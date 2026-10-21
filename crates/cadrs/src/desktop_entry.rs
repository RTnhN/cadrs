//! `cadrs --install-desktop`: a Linux launcher for this executable, with the app icon
//! (`assets/icons/cadrs.png`, the one `cadrs_app::app_icon` sets on the window).

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
    fs::write(&icon, include_bytes!("../../../assets/icons/cadrs.png"))?;
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
