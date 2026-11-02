## How TDLib's cache cleanup treats files in the Downloads list (2026-11-02)

The B2 brief asked for this check once Quill seeds Telegram Desktop's cache
limits on every account (`src/storage_limits.rs`: `storage_max_files_size`
1048576 KiB, `storage_max_time_from_last_access` 31 days,
`use_storage_optimizer` on). Read in TDLib's sources on tdlib/td master,
fetched 2026-11-02: `StorageManager.cpp`, `files/FileGcWorker.cpp`,
`files/FileGcParameters.cpp`, `files/FileStatsWorker.cpp`,
`files/FileManager.cpp` and `DownloadManager.cpp`. They were not compared
against the exact TDLib tag Quill builds.

### What the cleanup looks at

- `StorageManager::timeout_expired` runs `run_gc` about once a day while
  `use_storage_optimizer` is on. `FileStatsWorker` lists every file in the
  per-type folders, the temp folders, and the partial files the file
  database knows (`data.local_.partial()`).
- `FileGcWorker::run_gc` never consults the Downloads list
  (`DownloadManager`). Only the file type and age protect a file:
  - Immune types: Sticker, ProfilePhoto, Thumbnail, Wallpaper, Background,
    Ringtone and EncryptedThumbnail. Documents, videos, audio, photos and
    voice notes, which is what the Downloads list holds, are not immune.
  - A file modified within `storage_immunity_delay` (1 hour by default) is
    kept.
  - Otherwise a file is removed when its last access is older than
    `storage_max_time_from_last_access`. Then, oldest first, files are
    removed until the total is under `storage_max_files_size`.

### What happens to a removed file

`do_remove_file` unlinks it and calls `FileManager::on_file_unlink`, which
drops the local copy and calls `DownloadManager::remove_file_if_finished`:

- A finished download leaves the Downloads list. Its message still shows
  the file, and opening it downloads it again.
- An unfinished (active or paused) download stays in the list; only its
  partial bytes go, and a resume starts from zero.

### Consequences for Quill

- A file a user downloaded and has not opened for 31 days, or the oldest
  ones past 1 GB of cache, disappear from the Downloads list. This is what
  Telegram Desktop's own default limits do too.
- Files the user saved (Save, Save as) are copies outside TDLib's folders
  and are never touched.
- A download the disk paused (B2-2) and left paused for over 31 days loses
  its partial bytes. It is still found and resumed, from the start.
- Quill does not exclude the Downloads list from the cleanup: TDLib has no
  option for it, and `FileGcParameters` filters only by type and chat.

The release note says this in plain words (`release-notes/unreleased-disk-space.md`).
