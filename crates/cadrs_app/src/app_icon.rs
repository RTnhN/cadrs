//! The window's icon (issue #14): the cadrs logo (`assets/icons/cadrs.png`, built into the
//! binary) set on the window once winit has made it, for the taskbar and title bar on Windows
//! and X11. (The Windows executable carries the same icon as a resource: `crates/cadrs/build.rs`.
//! Wayland shows a desktop entry's icon instead.)
//!
//! On Windows the icons come from that resource at the sizes the title bar (16 px) and the
//! taskbar (32 px) show them, times the display scale: the `.ico` has an image drawn for each
//! common size, where Windows shrinking the 256 px logo to 16 px looks pixelated. They are set
//! again when the window moves to a display with another scale.

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::winit::WINIT_WINDOWS;

pub struct AppIconPlugin;

impl Plugin for AppIconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, set_window_icon);
    }
}

const ICON_PNG: &[u8] = include_bytes!("../../../assets/icons/cadrs.png");

/// `WINIT_WINDOWS` is thread-local and only filled on the main thread: the marker keeps this
/// system there (on another thread it never finds the window).
fn set_window_icon(
    windows: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut set_for_scale: Local<Option<f64>>,
    _main_thread: NonSendMarker,
) {
    let Ok(entity) = windows.single() else { return };
    WINIT_WINDOWS.with_borrow(|w| {
        // Not made yet (or headless: no winit window at all, nothing to do then).
        let Some(window) = w.get_window(entity) else {
            return;
        };
        let scale = window.scale_factor();
        if *set_for_scale == Some(scale) {
            return;
        }
        *set_for_scale = Some(scale);
        #[cfg(windows)]
        if set_from_resource(window, scale) {
            return;
        }
        let icon = image::load_from_memory(ICON_PNG)
            .ok()
            .map(|i| i.into_rgba8())
            .and_then(|i| {
                let (wd, ht) = i.dimensions();
                winit::window::Icon::from_rgba(i.into_raw(), wd, ht).ok()
            });
        if icon.is_none() {
            warn!("the app icon couldn't be read");
        }
        window.set_window_icon(icon);
    });
}

/// Sets the title bar and taskbar icons from the executable's icon resource (ordinal 1) at their
/// sizes for `scale`. False if the executable has no icon resource (built without `windres`).
#[cfg(windows)]
fn set_from_resource(window: &winit::window::Window, scale: f64) -> bool {
    use winit::dpi::PhysicalSize;
    use winit::platform::windows::{IconExtWindows, WindowExtWindows};
    use winit::window::Icon;
    let at = |px: f64| {
        let s = (px * scale).round() as u32;
        Icon::from_resource(1, Some(PhysicalSize::new(s, s))).ok()
    };
    let (Some(small), Some(big)) = (at(16.0), at(32.0)) else {
        return false;
    };
    window.set_window_icon(Some(small));
    window.set_taskbar_icon(Some(big));
    true
}
