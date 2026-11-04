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

/// Telegram Desktop's window controls (`defaultWindowTitle`): its masks in
/// `titleButtonFg`, `titleButtonFgOver` on `titleButtonBgActiveOver` when
/// hovered, and the close button white on `titleButtonCloseBgOver`.
pub(crate) fn install_window_controls(cx: &mut gpui_kit::App) {
    use gpui_kit::component::{WindowControlKind as Kind, WindowControlStyle};
    use gpui_kit::*;
    use quill::app_theme::PaletteKey as K;
    use std::rc::Rc;
    cx.set_global(WindowControlStyle {
        icon: Rc::new(|kind, hovered| {
            let icon = match kind {
                Kind::Minimize => TdIcon::TitleMinimize,
                Kind::Maximize => TdIcon::TitleMaximize,
                Kind::Restore => TdIcon::TitleRestore,
                Kind::Close => TdIcon::TitleClose,
            };
            let color = match (kind, hovered) {
                (_, false) => K::TitleButtonFg,
                (Kind::Close, true) => K::TitleButtonCloseFgOver,
                (_, true) => K::TitleButtonFgOver,
            };
            td_icon(icon, td(color)).into_any_element()
        }),
        hover_bg: Rc::new(|kind| {
            td(if kind == Kind::Close {
                K::TitleButtonCloseBgOver
            } else {
                K::TitleButtonBgActiveOver
            })
            .into()
        }),
    });
}
