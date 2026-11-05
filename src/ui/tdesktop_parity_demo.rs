//! Offline Telegram Desktop parity capture fixture.
#![deny(clippy::indexing_slicing)]

use super::app::QuillApp;
use super::demo::{demo_file_json, demo_media_allowlist, seed_ready_chats_session};
use super::screenshot_demo::{DemoSpec, register_demos};
use base64::Engine as _;
use gpui_kit::{Context, Window};
use quill::diagnostics::{DiagnosticSink, MemorySink};
use quill::ids::ChatId;
use quill::state::Session;
use quill::telegram::client::copy_and_parse;
use serde_json::{Value, json};
use std::sync::{Arc, atomic::AtomicU64};

const CHAT: i64 = 201;
const ME: i64 = 200;
const FOLDER: i32 = 5;
const STEP: i64 = 1_048_576;
/// The custom emoji behind replies to Andrey (cut from the reference).
const PATTERN_EMOJI: i64 = 777_001;

register_demos![
    DemoSpec::ready(
        "ready-tdesktop-parity",
        seed,
        "screenshot demo — Telegram Desktop parity"
    )
    .setup(QuillApp::demo_tdesktop_parity)
];

impl QuillApp {
    fn demo_tdesktop_parity(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.appearance.folder_tabs_view = quill::folder_icons::FolderTabsView::Left;
        self.frame.sidebar_width = gpui_kit::px(454.);
        self.open_folder_tab(Some(FOLDER), cx);
        // Telegram Desktop gives the open chat's field the focus.
        self.composer
            .update(cx, |input, cx| input.focus(window, cx));
    }
}

fn file(id: i32, name: &str, size: i64) -> Value {
    let path = demo_media_allowlist().join(name);
    let mut file: Value = serde_json::from_str(&demo_file_json(id, &path.to_string_lossy(), true))
        .unwrap_or_default();
    if let Some(object) = file.as_object_mut() {
        object.insert("size".into(), json!(size));
        object.insert("expected_size".into(), json!(size));
    }
    file
}

fn photo(id: i32, name: &str, caption: &str) -> Value {
    photo_sized(id, name, caption, (1100, 740))
}

fn photo_sized(id: i32, name: &str, caption: &str, (width, height): (u32, u32)) -> Value {
    let donor_mini = std::path::Path::new(name)
        .is_absolute()
        .then(|| super::tdesktop_parity_donor::minithumbnail(std::path::Path::new(name)))
        .flatten();
    let mini = donor_mini.or_else(|| {
        image::open(demo_media_allowlist().join(name))
            .ok()
            .and_then(|image| {
                let image = image.thumbnail(40, 40);
                let mut bytes = std::io::Cursor::new(Vec::new());
                image.write_to(&mut bytes, image::ImageFormat::Jpeg).ok()?;
                Some(
                    json!({"@type":"minithumbnail", "width":image.width(), "height":image.height(),
            "data":base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())}),
                )
            })
    });
    json!({"@type":"messagePhoto", "photo":{"@type":"photo", "has_stickers":false,
        "minithumbnail":mini,"sizes":[{"@type":"photoSize","type":"m",
        "photo":file(id,name,24),"width":width,"height":height,"progressive_sizes":[]}]},
        "caption":{"@type":"formattedText","text":caption,"entities":[]},
        "show_caption_above_media":false,"has_spoiler":false,"is_secret":false})
}

fn text(s: &str) -> Value {
    json!({"@type":"messageText","text":{"@type":"formattedText","text":s,"entities":[]}})
}

fn avatar(id: i32, name: &str) -> Value {
    json!({"@type":"chatPhotoInfo","small":file(id,name,24),"big":null,
        "minithumbnail":null,"has_animation":false,"is_personal":false})
}

fn message(id: i64, sender: i64, date: i64, content: Value, reply: Option<i64>) -> Value {
    let mut m = json!({"id":id,"chat_id":CHAT,"sender_id":{"@type":"messageSenderUser","user_id":sender},
        "is_outgoing":sender == ME,"date":date,"content":content});
    if let Some(reply) = reply
        && let Some(object) = m.as_object_mut()
    {
        object.insert(
            "reply_to".into(),
            json!({"@type":"messageReplyToMessage","chat_id":CHAT,"message_id":reply,
            "quote":null,"checklist_task_id":0,"poll_option_id":"","origin":null,
            "origin_send_date":0,"content":null}),
        );
    }
    json!({"@type":"updateNewMessage","message":m})
}

/// The waveforms of the reference's two voice notes, read back from its
/// bars (`PaintWaveform` with the loudest sample at 31).
const WAVE_FIRST: [u8; 100] = [
    2, 8, 29, 15, 18, 22, 13, 24, 0, 8, 0, 22, 24, 13, 13, 29, 13, 11, 13, 29, 15, 13, 29, 6, 29,
    15, 6, 31, 27, 6, 22, 4, 27, 6, 15, 11, 20, 22, 24, 29, 15, 15, 15, 8, 13, 0, 20, 0, 27, 11, 6,
    15, 6, 29, 13, 24, 31, 15, 22, 22, 2, 2, 15, 22, 22, 29, 4, 13, 2, 27, 4, 22, 20, 11, 27, 8, 0,
    15, 2, 0, 2, 15, 8, 15, 11, 8, 0, 0, 15, 27, 27, 31, 31, 13, 27, 18, 8, 4, 0, 0,
];
const WAVE_SECOND: [u8; 100] = [
    0, 8, 15, 22, 18, 24, 2, 11, 22, 6, 2, 11, 18, 15, 11, 18, 31, 22, 15, 18, 22, 18, 0, 0, 31,
    31, 31, 11, 11, 11, 11, 8, 8, 31, 31, 31, 24, 22, 18, 24, 22, 18, 18, 15, 15, 15, 18, 11, 18,
    0, 6, 6, 0, 2, 22, 31, 24, 24, 31, 31, 27, 31, 31, 31, 18, 22, 15, 15, 22, 18, 8, 11, 15, 15,
    8, 8, 8, 6, 8, 8, 8, 6, 6, 0, 2, 15, 0, 0, 11, 31, 27, 18, 24, 11, 24, 18, 15, 11, 0, 0,
];

fn voice(id: i32, duration: i32, size: i64) -> Value {
    let wave = quill::voice::waveform_base64(match id {
        6504 => &WAVE_SECOND,
        _ => &WAVE_FIRST,
    });
    json!({"@type":"messageVoiceNote","voice_note":{"@type":"voiceNote","duration":duration,
        "waveform":wave,"mime_type":"audio/ogg","speech_recognition_result":null,
        "voice":file(id,"demo-voice.ogg",size)},
        "caption":{"@type":"formattedText","text":"","entities":[]},"is_listened":true})
}

fn seed(sink: Arc<MemorySink>) -> Session {
    // With `QUILL_PARITY_DONOR`, the pictures come from the reference.
    let donor = super::tdesktop_parity_donor::prepare();
    let pick = |path: Option<&std::path::PathBuf>, fallback: &str| -> String {
        path.map_or_else(
            || fallback.to_string(),
            |path| path.to_string_lossy().into_owned(),
        )
    };
    let me_avatar = pick(donor.as_ref().map(|d| &d.me_avatar), "showcase/av05.png");
    let lain_avatar = pick(donor.as_ref().map(|d| &d.lain_avatar), "showcase/av01.png");
    let reply_thumb = pick(donor.as_ref().map(|d| &d.reply_thumb), "showcase/p5.jpg");
    let photo_file = pick(donor.as_ref().map(|d| &d.photo), "showcase/p6.jpg");
    let photo_size = if donor.is_some() {
        super::tdesktop_parity_donor::PHOTO_SIZE
    } else {
        (1100, 740)
    };
    let gif_still = pick(donor.as_ref().map(|d| &d.gif), "demo-gif-1.png");
    // The reference shows one frame: the clip is that frame.
    let gif_clip = pick(donor.as_ref().map(|d| &d.gif_clip), "demo-gif.gif");
    let list_avatar = |n: usize, fallback: &str| {
        pick(donor.as_ref().and_then(|d| d.list_avatars.get(n)), fallback)
    };
    let row_mini =
        |n: usize, fallback: &str| pick(donor.as_ref().and_then(|d| d.row_minis.get(n)), fallback);
    let album_mini = |n: usize, fallback: &str| {
        pick(donor.as_ref().and_then(|d| d.album_minis.get(n)), fallback)
    };
    let mut session = seed_ready_chats_session(sink.clone());
    let seq = AtomicU64::new(session.last_seq);
    let dyn_sink: Arc<dyn DiagnosticSink> = sink;
    let mut updates = Vec::new();
    for id in [11, 12, 13] {
        updates.push(
            json!({"@type":"updateChatPosition","chat_id":id,"position":{
            "@type":"chatPosition","list":{"@type":"chatListMain"},"order":"0","is_pinned":false}}),
        );
    }
    let now = quill::local_time::now_unix();
    let local = quill::local_time::civil_local(now);
    let midnight =
        now - i64::from(local.hour) * 3600 - i64::from(local.minute) * 60 - now.rem_euclid(60);
    let at = |h: i64, m: i64| midnight + h * 3600 + m * 60;
    let today = at(22, 13);
    let monday = midnight - i64::from(local.weekday) * 86400 + 12 * 3600;
    let monday = if monday > now {
        monday - 7 * 86400
    } else {
        monday
    };
    let offset = i64::from(quill::local_time::utc_offset_at(now));
    let old_oct = 1_793_188_800 - offset; // 2026-10-28 12:00 UTC interpreted locally
    let old_sep = 1_790_683_200 - offset; // 2026-09-29 12:00
    let folders: Vec<_> = [("Humanoid","Groups"),("Bots","Bots"),("Work","Work"),
        ("News","Channels"),("AI News","Study"),("Info","Custom")].iter().enumerate().map(|(i,(name,icon))| json!({
        "@type":"chatFolderInfo","id":i+1,"name":{"@type":"chatFolderName","text":{
        "@type":"formattedText","text":name,"entities":[]}},"icon":{"@type":"chatFolderIcon","name":icon},
        "color_id":-1,"is_shareable":false})).collect();
    updates.push(json!({"@type":"updateChatFolders","chat_folders":folders,"main_chat_list_position":0,"are_tags_enabled":false}));
    for (id, first, last, picture, status) in [
        (
            ME,
            "Andrey",
            "Bash",
            me_avatar.as_str(),
            json!({"@type":"userStatusRecently"}),
        ),
        (
            CHAT,
            "Lain",
            "",
            lain_avatar.as_str(),
            json!({"@type":"userStatusOffline","was_online":now-60}),
        ),
    ] {
        let small = file(id as i32 + 6000, picture, 24);
        updates.push(json!({"@type":"updateFile","file":small}));
        // The reference's replies to Andrey show his background emoji.
        let background = if id == ME && donor.is_some() {
            PATTERN_EMOJI
        } else {
            0
        };
        updates.push(json!({"@type":"updateUser","user":{"@type":"user","id":id,"first_name":first,
            "last_name":last,"accent_color_id":0,"background_custom_emoji_id":background,
            "status":status,"profile_photo":{"@type":"profilePhoto",
            "id":id,"small":small,"big":null,"minithumbnail":null,"has_animation":false,"is_personal":false},
            "is_contact":true,"type":{"@type":"userTypeRegular"}}}));
    }
    updates.push(json!({"@type":"updateOption","name":"my_id","value":{"@type":"optionValueInteger","value":ME}}));
    updates.push(json!({"@type":"updateNewChat","chat":{"id":CHAT,"title":"Lain",
        "type":{"@type":"chatTypePrivate","user_id":CHAT},"photo":avatar(6201,&lain_avatar),"unread_count":0}}));
    let rows = [
        (
            301,
            "Сиолошная",
            today,
            "Пока в твиттере пытаются вычислить новые Astra 6.2 и обсуждают последние новости искусственного интеллекта",
            "showcase/av06.png",
            "tdesktop_parity/white.png",
        ),
        (
            302,
            "Denis Sexy IT 🤖",
            monday,
            "Я всё думал, как объяснить это странное чувство, когда технологии меняют привычный мир",
            "showcase/av07.png",
            "showcase/p2.jpg",
        ),
        (
            303,
            "epsilon correct",
            old_oct,
            "Если вам зашёл Gemini 4 Argon, попробуйте Gemini 4 и поделитесь своими впечатлениями",
            "showcase/av08.png",
            "showcase/p3.jpg",
        ),
        (
            304,
            "Tensor Banana",
            old_sep,
            "Strata и ninfer - 2 быстрых движка для LLM  Взамен привычных решений появились новые возможности",
            "showcase/av09.png",
            "showcase/p4.jpg",
        ),
    ];
    for (n, (id, title, date, caption, av, pic)) in rows.iter().enumerate() {
        let av = list_avatar(n, av);
        let pic = if *id == 304 {
            album_mini(2, pic)
        } else {
            row_mini(n, pic)
        };
        updates.push(
            json!({"@type":"updateNewChat","chat":{"id":id,"title":title,
            "type":{"@type":"chatTypeSupergroup","supergroup_id":id,"is_channel":true},
            "photo":avatar(6300+n as i32,&av),"unread_count":0}}),
        );
        updates.push(json!({"@type":"updateSupergroup","supergroup":{"@type":"supergroup","id":id,
            "is_channel":true,"status":{"@type":"chatMemberStatusMember"},"member_count":1000,
            "verification_status":{"@type":"verificationStatus","is_verified":*id==302,"is_scam":false,"is_fake":false}}}));
        updates.push(json!({"@type":"updateChatNotificationSettings","chat_id":id,"notification_settings":{
            "@type":"chatNotificationSettings","use_default_mute_for":false,"mute_for":2147483647}}));
        let order = (5000 - n * 10).to_string();
        if *id == 304 {
            for (index, name) in ["showcase/p4.jpg", "showcase/p5.jpg", "showcase/p6.jpg"]
                .iter()
                .enumerate()
            {
                let name = album_mini(index, name);
                updates.push(json!({"@type":"updateNewMessage","message":{
                    "id": (index as i64 + 1) * STEP,"chat_id":id,
                    "sender_id":{"@type":"messageSenderChat","chat_id":id},
                    "date":date,"is_channel_post":true,"media_album_id":"88001",
                    "content":photo(6600 + index as i32,&name,if index == 2 { caption } else { "" })}}));
            }
        }
        updates.push(json!({"@type":"updateChatLastMessage","chat_id":id,"last_message":{
            "id":if *id == 304 { 3*STEP } else { STEP },"chat_id":id,"sender_id":{"@type":"messageSenderChat","chat_id":id},
            "media_album_id":if *id == 304 { "88001" } else { "0" },
            "date":date,"is_outgoing":false,"is_channel_post":true,"content":photo(6400+n as i32,&pic,caption)},
            "positions":[{"@type":"chatPosition","list":{"@type":"chatListMain"},"order":order,"is_pinned":false},
            {"@type":"chatPosition","list":{"@type":"chatListFolder","chat_folder_id":FOLDER},"order":order,"is_pinned":false}]}));
    }
    let mut link = text("https://github.com/shinycake/quill");
    if let Some(object) = link.as_object_mut() {
        object.insert("link_preview".into(), json!({"@type":"linkPreview","url":"https://github.com/shinycake/quill",
        "display_url":"github.com/shinycake/quill","site_name":"GitHub","title":"quill",
        "description":{"@type":"formattedText","text":"","entities":[]},"author":"",
        "type":{"@type":"linkPreviewTypeArticle","photo":{"@type":"photo","sizes":[{"@type":"photoSize",
        "type":"m","photo":file(6500,&reply_thumb,24),"width":1100,"height":740}]}},
        "has_large_media":true,"show_large_media":true,"show_media_above_description":false,
        "skip_confirmation":true,"show_above_text":false,"instant_view_version":0}));
    }
    for id in (1..=4).rev() {
        updates.push(message(
            STEP - id,
            ME,
            at(8, 0),
            text("Earlier pinned message"),
            None,
        ));
        updates.push(json!({"@type":"updateMessageIsPinned","chat_id":CHAT,"message_id":STEP-id,"is_pinned":true}));
    }
    updates.push(message(STEP, ME, at(9, 0), link, None));
    updates.push(message(2 * STEP, ME, at(9, 5), voice(6501, 5, 10000), None));
    updates.push(message(
        3 * STEP,
        ME,
        at(9, 10),
        text("https://tam-net-lyudey-tam-tolko.ass-peak.de/"),
        None,
    ));
    updates.push(json!({"@type":"updateMessageIsPinned","chat_id":CHAT,"message_id":3*STEP,"is_pinned":true}));
    updates.push(message(
        4 * STEP,
        ME,
        at(10, 40),
        photo_sized(6502, &photo_file, "", photo_size),
        None,
    ));
    updates.push(message(5*STEP, CHAT, at(11,3), text("DHCP-сервер LAN включён, три LAN-порта объединены в мост.\nWAN настроен на DHCP, WAN6 — DHCPv6.\nWi-Fi выключен; постоянный flow offloading оставлен выключенным по умолчанию.\nСтарые пароли, наши разрешённые SSH-ключи, тестовые настройки и дополнительное лабораторное логирование удалены.\nСерверные SSH-ключи созданы заново.\nСобственные MAC, калибровки, загрузчики и разметка сохранены."), None));
    updates.push(message(
        6 * STEP,
        ME,
        at(11, 12),
        voice(6503, 78, 338750),
        Some(STEP),
    ));
    updates.push(message(
        7 * STEP,
        ME,
        at(11, 16),
        voice(6504, 38, 144282),
        Some(2 * STEP),
    ));
    updates.push(message(8*STEP, ME, at(11,16), json!({"@type":"messageAudio","audio":{"@type":"audio",
        "duration":6,"title":"","performer":"","file_name":"хуем_по_батареям_КТО_ТО_БЬЁТ_ПО_BATAREI_NOChЬYu.mp3",
        "mime_type":"audio/mpeg","album_cover_minithumbnail":null,"album_cover_thumbnail":null,"external_album_covers":[],
        "audio":file(6505,"demo-voice.ogg",24000)},"caption":{"@type":"formattedText","text":"","entities":[]}}), None));
    updates.push(message(9*STEP, CHAT, at(11,18), json!({"@type":"messageAnimation","animation":{"@type":"animation",
        "duration":1,"width":480,"height":270,"file_name":"demo-gif.gif","mime_type":"image/gif","has_stickers":false,
        "minithumbnail":null,"thumbnail":{"@type":"thumbnail","format":{"@type":"thumbnailFormatJpeg"},
        "width":480,"height":270,"file":file(6506,&gif_still,24)},"animation":file(6507,&gif_clip,24)},
        "caption":{"@type":"formattedText","text":"","entities":[]},"show_caption_above_media":false,
        "has_spoiler":false,"is_secret":false}), Some(7*STEP)));
    for (id, name, size) in [
        (6500, reply_thumb.as_str(), 24),
        (6502, photo_file.as_str(), 24),
        (6503, "demo-voice.ogg", 338750),
        (6504, "demo-voice.ogg", 144282),
        (6505, "demo-voice.ogg", 24000),
        (6506, gif_still.as_str(), 24),
    ] {
        updates.push(json!({"@type":"updateFile", "file":file(id, name, size)}));
    }
    updates.push(json!({"@type":"updateFile", "file":file(6507, &gif_clip, 24)}));
    updates.push(
        json!({"@type":"updateChatReadOutbox","chat_id":CHAT,"last_read_outbox_message_id":8*STEP}),
    );
    for update in updates {
        if let Some(owned) = copy_and_parse(&update.to_string(), &seq, &dyn_sink) {
            session.apply(owned);
        }
    }
    if let Some(donor) = donor.as_ref() {
        let path = donor.pattern_emoji.to_string_lossy().into_owned();
        let update = json!({"@type":"updateFile","file":file(6508, &path, 24)});
        if let Some(owned) = copy_and_parse(&update.to_string(), &seq, &dyn_sink) {
            session.apply(owned);
        }
        session.accept_custom_emoji_stickers(vec![quill::telegram::envelope::StickerItem {
            custom_emoji_id: Some(PATTERN_EMOJI),
            id: PATTERN_EMOJI,
            set_id: 0,
            emoji: "🍌".into(),
            width: 20,
            height: 20,
            format: quill::telegram::envelope::StickerFormat::Webp,
            file_id: quill::ids::FileId(6508),
            thumb_file_id: None,
            thumb_width: 0,
            thumb_height: 0,
            requires_premium: false,
            premium_animation: None,
        }]);
    }
    session.my_user_id = Some(ME);
    session.settings.proxy.list = Some(vec![quill::proxy::ProxyEntry {
        id: 1,
        last_used_date: 0,
        is_enabled: true,
        comment: String::new(),
        proxy: quill::proxy::ProxyDraft::empty(quill::calls::proxy::ProxyKind::Socks5),
    }]);
    session.open_chat(ChatId(CHAT));
    session
}
