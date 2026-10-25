//! Debugging aids, each off unless its environment variable is set (so they cost nothing
//! otherwise):
//!
//! - `CADRS_SLOW_FRAMES=<ms>`: prints every frame that took longer than this (wall clock, so
//!   scripted runs' fixed time steps don't hide it) with the systems that took over a tenth of
//!   it, slowest first, from the frame profile ([`crate::frame_profile`]; needs the default
//!   `profiler` feature for the system times). `CADRS_SLOW_FRAMES=1` (or any value that isn't a
//!   number) uses 50 ms.

use std::time::Instant;

use bevy::prelude::*;

use crate::frame_profile::{FrameProfile, Kind};

pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut App) {
        if let Some(ms) = slow_frame_threshold() {
            app.insert_resource(SlowFrames { ms }).add_systems(
                First,
                report_slow_frames.after(crate::frame_profile::take_frame),
            );
        }
    }
}

/// `CADRS_SLOW_FRAMES` in ms, if set.
fn slow_frame_threshold() -> Option<f32> {
    let v = std::env::var("CADRS_SLOW_FRAMES").ok()?;
    Some(match v.trim().parse::<f32>() {
        Ok(ms) if ms > 1.0 => ms,
        _ => 50.0,
    })
}

#[derive(Resource)]
struct SlowFrames {
    ms: f32,
}

fn report_slow_frames(
    slow: Res<SlowFrames>,
    profile: Res<FrameProfile>,
    mut last: Local<Option<Instant>>,
) {
    let now = Instant::now();
    let Some(ms) = last.replace(now).map(|t| (now - t).as_secs_f32() * 1000.0) else {
        return;
    };
    if ms < slow.ms {
        return;
    }
    let mut systems: Vec<_> = profile
        .last()
        .map(|f| {
            f.cpu
                .iter()
                .filter(|(kind, _, t, _)| *kind == Kind::System && *t >= ms / 10.0)
                .collect()
        })
        .unwrap_or_default();
    systems.sort_by(|a, b| b.2.total_cmp(&a.2));
    let mut out = format!("slow frame: {ms:.0} ms");
    for (_, name, t, runs) in systems.iter().take(12) {
        out.push_str(&format!("\n  {t:8.1} ms  x{runs}  {name}"));
    }
    warn!("{out}");
}
