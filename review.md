CHANGES

# Last review of sweep-all at 2e1b8030

Scope: the work merged into origin/sweep-all that no independent review had seen. That is calls-followups (18a06b0c), sticker-timing-2 (ca3c1590), vsync-share (7e733245), long-text (ba0cf717), rich-lists (f5e36aae), live-replay (e8864eae), rich-and-reset (fde4b990, 008bb325, 81494e96, 559a5f8a and ace316f0), and claude/fix-final-a (d26d33d4), -b (9f176f3b), -c (761be35c) and -d (a2bd103d, plus 2e1b8030). No product branch was changed.

The verdict is CHANGES. Two findings are High and the UI bin suite is red:

- After a GPU loss with two or more windows open, Quill is left running with no window and cannot be reached.
- The quill binary no longer compiles for Windows without the `ui` feature, which breaks the core test step of the Windows release build.
- One UI bin test fails on sweep-all.
- Seven more findings are Medium. They change the text a user sends, how fast stickers play, a download the user paused, and what happens to windows and IPv6 routers after a change.

## How each finding was proven

- **Confirmed (test):** a test was added in a scratch tree and run on Linux, and it fails on sweep-all as described. The test code is under each finding. It is not committed, because a red test on this branch would break CI for anyone who merges it. Paste it in with the fix.
- **Confirmed (trace):** the code path was followed end to end, with file and line.
- **Plausible:** the path is real but the trigger, or the OS behavior it needs, was not reproduced here.

## Suites run (Linux, x86_64, toolchain 1.99.1)

| Suite | Result |
|---|---|
| `cargo test --no-default-features --locked --lib` | 7058 passed, 0 failed, 23 ignored |
| `cargo test --features demo-capture --locked --lib` | 7082 passed, 0 failed, 23 ignored |
| `cargo test --features demo-capture --locked --bin quill` | **2658 passed, 1 failed**, 2 ignored (finding T1) |

## Findings, most severe first

### R1. High, Confirmed (test). After a lost GPU with two or more windows open, recovery never runs and Quill is left alive with no window

- **Where:** `src/ui/startup_failure/device_loss.rs:136-140` (`Step::CloseTheRest`), with `third_party/gpui-pre/src/app.rs:3462` and `third_party/gpui-pre/src/subscription.rs:129-137`.
- **How it fails:**
  - The first close runs the watcher, which calls `remove_window` on every other window from inside the window-closed callback.
  - Each nested close reaches `window_closed_observers.retain`. That list is taken out while the outer callback runs (`.and_then(|s| s.take())` returns `None`), so the watcher is never called for those closes.
  - So no close ever sees `windows_left == 0`, and `Recover` never runs. Because the watcher set `QuitMode::Explicit`, GPUI does not quit either.
  - The user sees every window vanish. The process stays alive with the session running. Tray Open/Quit and a second launch all go to the dead main handle, so Quill cannot be opened again until it is killed.
  - Any second window is enough to trigger it: a chat window, the call window, PiP or the media viewer.
- **Fix:** run the closes after the callback returns: `Step::CloseTheRest => cx.defer(|cx| { for window in cx.windows() { let _ = window.update(cx, |_, w, _| w.remove_window()); } })`.
- **Test** (`src/ui/startup_failure/device_loss_tests.rs`, in `reset_tests`). It fails today with `left: 1, right: 2`, because the second close never reaches the watcher:

```rust
#[gpui_kit::test]
fn closing_the_rest_after_a_lost_gpu_still_reaches_the_last_close(cx: &mut TestAppContext) {
    use std::cell::Cell;
    use std::rc::Rc;
    let opened = open(cx);
    // A second window (a PiP or chat window) that the platform left open.
    let _pip = cx.open_window(size(px(200.), px(200.)), |_, _| Probe::default());
    let seen = Rc::new(Cell::new(0usize));
    let seen_in = seen.clone();
    cx.update(|cx| {
        watch_closed_windows(
            move |_| {
                seen_in.set(seen_in.get() + 1);
                true
            },
            |_, _| Ok(()),
            cx,
        )
    });
    let _ = cx.update_window(opened.window, |_, window, _| window.remove_window());
    cx.run_until_parked();
    assert!(cx.windows().is_empty());
    assert_eq!(seen.get(), 2, "every close reaches the watcher");
}
```

### W0. High, Confirmed (Windows cross-check). The quill binary fails to compile on Windows with `--no-default-features`

- **Where:** `src/main.rs:305`. fde4b990 gated `reopen_main_window` with `#[cfg(windows)]`. Its body uses `main_window_options`, `ui::…` and `gpui_kit`, which are all `#[cfg(feature = "ui")]`.
- **How it fails:** `cargo check --target x86_64-pc-windows-gnu --no-default-features --lib --tests` stops with E0433/E0425 at `src/main.rs:307-311` (five errors). `package-windows.yml`'s "tests (core + replay)" step runs `cargo test --no-default-features --locked`, which builds this binary. The Windows release build therefore fails before it packages anything.
  - PR CI misses it because `windows-check` builds with `--features demo-capture`.
  - Linux misses it because the item is Windows only.
- **Fix:** `#[cfg(all(windows, feature = "ui"))]` on `reopen_main_window`.

### T1. Medium, Confirmed (test). The UI bin suite fails on sweep-all: `ui::inline_video::tests::small_clips_are_never_upscaled`

- **Where:** `src/ui/inline_video.rs:651`. a6e39968 (fix-final-d) made `decode_bytes` count `quill::video_decode::BGRA_KEPT` idle buffers (`inline_video.rs:179`), but did not update the test. It expects `100*80*4*4` = 128000 and gets 224000.
- **Impact:** `cargo test --features demo-capture --bin quill` (CI job `linux-fmt-clippy-test`) is red. The budget also changes for users: each inline clip is now charged 3 more frames. That holds on Linux too, where NV12 surfaces mean the BGRA pool is idle, so fewer clips decode at full size than before.
- **Fix:** either change the assertion to `100 * 80 * 4 * (3 + 1 + quill::video_decode::BGRA_KEPT)`, or charge `BGRA_KEPT` only while the clip hands out BGRA pictures. The second matches what a6e39968 says the pool does.

### F-rich-1. Medium, Confirmed (test). The long-text splitter cuts inside a single visible character

- **Where:** `src/text_split.rs:111-137` (`next_cut`). When no whitespace fits, `hard` is the last code point that fits. Nothing checks that the code point ends a grapheme.
- **How it fails:** `"x" + "🇺🇸"×10` with a limit of 16 splits between the two halves of the fourth flag. Message 1 ends with a lone 🇺, and every flag in message 2 pairs wrongly (🇸🇺…). The same happens to `e\u{301}`, `❤\u{FE0F}`, skin tones and ZWJ families: the next message starts with a lone mark.
- **Fix:** walk `rest.grapheme_indices(true)` (unicode-segmentation is already a dependency) and set `hard` at grapheme ends. Fall back to a scalar boundary only when one grapheme alone is wider than the limit.
- **Tests** (`src/text_split.rs`, `mod tests`). Both fail today:

```rust
#[test]
fn a_hard_cut_keeps_a_flag_whole() {
    let text = format!("x{}", "🇺🇸".repeat(10));
    let parts = split_markup_text(&text, 16);
    assert_eq!(parts.concat(), text);
    for part in &parts {
        let halves = part
            .chars()
            .filter(|c| ('\u{1F1E6}'..='\u{1F1FF}').contains(c))
            .count();
        assert!(halves % 2 == 0, "a flag cut in two: {parts:?}");
    }
}

#[test]
fn a_hard_cut_keeps_a_combining_mark_with_its_letter() {
    let text = format!("x{}", "e\u{301}".repeat(20));
    let parts = split_markup_text(&text, 16);
    assert_eq!(parts.concat(), text);
    for part in parts.iter().skip(1) {
        assert_ne!(part.chars().next(), Some('\u{301}'), "{parts:?}");
    }
}
```

### F-rich-2. Medium, Confirmed (test). Rich send and rich draft save drop formatting nested inside other formatting

- **Where:** `src/rich/input.rs:11-29` (`formatted_to_rich_text`) keeps a span only if it starts after the previous one ends, so every nested span is dropped.
- **How it fails:** the composer markup nests and the preview shows every span. `**see [docs](https://example.com) now**` previews with the link but is sent as `{"@type":"richTextBold","text":{"@type":"richTextPlain","text":"see docs now"}}`, with no URL. `**a *b* c**` loses its italic.
- **Why it matters now:** the drop is older, but f5e36aae and 008bb325 now write received list items and details back as markup, and the AI-answer path goes through `RichDraft::from_message(..).markup()`. So far more text reaches it.
- **Fix:** build the rich text as a tree. Sort spans by `(start, Reverse(end))` and recurse into the spans that lie inside each span. Split a span that partly overlaps another at that span's end.
- **Test** (`src/rich/input.rs`, `mod tests`). It fails today:

```rust
#[test]
fn formatting_inside_bold_is_sent() {
    let link = block_rich_text("**see [docs](https://example.com) now**").to_string();
    assert!(link.contains("https://example.com"), "{link}");
    let italic = block_rich_text("**a *b* c**").to_string();
    assert!(italic.contains("richTextItalic"), "{italic}");
}
```

### F-rich-3. Medium, Confirmed (test). A rich draft or AI answer holding a code block is sent differently from how it is shown

- **Where:** `src/ui/rich_draft/mod.rs:254-255`. `submit_rich_draft` sends `rich.markup()`, which `markup_to_blocks` re-parses (`src/rich/markup.rs:7-80`). That parser is line based and does not know code fences. `block_to_markup` also runs `one_line` over list items.
- **How it fails:** a `pageBlockPreformatted{language:"python", text:"# setup\nx = 1"}` followed by a divider shows as one code block. It is sent as the paragraph "```python", the heading "setup" and the paragraph "x = 1\n```". A blank line inside the code also splits it, and a multi-line list item is joined with spaces.
- **Fix:**
  - Send `rich.blocks()`, as `input()` already does.
  - Make `markup_to_blocks` treat a fence as one unit up to its closing fence.
  - Keep `MarkupBlocks::chunks_of` (`src/rich/preview.rs:25-40`) from cutting a chunk at a blank line inside an open fence, so the incremental parse still matches the full parse.
- **Test** (`src/rich/draft_tests.rs`). It fails today, showing exactly the three blocks above:

```rust
#[test]
fn a_code_block_with_a_comment_line_survives_the_editor_markup() {
    use crate::text::{TextEntity, TextEntityKind};
    let code = RichBlock::Paragraph {
        text: "# setup\nx = 1".into(),
        entities: vec![TextEntity {
            utf8_start: 0,
            utf8_end: 13,
            kind: TextEntityKind::PreCode { language: "python".into() },
        }],
        buttons: Vec::new(),
    };
    let draft = RichDraft::from_message(&[code, RichBlock::Divider]).expect("visible");
    let again = RichDraft::from_markup(&draft.markup()).expect("structured");
    assert_eq!(again.blocks(), draft.blocks());
}
```

### F-frame-1. Medium, Confirmed (test). Stickers and emoji play up to 1.5x fast when a paint reads an earlier content time

- **Where:** `src/sticker_playback/player/head.rs:138-145` (`PlayHead::advance`), fed by `src/ui/frame_clock.rs:633-644` (`content_at`).
  - A frame that shows a clip picture gets its own vsync time. Any other frame gets the last lattice tick, which can be earlier than the time the previous paint used.
  - `advance` treats the backwards gap as zero but still stores the earlier `now` as `last`, so the next paint counts that stretch again.
  - `Woke::Later` moving the lattice onto a clip's beat (`frame_clock.rs:137-139`) has the same effect.
- **How it fails:** on a 120 Hz display with 30 fps stickers, a 60 fps inline clip playing, while scrolling. Paints read L+16.7, then L, then L+33.3, so the head moves 50 ms in 33.3 ms. Since 39e879b5 every window reads this time (`src/ui/sticker_player/mod.rs:463`).
- **Fix:** in `advance`, take `let now = self.last.map_or(now, |last| last.max(now));` before computing the gap.
- **Test** (`src/sticker_playback/player/tests.rs`). It fails today with "played 50ms in 33.3 ms":

```rust
#[test]
fn a_head_never_counts_time_twice_when_a_paint_reads_earlier() {
    let tl = timeline(60, 60.0, 1);
    let t0 = Instant::now();
    let ms_f = |n: f64| Duration::from_secs_f64(n / 1000.0);
    let mut head = PlayHead::default();
    head.start(t0);
    head.advance(t0 + ms_f(16.7), PlayMode::Loop, tl, false, 120.0);
    head.advance(t0, PlayMode::Loop, tl, false, 120.0);
    head.advance(t0 + ms_f(33.3), PlayMode::Loop, tl, false, 120.0);
    assert!(head.pos() <= ms_f(33.4), "played {:?} in 33.3 ms", head.pos());
}
```

### F-cd-1. Medium, Confirmed (test). A download the user paused by hand is resumed when disk space comes back, and now after a restart too

- **Where:** the Downloads panel's Pause and Resume (`src/ui/downloads.rs:136,158`) call the same `pause_download`/`resume_download` as the disk code (`src/connect/media.rs:458-465`). Neither clears the in-memory `disk_paused` set or the set fix-final-c now saves.
  - A Healthy reading resumes the download (`media.rs:401-407`).
  - In a later launch, the saved-pause search does the same (`src/state/updates_sync.rs:307-321`).
- **Fix:** a pause or resume the user makes removes the file from both sets and saves them. The disk code uses a private toggle that leaves the bookkeeping alone.
- **Test** (`src/connect/tests/messaging/downloads.rs`, using that file's helpers). It fails today: `toggleDownloadIsPaused(51, false)` goes out after the user's pause.

```rust
#[test]
fn a_download_the_user_paused_stays_paused_when_the_disk_recovers() {
    const MIB: u64 = 1 << 20;
    let store = MemorySecretStore::new();
    let (dir, prepared) = prepared_tmp(&store);
    let sink: Arc<dyn DiagnosticSink> = Arc::new(MemorySink::new());
    let recorder = Arc::new(RecordingSender::new());
    let session = Session::new(AccountKey::primary(), sink.clone());
    let mut driver = ConnectDriver::new(session, recorder.clone(), test_credentials(), prepared);
    let seq = AtomicU64::new(0);
    let ingest = |driver: &mut ConnectDriver<Arc<RecordingSender>>, json: &str| {
        driver.ingest(copy_and_parse(json, &seq, &sink).unwrap()).unwrap();
    };
    ingest(
        &mut driver,
        r#"{"@type":"updateAuthorizationState","authorization_state":{"@type":"authorizationStateReady"}}"#,
    );
    driver.download_user_file(FileId(51), Some((ChatId(7), 99))).unwrap();
    ingest(&mut driver, &file_update(51, 900 * MIB as i64, 0));
    driver.note_disk(DiskState::Low, Some(FLOORS.close + 500 * MIB), &FLOORS).unwrap();
    ingest(&mut driver, &paused_update(51, true));
    driver.resume_download(FileId(51)).unwrap();
    ingest(&mut driver, &paused_update(51, false));
    driver.pause_download(FileId(51)).unwrap();
    ingest(&mut driver, &paused_update(51, true));
    let base = recorder.snapshot().len();
    driver.note_disk(DiskState::Healthy, Some(FLOORS.warn_clear), &FLOORS).unwrap();
    let sent = recorder.snapshot();
    let resumed = sent[base..].iter().any(|json| {
        let v: Value = serde_json::from_str(json).unwrap();
        v["@type"] == "toggleDownloadIsPaused" && v["file_id"] == 51 && v["is_paused"] == false
    });
    assert!(!resumed, "the user's pause was overridden: {sent:?}");
    assert!(driver.session.settings.data_storage.disk_paused.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
```

### R2. Medium, Confirmed (test). A chat, call or group-call window the platform destroys leaves stale entries in the kept app

- **Where:** these windows clean up only through `on_window_should_close` and their own close actions:
  - chat window: `src/ui/chat_window/view.rs:396-399`
  - call window: `src/ui/call_panel.rs:319-326`
  - group-call window: `src/ui/group_call_window.rs:156-168`

  A platform destroy (device lost, `WM_DESTROY` then `remove_window`) and `CloseTheRest` skip that path. Since fde4b990 the `QuillApp` is kept across a reset, so the stale state now outlives it.
- **How it fails:**
  - `windows.open` keeps the dead handle, and `ChatWindows::activate` (`src/ui/chat_window/mod.rs:264-275`) keeps "succeeding" on it. "Open in new window" and picking that chat in the main list both do nothing for the rest of the session.
  - The window's unsent draft is never flushed.
  - TDLib keeps the chat open.
  - `calls.window` and `group_call.window` keep the dead handle, so a running call never gets its window back.
  - This also happens today without R1, whenever one secondary window's renderer gives up (`Step::Nothing`).
- **Fix:** do the cleanup in an `on_release` (or window-closed) hook on each view instead of only in should-close.
- **Test** (`src/ui/chat_window/tests.rs`). It fails today with "stale chat window entry":

```rust
#[gpui_kit::test]
fn a_chat_window_the_platform_destroys_frees_its_chat(cx: &mut TestAppContext) {
    let h = new_app(cx);
    let chat_window = h.open_in_new_window(OTHER, cx);
    // A lost renderer: the platform destroys the window without asking it to close.
    cx.update_window(chat_window, |_, window, _| window.remove_window()).unwrap();
    cx.run_until_parked();
    assert!(h.other_windows(cx).is_empty());
    assert!(h.app.read_with(cx, |app, _| !app.windows.has(OTHER)), "stale chat window entry");
    assert!(h.window_chats(cx).is_empty());
    cx.update_window(h.main, |_, window, cx| {
        h.app.update(cx, |app, cx| app.open_chat_in_new_window(OTHER, window, cx));
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(h.other_windows(cx).len(), 1, "asking again opens a window");
}
```

### F-ab-1. Medium, Confirmed (trace; kernel behavior not run here, the container has no IPv6). The IPv6 router lookup never leaves the machine on Linux or macOS

- **Where:** `src/net_watch/link.rs:26-38` (`resolve`) sends to `(gateway, 9)` with scope id 0.
- **How it fails:**
  - IPv6 routers are almost always link-local (`fe80::…`). Linux returns `EINVAL` for a link-local destination with no scope and no bound device (ipv6(7)), and macOS returns `EHOSTUNREACH`. The error is ignored.
  - `Tracker::resolving` marks the gateway as asked, so it is not asked again.
  - Moving between two networks whose routers share `fe80::1` is then not seen until unrelated IPv6 traffic resolves the neighbour, and TDLib's connections stay on the old network.
- **Fix:** pass `Link.interface` into `resolve` and send to `SocketAddrV6::new(gw, 9, 0, interface)` when `gw.is_unicast_link_local()`.

### F-calls-1. Low (Medium on Windows exit crashes), Plausible. The device-list thread is not waited for at quit, and it now runs on every join

- **Where:** `src/calls/engine/tgcalls/call_engine.rs:401-409` (`list_media_devices`) starts a detached thread that calls tgcalls' C++ (`devices::cameras`) without a reaper hold. The reaper (`src/calls/engine/tgcalls/reaper.rs:95`) exists because an exit during tgcalls work crashes in static destructors.
- **How it fails:** before, this ran once per process. `refresh_call_devices` now starts it on every join, rejoin, connect, camera toggle and when the Speakers box opens. Join a call and quit at once, and the process can crash at exit.
- **Fix:** move `self.reaper.hold()` into the thread.

### F-calls-2. Low, Plausible. Closing the call's sound output moved off the reaper

- **Where:** `retire`, `retire_instance` and `retire_group` now drop the sound's `DeviceThread` on the UI thread. The device then closes (or finishes a slow Bluetooth open) on a detached `quill-call-sound` thread that quit does not wait for.
- **Fix:** move a `ReaperHold` into `SoundSlot::open_with`'s device thread.

### F-calls-3. Low, Plausible. A join whose payload lands late skips the 1:1 call check

- **Where:** `join_video_chat` refuses while a 1:1 call is active, but `offer_wanted` for `OfferSend::Join` (`src/connect/group_calls/offers.rs`) does not check again when the payload lands, up to 2 s later.
- **How it fails:** press Join, then answer an incoming 1:1 call during the wait. `joinVideoChat` goes out during the 1:1 call.
- **Fix:** add `&& calls.active_call.is_none()` to the Join case.

### F-rich-4. Low, Confirmed (test). A cut inside a code block strips the next part's first-line indentation

- **Where:** `src/text_split.rs:178-198`. `skip_space` skips all whitespace after a seam, including indentation inside a `Pre` span.
- **How it fails:** the test gives `"…let a = 1;\nlet b = 2;…"` where `"    let b = 2;"` was typed. Indentation matters for Python and YAML.
- **Fix:** inside a `Pre` span, skip only the one `\n` at the seam.
- **Test** (`src/text_split.rs`). It fails today:

```rust
#[test]
fn a_cut_inside_a_code_block_keeps_the_next_line_indented() {
    let code = "fn f() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n}";
    let parts = split_markup_text(&format!("```\n{code}\n```"), 30);
    assert!(parts.len() > 1);
    let texts: Vec<String> = parts.iter().map(|p| ComposerDoc::from_markup(p).text).collect();
    assert_eq!(texts.join("\n"), code);
}
```

### F-ab-4. Low, Confirmed (trace), Windows. A shutdown that is allowed and then cancelled leaves "quit asked" set for the rest of the run

- **Where:** `src/ui/interface_zoom/mod.rs:257-266`. `noting_allowed_quit` calls `note_quit_asked()` on the `WM_QUERYENDSESSION` answer. Another app can still cancel the shutdown afterwards (`WM_ENDSESSION(FALSE)`), and nothing clears the flag (`src/ui/startup_failure.rs:83`).
- **How it fails:** if the GPU is lost later in that run, `device_loss.rs:128-129` sees `quit_asked()`, does not reopen the main window, and the app is left with none.
- **Fix:** drop `noting_allowed_quit`. `noting_quit` already covers the real session end.

### F-cd-2. Low, Confirmed (trace), Windows. One failed switch to HDR output tone-maps every later HDR photo to SDR for the session

- **Where:** `src/ui/hdr_image/viewer.rs:290` sets `edr_lost` and nothing clears it. Windows failures are transient: the renderer clears its own flag when HDR is released. Before 9f210de3 this check was always false on Windows.
- **Fix:** clear `edr_lost` when the viewer releases HDR output, or latch it only on macOS.

### F-cd-3. Low, Confirmed (trace). Storage figures stop after the lock-screen log-out

- **Where:** `note_account_files` (`src/ui/disk_state.rs:190`) drops readings for any account other than the one recorded at the last `switch_account`. The lock-screen log-out (`src/ui/background_accounts/deleting.rs:208-241`) falls back to the primary account without going through that path.
- **How it fails:** Manage local storage shows no figures, and the full-disk screen loses "Show cached files".
- **Fix:** record the on-screen account in `start_connection`/`adopt_live`.

### R4. Low, Confirmed (trace). Live story volume keys stop working in the main window after a reset

- **Where:** `src/calls/stream_volume.rs:86-90` keeps the first window's handle and compares by equality. After the reset the main window has a new id.
- **Fix:** compare against `link_inbox::main_window_id(cx)` at key time, or let the kept-window takeover update the stored handle.

### W1. Low, Confirmed (trace), Windows. The `.part` sweep forgets a part it could not delete

- **Where:** `src/export/leftovers.rs` `sweep_parts`. `let _ = fs::remove_file(part)` and then `let _ = fs::remove_file(item.path())` run unconditionally.
- **How it fails:** on Windows, a part an antivirus scanner or the indexer holds without delete sharing is not removed, but its journal entry is. The half-written file stays in the user's export folder for good.
- **Fix:** remove the entry only when the part's removal succeeded or the part is already `NotFound`.

### F-ab-6. Low, Plausible, Windows. `export::copy::settle` gives up at once when the rename is refused

- **Where:** `src/export/copy.rs:39-56` uses a plain `fs::rename`. A scanner briefly holding the finished `.part`, or a player holding last export's file, makes the rename fail. The whole copied part is then deleted and the export reports an error.
- **Evidence:** the Windows runner test `windows_a_copy_onto_a_held_file_fails_and_keeps_that_file` added on this branch pins that nothing is lost when the hold lasts. The swap tests already show a held source fails with error 32.
- **Fix:** `crate::fs_retry::rename_retrying(part, to, fs_retry::LONG)` (the copy runs on the export worker), as the settings writer does with `SHORT`.

### Other Low and Plausible findings

| Id | Where | Failure | Fix |
|---|---|---|---|
| F-ab-2 | `src/net_watch/platform.rs:62-71` (macOS) | An IPv6 router's neighbour entry appearing changes no watched configd key, so a carried-over v6 router is seen only at the next unrelated change. | Run `network_changed()` once more after the resolve, or listen on a `PF_ROUTE` socket. |
| F-ab-3 | `src/single_instance.rs:475-492` (Windows) | `Unserved::drop` calls `flush()` (`FlushFileBuffers`) on the accept thread. An over-cap client with a long argument list that never reads blocks every other launch until it times out. | Drop the explicit flush, or answer busy on its own thread. |
| F-ab-5 | `src/connect/group_calls/join.rs:519-531` | A video chat opened from a link before its chat loads keys its streams by chat 0 for the whole call. Stream video and quality controls find nothing, and it collides with a conference on key 0. | Re-key once the chat id is known, or key streams by group call id. |
| F-ab-7 | `src/export/leftovers.rs` `sweep_parts` | Account A's sweep can delete account B's live part when both export to the same path. | Put a per-run nonce in the `.part` name. |
| F-cd-4 | `apply_disk_paused_found` | A pause made while the saved-pause search is in flight is dropped, or resumed on a low disk. | Resolve only the keys sent with the search. |
| F-cd-5 | `src/connect/history_prune.rs:578-591` | An older topic page that lands after its history was evicted becomes the topic's newest content, and the latest page is never fetched on reopen. | Keep stale page ids at session level, or accept only a first page into a fresh entry. |
| F-cd-6 | `src/video_decode/mod.rs:377-385, 548-555, 884-890` | The heard position and the seek reset each write two atomics without a lock. `clock.follow(heard)` can adopt a stale value and jump the clock or end the clip. | Check the generation in `publish_played`, or bound `follow` by the seek origin. |
| R3 | `src/main.rs:424-451`, `src/ui/link_inbox.rs:31-33` | While `Recover` waits for a device, tray actions (Quit included) and a second launch's activation fail on the old handle and are lost. | Queue them on the kept app and replay them after `reopen`. |
| R5 | `third_party/gpui-pre/src/window.rs` (`into_kept`) | `use_keyed_state` element state (chart zoom, sliders, kit widget internals) is not carried over and comes back at defaults after a reset. | Move the frame's element states into `KeptWindow`, or accept and document it. |

## Checked and found clean

- **Group call join payload off the UI thread:** a payload after leave, rejoin or a share stop is dropped. The engine keeps one pending run per call and the driver's waiting map is cleared on every teardown path. A landed payload re-checks whether it is still wanted.
- **Sound slot generation and the mic test stop:** clean. The large tile publish, the halo arithmetic and the late answer failure are also clean.
- **Frame clock:**
  - Grid math is in saturating i128 nanoseconds. Zero or odd refresh rates give no grid.
  - Per-display grids are rebuilt on a rate change and dropped when their window closes.
  - Ticks that skip their draw are forced to draw after `REPAINT_GAP`, and two idle wakes stop the clock.
  - A failed or cancelled WebM decode frees the half-built store, and a stale result is dropped by the clip id.
- **live-replay:** the wheel hitbox does not block, and its check reads hit-test data from the current frame.
- **Incremental rich parse (`MarkupBlocks::reparse`):** matches a full parse across paste, undo, cross-block deletes, `\r\n` and empty text. The preview cache is keyed on the composer revision, and `find_urls` matches the old scan.
- **KeptWindow takeover:** subscriptions and handles follow the new window, the old platform window is dropped before the new one is noted, and quit during or after a reset works. The removed handoff code is no longer needed.
- **Quit ordering:** the settings flush runs after the windows' drops and always completes (`PlatformScheduler::block` polls before checking the deadline). Writes under a folder being deleted are refused without a race.
- **Export:**
  - Held deletes are released per `ExportId`.
  - A sticker pick replaces one still decoding.
  - A launch during the link inbox start is answered.
  - The OS quit cap uses saturating arithmetic.
- **fix-final-c and -d:**
  - Toasts, update download hold, the Linux quota grace time and macOS important-usage are clean.
  - Video clock arithmetic is clean, and the decode-thread wake neither loses a wakeup nor deadlocks.
  - GPU texture slots are neither freed in use nor leaked.
  - The reply-target index is rebuilt on edit and delete, and the supergroup id math saturates.

## Windows runner tests added on this branch

None of these need hardware. CI's `windows-check` job now runs them on every pull request (`.github/workflows/ci.yml`). The core ones also run in `package-windows`' core tests.

- **A rename held by a real handle:**
  - `export::copy::tests::windows_a_copy_onto_a_held_file_fails_and_keeps_that_file`: the export's final name is held open, the copy fails, the held file keeps its bytes and no `.part` is left.
  - `settings::tests::windows_a_settings_write_onto_a_held_file_keeps_the_old_one`: an atomic settings write onto a held file fails and leaves the old file whole and no temp file.
  - `settings::tests::windows_a_settings_write_lands_once_a_brief_hold_ends`: a hold that ends while the write retries still lands the new settings.
- **A failing ShellExecuteW:** `platform::tests::windows_a_failing_shell_execute_answers_not_opened` calls the real `ShellExecuteW` on a missing file. It returns false directly and through `open_off_thread`. `shell_execute` is now compiled for Windows test builds too (`src/platform.rs`).
- **Timer granularity:** `gpui-pre-windows` `timer_thread::tests::short_waits_are_not_rounded_up_to_the_system_tick` does twenty 1 ms waits on the timer thread's waitable timer. They must take at least 20 ms and less than 150 ms, well under twenty 15.6 ms ticks.
- **The file picker path:** `gpui-pre-windows` `file_picker_tests::a_picked_file_gives_back_its_exact_path` checks that a shell item for a file with a Hebrew and emoji name gives back its exact long path through `file_system_path`, the picker's conversion.

How they were checked here: with the mingw-w64 toolchain, `cargo check --target x86_64-pc-windows-gnu --no-default-features --lib --profile test` and `cargo clippy --target x86_64-pc-windows-gnu -p gpui-pre-windows -p quill --lib --features quill/demo-capture --profile test` both compile them, with no warning in the touched files.

The vendored crate's tests need GPUI's `test-support` feature, so CI runs them the way its QUILL-CHANGES.md says: `cargo test -p gpui-pre-windows -p quill --lib --features quill/demo-capture`.

The Linux code these tests touch is unchanged. The only non-test edit is the `shell_execute` cfg, which is Windows only.

## What did not run

- No Windows or macOS test ran here. The Windows tests above are written for the runner and have not run on one yet.
- F-ab-1 needs IPv6 networking, which this container does not have.
- The macOS-only paths (F-ab-2, the CVDisplayLink vsync source) were read, not compiled.
- None of the fixes are applied, because this review changes no product code.
