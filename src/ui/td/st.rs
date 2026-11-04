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
