//! Metrics from tdesktop `intro/intro.style` and the lib_ui styles the
//! sign-in screens use (`widgets.style`, `layers.style`, `basic.style`).
#![allow(dead_code)]

/// `introCoverHeight`.
pub(crate) const COVER_HEIGHT: f32 = 208.;
/// `introCoverMaxWidth`.
pub(crate) const COVER_MAX_WIDTH: f32 = 880.;
/// `introCoverIconsMinSkip`.
pub(crate) const COVER_ICONS_MIN_SKIP: f32 = 120.;
/// `introCoverIconLeft`.
pub(crate) const COVER_ICON_LEFT: f32 = 50.;
/// `introCoverIconTop`.
pub(crate) const COVER_ICON_TOP: f32 = 46.;
/// `introCoverTitleTop`.
pub(crate) const COVER_TITLE_TOP: f32 = 136.;
/// `introCoverDescriptionTop`.
pub(crate) const COVER_DESCRIPTION_TOP: f32 = 174.;
/// `introCoverTitle.style.font`: 22px semibold.
pub(crate) const COVER_TITLE_FONT: f32 = 22.;
/// `introCoverDescription.style`: 15px on 24px lines.
pub(crate) const COVER_DESCRIPTION_FONT: f32 = 15.;
pub(crate) const COVER_DESCRIPTION_LINE: f32 = 24.;

/// `introSettingsSkip`.
pub(crate) const SETTINGS_SKIP: f32 = 10.;
pub(crate) const SETTINGS_WIDTH: f32 = 392.;
pub(crate) const SETTINGS_TOP_BAR: f32 = 56.;
pub(crate) const SETTINGS_ROW: f32 = 40.;
pub(crate) const SETTINGS_RADIUS: f32 = 8.;
pub(crate) const SETTINGS_TEXT_LEFT: f32 = 22.;
pub(crate) const SETTINGS_TEXT_TOP: f32 = 10.;
pub(crate) const SETTINGS_LINE: f32 = 20.;
pub(crate) const SETTINGS_FONT: f32 = 14.;
pub(crate) const SETTINGS_HEIGHT: f32 = SETTINGS_TOP_BAR + 637. + 38. + SETTINGS_RADIUS;
pub(crate) const SETTINGS_DIVIDER: f32 = 8.;
/// `introPhotoTop`.
pub(crate) const PHOTO_TOP: f32 = 10.;
pub(crate) const PHOTO_SIZE: f32 = 72.;

/// `introTitle.style.font`: 17px semibold.
pub(crate) const TITLE_FONT: f32 = 17.;
/// `introTitleTop`.
pub(crate) const TITLE_TOP: f32 = 1.;
/// `introDescription`: 300px wide, `fsize` on 20px lines.
pub(crate) const DESCRIPTION_WIDTH: f32 = 300.;
pub(crate) const DESCRIPTION_LINE: f32 = 20.;
/// `introDescriptionTop`.
pub(crate) const DESCRIPTION_TOP: f32 = 34.;

/// `introPlaneWidth`, `introPlaneHeight`.
pub(crate) const PLANE_WIDTH: f32 = 48.;
pub(crate) const PLANE_HEIGHT: f32 = 38.;
/// `introHeight`.
pub(crate) const HEIGHT: f32 = 406.;
/// `introStepTopMin`.
pub(crate) const STEP_TOP_MIN: f32 = 76.;
/// `introStepWidth`.
pub(crate) const STEP_WIDTH: f32 = 380.;
/// `introNextTop`.
pub(crate) const NEXT_TOP: f32 = 266.;
/// `introNextSlide`.
pub(crate) const NEXT_SLIDE: f32 = 200.;
/// `introStepHeight`.
pub(crate) const STEP_HEIGHT: f32 = 384.;
/// `introContentTopAdd`.
pub(crate) const CONTENT_TOP_ADD: f32 = 30.;
/// `introStepHeightFull`.
pub(crate) const STEP_HEIGHT_FULL: f32 = 590.;
/// `introSlideDuration`, `introCoverDuration` (ms).
pub(crate) const SLIDE_DURATION: u64 = 200;
pub(crate) const COVER_DURATION: u64 = 200;

/// `introNextButton`: 300x42, radius 6, text 11px down, `boxFontSize`
/// semibold.
pub(crate) const NEXT_WIDTH: f32 = 300.;
pub(crate) const NEXT_HEIGHT: f32 = 42.;
pub(crate) const NEXT_RADIUS: f32 = 6.;
pub(crate) const NEXT_TEXT_TOP: f32 = 11.;
pub(crate) const NEXT_FONT: f32 = 14.;

/// `introStepFieldTop`.
pub(crate) const STEP_FIELD_TOP: f32 = 96.;
/// `introPhoneTop`.
pub(crate) const PHONE_TOP: f32 = 6.;
/// `introLinkTop`.
pub(crate) const LINK_TOP: f32 = 24.;
/// `introCountry`: `defaultInputField` 300 wide, at least 61 high,
/// text margins (3, 27, 3, 6), 16px text.
pub(crate) const FIELD_WIDTH: f32 = 300.;
pub(crate) const FIELD_HEIGHT: f32 = 61.;
pub(crate) const FIELD_MARGIN_LEFT: f32 = 3.;
pub(crate) const FIELD_MARGIN_TOP: f32 = 27.;
pub(crate) const FIELD_FONT: f32 = 16.;
pub(crate) const FIELD_LINE: f32 = 22.;
pub(crate) const FIELD_MARGIN_BOTTOM: f32 = 6.;
pub(crate) const FIELD_BORDER: f32 = 1.;
pub(crate) const FIELD_BORDER_ACTIVE: f32 = 2.;
/// `introCountryCode`: 64 wide.
pub(crate) const COUNTRY_CODE_WIDTH: f32 = 64.;
/// `introPhone`: 225 wide, text margins (12, 27, 12, 6).
pub(crate) const PHONE_WIDTH: f32 = 225.;
pub(crate) const PHONE_MARGIN_LEFT: f32 = 12.;
/// `introQrLoginLinkTop`.
pub(crate) const QR_LOGIN_LINK_TOP: f32 = 368.;
/// `introPasswordTop`, `introPasswordHintTop`.
pub(crate) const PASSWORD_TOP: f32 = 74.;
pub(crate) const PASSWORD_HINT_TOP: f32 = 151.;

/// `introCodeDigitFont`, `introCodeDigitHeight`,
/// `introCodeDigitBorderWidth`, `introCodeDigitSkip`.
pub(crate) const CODE_DIGIT_FONT: f32 = 20.;
pub(crate) const CODE_DIGIT_WIDTH: f32 = 40.;
pub(crate) const CODE_DIGIT_HEIGHT: f32 = 50.;
pub(crate) const CODE_DIGIT_BORDER: f32 = 4.;
pub(crate) const CODE_DIGIT_SKIP: f32 = 10.;
pub(crate) const CODE_DIGIT_RADIUS: f32 = 8.;
/// `CodeWidget::updateControlsGeometry`: field top + 3 * input border skip.
pub(crate) const CODE_TOP: f32 = STEP_FIELD_TOP + 18.;
/// EmailWidget's vertical title/description/icon layout at 300px.
pub(crate) const EMAIL_ICON_TOP: f32 = 23. + 2. + DESCRIPTION_LINE * 4.;
pub(crate) const EMAIL_ICON_SIZE: f32 = 66.;
pub(crate) const EMAIL_FIELD_TOP: f32 = EMAIL_ICON_TOP + EMAIL_ICON_SIZE;

/// `introResetBottom`, `introTermsBottom`.
pub(crate) const RESET_BOTTOM: f32 = 20.;
pub(crate) const TERMS_BOTTOM: f32 = 20.;

/// `introCountryIconPosition`.
pub(crate) const COUNTRY_ICON_LEFT: f32 = 8.;
pub(crate) const COUNTRY_ICON_TOP: f32 = 37.;

/// `introErrorTop`, `introErrorBelowLinkTop`.
pub(crate) const ERROR_TOP: f32 = 235.;
pub(crate) const ERROR_BELOW_LINK_TOP: f32 = 220.;

/// `introBackButton`: 56x56, a 40px ripple at (8, 8).
pub(crate) const BACK_SIZE: f32 = 56.;
pub(crate) const BACK_RIPPLE_LEFT: f32 = 8.;
pub(crate) const BACK_RIPPLE_SIZE: f32 = 40.;

/// `introQr*`.
pub(crate) const QR_TOP: f32 = -18.;
pub(crate) const QR_PIXEL: f32 = 50.;
pub(crate) const QR_MAX_SIZE: f32 = 180.;
/// `PrepareQrWidget`'s waiting code size.
pub(crate) const QR_USUAL_MODULES: usize = 41;
pub(crate) const QR_BACKGROUND_SKIP: f32 = 12.;
pub(crate) const QR_BACKGROUND_RADIUS: f32 = 8.;
pub(crate) const QR_LABELS_WIDTH: f32 = 292.;
pub(crate) const QR_TITLE_WIDTH: f32 = 320.;
pub(crate) const QR_TITLE_FONT: f32 = 20.;
pub(crate) const QR_ERROR_TOP: f32 = 336.;
pub(crate) const QR_TITLE_TOP: f32 = 196.;
pub(crate) const QR_STEPS_TOP: f32 = 232.;
/// `introQrStepMargins`: (20, 8, 0, 0).
pub(crate) const QR_STEP_MARGIN_LEFT: f32 = 20.;
pub(crate) const QR_STEP_MARGIN_TOP: f32 = 8.;
pub(crate) const QR_SKIP_TOP: f32 = 360.;
pub(crate) const QR_CENTER_SIZE: f32 = 44.;
/// `QrActiveColor`, independent of the chosen palette.
pub(crate) const QR_DISC_COLOR: u32 = 0x40a7e3;
/// `radialLine`: the waiting arc's pen.
pub(crate) const RADIAL_LINE: f32 = 3.;

/// `buttonRadius`: the 4px the title and description sit right of the
/// content's left edge (`Step::updateLabelsPosition`).
pub(crate) const BUTTON_RADIUS: f32 = 4.;
/// `windowMinHeight` (`window.style`).
pub(crate) const WINDOW_MIN_HEIGHT: f32 = 480.;

/// `defaultBoxButton`: as wide as its text plus 30, 34 high, text 7px down,
/// 14px semibold; the corners take `buttonRadius`.
pub(crate) const BOX_BUTTON_ADDED_WIDTH: f32 = 30.;
pub(crate) const BOX_BUTTON_HEIGHT: f32 = 34.;
pub(crate) const BOX_BUTTON_TEXT_TOP: f32 = 7.;
pub(crate) const BOX_BUTTON_FONT: f32 = 14.;

/// `slideDuration` (the next button showing or hiding) and
/// `fadeWrapDuration` (errors, the back button, the QR code), in ms.
pub(crate) const NEXT_SHOWN_DURATION: u64 = 240;
pub(crate) const FADE_DURATION: u64 = 200;
/// `shakeShift`, `shakeDuration`.
pub(crate) const SHAKE_SHIFT: f32 = 4.;
pub(crate) const SHAKE_DURATION: u64 = 300;
/// `defaultInputField.duration`: focus and placeholder.
pub(crate) const FIELD_DURATION: u64 = 150;
/// `universalDuration`: a digit appearing in the code field.
pub(crate) const UNIVERSAL_DURATION: u64 = 120;

/// `Step::contentLeft` / `Step::contentTop`, excluding window chrome.
pub(crate) fn content_left(width: f32) -> f32 {
    ((width - NEXT_WIDTH) / 2.).floor()
}

pub(crate) fn content_top(height: f32, cover: bool) -> f32 {
    let top = ((height - HEIGHT) / 2.).floor().max(STEP_TOP_MIN);
    if !cover {
        return top;
    }
    let full = top + NEXT_TOP + CONTENT_TOP_ADD;
    let added =
        1. - ((full - WINDOW_MIN_HEIGHT) / (STEP_HEIGHT_FULL - WINDOW_MIN_HEIGHT)).clamp(0., 1.);
    top + (added * CONTENT_TOP_ADD).round()
}

pub(crate) fn qr_side(modules: usize) -> f32 {
    let pixel = (QR_MAX_SIZE / modules as f32).floor().clamp(1., QR_PIXEL);
    modules as f32 * pixel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_geometry_at_reference_and_small_windows() {
        assert_eq!(qr_side(QR_USUAL_MODULES), 164.);
        assert_eq!(qr_side(QR_USUAL_MODULES) + 2. * QR_BACKGROUND_SKIP, 188.);
        for (width, height, left, top, cover_top) in [
            (1920., 1012., 810., 303., 303.),
            (800., 576., 250., 85., 115.),
            (380., 456., 40., 76., 106.),
        ] {
            assert_eq!(content_left(width), left);
            assert_eq!(content_top(height, false), top);
            assert_eq!(content_top(height, true), cover_top);
            assert!(left + NEXT_WIDTH <= width);
            assert!(cover_top + NEXT_TOP + NEXT_HEIGHT <= height);
        }
    }
}
