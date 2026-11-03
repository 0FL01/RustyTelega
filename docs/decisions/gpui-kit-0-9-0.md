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

Done after this upgrade:

- The caret stops blinking, visibly, 10 s after the last focus, edit or move
  (`set_caret_policy`). Every blink woke a frame.
- macOS and Windows take the saver from `App::power_mode`, with no probe. GPUI had
  no public change event, so Quill's copy adds `App::on_power_mode_change`. Linux
  keeps `battery_signal`: GPUI reads only the profile daemon, so a sandboxed app
  would lose the portal.
- Windows are capped at about 30 fps in low power mode, upstream's default. The
  owner chose this knowing it holds video, scrolling and PiP to the same rate.
  Turning "Save Power on Low Battery" off lifts the cap
  (`App::set_low_power_frame_cap`).
- The kit's own strings follow the language pack where the kit has the language
  (ru, it, pt-BR, zh-CN, zh-HK, zh-TW). Quill never called `set_locale` before, so
  they were English for everyone.
- `Keystroke::is_modifier_key` replaces Quill's copy.

Looked at, no use in Quill:

- `SendForegroundExecutor`: `quill::wake` signals already wake waiting tasks from
  any thread without polling.
- `InputEditsEvent`: the composer's checks read the new text, and spellcheck shifts
  by its own comparison. A subscription would add state and remove none.
- `set_client_corner_radii`: Quill's Linux window frame is square
  (`window_border::BORDER_RADIUS` is 0).
- Test support for painted text: `painted_quads` and `painted_underlines` exist,
  painted text does not in this snapshot.
- Icon resolver, touch selection, `SpeechWaveform`, Editor and Diff gutters.

Still open:

- `CAMetalDisplayLink` replaces `CVDisplayLink` in its own change, with Quill's
  pacing fixes moved over. Compare idle CPU, scroll pacing and video pacing with a
  build from before it.

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
