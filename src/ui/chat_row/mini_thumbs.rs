//! The small pictures of a chat row: the chat type icon before the name and
//! the minithumbnails before the preview (`dialogs_layout.cpp`,
//! `dialogs_message_view.cpp`).

use super::row_glyph;
use crate::ui::td::{self, TdIcon, st::dialogs as st};
use gpui_kit::assets::IconName;
use gpui_kit::*;
use quill::ids::ChatId;
use quill::telegram::envelope::MiniThumbnail;
use std::sync::Arc;

/// The chat type icon (`ChatTypeIcon`): its mask at the icon's own offset
/// from the name's left, the name `dialogsChatTypeSkip` after it.
pub(super) fn chat_type_icon(icon: IconName, muted_fg: Hsla) -> AnyElement {
    let (mask, left) = match icon {
        IconName::Megaphone => (TdIcon::DialogsChannel, 3.),
        IconName::Users => (TdIcon::DialogsChat, 1.),
        IconName::Bot => (TdIcon::DialogsBot, 1.),
        _ => {
            return div()
                .mr_1()
                .child(row_glyph(icon, muted_fg))
                .into_any_element();
        }
    };
    div()
        .flex_none()
        .pl(px(left))
        .mr(px(st::CHAT_TYPE_SKIP))
        .child(td::td_icon(
            mask,
            td::td(quill::app_theme::PaletteKey::DialogsChatIconFg),
        ))
        .into_any_element()
}

/// `dialogsMiniPreview` images, each `dialogsMiniPreviewSkip` after the
/// last, then `dialogsMiniPreviewRight` before the text; `None` without any.
pub(super) fn mini_previews(id: ChatId, thumbs: &[Arc<MiniThumbnail>]) -> Option<Div> {
    if thumbs.is_empty() {
        return None;
    }
    Some(
        div()
            .flex()
            .flex_none()
            .mr(px(st::DIALOGS_MINI_PREVIEW_RIGHT))
            .children(thumbs.iter().enumerate().map(|(n, mini)| {
                img(crate::ui::image_budget::sized_image(
                    crate::ui::image_budget::SizedSource::Mini(mini.clone()),
                    px(st::DIALOGS_MINI_PREVIEW_SIZE),
                ))
                .id(("preview-thumb", (id.0 as u64) * 4 + n as u64))
                .size(px(st::DIALOGS_MINI_PREVIEW_SIZE))
                .aspect_square()
                .flex_none()
                .mr(px(st::DIALOGS_MINI_PREVIEW_SKIP))
                .rounded(px(st::DIALOGS_MINI_PREVIEW_RADIUS))
                .object_fit(ObjectFit::Cover)
            })),
    )
}
