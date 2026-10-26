//! Asking a shared document's owner for something: one QUIC stream, dialled to the owner's
//! key, on an ALPN of our own (bevy_iroh's direct messages stop at a megabyte; a document, or
//! an edit that pastes one in, need not).
//!
//! - **Join**: the document as it is now ([`Welcome`]: the permission granted, the edit it is
//!   at, the document).
//! - **Propose**: an edit to number ([`Verdict`]). One at a time per peer, so they arrive in
//!   the order they were made; the numbered edit itself reaches everyone over the room.
//!
//! Every request names the document and the room's topic, which only a holder of the link
//! knows. The owner's app decides ([`OwnerRequests`]).

use std::time::Duration;

use bevy::prelude::*;
use bevy_iroh::{
    Iroh, IrohTask,
    iroh::{
        Endpoint, EndpointAddr, EndpointId,
        endpoint::Connection,
        protocol::{AcceptError, ProtocolHandler},
    },
};
use cadrs_core::DocumentId;
use cadrs_core::collab::sync::Change;
use cadrs_core::collab::{self, Permission, Snapshot};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::sync::{mpsc, oneshot};

use crate::link::ShareLink;

/// Versioned, so a peer with another framing fails to connect rather than misreads.
pub const ALPN: &[u8] = b"cadrs/doc/2";

/// The largest request (an edit) and the largest answer (a document), packed.
const MAX_MESSAGE: usize = 1 << 30;
/// How long the owner's app has to answer before the peer is told it did not.
const ANSWER_WITHIN: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize)]
struct Wire {
    document: DocumentId,
    topic: [u8; 32],
    ask: WireAsk,
}

#[derive(Debug, Serialize, Deserialize)]
enum WireAsk {
    Join,
    Propose {
        epoch: u64,
        id: u64,
        base: u64,
        /// The packed [`Change`].
        change: Vec<u8>,
    },
}

/// What the owner sends someone they let in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Welcome {
    pub permission: Permission,
    pub owner_name: String,
    /// The owner's session: edits are numbered afresh each time the owner opens the document.
    pub epoch: u64,
    /// The document after edit `snapshot.seq`.
    pub snapshot: Snapshot,
}

/// The owner's answer to a proposed edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Numbered: it reaches everyone as this edit.
    Accepted(u64),
    /// Refused, and why (someone else changed the same thing first, view access, …).
    Refused(String),
    /// Made from a copy the owner no longer has (an old session, or too far behind): fetch
    /// the document again.
    Resync,
    /// The owner does not have the document open: nobody numbers edits.
    Closed,
}

/// What a peer asks the owner.
#[derive(Debug)]
pub enum Ask {
    Join,
    Propose {
        epoch: u64,
        id: u64,
        base: u64,
        change: Change,
    },
}

/// A peer's request. The app checks the topic and the peer against the document's sharing and
/// [`answer`](Self::answer)s; one dropped unanswered is a refusal.
#[derive(Debug)]
pub struct OwnerRequest {
    pub from: EndpointId,
    pub document: DocumentId,
    pub topic: [u8; 32],
    pub ask: Ask,
    reply: oneshot::Sender<Result<Vec<u8>, String>>,
}

impl OwnerRequest {
    /// Answers a Join with the document, or the reason it is refused (shown to the peer).
    pub fn welcome(self, welcome: Result<Welcome, String>) {
        let _ = self.reply.send(welcome.and_then(|w| collab::pack(&w)));
    }

    /// Answers a Propose.
    pub fn verdict(self, verdict: Verdict) {
        let _ = self.reply.send(collab::pack(&verdict));
    }

    /// Refuses whatever it asks, with the reason.
    pub fn refuse(self, why: impl Into<String>) {
        let _ = self.reply.send(Err(why.into()));
    }
}

/// The requests that arrived and are not answered yet.
#[derive(Resource)]
pub struct OwnerRequests(mpsc::UnboundedReceiver<OwnerRequest>);

impl OwnerRequests {
    pub fn drain(&mut self) -> Vec<OwnerRequest> {
        let mut out = Vec::new();
        while let Ok(r) = self.0.try_recv() {
            out.push(r);
        }
        out
    }
}

/// The accept side, and the [`OwnerRequests`] it feeds.
pub(crate) fn handler() -> (Handler, OwnerRequests) {
    let (tx, rx) = mpsc::unbounded_channel();
    (Handler { tx }, OwnerRequests(rx))
}

#[derive(Debug, Clone)]
pub(crate) struct Handler {
    tx: mpsc::UnboundedSender<OwnerRequest>,
}

impl Handler {
    async fn answer(&self, from: EndpointId, request: &[u8]) -> Result<Vec<u8>, String> {
        let wire: Wire = postcard::from_bytes(request).map_err(|_| "Not a request".to_string())?;
        let ask = match wire.ask {
            WireAsk::Join => Ask::Join,
            WireAsk::Propose {
                epoch,
                id,
                base,
                change,
            } => Ask::Propose {
                epoch,
                id,
                base,
                change: collab::unpack(&change)?,
            },
        };
        let (reply, answer) = oneshot::channel();
        self.tx
            .send(OwnerRequest {
                from,
                document: wire.document,
                topic: wire.topic,
                ask,
                reply,
            })
            .map_err(|_| "The owner's cadrs is closing".to_string())?;
        match tokio::time::timeout(ANSWER_WITHIN, answer).await {
            Ok(Ok(r)) => r,
            Ok(Err(_)) => Err("The owner's cadrs refused the request".into()),
            Err(_) => Err("The owner's cadrs did not answer".into()),
        }
    }
}

impl ProtocolHandler for Handler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let from = connection.remote_id();
        let Ok((mut send, mut recv)) = connection.accept_bi().await else {
            return Ok(());
        };
        let answer = match recv.read_to_end(MAX_MESSAGE).await {
            Ok(request) => self.answer(from, &request).await,
            Err(e) => Err(format!("Cannot read the request: {e}")),
        };
        // One byte of status, then the packed answer or the reason.
        let reply = match answer {
            Ok(bytes) => [&[0u8][..], &bytes].concat(),
            Err(why) => {
                debug!("cadrs_net: refused {}: {why}", from.fmt_short());
                [&[1u8][..], why.as_bytes()].concat()
            }
        };
        if send.write_all(&reply).await.is_ok() {
            let _ = send.finish();
        }
        connection.closed().await;
        Ok(())
    }
}

/// Sends `wire` to `owner` and reads the answer.
async fn ask<T: DeserializeOwned>(
    endpoint: Endpoint,
    owner: EndpointAddr,
    wire: Wire,
) -> Result<T, String> {
    let who = owner.id.fmt_short();
    let conn = endpoint
        .connect(owner, ALPN)
        .await
        .map_err(|e| format!("Cannot reach the owner ({who}): {e}"))?;
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| format!("Cannot reach the owner ({who}): {e}"))?;
    let request = postcard::to_stdvec(&wire).map_err(|e| e.to_string())?;
    send.write_all(&request)
        .await
        .map_err(|e| format!("Lost the owner ({who}): {e}"))?;
    let _ = send.finish();
    let reply = recv
        .read_to_end(MAX_MESSAGE)
        .await
        .map_err(|e| format!("Lost the owner ({who}): {e}"))?;
    conn.close(0u32.into(), b"done");
    match reply.split_first() {
        Some((0, packed)) => collab::unpack(packed),
        Some((_, why)) => Err(String::from_utf8_lossy(why).into_owned()),
        None => Err(format!("The owner ({who}) sent nothing")),
    }
}

/// Asks the link's owner for the document.
pub async fn fetch(endpoint: Endpoint, link: ShareLink) -> Result<Welcome, String> {
    let wire = Wire {
        document: link.document,
        topic: link.topic,
        ask: WireAsk::Join,
    };
    ask(endpoint, link.owner, wire).await
}

/// [`fetch`] on the network runtime: poll the task from a system.
pub fn start_fetch(iroh: &Iroh, link: ShareLink) -> IrohTask<Result<Welcome, String>> {
    iroh.spawn(fetch(iroh.endpoint(), link))
}

/// Sends edit `id` (made from edit `base` of session `epoch`) to the link's owner. `Err` is a
/// failure to ask (the owner unreachable), not a refusal.
pub fn start_propose(
    iroh: &Iroh,
    link: &ShareLink,
    epoch: u64,
    id: u64,
    base: u64,
    change: &Change,
) -> IrohTask<Result<Verdict, String>> {
    let wire = collab::pack(change).map(|change| Wire {
        document: link.document,
        topic: link.topic,
        ask: WireAsk::Propose {
            epoch,
            id,
            base,
            change,
        },
    });
    let (endpoint, owner) = (iroh.endpoint(), link.owner.clone());
    iroh.spawn(async move { ask(endpoint, owner, wire?).await })
}
