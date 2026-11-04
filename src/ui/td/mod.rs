//! Telegram Desktop's look, ported value for value: its fonts, its icon
//! masks, its palette keys and its style metrics (`*.style` files, the
//! `st::` namespace in tdesktop). The main window draws from here so it
//! matches tdesktop pixel for pixel (`docs/decisions/tdesktop-pixel-contract.md`).
#![deny(clippy::indexing_slicing)]

pub(crate) mod colors;
pub(crate) mod fonts;
pub(crate) mod icon;
pub(crate) mod st;

pub(crate) use colors::td;
pub(crate) use icon::{TdIcon, td_icon};

/// tdesktop's 1px `shadowFg` line under the top bar and the pinned bar
/// (`Ui::PlainShadow`): drawn over the top row of whatever comes next, so
/// the element above keeps its full height.
pub(crate) fn shadow_line() -> gpui_kit::Div {
    use gpui_kit::*;
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .h(px(st::basic::LINE_WIDTH))
        .bg(td(quill::app_theme::PaletteKey::ShadowFg))
}
