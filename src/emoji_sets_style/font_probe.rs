//! Is a color emoji font installed? Linux ships none by default, so minimal
//! installs draw tofu for every emoji with the "System" set. tdesktop avoids
//! this by shipping its own emoji art everywhere, and so does Quill now: the
//! bundled Telegram Desktop set is the default. The Noto fallback only
//! applies while the default set is the system font.
#![deny(clippy::indexing_slicing)]

use super::catalog::EmojiSetId;

/// Whether `fc-list :charset=1f600 color=true family` output names a font.
/// Any non-blank line means fontconfig knows a color font covering U+1F600.
pub fn fc_list_has_color_font(output: &str) -> bool {
    output.lines().any(|line| !line.trim().is_empty())
}

/// Ask fontconfig whether a color emoji font exists. `None` when the answer
/// is unknown (not Linux, `fc-list` missing or failed): the caller keeps the
/// system set then, since guessing wrong would replace working emoji.
/// Blocking; call off the UI thread.
#[cfg(target_os = "linux")]
pub fn color_emoji_font_installed() -> Option<bool> {
    // A fontconfig cache rebuild on a first run can take a while.
    let listing = crate::child_timeout::stdout_within(
        "fc-list",
        &[":charset=1f600", "color=true", "family"],
        std::time::Duration::from_secs(30),
    )?;
    Some(fc_list_has_color_font(&listing))
}

#[cfg(not(target_os = "linux"))]
pub fn color_emoji_font_installed() -> Option<bool> {
    None
}

/// The set to switch to at startup, or `None` to leave the choice alone.
/// An explicit saved choice always wins; otherwise only a definite "no
/// color font" answer picks the Noto pack.
pub fn fallback_set(has_saved_choice: bool, font_installed: Option<bool>) -> Option<EmojiSetId> {
    if has_saved_choice || !EmojiSetId::default().is_system() {
        return None;
    }
    (font_installed == Some(false)).then_some(EmojiSetId::Noto)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fc_list_output_is_read() {
        assert!(fc_list_has_color_font("Noto Color Emoji\n"));
        assert!(!fc_list_has_color_font(""));
        assert!(!fc_list_has_color_font("\n  \n"));
    }

    #[test]
    fn the_bundled_default_needs_no_fallback() {
        // The default draws everywhere, so no answer switches sets.
        assert_eq!(fallback_set(false, Some(false)), None);
        assert_eq!(fallback_set(false, Some(true)), None);
        assert_eq!(fallback_set(false, None), None);
        assert_eq!(fallback_set(true, Some(false)), None);
    }
}
