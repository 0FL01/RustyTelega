//! The id the desktop knows Quill by: the window app_id (Wayland) and
//! WM_CLASS (X11), the desktop file name, the notification `desktop-entry`
//! hint, the launcher badge URI and the GeoClue `DesktopId`.
//!
//! A tarball, AppImage or distro package uses `org.shinycake.quill`, the name
//! of the desktop file Quill ships. Inside Flatpak the exported desktop file
//! and icon are named after the Flatpak app id (`io.github.shinycake.Quill`
//! on Flathub), so the shell only groups the window with its launcher when
//! the window carries that id. Flatpak sets `FLATPAK_ID` for that.
#![deny(clippy::indexing_slicing)]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The id outside Flatpak.
pub const DEFAULT: &str = "org.shinycake.quill";

/// Present in every Flatpak sandbox (flatpak-metadata(5)).
const FLATPAK_INFO: &str = "/.flatpak-info";

/// The id for this process. Read once: the environment does not change.
pub fn get() -> &'static str {
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| {
        let flatpak_id = std::env::var("FLATPAK_ID").ok();
        choose(flatpak_id.as_deref(), in_flatpak()).to_string()
    })
}

/// `<id>.desktop`.
pub fn desktop_file_name() -> String {
    format!("{}.desktop", get())
}

/// Whether Quill runs inside a Flatpak sandbox.
pub fn in_flatpak() -> bool {
    Path::new(FLATPAK_INFO).exists()
}

/// The id to use given `FLATPAK_ID` and whether this is a Flatpak sandbox.
/// A missing or malformed id keeps [`DEFAULT`].
pub fn choose(flatpak_id: Option<&str>, sandboxed: bool) -> &str {
    match flatpak_id.map(str::trim) {
        Some(id) if sandboxed && well_formed(id) => id,
        _ => DEFAULT,
    }
}

/// Flatpak's application id rules: at least three dot-separated elements of
/// ASCII letters, digits, `_` and `-`, none empty or starting with a digit,
/// at most 255 bytes.
pub fn well_formed(id: &str) -> bool {
    id.len() <= 255
        && id.split('.').count() >= 3
        && id.split('.').all(|element| {
            element
                .chars()
                .next()
                .is_some_and(|first| !first.is_ascii_digit())
                && element
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
}

/// The command a terminal or another app outside the sandbox runs to start
/// this Flatpak: the launcher Flatpak exports for it
/// (`<installation>/exports/bin/<id>`), found from `app-path` in
/// `/.flatpak-info`. `None` outside Flatpak.
pub fn flatpak_launcher() -> Option<PathBuf> {
    if !in_flatpak() {
        return None;
    }
    let id = get();
    let info = std::fs::read_to_string(FLATPAK_INFO).unwrap_or_default();
    Some(exported_launcher(&info, id).unwrap_or_else(|| PathBuf::from(id)))
}

/// `<installation>/exports/bin/<id>` from a `/.flatpak-info` whose
/// `[Instance]` `app-path` is `<installation>/app/<id>/<arch>/<branch>/<commit>/files`.
pub fn exported_launcher(info: &str, id: &str) -> Option<PathBuf> {
    let app_path = info
        .lines()
        .find_map(|line| line.trim().strip_prefix("app-path="))?
        .trim();
    let marker = format!("/app/{id}/");
    let installation = app_path.find(&marker).and_then(|at| app_path.get(..at))?;
    if installation.is_empty() {
        return None;
    }
    Some(Path::new(installation).join("exports/bin").join(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flathub_id_is_used_only_inside_flatpak() {
        assert_eq!(
            choose(Some("io.github.shinycake.Quill"), true),
            "io.github.shinycake.Quill"
        );
        assert_eq!(choose(Some("io.github.shinycake.Quill"), false), DEFAULT);
        assert_eq!(choose(None, true), DEFAULT);
    }

    #[test]
    fn malformed_ids_keep_the_default() {
        for bad in [
            "",
            "quill",
            "a.b",
            "io.github.1quill",
            "io..Quill",
            "io.github.Qu ill",
        ] {
            assert_eq!(choose(Some(bad), true), DEFAULT, "{bad:?}");
        }
        assert!(well_formed(DEFAULT));
        assert!(well_formed("io.github.shinycake.Quill"));
    }

    #[test]
    fn launcher_comes_from_the_installation_of_the_app_path() {
        let id = "io.github.shinycake.Quill";
        let system = "[Application]\nname=io.github.shinycake.Quill\n\n[Instance]\n\
            app-path=/var/lib/flatpak/app/io.github.shinycake.Quill/x86_64/stable/abc/files\n";
        assert_eq!(
            exported_launcher(system, id),
            Some(PathBuf::from(
                "/var/lib/flatpak/exports/bin/io.github.shinycake.Quill"
            ))
        );
        let user = "app-path=/home/u/.local/share/flatpak/app/io.github.shinycake.Quill/aarch64/master/f/files";
        assert_eq!(
            exported_launcher(user, id),
            Some(PathBuf::from(
                "/home/u/.local/share/flatpak/exports/bin/io.github.shinycake.Quill"
            ))
        );
        assert_eq!(exported_launcher("app-path=/app\n", id), None);
        assert_eq!(exported_launcher("", id), None);
    }
}
