//! Sharing and collaboration, peer to peer ([`cadrs_net`]; Onshape's "Sharing and
//! Collaboration").
//!
//! - **Share** (the document's top bar) makes the open document shareable and copies its link:
//!   the room it is shared in is kept in its metadata ([`cadrs_core::collab::Sharing`]), so the
//!   link stays the same. For now the link lets anyone who has it in with **Can edit**; the
//!   Share dialog will offer the choice.
//! - The owner's cadrs answers whoever opens the link ([`serve_requests`]), from the open
//!   document or from the store, while link sharing is on.
//! - **Create ▸ Join shared document…** on the documents page fetches a document by its link,
//!   keeps it in the store marked as shared ([`cadrs_core::collab::SharedInfo`]: it is listed
//!   under **Shared with me**) and opens it. Until edits travel between peers it opens
//!   read-only.

use bevy::prelude::*;
use cadrs_core::DocumentMeta;
use cadrs_core::collab::{Permission, SharedInfo, Sharing};
use cadrs_net::bevy_iroh::{Iroh, IrohTask};
use cadrs_net::{ShareLink, Welcome, transfer};

use crate::{ActiveDocument, AppClock, AppState, DocumentStore};

pub mod live;

pub struct CollabPlugin;

impl Plugin for CollabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Joins>().add_systems(Update, poll_joins);
        live::plugin(app);
    }
}

/// Why a shared document opens read-only: until its room is joined ([`live`]), or for good
/// without the network.
pub fn read_only_reason(shared: &SharedInfo) -> String {
    format!(
        "Connecting to {}… (with the network off, CADRS_NET=0, this is the last copy)",
        shared.owner_name
    )
}

/// A fresh room topic.
fn random_topic() -> [u8; 32] {
    let mut t = [0; 32];
    t[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    t[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    t
}

/// Share: the open document's link, on the clipboard.
pub fn share_current(world: &mut World) {
    let text = match make_link(world) {
        Ok(link) => {
            info!("share link: {link}");
            if let Some(mut clip) = world.get_resource_mut::<bevy::clipboard::Clipboard>()
                && let Err(e) = clip.set_text(link.to_string())
            {
                warn!("clipboard: {e:?}");
            }
            "Link copied: anyone with it can open and edit this document".to_string()
        }
        Err(why) => why,
    };
    crate::linked::toast(world, text);
}

/// The open document's link, sharing it (and saving that) if it wasn't yet.
fn make_link(world: &mut World) -> Result<ShareLink, String> {
    let owner = world
        .get_resource::<Iroh>()
        .and_then(Iroh::addr)
        .ok_or("Sharing needs the network, which is off (CADRS_NET=0)")?;
    let store = world.resource::<DocumentStore>().0.clone();
    let mut d = world.resource_mut::<ActiveDocument>();
    let (document, name) = (d.doc.id, d.doc.name.clone());
    let meta = d
        .meta
        .as_mut()
        .ok_or("Only a saved document can be shared")?;
    if let Some(shared) = &meta.shared {
        return Err(format!(
            "Only {} can share this document",
            shared.owner_name
        ));
    }
    let sharing = meta
        .sharing
        .get_or_insert_with(|| Sharing::new(random_topic()));
    sharing.link.get_or_insert(Permission::Edit);
    let topic = sharing.topic;
    d.save_now(&store)
        .map_err(|e| format!("Cannot share the document: {e}"))?;
    Ok(ShareLink {
        document,
        name,
        topic,
        owner,
    })
}

/// Documents being fetched by their links.
#[derive(Resource, Default)]
struct Joins(Vec<(ShareLink, IrohTask<Result<Welcome, String>>)>);

/// Join shared document: fetches the document a link names.
pub fn start_join(world: &mut World, text: &str) {
    let link = match text.parse::<ShareLink>() {
        Ok(l) => l,
        Err(why) => {
            crate::linked::toast(world, why);
            return;
        }
    };
    let Some(iroh) = world.get_resource::<Iroh>() else {
        crate::linked::toast(
            world,
            "Joining needs the network, which is off (CADRS_NET=0)",
        );
        return;
    };
    if link.owner.id == iroh.id() {
        crate::linked::toast(world, "That is a link to your own document");
        return;
    }
    let task = transfer::start_fetch(iroh, link.clone());
    let text = format!("Joining \u{201c}{}\u{201d}…", link.name);
    world.resource_mut::<Joins>().0.push((link, task));
    crate::linked::toast(world, text);
}

fn poll_joins(world: &mut World) {
    let mut done = Vec::new();
    world.resource_mut::<Joins>().0.retain_mut(|(link, task)| {
        let Some(r) = task.try_take() else {
            return true;
        };
        done.push((link.clone(), r));
        false
    });
    for (link, r) in done {
        match r.and_then(|w| keep_joined(world, &link, w)) {
            Ok(id) => {
                // Opened at once from the documents page; from anywhere else it waits in Shared
                // with me.
                if *world.resource::<State<AppState>>().get() == AppState::Landing {
                    crate::landing::open_document(world, id);
                } else {
                    let text = format!("\u{201c}{}\u{201d} is in Shared with me", link.name);
                    crate::linked::toast(world, text);
                }
            }
            Err(why) => crate::linked::toast(world, format!("Cannot join: {why}")),
        }
    }
}

/// Stores a fetched document as shared with this user (over the copy from an earlier join).
fn keep_joined(
    world: &mut World,
    link: &ShareLink,
    welcome: Welcome,
) -> Result<cadrs_core::DocumentId, String> {
    let store = world.resource::<DocumentStore>().0.clone();
    let now = world.resource::<AppClock>().now();
    let doc = welcome.snapshot.install()?;
    if doc.id != link.document {
        return Err("The owner sent another document".into());
    }
    let shared = SharedInfo {
        owner: link.owner.id.to_string(),
        owner_name: welcome.owner_name.clone(),
        link: link.to_string(),
        permission: welcome.permission,
    };
    let earlier = store.read(doc.id).ok().map(|f| f.meta);
    let mut meta = earlier
        .clone()
        .unwrap_or_else(|| DocumentMeta::new(&welcome.owner_name, now));
    meta.shared = Some(shared);
    meta.trashed = None;
    let saved = if earlier.is_some() {
        store.save(&doc, &meta)
    } else {
        store.create(&doc, &meta).map(|_| ())
    };
    saved.map_err(|e| e.to_string())?;
    Ok(doc.id)
}
