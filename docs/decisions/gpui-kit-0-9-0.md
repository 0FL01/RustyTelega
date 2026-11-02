# gpui-kit 0.9.0 and gpui-pre 0.3.11

gpui-kit, gpui-base and gpui-component go from `=0.8.1` to `=0.9.0`. GPUI moves
from the gpui-pre 0.3.10 snapshot (zed@08cbb88) to 0.3.11 (zed@f816e1f), and the
optional, unbuilt gpui-fast crates to 0.1.13. All eight crates in `third_party/`
were re-vendored.

## What tdesktop does

Nothing relevant. This is Quill's own dependency policy
(`docs/dependency-updates.md`).

## What changed

### Vendored crates

Commit `1398c875` adds the unmodified registry copies, leaving out the same
files as before (`.cargo-ok`, `.cargo_vcs_info.json`, `Cargo.lock`,
`Cargo.toml.orig`, gpui-pre's `examples/` and `[[example]]` tables). The next
commit three-way applies each crate's diff from `6be84cd7`.

| Crate | What had to change |
|---|---|
| gpui-pre | Upstream's `SingleSubscriberSet` replaces `SubscriberSet<(), _>` for per-window observers, so `KeptWindow` and `live_subscription_counts` use it. Upstream's `FramePacer` and power-aware `minimum_frame_interval` replace the old throttle; `InputRateTracker` stays in Quill's `input_rate.rs`. The removed `shape_line_by_hash` family stays removed. |
| gpui-base | `text_wrapper.rs` stays on Quill's `row_wrap.rs` (see below). `is_blinking` follows upstream's `BlinkCursor` rewrite. |
| gpui-component | Merged cleanly. |
| gpui-pre-apple | Upstream's move from `metal` to `objc2-metal` rewrote the renderer. Quill's HDR, drawable trimming, capture and shader retry were ported. Upstream now returns errors where Quill had patched out panics, so those patches are gone. |
| gpui-pre-macos | `MacWindowState` is in an `Rc`. Quill's frame pacing is kept (see below). |
| gpui-pre-linux | Import and module lines only. |
| gpui-pre-windows | Upstream's low power handling sits next to Quill's GPU error and vsync state. |
| gpui-pre-wgpu | Quill's `Adapters` and device recovery replace upstream's `new_rejecting_software`. |

### Two upstream changes Quill does not take

**`CAMetalDisplayLink` on macOS 14 and later.** Upstream paces each window with a
Metal display link that hands out the drawable and pauses when idle. Quill stays
on `CVDisplayLink` for every macOS version. `link_pace`, the frame waker,
`present_timing` for video and the drawable release all build on it, and the
repaint stall fixed in #1199 shows how much that path depends on its details. A
switch deserves its own change with frame pacing and idle CPU measured before
and after, so it is listed below.

**Live resize wrap cache.** Upstream's width-range cache in `text_wrapper.rs`
works on the character-first wrapper that Quill's `row_wrap.rs` replaced. Quill
keeps its own wrapper, which already resumes from the edited row.

### Quill code

`ThemeColor` is `#[non_exhaustive]`, so two tests in `kit_colors.rs` assign
fields on a default value. Nothing else in Quill needed a change.

### Other updates in this sweep

- `cargo update`: 23 compatible updates, including `zip` 9.2, `ureq` 3.5.1 and
  `ureq-proto` 0.7.
- TDLib (`4c6377a` is still master), FFmpeg (n9.0.2), ThorVG (1.1.3), Rust
  (1.99.1) and Actions were already current. tgcalls is skipped by request.
- Held, rechecked 2026-11-02: moxcms 0.9 (image 0.25.10 and jxl-oxide 0.12.6
  still ask for 0.8), x11rb 0.14 (gpui-pre-linux 0.3.11 still asks for 0.13),
  zbus 6 (ashpd 0.13.13 and ksni 0.3.7 still ask for 5).

## For follow-up work

Use these before writing Quill-side code for the same thing.

Worth doing soon:

- `Theme::cursor_blink` with `CursorBlink::blinking(interval).with_idle_timeout(…)`
  stops the caret after a while with no typing. Every blink wakes a frame, so
  this cuts idle wakeups in a focused composer.
- `App::power_mode` and `on_power_mode_change` report the OS low power mode on
  macOS, Windows and Linux. `ZoomPlatform` forwards them. They could replace
  `src/battery_signal.rs` and the battery watcher.
- `low_power_frame_interval` caps a window's frame rate in low power mode.
  Upstream defaults it to 30 fps. Quill's copy defaults to `None`, so nothing
  changes for video or animation until someone decides on a cap with the video
  player in mind.
- `SendForegroundExecutor` (`ForegroundExecutor::to_send`) runs a `Send` future
  on the main thread from any thread. It can replace hand-made wake paths from
  worker threads to the UI.
- `CAMetalDisplayLink` pacing on macOS 14 and later (see above). Measure idle
  CPU, scroll pacing and video pacing on the old and new paths.
- Test support can observe painted text and alternative text, which makes
  assertions in the demo tests possible without pixel captures.
- Kit `ru` translations: check them against the language packs for strings the
  kit draws itself.

Worth knowing:

- `InputEditsEvent` comes with each `InputEvent::Change` and carries the edits.
  The composer's word bar and draft sync could use it where they compare whole
  texts today.
- `Window::set_client_corner_radii` tells Wayland the corner radii of client-side
  decorations. Quill could use it for its Linux window frame. The zoom wrapper
  already forwards it, scaled.
- `set_icon_resolver` and `resolve_icon_path` pick the path of every icon the kit
  draws, for runtime icon themes.
- `set_native_touch_selection` and `set_selected_text_range`, for IME and touch
  hardware.
- `Keystroke::is_modifier_key`, for the shortcut recorder.
- `SpeechWaveform`'s liquid look, if Quill turns on the `speech` feature.
- Editor and Diff gutter decorations have no use in Quill today.

These fixes reach Quill without any change: the shutdown deadlock in text layout
cleanup, `MessageScroller`'s scrollbar reaching the end of padded content,
submenus opening left only without room, Popover clearing focus,
table and menu accessibility, Windows ignoring its synthetic
mouse moves, Unicode line separators in DirectWrite, and text wrapping that now
works out break points with `icu_segmenter`.

## How verified

- Gate: fmt, both clippies, core and UI tests, license list.
- 17 demo screens in light and dark, compared pixel by pixel with a build from
  before the upgrade. Code highlight, the HDR viewer, media and the poll forms
  match exactly. Text bubbles came out 1 pt narrower in the demo chat. The other
  differences under 0.1% are in small text and emoji. Menus differ because the
  older build predates the poll redesign.
- CI builds and tests Linux and Windows and runs the Metal tests. An independent
  review of the first push found Linux, Windows and the Metal tests did not
  compile; all three were fixed before merge.
- `scripts/test-frame-idle.sh` for the pacing rules. Idle CPU, real video and real
  HDR displays were not measured here.
