//! Connect-driver tests: a shown chat's wallpaper file downloads without
//! the picker being open.
use super::super::*;
use super::*;
use crate::diagnostics::{DiagnosticSink, MemorySink};
use crate::ids::{AccountKey, ChatId};
use crate::platform::MemorySecretStore;
use crate::state::Session;
use crate::telegram::client::copy_and_parse;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

/// The `updateChatBackground` of a photo set by the other person, shaped as
/// TDLib sends it: the background id is a string.
fn photo_update(chat: i64, file: i32) -> String {
    format!(
        r#"{{"@type":"updateChatBackground","chat_id":{chat},"background":{{"@type":"chatBackground","dark_theme_dimming":20,"background":{{"@type":"background","id":"6012836012836012836","is_default":false,"is_dark":false,"name":"","document":{{"@type":"document","file_name":"","mime_type":"image/jpeg","document":{{"@type":"file","id":{file},"size":9,"local":{{"@type":"localFile","path":"","can_be_downloaded":true,"is_downloading_completed":false}},"remote":{{"@type":"remoteFile","id":"r","unique_id":"u"}}}}}},"type":{{"@type":"backgroundTypeWallpaper","is_blurred":false,"is_moving":false}}}}}}}}"#
    )
}

struct Fixture {
    driver: ConnectDriver<Arc<RecordingSender>>,
    recorder: Arc<RecordingSender>,
    sink: Arc<dyn DiagnosticSink>,
    seq: AtomicU64,
    dir: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let store = MemorySecretStore::new();
        let (dir, prepared) = prepared_tmp(&store);
        let sink: Arc<dyn DiagnosticSink> = Arc::new(MemorySink::new());
        let recorder = Arc::new(RecordingSender::new());
        let session = Session::new(AccountKey::primary(), sink.clone());
        let driver = ConnectDriver::new(session, recorder.clone(), test_credentials(), prepared);
        let mut fixture = Self {
            driver,
            recorder,
            sink,
            seq: AtomicU64::new(0),
            dir,
        };
        for json in [
            r#"{"@type":"updateAuthorizationState","authorization_state":{"@type":"authorizationStateReady"}}"#,
            r#"{"@type":"updateNewChat","chat":{"id":7,"title":"Alice","type":{"@type":"chatTypePrivate","user_id":7},"unread_count":0,"permissions":{"@type":"chatPermissions","can_send_basic_messages":true}}}"#,
            r#"{"@type":"updateNewChat","chat":{"id":8,"title":"Bob","type":{"@type":"chatTypePrivate","user_id":8},"unread_count":0,"permissions":{"@type":"chatPermissions","can_send_basic_messages":true}}}"#,
        ] {
            fixture.ingest(json);
        }
        fixture
    }

    fn ingest(&mut self, json: &str) {
        self.driver
            .ingest(copy_and_parse(json, &self.seq, &self.sink).unwrap())
            .unwrap();
    }

    fn downloads(&self) -> Vec<i64> {
        self.recorder
            .snapshot()
            .into_iter()
            .filter(|json| json.contains("\"@type\":\"downloadFile\""))
            .filter_map(|json| serde_json::from_str::<Value>(&json).ok()?["file_id"].as_i64())
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn opening_a_chat_downloads_its_wallpaper_once() {
    let mut f = Fixture::new();
    f.ingest(&photo_update(7, 70));
    assert!(f.downloads().is_empty(), "the chat is not on screen");
    f.driver.select_chat(ChatId(7)).unwrap();
    assert_eq!(f.downloads(), [70]);
    f.driver.select_chat(ChatId(8)).unwrap();
    f.driver.select_chat(ChatId(7)).unwrap();
    assert_eq!(f.downloads(), [70], "already on its way");
}

#[test]
fn a_new_wallpaper_in_the_open_chat_downloads_at_once() {
    let mut f = Fixture::new();
    f.driver.select_chat(ChatId(7)).unwrap();
    f.ingest(&photo_update(8, 80));
    assert!(f.downloads().is_empty(), "another chat's wallpaper waits");
    f.ingest(&photo_update(7, 70));
    assert_eq!(f.downloads(), [70]);
}
