//! A shared document's room: the owner hosts it while the document is open, everyone it is
//! shared with joins it. Numbered edits ([`Sequenced`]) reach everyone over it.

use bevy::prelude::*;
use bevy_iroh::RoomTicket;
use bevy_iroh::TopicId;
use bevy_iroh::message::MessageAppExt;
use bevy_iroh::prelude::Room;
use cadrs_core::DocumentId;
use cadrs_core::collab::{self, sync::Change};
use serde::{Deserialize, Serialize};

use crate::link::ShareLink;

/// The most an edit may take, packed, to travel in the room (gossip carries a megabyte, less
/// its envelope). A bigger one is announced without it, and peers fetch the document.
pub const MAX_IN_ROOM: usize = 900 * 1024;

/// Edit `seq` of the owner's session `epoch`, as the owner applied it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sequenced {
    pub document: DocumentId,
    pub epoch: u64,
    pub seq: u64,
    /// Who made it: their cadrs ID and name.
    pub author: String,
    pub author_name: String,
    /// The id its author gave it.
    pub id: u64,
    /// The packed [`Change`]; `None` when it was too big for the room.
    pub change: Option<Vec<u8>>,
    /// [`cadrs_core::history_log::document_hash`] of the owner's document after it.
    pub hash: u64,
}

impl Sequenced {
    /// The message for `change`, without it if it is too big.
    pub fn new(
        document: DocumentId,
        epoch: u64,
        seq: u64,
        author: (&str, &str),
        id: u64,
        change: &Change,
        hash: u64,
    ) -> Self {
        let change = collab::pack(change).ok().filter(|b| b.len() <= MAX_IN_ROOM);
        Self {
            document,
            epoch,
            seq,
            author: author.0.to_string(),
            author_name: author.1.to_string(),
            id,
            change,
            hash,
        }
    }

    pub fn change(&self) -> Option<Change> {
        collab::unpack(self.change.as_deref()?).ok()
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_net_message::<Sequenced>();
}

/// The owner's room for a document shared in `topic`: always the same topic, so links keep
/// working (a ticket naming no one opens the topic and waits for others).
pub fn host_room(commands: &mut Commands, topic: [u8; 32], name: &str) -> Entity {
    let ticket = RoomTicket::new(TopicId::from_bytes(topic), name, Vec::new());
    commands.spawn(Room::join(ticket)).id()
}

/// The room a link names, joined through its owner.
pub fn join_room(commands: &mut Commands, link: &ShareLink) -> Entity {
    let ticket = RoomTicket::new(
        TopicId::from_bytes(link.topic),
        link.name.clone(),
        vec![link.owner.clone()],
    );
    commands.spawn(Room::join(ticket)).id()
}
