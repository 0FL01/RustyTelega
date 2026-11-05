//! Pieces of a history row that `session_history_row` decides on: the
//! countdown chips, when a picture shows its time, and how wide a plain text
//! bubble is.

use super::*;

/// The self-destruct badge and the auto-delete chip (Phase B4,
/// `message.auto_delete_in`, "🗑 59m left"), each counting down locally
/// against `unix_ms_now()` on the 1-second tick; TDLib deletes the row when
/// the timer fires.
pub(super) fn countdowns(
    message: &HistoryMessage,
) -> (Option<Stateful<Div>>, Option<Stateful<Div>>) {
    let now = unix_ms_now();
    (
        message
            .self_destruct_badge(now)
            .map(|label| countdown_chip("self-destruct-badge", message, label)),
        message
            .auto_delete_chip(now)
            .map(|label| countdown_chip("auto-delete-chip", message, label)),
    )
}

/// A countdown under the message (self-destruct, auto-delete), on the
/// 1-second tick. Outgoing bubbles are accent-filled: it inherits their text
/// color like the time footer instead of warning orange.
fn countdown_chip(id: &'static str, message: &HistoryMessage, label: String) -> Stateful<Div> {
    let outgoing = message.is_outgoing;
    div()
        .id((id, message.id.0 as u64))
        .mt_1()
        .text_xs()
        .map(|this| {
            if outgoing {
                this.opacity(0.8)
            } else {
                this.text_color(warning_text())
            }
        })
        .child(label)
}

/// How a picture's time pill sits and whether it waits for the pointer
/// (`Photo::needInfoDisplay`: sending, failed, the chat's last own message
/// or under the cursor; a GIF or video keeps it, `msgDateImgDelta` higher).
pub(super) fn media_info(
    message: &HistoryMessage,
    session: Option<&Session>,
) -> crate::ui::synthetic::MediaInfo {
    let content = effective_content(&message.content, message.ephemeral.as_ref());
    let last_own = message.is_outgoing
        && session
            .and_then(|s| s.chats.get(&message.chat_id.0))
            .and_then(|chat| chat.last_message.as_ref())
            .is_some_and(|last| last.id == message.id);
    crate::ui::synthetic::MediaInfo {
        video: matches!(
            content,
            MessageContent::Animation(_) | MessageContent::Video(_)
        ),
        on_hover: matches!(content, MessageContent::Photo(_))
            && !message.pending
            && !message.failed
            && !last_own,
    }
}

/// A plain text message's text width: the bubble fits its wrapped lines
/// (tdesktop `textRealWidth`) inside `outer` less the bubble's padding, with
/// `extra` (the time) after the last line. Formatted blocks and link cards
/// keep their own widths (`None`).
pub(super) fn plain_text_width(
    message: &HistoryMessage,
    font: Pixels,
    outer: f32,
    extra: f32,
    cx: &App,
) -> Option<Pixels> {
    let MessageContent::Text(text) =
        effective_content(&message.content, message.ephemeral.as_ref())
    else {
        return None;
    };
    if text.link_preview.is_some() || !text.entities.is_empty() {
        return None;
    }
    let chrome_x = 2. * crate::ui::td::st::chat::MSG_PADDING_X;
    Some(px(super::text_measure::wrapped_width(
        cx,
        &text.text,
        font,
        outer - chrome_x,
        extra,
    )
    .ceil()))
}

/// Bubble-less media carries its time itself, so reactions under it don't
/// push the pill off the picture.
pub(super) fn bubbleless_media(
    media: AnyElement,
    footer: Option<&FooterMeta>,
    info: crate::ui::synthetic::MediaInfo,
) -> AnyElement {
    let pill = footer.and_then(|meta| {
        let footer = message_footer_meta(meta)?;
        let mirrored = meta.mirrored();
        Some(crate::ui::synthetic::media_footer_pill(
            footer,
            Some(std::rc::Rc::new(move || message_footer_meta(&mirrored))),
            info,
        ))
    });
    div()
        .relative()
        .group(crate::ui::synthetic::MEDIA_INFO_GROUP)
        .child(media)
        .children(pill)
        .into_any_element()
}

/// What decides a picture's rounded corners besides the message.
pub(super) struct CornerFacts {
    pub(super) header: bool,
    pub(super) media_bleed: bool,
    pub(super) has_caption: bool,
    pub(super) has_sender: bool,
    pub(super) has_chips: bool,
}

/// The corners of the message's picture: bare for bubble-less media, else
/// the bubble's where the picture touches its edges.
pub(super) fn media_corners(
    message: &HistoryMessage,
    bubbleless: bool,
    look: BubbleLook,
    facts: CornerFacts,
    cx: &App,
) -> MediaCorners {
    if bubbleless {
        return MediaCorners::bare(
            cx,
            quill::chat_wide::has_right_layout(message.is_outgoing, look.wide),
            look.joined_above,
        );
    }
    let media_led = !facts.header
        && leads_with_media(effective_content(
            &message.content,
            message.ephemeral.as_ref(),
        ));
    let caption_above = facts.has_caption && caption_above_media(&message.content);
    let caption_below = facts.has_caption && !caption_above;
    let badges = message.self_destruct_badge(unix_ms_now()).is_some()
        || message.auto_delete_chip(unix_ms_now()).is_some();
    MediaCorners::in_bubble(
        cx,
        message.is_outgoing,
        look.plain,
        media_led || facts.media_bleed,
        !facts.has_sender && !caption_above && !facts.media_bleed,
        !caption_below && !badges && !facts.has_chips,
    )
}
