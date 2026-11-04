//! Open Sans, the face tdesktop draws in (`lib_ui/fonts`, SIL OFL 1.1),
//! embedded so every platform gets the same metrics.

use gpui_kit::App;
use std::borrow::Cow;

/// The family name the embedded files register.
pub(crate) const FAMILY: &str = "Open Sans";

/// tdesktop `fsize`: the interface text size, in px (`basic.style`).
pub(crate) const FSIZE: f32 = 13.;

/// Register the embedded faces with the text system and draw text with
/// greyscale antialiasing, as tdesktop does. Call once at startup, before
/// the first window.
pub(crate) fn register(cx: &mut App) {
    cx.set_text_rendering_mode(gpui_kit::TextRenderingMode::Grayscale);
    super::install_window_controls(cx);
    let faces: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../../../assets/fonts/OpenSans-Regular.ttf")),
        Cow::Borrowed(include_bytes!(
            "../../../assets/fonts/OpenSans-SemiBold.ttf"
        )),
        Cow::Borrowed(include_bytes!("../../../assets/fonts/OpenSans-Italic.ttf")),
        Cow::Borrowed(include_bytes!(
            "../../../assets/fonts/OpenSans-SemiBoldItalic.ttf"
        )),
    ];
    if let Err(error) = cx.text_system().add_fonts(faces) {
        log::warn!("could not register the embedded Open Sans faces: {error}");
    }
}
