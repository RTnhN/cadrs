//! Voice while editing together (the `voice` feature: bevy_iroh's media, off by default since
//! it builds libopus and openh264 and opens the microphone).
//!
//! **Join voice** beside the avatars puts a [`Voice`] on an entity shared in the document's
//! room: the microphone goes to everyone there, and everyone else's voice is played back (flat:
//! no listener, so no panning). Mute stops sending; leave ends it, as does leaving the
//! document. An avatar is outlined green while its owner speaks ([`VoiceUi::speaking`]).

use bevy::prelude::*;
use cadrs_net::bevy_iroh::Iroh;
use cadrs_net::bevy_iroh::prelude::{InRoom, Owner, Remote, Shared, Voice, VoiceLevel};

use super::live::Live;
use super::presence::VoiceUi;

/// Louder than this (on the meter's 0 to 1) is speaking.
const SPEAKING: f32 = 0.3;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (buttons, keep_in_room, speaking)
            .chain()
            .run_if(resource_exists::<Iroh>),
    )
    .add_systems(Startup, |mut ui: ResMut<VoiceUi>| ui.available = true);
}

/// This user's voice in the room.
#[derive(Component)]
struct MyVoice;

fn buttons(mut ui: ResMut<VoiceUi>, mut mine: Query<&mut Voice, With<MyVoice>>) {
    if let Some(join) = ui.want_join.take() {
        ui.joined = join;
        ui.muted = false;
    }
    if std::mem::take(&mut ui.toggle_mute) {
        ui.muted = !ui.muted;
    }
    let muted = ui.muted;
    for mut v in &mut mine {
        if v.muted != muted {
            v.muted = muted;
        }
    }
}

/// One voice of this user's, in the open document's room while joined (a new room when the
/// session moves to one).
fn keep_in_room(
    ui: Res<VoiceUi>,
    live: Option<Res<Live>>,
    mine: Query<(Entity, &InRoom), With<MyVoice>>,
    mut commands: Commands,
) {
    let want = live.filter(|_| ui.joined).map(|l| l.room());
    let mut have = false;
    for (e, room) in &mine {
        if Some(room.0) == want && !have {
            have = true;
        } else {
            commands.entity(e).despawn();
        }
    }
    if let (Some(room), false) = (want, have) {
        commands.spawn((
            Name::new("my-voice"),
            MyVoice,
            Voice { muted: ui.muted },
            Shared::default(),
            InRoom(room),
        ));
    }
}

/// Who is speaking: everyone else's voice level, by their cadrs ID.
fn speaking(voices: Query<(&Owner, &VoiceLevel), With<Remote>>, mut ui: ResMut<VoiceUi>) {
    let mut now: Vec<String> = voices
        .iter()
        .filter(|(_, level)| level.0 > SPEAKING)
        .map(|(owner, _)| owner.0.to_string())
        .collect();
    now.sort();
    if ui.speaking != now {
        ui.speaking = now;
    }
}
