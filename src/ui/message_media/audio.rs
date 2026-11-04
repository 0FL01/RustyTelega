//! Voice notes, music files, waveforms, seek bars and speech
//! transcription.

use super::scrubber::waveform_canvas;
use super::*;
use crate::ui::playback_state::SeekSite;
use quill::lang::{keys, tr, tr_with};

/// Round video note. tdesktop paints `history/view/media` round video as a
/// circle (JPEG thumb, duration, play). Quill uses the same ffmpeg frames as
/// `messageVideo`, clipped to a circle. Diameter on screen is fixed; schema
/// `length` is the sender's pixel size, shown when the file is not local yet.
/// MED2: the transcription row under a voice/video note. When TDLib has
/// delivered a `speechRecognitionResult` (via `updateMessageContent`), the
/// transcript shows; otherwise the row offers a real `recognizeSpeech`
/// request — pending/error states are shown honestly, never faked.
pub(in crate::ui) fn transcription_row(
    chat_id: ChatId,
    message_id: MessageId,
    transcription: &Option<SpeechRecognition>,
    accent: Hsla,
    cx: &mut Context<QuillApp>,
) -> AnyElement {
    let row = message_id.0 as u64;
    match transcription {
        None => div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(
                inline_link(("transcribe", row), tr(keys::Q_AV_TRANSCRIBE), accent)
                    .tooltip(|window, cx| {
                        gpui_kit::component::tooltip::Tooltip::new(tr(
                            keys::Q_AV_SEND_SPEECH_RECOGNITION_REQUEST_TO,
                        ))
                        .build(window, cx)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.request_transcription(chat_id, message_id, cx);
                    })),
            )
            .into_any_element(),
        Some(SpeechRecognition::Pending { partial_text }) => div()
            .text_xs()
            .opacity(0.7)
            .child(if partial_text.is_empty() {
                tr(keys::Q_AV_TRANSCRIBING).to_string()
            } else {
                tr_with(
                    keys::Q_AV_TRANSCRIBING_2,
                    &[("partial_text", &partial_text.to_string())],
                )
            })
            .into_any_element(),
        Some(SpeechRecognition::Text { text }) => div()
            .text_sm()
            .child(format!("“{text}”"))
            .into_any_element(),
        // A column, so a long error wraps inside the bubble instead of
        // running past its edge.
        Some(SpeechRecognition::Error { message }) => div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .items_start()
            .gap_1()
            .text_xs()
            .child(div().w_full().opacity(0.7).child(tr_with(
                keys::Q_AV_TRANSCRIPTION_FAILED,
                &[("message", &message.to_string())],
            )))
            .child(
                inline_link(("transcribe-retry", row), tr(keys::Q_AV_RETRY), accent).on_click(
                    cx.listener(move |this, _, _, cx| {
                        this.request_transcription(chat_id, message_id, cx);
                    }),
                ),
            )
            .into_any_element(),
    }
}

/// Voice waveform bars at a fixed pitch (the record bar); the first
/// `played` fraction is drawn solid, the rest faded. Bubbles use
/// [`waveform_canvas`], which fills its row.
pub(in crate::ui) fn waveform_row(
    row_key: u64,
    bars: &[u8],
    color: Hsla,
    played: f32,
) -> impl IntoElement {
    let mut row = div()
        .id(("waveform", row_key))
        .flex()
        .items_center()
        .gap(px(2.))
        .h(px(24.));
    let shown: Vec<u8> = if bars.is_empty() {
        vec![6, 10, 14, 8, 12]
    } else {
        bars.iter().copied().take(48).collect()
    };
    let count = shown.len().max(1) as f32;
    for (index, bar) in shown.into_iter().enumerate() {
        let h = 3.0 + f32::from(bar.min(31)) * 0.65;
        let lit = (index as f32 + 0.5) / count <= played;
        row = row.child(
            div()
                .id(("wave-bar", row_key * 64 + index as u64))
                .w(px(2.))
                .h(px(h))
                .rounded_full()
                .bg(if lit { color } else { color.opacity(0.35) }),
        );
    }
    row
}

/// `historyTranscribeSize` and `historyTranscribeRadius` from chat.style.
fn transcribe_button(
    chat_id: ChatId,
    message_id: MessageId,
    outgoing: bool,
    cx: &mut Context<QuillApp>,
) -> AnyElement {
    use crate::ui::td::st::chat;
    use crate::ui::td::{TdIcon, td, td_icon};
    use quill::app_theme::PaletteKey;
    let color = td(if outgoing {
        PaletteKey::MsgFileOutBg
    } else {
        PaletteKey::MsgFileInBg
    });
    div()
        .id(("transcribe", message_id.0 as u64))
        .w(px(chat::HISTORY_TRANSCRIBE_WIDTH))
        .h(px(chat::HISTORY_TRANSCRIBE_HEIGHT))
        .flex_none()
        .rounded(px(chat::HISTORY_TRANSCRIBE_RADIUS))
        .bg(Hsla::from(color).opacity(if outgoing { 0.18 } else { 0.12 }))
        .role(Role::Button)
        .aria_label(tr(keys::Q_AV_TRANSCRIBE))
        .tab_index(0)
        .cursor_pointer()
        .child(td_icon(TdIcon::VoiceToText, color))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.request_transcription(chat_id, message_id, cx);
        }))
        .into_any_element()
}

pub(in crate::ui) fn voice_note_row(
    chat_id: ChatId,
    message_id: MessageId,
    outgoing: bool,
    note: &quill::telegram::envelope::VoiceNoteContent,
    files: &quill::state::FileMap,
    downloading: &quill::state::DownloadIntents,
    seek: &SeekBarView,
    // The bubble's room (plain bubbles have no padding to take off the
    // row's width) and the time footer's width, 0 when hidden.
    room: crate::ui::history::bubble_width::BubbleRoom,
    info_width: i32,
    cx: &mut Context<QuillApp>,
) -> AnyElement {
    let file_id = note.file_id;
    let downloading_now = file_is_downloading(file_id, files, downloading);
    let bars = voice::waveform_bars_from_bytes(&note.waveform);
    let listened = note.is_listened;
    let note_duration = f64::from(note.duration);
    let active = seek.slider.is_some();
    let accent = bubble_accent(outgoing, cx);
    let wave_color = Hsla::from(crate::ui::td::td(
        match (outgoing, !note.is_listened && !outgoing) {
            (true, _) => quill::app_theme::PaletteKey::MsgWaveformOutInactive,
            (false, true) => quill::app_theme::PaletteKey::MsgWaveformInActive,
            (false, false) => quill::app_theme::PaletteKey::MsgWaveformInInactive,
        },
    ));
    // Phase 4.6: the active row shows elapsed / total (tdesktop-style).
    let total = format_voice_duration(note.duration);
    // `Document::composeStatusText` uses FormatDurationAndSizeText while
    // the file is not yet downloaded. Keep the duration alone if TDLib
    // has not supplied a size instead of inventing one.
    let file = files.get(&file_id.0);
    let size = file
        .map(|file| file.size.max(file.expected_size))
        .filter(|size| *size > 0);
    let idle_status = size.map_or_else(
        || total.clone(),
        |size| {
            format!(
                "{total}, {}",
                quill::telegram::envelope::format_size_text(size)
            )
        },
    );
    // Include both idle and playback labels when estimating width.
    let playback_status = format!("{total} / {total}");
    let widest_status = if idle_status.len() > playback_status.len() {
        &idle_status
    } else {
        &playback_status
    };
    let meta = if downloading_now && !seek.is_playing {
        tr(keys::Q_AV_DOWNLOADING_4).to_string()
    } else if active {
        // Hovering the waveform shows the time under the pointer (tdesktop).
        format!(
            "{} / {total}",
            format_voice_duration(seek.label_secs() as i32)
        )
    } else {
        idle_status.clone()
    };
    let unheard = !outgoing && !note.is_listened && !active;
    let (icon, label, ring) = if seek.is_playing {
        (
            gpui_kit::assets::IconName::Pause,
            tr(keys::MAC_MENU_PLAYER_PAUSE),
            None,
        )
    } else if downloading_now {
        (
            gpui_kit::assets::IconName::X,
            tr(keys::SR_MESSAGE_DOWNLOADING),
            Some(download_fraction(file_id, files)),
        )
    } else {
        (
            gpui_kit::assets::IconName::Play,
            tr(keys::Q_AV_PLAY_VOICE_MESSAGE),
            None,
        )
    };
    let row_key = message_id.0 as u64;
    // Telegram Desktop: an unplayed incoming note shows its whole
    // waveform in the active color; otherwise the played part does.
    let progress = if unheard { 1. } else { seek.fraction() as f32 };
    // The row is as wide as a full waveform (`Document::countOptimalSize`
    // for a voice note); the status label is the widest it gets.
    let status_width = crate::ui::history::bubble_width::longest_line_width(
        cx,
        &widest_status,
        px(crate::ui::td::st::chat::info::FONT),
        FontWeight::NORMAL,
        0,
    );
    let width = crate::ui::history::bubble_width::voice_row_width(status_width, info_width, room);
    // tdesktop `Document::draw` for a voice note (`msgFileLayout`): the
    // play button, the waveform with the transcribe button after it, and the
    // status under the waveform, laid out over the bubble's whole width.
    use crate::ui::td::st::{chat::file, font};
    let full = width + px(2. * crate::ui::td::st::chat::MSG_PADDING_X);
    let status_color = crate::ui::td::td(if outgoing {
        quill::app_theme::PaletteKey::MsgOutDateFg
    } else {
        quill::app_theme::PaletteKey::MsgInDateFg
    });
    let wave_width = full
        - px(file::NAME_LEFT + file::PADDING_RIGHT)
        - px(crate::ui::td::st::chat::HISTORY_TRANSCRIBE_WIDTH
            + crate::ui::td::st::chat::HISTORY_TRANSCRIBE_SKIP);
    let block = div()
        .relative()
        .w(full)
        .h(px(file::HEIGHT))
        .child(
            action_disc(("voice-play", row_key), outgoing, icon, ring, label, cx)
                .absolute()
                .left(px(file::PADDING_LEFT))
                .top(px(file::PADDING_TOP))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.press_media(file_id, cx, |this, cx| {
                        this.toggle_voice_playback(
                            chat_id,
                            message_id,
                            file_id,
                            listened,
                            note_duration,
                            cx,
                        );
                    });
                })),
        )
        .child(
            div()
                .absolute()
                .left(px(file::NAME_LEFT))
                .top(px(file::PADDING_TOP))
                .w(wave_width)
                .child(match &seek.slider {
                    Some(slider) => AudioScrubber::new(
                        ("voice-scrubber", row_key),
                        slider,
                        ScrubberFace::Waveform(bars),
                        accent,
                        QuillApp::seek_hover_handler(SeekSite::Row, cx),
                    )
                    .progress(progress)
                    .hover(seek.hover)
                    .dragging(seek.dragging)
                    .into_any_element(),
                    None => waveform_canvas(bars, wave_color, 1.).into_any_element(),
                }),
        )
        .child(
            div()
                .absolute()
                .right(px(file::PADDING_RIGHT))
                .top(px(file::PADDING_TOP))
                .child(transcribe_button(chat_id, message_id, outgoing, cx)),
        )
        .child(
            div()
                .absolute()
                .left(px(file::NAME_LEFT))
                .top(px(file::STATUS_TOP))
                .flex()
                .items_center()
                .gap_1()
                .text_size(px(crate::ui::td::st::chat::info::FONT))
                .line_height(px(font::HEIGHT))
                .text_color(status_color)
                .child(meta)
                // Unheard incoming note: a small accent dot.
                .when(unheard, |this| {
                    this.child(div().size(px(6.)).rounded_full().bg(accent))
                }),
        );
    div()
        .id(("voice-note", row_key))
        .flex()
        .flex_col()
        .w(full)
        .mx(px(-crate::ui::td::st::chat::MSG_PADDING_X))
        .child(block)
        // MED1: speed + mute on the active row.
        .when(active, |this| {
            this.child(row_playback_controls(row_key, "voice", seek, accent, cx))
        })
        // MED2: transcription (schema 1.8.67 `speechRecognitionResult`
        // on `voiceNote`).
        .when(note.transcription.is_some(), |this| {
            this.child(transcription_row(
                chat_id,
                message_id,
                &note.transcription,
                accent,
                cx,
            ))
        })
        .into_any_element()
}

pub(in crate::ui) fn audio_row(
    message_id: MessageId,
    outgoing: bool,
    audio: &quill::telegram::envelope::AudioContent,
    files: &quill::state::FileMap,
    downloading: &quill::state::DownloadIntents,
    media_roots: &[PathBuf],
    seek: &SeekBarView,
    // The bubble's room: no bubble padding to leave out in the plain look.
    room: crate::ui::history::bubble_width::BubbleRoom,
    cx: &mut Context<QuillApp>,
) -> AnyElement {
    let file_id = audio.file_id;
    let downloading_now = file_is_downloading(file_id, files, downloading);
    let audio_duration = f64::from(audio.duration);
    let active = seek.slider.is_some();
    let accent = bubble_accent(outgoing, cx);
    let title = if !audio.title.is_empty() {
        audio.title.clone()
    } else if !audio.file_name.is_empty() {
        audio.file_name.clone()
    } else {
        tr(keys::Q_AV_AUDIO).to_string()
    };
    // Phase 4.6: the active row shows elapsed / total (tdesktop-style).
    let total = voice::format_voice_duration(audio.duration);
    let duration_label = if active {
        format!(
            "{} / {total}",
            voice::format_voice_duration(seek.label_secs() as i32)
        )
    } else {
        total
    };
    let meta = if downloading_now && !seek.is_playing {
        tr(keys::Q_AV_DOWNLOADING_4).to_string()
    } else if audio.performer.is_empty() {
        duration_label
    } else {
        format!("{} · {duration_label}", audio.performer)
    };
    let (icon, label, ring) = if seek.is_playing {
        (
            gpui_kit::assets::IconName::Pause,
            tr(keys::MAC_MENU_PLAYER_PAUSE),
            None,
        )
    } else if downloading_now {
        (
            gpui_kit::assets::IconName::X,
            tr(keys::SR_MESSAGE_DOWNLOADING),
            Some(download_fraction(file_id, files)),
        )
    } else {
        (gpui_kit::assets::IconName::Play, tr(keys::Q_AV_PLAY), None)
    };
    let row_key = message_id.0 as u64;
    let cover_id = audio.cover_file_id().unwrap_or(FileId(0));
    let cover = files
        .get(&cover_id.0)
        .and_then(|file| file.usable_path())
        .and_then(|path| sandboxed_display_path(path, media_roots));
    let play = cx.listener(move |this, _, _, cx| {
        if let Some(chat_id) = this.open_chat_id() {
            this.press_media(file_id, cx, |this, cx| {
                this.toggle_audio_playback(chat_id, message_id, file_id, audio_duration, cx);
            });
        }
    });
    // Album art, when there is one, carries the play glyph on a scrim;
    // otherwise the plain accent disc.
    let disc = match cover {
        Some(path) => div()
            .id(("audio-play", row_key))
            .relative()
            .size(px(44.))
            .flex_none()
            .rounded_md()
            .overflow_hidden()
            .role(gpui_kit::Role::Button)
            .aria_label(label)
            .tab_index(0)
            .cursor_pointer()
            .child(
                img(crate::ui::image_budget::sized_media(
                    &path,
                    (px(44.), px(44.)),
                    None,
                    crate::ui::image_budget::Fit::Cover,
                ))
                .id(("audio-cover", row_key))
                .size_full()
                .object_fit(ObjectFit::Cover),
            )
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .bg(gpui_kit::black().opacity(0.35))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).size(px(20.)).text_color(gpui_kit::white())),
            )
            .on_click(play),
        None => {
            action_disc(("audio-play", row_key), outgoing, icon, ring, label, cx).on_click(play)
        }
    };
    // tdesktop `Document::draw` for a named file (`msgFileLayout`): the
    // button, the name in semibold cut in the middle, the status under it,
    // over the bubble's whole width (`msgFileMinWidth` to `msgMaxWidth`).
    use crate::ui::td::st::{chat::file, font};
    let (min_width, max_width) = crate::ui::history::bubble_width::file_row_bounds(room);
    let chrome = px(2. * crate::ui::td::st::chat::MSG_PADDING_X);
    let name_size = px(crate::ui::td::st::basic::FSIZE);
    let name_width = crate::ui::history::bubble_width::longest_line_width(
        cx,
        &title,
        name_size,
        FontWeight::SEMIBOLD,
        0,
    ) as f32;
    let full = px(file::NAME_LEFT + name_width + file::PADDING_RIGHT)
        .clamp(min_width + chrome, max_width + chrome);
    let name_room = f32::from(full) - file::NAME_LEFT - file::PADDING_RIGHT;
    let name_color = crate::ui::td::td(if outgoing {
        quill::app_theme::PaletteKey::HistoryTextOutFg
    } else {
        quill::app_theme::PaletteKey::HistoryTextInFg
    });
    let status_color = crate::ui::td::td(if outgoing {
        quill::app_theme::PaletteKey::MsgOutDateFg
    } else {
        quill::app_theme::PaletteKey::MsgInDateFg
    });
    let name = match crate::ui::history::text_measure::middle_elision(
        cx,
        &title,
        name_size,
        FontWeight::SEMIBOLD,
        name_room,
    ) {
        None => div().child(title).into_any_element(),
        Some(cut) => {
            let gap = name_room - cut.start_width - cut.end_width;
            div()
                .relative()
                .w(px(name_room))
                .h(px(font::HEIGHT))
                .child(div().absolute().left_0().child(cut.start))
                .child(
                    div()
                        .absolute()
                        .left(px(cut.start_width + (gap - cut.ellipsis_width) / 2.))
                        .child("..."),
                )
                .child(div().absolute().right_0().child(cut.end))
                .into_any_element()
        }
    };
    let block = div()
        .relative()
        .w(full)
        .h(px(file::HEIGHT))
        .text_size(name_size)
        .line_height(px(font::HEIGHT))
        .child(
            disc.absolute()
                .left(px(file::PADDING_LEFT))
                .top(px(file::PADDING_TOP)),
        )
        .child(
            div()
                .absolute()
                .left(px(file::NAME_LEFT))
                .top(px(file::NAME_TOP))
                .w(px(name_room))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(name_color)
                .child(name),
        )
        .child(
            div()
                .absolute()
                .left(px(file::NAME_LEFT))
                .top(px(file::STATUS_TOP))
                .w(px(name_room))
                .truncate()
                .text_color(status_color)
                .child(meta),
        );
    div()
        .id(("audio", row_key))
        .flex()
        .flex_col()
        .w(full)
        .mx(px(-crate::ui::td::st::chat::MSG_PADDING_X))
        .child(block)
        .when_some(seek.slider.as_ref(), |this, slider| {
            this.child(
                AudioScrubber::new(
                    ("audio-scrubber", row_key),
                    slider,
                    ScrubberFace::Line,
                    accent,
                    QuillApp::seek_hover_handler(SeekSite::Row, cx),
                )
                .progress(seek.fraction() as f32)
                .hover(seek.hover)
                .dragging(seek.dragging),
            )
            // MED1: speed + mute on the active row.
            .child(row_playback_controls(row_key, "audio", seek, accent, cx))
        })
        .into_any_element()
}
