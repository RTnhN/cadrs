//! `cadrs --install`: installs this executable for the current user (Linux), so a checkout can be
//! the source of truth for the cadrs on a machine. `cargo install-local` (an alias in
//! `.cargo/config.toml`) builds in release and runs it.
//!
//! - `~/.local/opt/cadrs/`: a stripped copy of the executable and the built-in `libraries/`,
//!   which the app reads beside itself. A copy, so rebuilding the checkout never pulls the
//!   executable from under a running install.
//! - `~/.local/bin/cadrs`: a symlink to it.
//! - `~/.local/share/applications/cadrs.desktop` and its icon. The window's Wayland app id is
//!   `cadrs` (see `cadrs_harness::default_plugins`), which ties the window to this entry.

use std::path::{Path, PathBuf};

const DESKTOP: &str = include_str!("../../../tools/linux/cadrs.desktop");
const ICON: &str = include_str!("../../../tools/linux/cadrs.svg");

pub fn run() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or("HOME is not set")?;
    let exe = std::env::current_exe().map_err(|e| format!("cannot find this executable: {e}"))?;
    let libraries = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libraries");
    let prefix = home.join(".local/opt/cadrs");
    let err = |what: &str, p: &Path, e: std::io::Error| format!("cannot {what} {}: {e}", p.display());

    // Assemble the new install beside the old one, then swap them.
    let staging = prefix.with_extension("new");
    let old = prefix.with_extension("old");
    for d in [&staging, &old] {
        if d.exists() {
            std::fs::remove_dir_all(d).map_err(|e| err("remove", d, e))?;
        }
    }
    std::fs::create_dir_all(&staging).map_err(|e| err("create", &staging, e))?;
    let installed = staging.join("cadrs");
    std::fs::copy(&exe, &installed).map_err(|e| err("copy", &exe, e))?;
    // The debug symbols are a third of the executable; without `strip` the copy keeps them.
    let _ = std::process::Command::new("strip").arg(&installed).status();
    copy_dir(&libraries, &staging.join("libraries")).map_err(|e| err("copy", &libraries, e))?;
    if prefix.exists() {
        std::fs::rename(&prefix, &old).map_err(|e| err("move", &prefix, e))?;
    }
    std::fs::rename(&staging, &prefix).map_err(|e| err("move", &staging, e))?;
    if old.exists() {
        std::fs::remove_dir_all(&old).map_err(|e| err("remove", &old, e))?;
    }

    let bin = home.join(".local/bin");
    let apps = home.join(".local/share/applications");
    let icons = home.join(".local/share/icons/hicolor/scalable/apps");
    for d in [&bin, &apps, &icons] {
        std::fs::create_dir_all(d).map_err(|e| err("create", d, e))?;
    }
    let link = bin.join("cadrs");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(prefix.join("cadrs"), &link).map_err(|e| err("link", &link, e))?;
    let icon = icons.join("cadrs.svg");
    std::fs::write(&icon, ICON).map_err(|e| err("write", &icon, e))?;
    // The icon by its path: a user hicolor theme without an index.theme keeps a stale icon cache
    // that hides a new icon named by theme name.
    let desktop = DESKTOP.replace("@EXEC@", &prefix.join("cadrs").to_string_lossy()).replace("@ICON@", &icon.to_string_lossy());
    let entry = apps.join("cadrs.desktop");
    std::fs::write(&entry, desktop).map_err(|e| err("write", &entry, e))?;
    let _ = std::process::Command::new("update-desktop-database").arg(&apps).stderr(std::process::Stdio::null()).status();
    Ok(prefix)
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
