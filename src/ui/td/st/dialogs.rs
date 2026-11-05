//! Metrics from tdesktop `dialogs/dialogs.style`.
#![allow(dead_code)]
/// Search field height (`dialogsFilter`).
pub(crate) const DIALOGS_FILTER_HEIGHT: f32 = 35.;
/// Search field horizontal inset (`dialogsFilter`).
pub(crate) const DIALOGS_FILTER_SKIP: f32 = 11.;
/// Search field container height (`dialogsFilter`).
pub(crate) const DIALOGS_FILTER_CONTAINER_HEIGHT: f32 = 54.;
/// The search field's top in its area, `(areaHeight - height) / 2` in
/// integers (`Widget::updateControlsGeometry`).
pub(crate) fn dialogs_filter_top() -> f32 {
    ((DIALOGS_FILTER_CONTAINER_HEIGHT - DIALOGS_FILTER_HEIGHT) / 2.).floor()
}
/// `defaultDialogRow` height.
pub(crate) const DEFAULT_DIALOG_ROW_HEIGHT: f32 = 62.;
/// `dialogsMiniPreview` size.
pub(crate) const DIALOGS_MINI_PREVIEW_SIZE: f32 = 16.;

/// `dialogsFilter.borderRadius`.
pub(crate) const DIALOGS_FILTER_RADIUS: f32 = 18.;
/// `dialogsFilter.textMargins.left` plus placeholder left margin.
pub(crate) const DIALOGS_FILTER_TEXT_LEFT: f32 = 17.;
/// `dialogsFilter.textMargins.top`.
pub(crate) const DIALOGS_FILTER_TEXT_TOP: f32 = 8.;
/// `dialogsMiniPreviewRadius`.
pub(crate) const DIALOGS_MINI_PREVIEW_RADIUS: f32 = 2.;
/// `dialogsMiniPreviewSkip`: after each mini preview.
pub(crate) const DIALOGS_MINI_PREVIEW_SKIP: f32 = 2.;
/// `dialogsMiniPreviewRight`.
pub(crate) const DIALOGS_MINI_PREVIEW_RIGHT: f32 = 3.;
/// `defaultDialogRow.padding.left`.
pub(crate) const ROW_PADDING_LEFT: f32 = 10.;
/// `defaultDialogRow.nameLeft` and `textLeft`.
pub(crate) const NAME_LEFT: f32 = 68.;
/// `dialogsChatTypeSkip`: after the chat type icon.
pub(crate) const CHAT_TYPE_SKIP: f32 = 3.;
/// `defaultDialogRow.nameTop` and `textTop`.
pub(crate) const NAME_TOP: f32 = 10.;
pub(crate) const TEXT_TOP: f32 = 34.;
