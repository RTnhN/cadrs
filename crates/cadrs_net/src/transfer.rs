//! Fetching a shared document from its owner: one QUIC stream, dialled to the owner's key, on
//! an ALPN of our own (bevy_iroh's direct messages stop at a megabyte; a document need not).
//!
//! The joiner sends the document's id and the room's topic, which only a holder of the link
//! knows. The owner's app decides ([`SnapshotRequests`]) and answers with a [`Welcome`]: the
//! permission granted and the document as it is now.

use std::time::Duration;

use bevy::prelude::*;
use bevy_iroh::{
    Iroh, IrohTask,
    iroh::{
        Endpoint, EndpointId,
        endpoint::Connection,
        protocol::{AcceptError, ProtocolHandler},
    },
};
use cadrs_core::DocumentId;
use cadrs_core::collab::{self, Permission, Snapshot};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::link::ShareLink;

/// Versioned, so a peer with another framing fails to connect rather than misreads.
pub const ALPN: &[u8] = b"cadrs/doc/1";

/// A request is an id and a topic; anything longer is not one.
const MAX_REQUEST: usize = 4096;
/// The largest document a joiner accepts, packed.
const MAX_WELCOME: usize = 1 << 30;
/// How long the owner's app has to answer before the joiner is told it did not.
const ANSWER_WITHIN: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    document: DocumentId,
    topic: [u8; 32],
}

/// What the owner sends someone they let in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Welcome {
    pub permission: Permission,
    pub owner_name: String,
    pub snapshot: Snapshot,
}

/// A peer asking for a document. The app checks the topic and the peer against the
/// document's sharing and [`answer`](Self::answer)s; one dropped unanswered is a refusal.
#[derive(Debug)]
pub struct SnapshotRequest {
    pub from: EndpointId,
    pub document: DocumentId,
    pub topic: [u8; 32],
    reply: oneshot::Sender<Result<Vec<u8>, String>>,
}

impl SnapshotRequest {
    /// Sends the document, or the reason it is refused (shown to the joiner).
    pub fn answer(self, welcome: Result<Welcome, String>) {
        let _ = self.reply.send(welcome.and_then(|w| collab::pack(&w)));
    }
}

/// The requests that arrived and are not answered yet.
#[derive(Resource)]
pub struct SnapshotRequests(mpsc::UnboundedReceiver<SnapshotRequest>);

impl SnapshotRequests {
    pub fn drain(&mut self) -> Vec<SnapshotRequest> {
        let mut out = Vec::new();
        while let Ok(r) = self.0.try_recv() {
            out.push(r);
        }
        out
    }
}

/// The accept side, and the [`SnapshotRequests`] it feeds.
pub(crate) fn handler() -> (Handler, SnapshotRequests) {
    let (tx, rx) = mpsc::unbounded_channel();
    (Handler { tx }, SnapshotRequests(rx))
}

#[derive(Debug, Clone)]
pub(crate) struct Handler {
    tx: mpsc::UnboundedSender<SnapshotRequest>,
}

impl Handler {
    async fn answer(&self, from: EndpointId, request: &[u8]) -> Result<Vec<u8>, String> {
        let req: Request =
            postcard::from_bytes(request).map_err(|_| "Not a document request".to_string())?;
        let (reply, answer) = oneshot::channel();
        self.tx
            .send(SnapshotRequest {
                from,
                document: req.document,
                topic: req.topic,
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
        let answer = match recv.read_to_end(MAX_REQUEST).await {
            Ok(request) => self.answer(from, &request).await,
            Err(e) => Err(format!("Cannot read the request: {e}")),
        };
        // One byte of status, then the packed welcome or the reason.
        let reply = match answer {
            Ok(bytes) => [&[0u8][..], &bytes].concat(),
            Err(why) => {
                debug!("cadrs_net: refused {} a document: {why}", from.fmt_short());
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

/// Asks the link's owner for the document.
pub async fn fetch(endpoint: Endpoint, link: ShareLink) -> Result<Welcome, String> {
    let owner = link.owner.id.fmt_short();
    let conn = endpoint
        .connect(link.owner.clone(), ALPN)
        .await
        .map_err(|e| format!("Cannot reach the owner ({owner}): {e}"))?;
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| format!("Cannot reach the owner ({owner}): {e}"))?;
    let request = postcard::to_stdvec(&Request {
        document: link.document,
        topic: link.topic,
    })
    .map_err(|e| e.to_string())?;
    send.write_all(&request)
        .await
        .map_err(|e| format!("Lost the owner ({owner}): {e}"))?;
    let _ = send.finish();
    let reply = recv
        .read_to_end(MAX_WELCOME)
        .await
        .map_err(|e| format!("Lost the owner ({owner}): {e}"))?;
    conn.close(0u32.into(), b"done");
    match reply.split_first() {
        Some((0, packed)) => collab::unpack(packed),
        Some((_, why)) => Err(String::from_utf8_lossy(why).into_owned()),
        None => Err(format!("The owner ({owner}) sent nothing")),
    }
}

/// [`fetch`] on the network runtime: poll the task from a system.
pub fn start_fetch(iroh: &Iroh, link: ShareLink) -> IrohTask<Result<Welcome, String>> {
    iroh.spawn(fetch(iroh.endpoint(), link))
}
