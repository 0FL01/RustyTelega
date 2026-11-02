//! Connect driver: the listed downloads a low or full disk paused, kept
//! across a closed client or a restart. TDLib keeps them paused in its
//! Downloads list, under new file ids in the next instance, so the
//! message each came from is saved per account and finds them again.
use super::*;
use crate::ids::FileId;
use crate::state::{MediaPurpose, RequestPurpose};
use crate::telegram::requests::search_active_file_downloads;

/// More than any Downloads list a disk pause leaves behind.
const SEARCH_LIMIT: i32 = 1000;

impl<S: JsonSender> ConnectDriver<S> {
    /// The disk paused `file_id`: save the message it came from.
    pub(super) fn save_disk_pause(&mut self, file_id: i32) {
        if let Some((chat, message)) = self.session.media.downloads.origin_of(file_id) {
            let saved = &mut self.session.settings.data_storage.disk_paused;
            if saved.insert((chat.0, message)) {
                self.session.media.disk_paused_saved_changed = true;
            }
        }
    }

    /// The disk's pause of `file_id` is undone.
    pub(super) fn forget_disk_pause(&mut self, file_id: i32) {
        if let Some((chat, message)) = self.session.media.downloads.origin_of(file_id) {
            let saved = &mut self.session.settings.data_storage.disk_paused;
            if saved.remove(&(chat.0, message)) {
                self.session.media.disk_paused_saved_changed = true;
            }
        }
    }

    /// On a healthy disk: saved pauses this session did not make were made
    /// by an earlier TDLib instance, so the Downloads list is searched for
    /// them, once per session.
    pub(super) fn search_disk_paused_elsewhere(&mut self) -> Result<(), ConnectSendError> {
        let media = &mut self.session.media;
        if media.disk_resume_searched || self.session.settings.data_storage.disk_paused.is_empty() {
            return Ok(());
        }
        media.disk_resume_searched = true;
        let extra = self.session.request(
            RequestPurpose::Media(MediaPurpose::SearchDiskPausedDownloads),
            None,
        );
        if let Err(err) = self
            .sender
            .send_json(&search_active_file_downloads(extra, SEARCH_LIMIT))
        {
            self.session.requests.take(extra);
            self.session.media.disk_resume_searched = false;
            return Err(err);
        }
        Ok(())
    }

    /// After an ingest: resume what that search found, and save the
    /// pauses when they changed.
    pub(crate) fn resume_disk_paused_found(&mut self) -> Result<(), ConnectSendError> {
        let due = std::mem::take(&mut self.session.media.disk_resume_due);
        for id in due {
            self.resume_download(FileId(id))?;
        }
        self.save_disk_pauses_if_changed();
        Ok(())
    }

    pub(super) fn save_disk_pauses_if_changed(&mut self) {
        if std::mem::take(&mut self.session.media.disk_paused_saved_changed) {
            let _ = self.save_data_storage_prefs();
        }
    }
}
