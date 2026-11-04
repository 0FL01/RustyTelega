//! Palette keys as tdesktop paints them: the applied theme's colour, else
//! tdesktop's own dark (`night`) or light (`colors.palette`) value.

use crate::ui::chat_theme;
use gpui_kit::Rgba;
use quill::app_theme::{BuiltinTheme, PaletteKey};
use std::sync::OnceLock;

fn builtin(key: PaletteKey, dark: bool) -> u32 {
    static PALETTES: OnceLock<[quill::app_theme::Palette; 2]> = OnceLock::new();
    let [light, night] = PALETTES.get_or_init(|| {
        [
            BuiltinTheme::Classic.palette(),
            BuiltinTheme::Tinted.palette(),
        ]
    });
    if dark { night } else { light }.rgba(key)
}

/// The colour of `key` (tdesktop `st::<key>`).
pub(crate) fn td(key: PaletteKey) -> Rgba {
    let rgba = crate::ui::app_theme::installed(key)
        .unwrap_or_else(|| builtin(key, chat_theme::is_dark_palette()));
    crate::ui::chat_palette::recolor(rgba_of(rgba))
}

/// 0xRRGGBBAA as a GPUI colour.
pub(crate) fn rgba_of(rgba: u32) -> Rgba {
    Rgba {
        r: ((rgba >> 24) & 0xff) as f32 / 255.,
        g: ((rgba >> 16) & 0xff) as f32 / 255.,
        b: ((rgba >> 8) & 0xff) as f32 / 255.,
        a: (rgba & 0xff) as f32 / 255.,
    }
}
