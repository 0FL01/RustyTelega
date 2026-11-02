### Disk space and the cache

- Quill now keeps its cache to 1 GB and clears files nobody opened for 31 days, the limits Telegram Desktop starts with. This is set once per account, only on accounts that never chose limits. If you pick "No limit" in Settings › Data and Storage, it stays that way.
- The cleanup also covers files in the Downloads list. A downloaded file you have not opened for 31 days, or the oldest ones once the cache passes 1 GB, leave the list and download again when you open them. Files you saved with Save or Save as are copies outside the cache and are never removed.
- The low disk warning now shows in every Quill window: chat windows, calls and voice chats too. It stays until enough space is free.
- Downloads that Quill paused because the disk was low now resume by themselves once space is free, also after Quill restarts or reconnects.
- Freeing space during a call no longer ends the call.
- While a call keeps Quill connected on a full disk, downloads and an update download pause instead of filling the disk.
- Quill warns before a low disk would stop it from opening your account, even with a large message database or several accounts.
- A data folder on a drive that was unplugged now counts as unwritable, so Quill disconnects safely instead of reading the internal disk.
- The full disk notification no longer offers Reply and Mark as read.
- "Show cached files" on the full disk screen says so when the folder cannot open.
