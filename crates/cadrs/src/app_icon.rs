//! Native app identity, reusing the Documents logo's icon-rs cube.

use bevy::{ecs::system::NonSendMarker, prelude::*, winit::WINIT_WINDOWS};
use cadrs_ui::icon::{APP_ICON_PNG, APP_ICON_RGBA, APP_ICON_SIZE};

#[derive(Component)]
pub(crate) struct IconSet;

/// Retry until the native window exists; the marker keeps headless runs harmless and ensures
/// each new window receives its icon. winit requires this to run on the main thread.
pub(crate) fn set_window_icon(
    windows: Query<Entity, (With<Window>, Without<IconSet>)>,
    mut commands: Commands,
    _main_thread: NonSendMarker,
) {
    if windows.is_empty() { return; }
    WINIT_WINDOWS.with_borrow(|native| {
        for entity in &windows {
            if let Some(window) = native.get_window(entity) {
                let icon = winit::window::Icon::from_rgba(APP_ICON_RGBA.to_vec(), APP_ICON_SIZE, APP_ICON_SIZE)
                    .expect("build-time app icon has valid RGBA dimensions");
                window.set_window_icon(Some(icon));
                commands.entity(entity).insert(IconSet);
            }
        }
    });
}

/// Wayland docks obtain icons from desktop entries rather than set_window_icon. Register the
/// same app ID as Window::name, pointing at this executable (no shell or cargo dependency).
#[cfg(target_os = "linux")]
pub(crate) fn install_desktop() -> std::io::Result<()> {
    use std::{fs, io, path::Path};
    let dirs = directories::BaseDirs::new().ok_or_else(|| io::Error::other("no user data directory"))?;
    let applications = dirs.data_dir().join("applications");
    let icons = dirs.data_dir().join("icons/hicolor/256x256/apps");
    let exe = std::env::current_exe()?;
    fn quote(path: &Path) -> io::Result<String> {
        let s = path.to_str().ok_or_else(|| io::Error::other("executable path is not UTF-8"))?;
        // Desktop Entry Exec quoting is distinct from shell quoting. Percent is a field code,
        // and backslash is escaped once for Exec and once for the desktop string value.
        Ok(format!("\"{}\"", s.replace('\\', "\\\\\\\\").replace('"', "\\\\\"").replace('`', "\\\\`").replace('$', "\\\\$").replace('%', "%%").replace('\n', "\\n").replace('\r', "\\r")))
    }
    let exec = quote(&exe)?;
    fs::create_dir_all(&applications)?;
    fs::create_dir_all(&icons)?;
    let icon = icons.join("cadrs-logo.png");
    fs::write(&icon, APP_ICON_PNG)?;
    // GNOME can retain a cached missing themed icon after a first-time install. A file icon
    // resolves immediately, including when the user's hicolor directory has no theme index.
    let icon = icon.to_str().ok_or_else(|| io::Error::other("icon path is not UTF-8"))?
        .replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r");
    fs::write(applications.join("cadrs.desktop"), format!(
        "[Desktop Entry]\nType=Application\nName=cadrs\nComment=CAD parts, assemblies and drawings\nExec={exec}\nIcon={icon}\nTerminal=false\nCategories=Graphics;Engineering;\nStartupWMClass=cadrs\n"
    ))?;
    println!("Installed cadrs launcher and cube icon in {}", dirs.data_dir().display());
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn install_desktop() -> std::io::Result<()> {
    Err(std::io::Error::other("desktop launcher installation is supported on Linux"))
}
