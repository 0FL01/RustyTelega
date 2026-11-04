//! Message history metrics from `SourceFiles/ui/chat/chat.style`, at 100%.

/// `msgPadding`: left and right inset.
pub(crate) const MSG_PADDING_X: f32 = 11.;
/// `msgPadding`: top and bottom inset.
pub(crate) const MSG_PADDING_Y: f32 = 8.;
/// `msgMargin`: top of a message that starts a group of its sender's.
pub(crate) const MSG_MARGIN_TOP: f32 = 6.;
/// `msgMargin`: bottom of every message.
pub(crate) const MSG_MARGIN_BOTTOM: f32 = 2.;
/// `historyPhotoLeft`: the sender photo's left, from the history's left.
pub(crate) const HISTORY_PHOTO_LEFT: f32 = 14.;
/// `msgMargin.left + msgPhotoSkip`: the bubble's left, from the history's
/// left, when a sender photo column is shown.
pub(crate) const MSG_PHOTO_BUBBLE_LEFT: f32 = 16. + 40.;
/// `historyPaddingBottom`.
pub(crate) const HISTORY_PADDING_BOTTOM: f32 = 8.;
/// `msgPhotoSize`.
pub(crate) const MSG_PHOTO_SIZE: f32 = 33.;
/// `bubbleRadiusLarge`.
pub(crate) const BUBBLE_RADIUS_LARGE: f32 = 16.;
/// `bubbleRadiusSmall` aliases `roundRadiusLarge` in lib_ui's basic.style.
pub(crate) const BUBBLE_RADIUS_SMALL: f32 = 6.;
/// `historyTranscribeSize`.
pub(crate) const HISTORY_TRANSCRIBE_WIDTH: f32 = 28.;
pub(crate) const HISTORY_TRANSCRIBE_HEIGHT: f32 = 22.;
/// `historyTranscribeRadius`.
pub(crate) const HISTORY_TRANSCRIBE_RADIUS: f32 = 4.;
/// `historyTranscribeSkip`.
pub(crate) const HISTORY_TRANSCRIBE_SKIP: f32 = 10.;

/// `mediaInBubbleSkip`: between media and the parts above or below it.
pub(crate) const MEDIA_IN_BUBBLE_SKIP: f32 = 5.;

/// `msgFileLayout`: a file or voice row without a thumbnail.
pub(crate) mod file {
    pub(crate) const PADDING_LEFT: f32 = 12.;
    pub(crate) const PADDING_TOP: f32 = 8.;
    pub(crate) const PADDING_RIGHT: f32 = 10.;
    pub(crate) const PADDING_BOTTOM: f32 = 8.;
    pub(crate) const NAME_TOP: f32 = 12.;
    pub(crate) const STATUS_TOP: f32 = 34.;
    pub(crate) const THUMB_SIZE: f32 = 44.;
    pub(crate) const THUMB_SKIP: f32 = 11.;
    /// `msgFileTopMinus`: how much a file row under other parts moves up.
    pub(crate) const TOP_MINUS: f32 = 6.;
    /// The row's height on top of a bubble.
    pub(crate) const HEIGHT: f32 = PADDING_TOP + THUMB_SIZE + PADDING_BOTTOM;
    /// Where the name or waveform starts.
    pub(crate) const NAME_LEFT: f32 = PADDING_LEFT + THUMB_SIZE + THUMB_SKIP;
}

/// The reply block inside a bubble (`history_view_reply.cpp`).
pub(crate) mod reply {
    /// `historyReplyTop` and `historyReplyBottom`.
    pub(crate) const TOP: f32 = 2.;
    pub(crate) const BOTTOM: f32 = 2.;
    /// `historyReplyPadding`.
    pub(crate) const PADDING_LEFT: f32 = 11.;
    pub(crate) const PADDING_TOP: f32 = 2.;
    pub(crate) const PADDING_RIGHT: f32 = 6.;
    pub(crate) const PADDING_BOTTOM: f32 = 2.;
    /// `historyReplyPreview` and `historyReplyPreviewMargin`.
    pub(crate) const PREVIEW: f32 = 32.;
    pub(crate) const PREVIEW_LEFT: f32 = 7.;
    pub(crate) const PREVIEW_TOP: f32 = 4.;
    pub(crate) const PREVIEW_RIGHT: f32 = 4.;
    /// `historyQuoteStyle`: the bar and the rounding.
    pub(crate) const OUTLINE: f32 = 3.;
    pub(crate) const RADIUS: f32 = 5.;
    /// `kDefaultBgOpacity` (`chat_style.h`): the block's fill over the
    /// bubble, in the bar's colour.
    pub(crate) const BG_OPACITY: f32 = 0.12;
    /// `kDefaultOutline1Opacity`: the bar's.
    pub(crate) const OUTLINE_OPACITY: f32 = 0.9;
    /// The name and text move right by this much past a preview.
    pub(crate) const PREVIEW_SKIP: f32 =
        OUTLINE + PREVIEW_LEFT + PREVIEW + PREVIEW_RIGHT - PADDING_LEFT;
}

/// The time and send state in a bubble (`BottomInfo`, `Message::drawInfo`).
pub(crate) mod info {
    /// `msgDateFont`.
    pub(crate) const FONT: f32 = 13.;
    /// `msgPadding.right - msgDateDelta.x`: the info's right, from the
    /// bubble's right.
    pub(crate) const RIGHT: f32 = 11. - 2.;
    /// `msgPadding.bottom - msgDateDelta.y`: its bottom, from the bubble's.
    pub(crate) const BOTTOM: f32 = 8. - 5.;
    /// `msgDateDelta.y`.
    pub(crate) const DATE_DELTA_Y: f32 = 5.;
    /// `historySendStateSpace`.
    pub(crate) const SEND_STATE_SPACE: f32 = 24.;
    /// `historySendStatePosition` plus the check icons' own offset
    /// (`historyReceivedIcon` `point(2px, 4px)`): the icon's left from the
    /// info's right, and its top from the line's top.
    pub(crate) const SEND_STATE_LEFT: f32 = -17. + 2.;
    pub(crate) const SEND_STATE_TOP: f32 = 18. - 19. + 4.;
    /// `msgDateImgDelta` and `msgDateImgPadding`: the time over media.
    pub(crate) const IMG_DELTA: f32 = 4.;
    pub(crate) const IMG_PADDING_X: f32 = 8.;
    pub(crate) const IMG_PADDING_Y: f32 = 2.;
}
