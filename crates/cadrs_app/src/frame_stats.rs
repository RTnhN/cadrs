//! Frame rate: the real time between frames over the last couple of seconds ([`FrameStats`]),
//! shown as a small "60 fps · 16.7 ms" label at the bottom left of the 3D view (F12 shows or
//! hides it) and returned by the MCP tool `get_frame_stats` ([`crate::mcp`]), so a slow view
//! can be measured at any time.

use std::collections::VecDeque;

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use cadrs_ui::input::TextInputField;
use cadrs_ui::{ShortcutKey, Theme};
use serde_json::{Value, json};

use crate::AppState;

/// How far back the stats look.
const WINDOW_SECS: f64 = 2.0;
/// How often the label's text changes (a number that changes every frame can't be read).
const LABEL_EVERY_SECS: f32 = 0.5;

pub struct FrameStatsPlugin;

impl Plugin for FrameStatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameStats>()
            .add_systems(First, record_frame)
            .add_systems(Update, (toggle_on_key, sync_label).chain().run_if(in_state(AppState::Document)));
    }
}

/// The last [`WINDOW_SECS`] of frames: when each ended (s since start) and how long it took (ms).
#[derive(Resource, Debug, Default)]
pub struct FrameStats {
    frames: VecDeque<(f64, f32)>,
    /// The label is shown.
    pub shown: bool,
}

impl FrameStats {
    pub fn push(&mut self, now: f64, ms: f32) {
        self.frames.push_back((now, ms));
        while self.frames.front().is_some_and(|(t, _)| now - t > WINDOW_SECS) {
            self.frames.pop_front();
        }
    }

    /// Frames per second over the window (frames / the time they took).
    pub fn fps(&self) -> f32 {
        let total: f32 = self.frames.iter().map(|(_, ms)| ms).sum();
        if total > 0.0 { self.frames.len() as f32 * 1000.0 / total } else { 0.0 }
    }

    /// Frame times in ms over the window: (mean, median, 95th percentile, max).
    pub fn frame_ms(&self) -> (f32, f32, f32, f32) {
        let mut ms: Vec<f32> = self.frames.iter().map(|(_, ms)| *ms).collect();
        if ms.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        ms.sort_by(f32::total_cmp);
        let at = |q: f32| ms[((ms.len() - 1) as f32 * q).round() as usize];
        (ms.iter().sum::<f32>() / ms.len() as f32, at(0.5), at(0.95), ms[ms.len() - 1])
    }

    /// What `get_frame_stats` returns.
    pub fn describe(&self) -> Value {
        let (mean, median, p95, max) = self.frame_ms();
        let r = |x: f32| (x as f64 * 10.0).round() / 10.0;
        json!({
            "fps": r(self.fps()),
            "frames": self.frames.len(),
            "window_s": WINDOW_SECS,
            "frame_ms": { "mean": r(mean), "median": r(median), "p95": r(p95), "max": r(max) },
        })
    }

    fn label(&self) -> String {
        let (mean, _, _, max) = self.frame_ms();
        format!("{:.0} fps · {mean:.1} ms (max {max:.0})", self.fps())
    }
}

fn record_frame(time: Res<Time<Real>>, mut stats: ResMut<FrameStats>) {
    let ms = time.delta_secs() * 1000.0;
    if ms > 0.0 {
        stats.push(time.elapsed_secs_f64(), ms);
    }
}

#[derive(Component)]
struct FrameStatsLabel;

/// Spawns the (hidden) label into the viewport area.
pub fn spawn_frame_stats(p: &mut ChildSpawnerCommands, t: &Theme) {
    p.spawn((
        Name::new("frame-stats"),
        FrameStatsLabel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(t.space[3]),
            left: Val::Px(t.space[3]),
            padding: UiRect::axes(Val::Px(t.space[2]), Val::Px(t.space[1])),
            border_radius: BorderRadius::all(Val::Px(t.radius_lg)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(t.popover),
        Pickable::IGNORE,
    ))
    .with_child((t.text("", t.font_sm, FontWeight::NORMAL, t.muted_foreground), Pickable::IGNORE));
}

/// F12 shows or hides the label (not while typing in a field).
fn toggle_on_key(
    mut keys_in: MessageReader<KeyboardInput>,
    focus: Res<InputFocus>,
    q_fields: Query<(), With<TextInputField>>,
    mut stats: ResMut<FrameStats>,
) {
    let typing = focus.get().is_some_and(|e| q_fields.contains(e));
    for k in keys_in.read() {
        if k.state == ButtonState::Pressed && !k.repeat && k.shortcut_code() == KeyCode::F12 && !typing {
            stats.shown = !stats.shown;
        }
    }
}

fn sync_label(
    stats: Res<FrameStats>,
    time: Res<Time<Real>>,
    mut last: Local<f32>,
    mut q: Query<(&mut Node, &Children), With<FrameStatsLabel>>,
    mut q_text: Query<&mut Text>,
) {
    let display = if stats.shown { Display::Flex } else { Display::None };
    let now = time.elapsed_secs();
    let update = stats.is_changed() || now - *last >= LABEL_EVERY_SECS;
    for (mut node, children) in &mut q {
        if node.display != display {
            node.display = display;
        }
        if stats.shown && update {
            *last = now;
            for c in children.iter() {
                if let Ok(mut text) = q_text.get_mut(c) {
                    text.0 = stats.label();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_cover_the_last_two_seconds() {
        let mut s = FrameStats::default();
        // Three seconds at 10 ms a frame, then one 100 ms frame.
        for i in 1..=300 {
            s.push(i as f64 * 0.01, 10.0);
        }
        s.push(3.105, 100.0);
        // The window is [1.105, 3.105]: 190 frames of 10 ms and the 100 ms one.
        assert_eq!(s.frames.len(), 191);
        let (mean, median, p95, max) = s.frame_ms();
        assert_eq!((median, p95, max), (10.0, 10.0, 100.0));
        assert!((mean - 2000.0 / 191.0).abs() < 1e-3);
        assert!((s.fps() - 191.0 / 2.0).abs() < 1e-3);
        assert_eq!(s.describe()["frames"], 191);
    }
}
