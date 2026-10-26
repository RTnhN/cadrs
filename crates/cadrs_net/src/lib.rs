//! Sharing and collaboration between cadrs peers, with no server: iroh through
//! [`bevy_iroh`] (QUIC dialled by public key, hole punching with relay fallback, gossip rooms,
//! signed typed messages).
//!
//! - [`NetPlugin`] starts this node: its identity is a key kept next to the documents
//!   ([`IDENTITY_FILE`]), so peers recognise it from run to run, and its public key is the
//!   user's **cadrs ID** ([`cadrs_id`]).
//! - Only bevy_iroh's transport, rooms and typed messages are used; its entity replication of
//!   `Transform` is off and its voice and video features are not built.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_iroh::{Identity, Iroh, IrohPlugin};

pub use bevy_iroh;

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
}

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        let identity = self
            .identity
            .clone()
            .map_or(Identity::Ephemeral, Identity::File);
        let mut iroh = IrohPlugin::default()
            .with_identity(identity)
            .with_display_name(self.display_name.clone());
        // Documents travel as their own messages, never as replicated entities.
        iroh.replicate_transform = false;
        app.add_plugins(iroh);
    }
}

/// This user's cadrs ID: the node's public key, what someone shares a document with. `None`
/// while networking is off.
pub fn cadrs_id(iroh: Option<&Iroh>) -> Option<String> {
    iroh.map(|i| i.id().to_string())
}
