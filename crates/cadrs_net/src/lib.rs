//! Sharing and collaboration between cadrs peers, with no server: iroh through
//! [`bevy_iroh`] (QUIC dialled by public key, hole punching with relay fallback, gossip rooms,
//! signed typed messages).
//!
//! - [`NetPlugin`] starts this node: its identity is a key kept next to the documents
//!   ([`IDENTITY_FILE`]), so peers recognise it from run to run, and its public key is the
//!   user's **cadrs ID** ([`cadrs_id`]).
//! - A document is shared by a [`ShareLink`]; whoever opens one fetches the document from its
//!   owner over [`transfer`], and proposes their edits the same way. The owner's app decides
//!   through [`Requests`], and its numbered edits ([`Sequenced`]) reach everyone over
//!   the document's room ([`session`]).
//! - Only bevy_iroh's transport, rooms and typed messages are used; its entity replication of
//!   `Transform` is off and its voice and video features are not built.

use std::path::PathBuf;

pub mod link;
pub mod session;
pub mod transfer;

use bevy::prelude::*;
use bevy_iroh::iroh::protocol::DynProtocolHandler;
use bevy_iroh::{Identity, Iroh, IrohPlugin, Relays};

pub use bevy_iroh;
pub use link::ShareLink;
pub use session::{AccessChanged, CommentChanged, Presence, Sequenced, View};

/// A question to another peer on its way: polled from a system until the answer (or why
/// there is none) arrives.
pub type Answer<T> = bevy_iroh::IrohTask<Result<T, String>>;
pub use transfer::{Ask, Invite, Request, Requests, Verdict, Welcome};

/// The node's secret key, in the documents folder (32 raw bytes, mode 0600). Each data
/// directory is a peer of its own, so two instances with their own `--data-dir` can share
/// with each other on one machine.
pub const IDENTITY_FILE: &str = "identity.key";

/// Starts this node. Added by the app itself, not by scenarios, headless runs or tests: the
/// endpoint binds and reaches the relays as soon as it is built.
pub struct NetPlugin {
    /// Where the identity is kept; `None` is a fresh key each run.
    pub identity: Option<PathBuf>,
    /// What peers see of this user.
    pub display_name: String,
    /// n0's relays for the app; none in tests, which stay on the machine.
    pub relays: Relays,
}

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        let identity = self
            .identity
            .clone()
            .map_or(Identity::Ephemeral, Identity::File);
        let (handler, requests) = transfer::handler();
        let mut iroh = IrohPlugin::default()
            .with_identity(identity)
            .with_relays(self.relays.clone())
            .with_display_name(self.display_name.clone())
            .with_protocol(
                transfer::ALPN,
                Box::new(handler) as Box<dyn DynProtocolHandler>,
            );
        // Documents travel as their own messages, never as replicated entities.
        iroh.replicate_transform = false;
        app.add_plugins(iroh).insert_resource(requests);
        if let Some(id) = cadrs_id(app.world().get_resource::<Iroh>()) {
            info!("cadrs ID: {id}");
        }
        session::register(app);
    }
}

/// This user's cadrs ID: the node's public key, what someone shares a document with. `None`
/// while networking is off.
pub fn cadrs_id(iroh: Option<&Iroh>) -> Option<String> {
    iroh.map(|i| i.id().to_string())
}
