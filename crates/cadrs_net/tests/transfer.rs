//! Two peers in one process, over real iroh with no relay: one shares a document, the other
//! fetches it by its link.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_iroh::{Iroh, Relays};
use cadrs_core::Document;
use cadrs_core::collab::{Permission, Snapshot};
use cadrs_net::{NetPlugin, ShareLink, SnapshotRequests, Welcome, transfer};

const PATIENCE: Duration = Duration::from_secs(30);
const TOPIC: [u8; 32] = [42; 32];

fn peer(name: &str) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        NetPlugin {
            identity: None,
            display_name: name.into(),
            relays: Relays::Disabled,
        },
    ));
    app
}

/// The owner's side: `doc` to anyone who knows the topic, refused otherwise.
#[derive(Resource)]
struct Shared(Document);

fn serve(mut requests: ResMut<SnapshotRequests>, shared: Res<Shared>) {
    for r in requests.drain() {
        let ok = r.document == shared.0.id && r.topic == TOPIC;
        let doc = shared.0.clone();
        r.answer(if ok {
            Ok(Welcome {
                permission: Permission::Edit,
                owner_name: "Owner".into(),
                snapshot: Snapshot {
                    seq: 3,
                    document: doc,
                    blobs: Vec::new(),
                },
            })
        } else {
            Err("Link sharing is off".into())
        });
    }
}

fn fetch(owner: &mut App, joiner: &mut App, link: ShareLink) -> Result<Welcome, String> {
    let mut task = transfer::start_fetch(joiner.world().resource::<Iroh>(), link);
    let start = Instant::now();
    loop {
        owner.update();
        joiner.update();
        if let Some(r) = task.try_take() {
            eprintln!("fetched in {:?}", start.elapsed());
            return r;
        }
        assert!(
            start.elapsed() < PATIENCE,
            "gave up waiting for the document"
        );
        std::thread::sleep(Duration::from_millis(16));
    }
}

#[test]
fn a_document_travels_by_its_link_and_a_wrong_topic_is_refused() {
    let mut owner = peer("Owner");
    let mut joiner = peer("Joiner");
    let mut doc = Document::new("Bracket");
    // Big enough to be well past bevy_iroh's one-megabyte messages, deflated or not.
    doc.name = (0..400_000).map(|i| format!("{i:x}")).collect::<String>();
    owner
        .insert_resource(Shared(doc.clone()))
        .add_systems(Update, serve);
    let addr = owner.world().resource::<Iroh>().addr().unwrap();
    let link = ShareLink {
        document: doc.id,
        name: doc.name.clone(),
        topic: TOPIC,
        owner: addr,
    };

    let welcome = fetch(&mut owner, &mut joiner, link.clone()).unwrap();
    assert_eq!(welcome.permission, Permission::Edit);
    assert_eq!(welcome.snapshot.seq, 3);
    assert_eq!(welcome.snapshot.document, doc);

    let wrong = ShareLink {
        topic: [0; 32],
        ..link
    };
    assert_eq!(
        fetch(&mut owner, &mut joiner, wrong),
        Err("Link sharing is off".into())
    );
}
