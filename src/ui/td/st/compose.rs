//! `chat_helpers/chat_helpers.style` and `ui/chat/chat.style` metrics.
pub(crate) const HISTORY_REPLY_HEIGHT: f32 = 49.;
pub(crate) const MSG_REPLY_PADDING_TOP: f32 = 6.;
pub(crate) const MSG_REPLY_BAR_SKIP: f32 = 10.;
pub(crate) const MSG_REPLY_BAR_WIDTH: f32 = 2.;
pub(crate) const MSG_REPLY_BAR_HEIGHT: f32 = 36.;
pub(crate) const HISTORY_ATTACH_WIDTH: f32 = 44.;
pub(crate) const HISTORY_ATTACH_HEIGHT: f32 = 46.;
pub(crate) const HISTORY_COMPOSE_FIELD_HEIGHT_MIN: f32 = 36.;
pub(crate) const HISTORY_SEND_RIGHT: f32 = 2.;
pub(crate) const HISTORY_SEND_PADDING: f32 = 9.;

/// The message field's document margin (`setDocumentMargin(4.)` in
/// `InitMessageField`).
pub(crate) const HISTORY_FIELD_DOCUMENT_MARGIN: f32 = 4.;
/// `historyComposeField.placeholderMargins.left`.
pub(crate) const HISTORY_FIELD_PLACEHOLDER_LEFT: f32 = 2.;

/// `historySendSize.height`.
pub(crate) const HISTORY_SEND_HEIGHT: f32 = 46.;
/// The message field's minimum height: `InitMessageFieldGeometry` sets
/// `historySendSize.height - 2 * historySendPadding`, not `heightMin`.
pub(crate) const HISTORY_FIELD_MIN_HEIGHT: f32 = HISTORY_SEND_HEIGHT - 2. * HISTORY_SEND_PADDING;

/// The composer field's text padding: the document margin on top and
/// right, plus the placeholder margin on the left (tdesktop draws typed
/// text two pixels further left, but GPUI pads both alike), and the rest of
/// the minimum height below one line.
pub(crate) fn history_field_padding() -> gpui_kit::Edges<gpui_kit::Pixels> {
    use gpui_kit::px;
    gpui_kit::Edges {
        top: px(HISTORY_FIELD_DOCUMENT_MARGIN),
        right: px(HISTORY_FIELD_DOCUMENT_MARGIN),
        bottom: px(HISTORY_FIELD_MIN_HEIGHT - HISTORY_FIELD_DOCUMENT_MARGIN - super::font::HEIGHT),
        left: px(HISTORY_FIELD_DOCUMENT_MARGIN + HISTORY_FIELD_PLACEHOLDER_LEFT),
    }
}
