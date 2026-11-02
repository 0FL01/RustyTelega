# Fix notes C: B2, B19 and B20

Branch `claude/fix-final-c`, from `origin/sweep-all` at ba0cf717. `origin/main` was merged at the start and before the last push; both merges were already up to date. Findings come from `review.md` on `claude/review-new-bundles`, and the rules from `sweep-briefs/rules.md`. Each finding has one commit and a test. Unless noted, the test was run with the fix reverted and it failed.

## Fixed

| Finding | Commit | What changed | Test |
| --- | --- | --- | --- |
| B2-1 | eb978ca3 | If space returns while a call holds the on-screen client at Full, only the background accounts restart (`DiskStep::ReconnectBackground`). The call goes on. | `leaving_full_during_a_held_call_restarts_only_the_background`, `space_freed_during_a_held_call_keeps_the_call_client` |
| B2-2 | d5968787 | Each disk pause saves the message the download came from, per account, in `data_storage.json`. On the first healthy reading after Ready, `searchFileDownloads` finds those downloads under their new file ids, and Quill resumes them. | `downloads_a_low_disk_paused_resume_in_the_next_tdlib_instance` |
| B2-3 | a1b7641c | At Full with a held call, the disk counts as Low with no room. Every listed download pauses. | `a_full_disk_held_open_for_a_call_pauses_every_listed_download` |
| B2-4 | 4bae53c0 | The low-disk bar now also shows under a chat window's title bar and over the call and voice chat windows. It stays until space returns. Its button raises the main window. | `a_low_disk_shows_its_bar_in_a_chat_window_until_space_returns`, `..._in_the_call_window`, `..._in_the_voice_chat_window` |
| B2-5 | 792591ef | `warn = max(1 GiB, open + 256 MiB)` and `warn_clear = warn + 256 MiB`. | `the_floors_keep_their_order` (6 accounts, 1.5 GB database), `a_disk_that_would_refuse_the_next_open_shows_the_bar_first` |
| B2-6 | 0ae162c0 | A data folder that is a link to a missing target makes the reading unwritable. | `a_files_folder_linked_to_a_pulled_drive_is_unwritable` |
| B2-7 | f9308f87 | An account switch clears the cache folder and database size, then reads them again at once. A read for another account is dropped. | `a_switch_drops_the_previous_accounts_storage_figures` |
| B2-8 | 4e6ded80 | `DownloadProgress` has a hold flag, set on the Low or Full edge. The fetch stops and keeps its partial file. A held download goes back to Available, so the existing restart on the Healthy edge resumes it. | `a_held_download_stops_at_its_next_read_and_keeps_what_it_has`, `a_running_update_download_waits_for_space_and_starts_again` |
| B2-9 | 12d350b1 | `OsNotification::about_app` builds the full-disk notice. All four backends ask `offers_actions()`, so the notice has no Reply or Mark as read. notify-send now also drops these buttons when the app is locked, which it did not do before. | `the_full_disk_notice_offers_no_reply_or_mark_as_read`, `a_notice_about_the_app_offers_no_reply_or_mark_as_read`, `locked_apps_offer_no_buttons` |
| B2-10 | ce6f6182 | The Linux quota room is taken under the hard limit. The soft limit counts only once its grace time (`dqb_btime`) has passed. | `writes_reach_the_hard_limit_until_the_soft_limits_grace_ends` |
| B2-11 | 8404685b | On macOS, an important-usage answer of 0 is treated as no reading, and statfs decides. | `a_zero_important_usage_answer_falls_back_to_statfs` |
| B2-12 | 12cba00b | `storage_limits_seeded` is saved only once TDLib's options show limits (the `updateOption` echo). A session sends the limits at most once. | `the_first_ready_sets_tdesktops_cache_limits_once` (changed), `limits_tdlib_never_took_are_sent_again_on_the_next_ready` |
| B2-13 | 21f95073 | Show cached files goes through `reveal_in_folder`. The raw `quill::platform` opens are clippy `disallowed-methods`, allowed only in `src/ui/os_open.rs`, and every caller was moved. | `the_full_screen_links_to_the_cached_files` (failed reveal shows the note) |
| B19-1 | 0d47b9d0 | A t.me link that opens its chat shows no note. `qll_misc_opened_link` is removed. | `a_link_that_opened_its_chat_shows_no_note` |
| B19-2 | ae09ce94 | On unix, an open answers `Pending` with the helper's exit status. A reveal checks that the path exists first, on every OS. The crash notice now waits for the answer. | `a_helper_that_exits_with_an_error_did_not_open`, `revealing_a_path_that_is_gone_fails_before_any_helper`, `show_log_says_so_when_the_folder_fails_to_open_late` |
| B19-3 | e7f29441 | A narrow forum window that shows only topics draws the toast in its body. | `a_narrow_forum_window_showing_its_topics_draws_the_toast` |
| B19-4 | 92998cd5 | Each toast keeps the window it was queued in. `report_failed_open` captures that window before the open runs. A closing chat window hands its toasts to the main window. | `a_note_from_a_chat_window_toasts_in_that_window` (changed), `a_late_open_failure_toasts_in_the_window_that_asked`, `closing_a_chat_window_moves_its_toast_to_the_main_window`, `the_same_note_for_two_windows_shows_in_each` |
| B19-5 | 23f92222 | When a closing window held the hovered toast, its hover ends. | `a_closed_windows_toasts_move_to_the_main_window_and_run_again` |
| B19-6 | 3f87fa69 | The Screen Recording and Input Monitoring panes open through `open_system_settings`, and a failure shows "Couldn't open System Settings." (new string in `lang/en/av.strings`). | `the_privacy_box_says_so_when_system_settings_cannot_open` |
| B20-1 | a71c5b9c | Test builds count the helper's keep-alives and wake on each one. The live wait fails after 5 keep-alives with no relay. | `a_page_that_never_relays_fails_after_some_keep_alives` |
| B20-2 | a7fed64a | A new test drives `QuillApp::mark_folder_read` in the demo app and reads the title badge memo. It fails when the old in-place loop is put back. | `marking_a_folder_read_drops_its_chats_from_the_title_badge` |
| B20-3 | 583d6a8b | The first stale-shim reason is kept and preferred in the error (`LoadFailure`). | `a_stale_shim_stays_the_reason_after_later_failures` |

The two B2 confirmations are in 2016f65a: `docs/decisions/disk-downloads-list-cleanup.md` (TDLib cleanup and the Downloads list) and `release-notes/unreleased-disk-space.md`.

## The Plausible findings

All of them turned out to be defects, so all were fixed:

- **B2-10.** Proven from the kernel source. `check_bdq` in fs/quota/dquot.c refuses a write only past the hard limit, or past the soft limit after `dqb_btime`. No real quota was mounted here.
- **B2-11.** The old `unwrap_or` took `Some(0)` over a positive statfs reading. Some volumes (exFAT, HFS+, network shares) answer 0, which was not checked on a Mac. The fix is safe either way: a truly full disk also reads near 0 in statfs.
- **B2-12.** Proven by the code path: the flag was written before any answer came back. The tests show both cases.
- **B19-4.** The new tests fail on the old code.
- **B19-5.** The new test fails on the old code.
- **B20-1.** `CarrierEvent::Alive` poked nothing, so a helper that stays alive with a silent page blocked `wait_since` forever.
- **B20-2.** Reverting the call site left the old test green; the new test fails.
- **B20-3.** The unit test fails on the old overwrite of `last`.

## TDLib cleanup and the Downloads list

TDLib's file cleanup (`FileGcWorker::run_gc`) never looks at the Downloads list. Documents, videos and audio there are removed like any cached file under the 31-day and 1 GB limits. When a finished download's file is removed, the download leaves the list (`FileManager::on_file_unlink` calls `DownloadManager::remove_file_if_finished`). An unfinished download stays in the list but can lose its partial bytes. The sources cited were read on tdlib/td master, not on the exact tag Quill builds.

## Not done, or not checked here

- **The release note file.** The release workflow reads only `release-notes/vX.Y.Z.md`, and the next version is not known. Move the text of `release-notes/unreleased-disk-space.md` into that file when the tag is cut.
- **B2-4, two kinds of window.** Picture-in-picture and notification popups still draw no bar. They are short-lived overlays next to a window that does show the bar, and a bar would cover the video or the message. The review's own fix named only the chat, call and voice chat windows. If the owner wants the bar in these too, it is a small follow-up.
- **Live client in tests.** A `LiveConnect` needs a real libtdjson, so UI tests cannot build one. The B2-1 UI test sets `held_for_call` by hand, as the review noted; the step logic itself has a unit test. The B19-1 live path is tested through `chat_link_note`, which is the only part that changed.
- **macOS and Windows code was not compiled.** `cargo check --target x86_64-pc-windows-gnu` and `--target aarch64-apple-darwin` both stop in C build scripts (gpui-pre, ring) without a cross toolchain. Code that only a Mac can build: `screen_mac.rs` and `ptt_global_mac.rs` (B19-6), the `read_available` call site in `probe.rs` (B2-11), and the `open -R` exit status (B19-2). Code that only Windows can build: the Windows `os_spawn` in `platform.rs` (B19-2). All of these edits are small.
- **Tests that need a desktop or the video shim.** The live carrier tests (`helper_live_*`) were not run. The B20-1 bound was checked with a stand-in helper instead.
- **`scripts/check-hotspots.sh` against origin/main.** It reports `src/main.rs`, `src/ui/shell.rs`, `src/state/session_apply.rs` and `src/ui/demos/mod.rs`. Those changes come from sweep-all's earlier bundles, not from this branch. Checked against `origin/sweep-all`, it reports `hotspots OK`; this branch's only hotspot change is one `mod disk_paused;` line in `src/connect/mod.rs`, which is allowed.
- **Measurements.** None of these findings gave a number to measure before and after.

## What ran (Linux, as root)

- `cargo fmt`, `scripts/check-file-size.sh` (OK), `scripts/check-hotspots.sh origin/sweep-all` (OK) and `python3 scripts/check-lang.py` (OK).
- `cargo clippy -- -D warnings` for `--features demo-capture --all-targets` and for `--no-default-features --all-targets`: both clean.
- `cargo test --features demo-capture --lib --bin quill`: see the next section.
