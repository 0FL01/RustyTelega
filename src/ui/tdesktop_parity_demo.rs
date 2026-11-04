//! Offline Telegram Desktop parity capture fixture.
#![deny(clippy::indexing_slicing)]

use super::app::QuillApp;
use super::demo::{demo_file_json, demo_media_allowlist, seed_ready_chats_session};
use super::screenshot_demo::{DemoSpec, register_demos};
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

register_demos![
    DemoSpec::ready(
        "ready-tdesktop-parity",
        seed,
        "screenshot demo — Telegram Desktop parity"
    )
    .setup(QuillApp::demo_tdesktop_parity)
];

impl QuillApp {
    fn demo_tdesktop_parity(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.appearance.folder_tabs_view = quill::folder_icons::FolderTabsView::Left;
        self.open_folder_tab(Some(FOLDER), cx);
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
    json!({"@type":"messagePhoto", "photo":{"@type":"photo", "has_stickers":false,
        "minithumbnail":null,"sizes":[{"@type":"photoSize","type":"m",
        "photo":file(id,name,24),"width":1100,"height":740,"progressive_sizes":[]}]},
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

fn voice(id: i32, duration: i32, size: i64) -> Value {
    let wave = quill::voice::waveform_base64(&[
        6, 14, 22, 30, 18, 10, 26, 31, 20, 12, 8, 24, 28, 16, 10, 20, 8, 23, 30, 12, 24, 15, 6, 27,
        31, 14, 9, 20, 28, 9, 15, 26,
    ]);
    json!({"@type":"messageVoiceNote","voice_note":{"@type":"voiceNote","duration":duration,
        "waveform":wave,"mime_type":"audio/ogg","speech_recognition_result":null,
        "voice":file(id,"demo-voice.ogg",size)},
        "caption":{"@type":"formattedText","text":"","entities":[]},"is_listened":true})
}

fn seed(sink: Arc<MemorySink>) -> Session {
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
    let today = at(22, 13).min(now - 60);
    let monday = midnight - i64::from(local.weekday) * 86400 + 12 * 3600;
    let monday = if monday > now {
        monday - 7 * 86400
    } else {
        monday
    };
    let offset = i64::from(quill::local_time::utc_offset_at(now));
    let old_oct = 1_793_188_800 - offset; // 2026-10-28 12:00 UTC interpreted locally
    let old_sep = 1_790_683_200 - offset; // 2026-09-29 12:00
    let folders: Vec<_> = [("Humanoid","Private"),("Bots","Bots"),("Work","Work"),
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
            "showcase/av05.png",
            json!({"@type":"userStatusRecently"}),
        ),
        (
            CHAT,
            "Lain",
            "",
            "showcase/av01.png",
            json!({"@type":"userStatusOffline","was_online":now-60}),
        ),
    ] {
        let small = file(id as i32 + 6000, picture, 24);
        updates.push(json!({"@type":"updateFile","file":small}));
        updates.push(json!({"@type":"updateUser","user":{"@type":"user","id":id,"first_name":first,
            "last_name":last,"accent_color_id":0,"status":status,"profile_photo":{"@type":"profilePhoto",
            "id":id,"small":small,"big":null,"minithumbnail":null,"has_animation":false,"is_personal":false},
            "is_contact":true,"type":{"@type":"userTypeRegular"}}}));
    }
    updates.push(json!({"@type":"updateOption","name":"my_id","value":{"@type":"optionValueInteger","value":ME}}));
    updates.push(json!({"@type":"updateNewChat","chat":{"id":CHAT,"title":"Lain",
        "type":{"@type":"chatTypePrivate","user_id":CHAT},"photo":avatar(6201,"showcase/av01.png"),"unread_count":0}}));
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
        updates.push(
            json!({"@type":"updateNewChat","chat":{"id":id,"title":title,
            "type":{"@type":"chatTypeSupergroup","supergroup_id":id,"is_channel":true},
            "photo":avatar(6300+n as i32,av),"unread_count":0}}),
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
                updates.push(json!({"@type":"updateNewMessage","message":{
                    "id": (index as i64 + 1) * STEP,"chat_id":id,
                    "sender_id":{"@type":"messageSenderChat","chat_id":id},
                    "date":date,"is_channel_post":true,"media_album_id":"88001",
                    "content":photo(6600 + index as i32,name,if index == 2 { caption } else { "" })}}));
            }
        }
        updates.push(json!({"@type":"updateChatLastMessage","chat_id":id,"last_message":{
            "id":if *id == 304 { 3*STEP } else { STEP },"chat_id":id,"sender_id":{"@type":"messageSenderChat","chat_id":id},
            "media_album_id":if *id == 304 { "88001" } else { "0" },
            "date":date,"is_outgoing":false,"is_channel_post":true,"content":photo(6400+n as i32,pic,caption)},
            "positions":[{"@type":"chatPosition","list":{"@type":"chatListMain"},"order":order,"is_pinned":false},
            {"@type":"chatPosition","list":{"@type":"chatListFolder","chat_folder_id":FOLDER},"order":order,"is_pinned":false}]}));
    }
    let mut link = text("https://github.com/shinycake/quill");
    if let Some(object) = link.as_object_mut() {
        object.insert("link_preview".into(), json!({"@type":"linkPreview","url":"https://github.com/shinycake/quill",
        "display_url":"github.com/shinycake/quill","site_name":"GitHub","title":"quill",
        "description":{"@type":"formattedText","text":"","entities":[]},"author":"",
        "type":{"@type":"linkPreviewTypeArticle","photo":{"@type":"photo","sizes":[{"@type":"photoSize",
        "type":"m","photo":file(6500,"showcase/p5.jpg",24),"width":1100,"height":740}]}},
        "has_large_media":true,"show_large_media":true,"show_media_above_description":false,
        "skip_confirmation":true,"show_above_text":false,"instant_view_version":0}));
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
        photo(6502, "showcase/p6.jpg", ""),
        None,
    ));
    updates.push(message(5*STEP, CHAT, at(11,3), text("DHCP-сервер LAN включён, три LAN-порта объединены в мост.\nWAN настроен на DHCP, WAN6 — DHCPv6.\nWi-Fi выключен; постоянный flow offloading оставлен выключенным по умолчанию.\nСтарые пароли, наши разрешённые SSH-ключи, тестовые настройки и дополнительное лабораторное логирование удалены.\nСерверные SSH-ключи созданы заново.\nСобственные MAC, калибровки, загрузчики и разметка сохранены."), None));
    updates.push(message(
        6 * STEP,
        ME,
        at(11, 12),
        voice(6503, 78, 338739),
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
        "duration":1,"width":240,"height":140,"file_name":"demo-gif.gif","mime_type":"image/gif","has_stickers":false,
        "minithumbnail":null,"thumbnail":{"@type":"thumbnail","format":{"@type":"thumbnailFormatJpeg"},
        "width":240,"height":140,"file":file(6506,"demo-gif-1.png",24)},"animation":file(6507,"demo-gif.gif",24)},
        "caption":{"@type":"formattedText","text":"","entities":[]},"show_caption_above_media":false,
        "has_spoiler":false,"is_secret":false}), Some(7*STEP)));
    updates.push(
        json!({"@type":"updateChatReadOutbox","chat_id":CHAT,"last_read_outbox_message_id":8*STEP}),
    );
    for update in updates {
        if let Some(owned) = copy_and_parse(&update.to_string(), &seq, &dyn_sink) {
            session.apply(owned);
        }
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
