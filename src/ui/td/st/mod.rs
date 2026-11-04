//! tdesktop style metrics at 100% scale, in logical px. Each block names
//! the `.style` file it comes from; the values are copied, not measured.
#![allow(dead_code)]

/// `lib_ui/ui/basic.style`.
pub(crate) mod basic {
    /// `fsize`: normal and semibold interface text.
    pub(crate) const FSIZE: f32 = 13.;
    pub(crate) const LINE_WIDTH: f32 = 1.;
    pub(crate) const ROUND_RADIUS_LARGE: f32 = 6.;
    pub(crate) const ROUND_RADIUS_SMALL: f32 = 3.;
}

/// `lib_ui/ui/widgets/widgets.style` (`defaultWindowTitle`).
pub(crate) mod window_title {
    pub(crate) const HEIGHT: f32 = 24.;
    /// `windowTitleButtonWidth`.
    pub(crate) const BUTTON_WIDTH: f32 = 36.;
}

// Each area of the main window keeps its metrics in its own file, named
// after the tdesktop style it ports.
pub(crate) mod chat;
pub(crate) mod compose;
pub(crate) mod dialogs;
pub(crate) mod top_bar;
pub(crate) mod window;

/// Text heights of `basic.style` fonts in Open Sans at 13px, as Qt rounds
/// them (`style::font->height`).
pub(crate) mod font {
    /// `normalFont` and `semiboldFont` height: one text line.
    pub(crate) const HEIGHT: f32 = 18.;
}
