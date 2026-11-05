# Goal: Pixel-exact main window after Telegram Desktop (Night)

Status: complete
Source: user instruction 2026-11-04 ("Идём за пиксель пёрфектом", then "Реализовать и коммит пуш потом, так же документируй в репо прогресс в цель"); reference `artifacts/telegram-donor.png`; follows `2026-11-04-tdesktop-pixel-perfect-main-window.md`
Last updated: 2026-11-05

## Objective

The previous goal left 2.11% of the main window differing at tolerance 24, with avatars, photos and the GIF masked out, and called the rest glyph antialiasing and fixture content. A per-run alignment check showed part of it is geometry: seven text runs sit 1 to 3 px off. This goal takes the window to the reference without masks: every text run at shift (0,0), glyphs rasterised the way Qt/FreeType draws them, emoji from Telegram Desktop's sprite sheets, and fixture pictures taken from the reference.

## Execution Directive

Complete the frozen Required Outcomes using the listed Change Envelope and Primary Evidence. Work on the smallest unresolved outcome. Do not add requirements from reviews, tests, tools, speculative risks, or optional source text. Finish when every required outcome is resolved and affected constraints remain satisfied.

## Frozen Contract

### Required Outcomes

Primary evidence for R2–R5: a capture of `ready-tdesktop-parity` (`scripts/tdesktop_parity/build.sh`, then `capture.sh`) checked with `align.py` (text-run shift and ink) and `diff.py --no-masks` (share at tolerances 24, 8 and 0).

- **R1: Strict tooling.**
  - Acceptance:
    - `align.py` reports every text run's best shift and ink ratio against the reference.
    - `diff.py` reports tolerances 24, 8 and 0 and the largest delta per region, and has `--no-masks`.
    - `build.sh` builds the capture binary in its own target directory (a `cargo test` relink no longer breaks captures).
  - Status: verified
  - Evidence: baseline on the previous commit — 7 runs shifted (search placeholder (0,+1), list date (+1,0), header name and status (0,+1), message time (+1,0), GIF time (0,+3), composer placeholder (−1,+1)); ink ratio 1.06–1.22; `diff.py` total 2.11% masked, 7.26% `--no-masks`.
- **R2: Geometry.**
  - Source: plan, Phase 1.
  - Acceptance:
    - `align.py` reports (0,0) for every run.
    - Media timestamps follow `Photo::needInfoDisplay`, `Gif::needInfoDisplay` and the grouped variant.
    - Album previews show up to three mini thumbnails with tdesktop metrics.
    - The quote extras of the reply pattern take the +12 shift.
  - Status: verified
  - Evidence (2026-11-05 continuation capture `separator.png`): all 22 canonical runs at (0,0). `chat_list_stamp` retains the narrow no-break separator of the English locale short time instead of deleting its advance (`FormatDialogsDate`); the existing deterministic date test covers the separator. Earlier source-derived fixes remain:
    - header name and status on 18px lines at top 8 and 28;
    - search field at `filterTop = (54-35)/2` (integer), 18px lines;
    - composer field min height `historySendSize.height - 2*padding` (28, `InitMessageFieldGeometry`), document margin 4 plus placeholder margin 2, 18px lines (the kit input set its own 1.25rem line height, so the line height goes on the textarea);
    - bubble width from shaped words, kerning included (`ceil(400.21)` = 401, where the per-character sum gave 402), which moved the message time;
    - GIF and video info `msgDateImgDelta` higher (`Gif::draw` passes `fullBottom - msgDateImgDelta`);
    - photo time only under the pointer, while sending or failed, or on the last own message (`Photo::needInfoDisplay`);
    - up to three album mini previews with `dialogsMiniPreviewSkip`;
    - row text at `textLeft` 68 with chat type icons at their own offsets (3/1 px) and `dialogsChatTypeSkip`;
    - quote extras shifted by 12;
    - right-aligned labels ceiled (`style::font::width`, `src/ui/td/label.rs`).
  - The date's canonical best shift is now (0,0); internal glyph-phase differences remain (mean grayscale delta 8.71 in this run). No string-specific offset or shared shaping/quantization change was added.
  - Region diff (tolerance 24, masks on): rail 1.43, list 1.45, header 0.55, pinned 2.69, history 2.46, composer 0.47, total 2.00 (was 2.11).
- **R3: Glyph rasterisation.**
  - Source: plan, Phase 2 (user: FreeType LIGHT on Linux).
  - Acceptance:
    - On Linux, grayscale glyphs are rasterised by FreeType with light (hintslight) hinting.
    - Blending matches Qt's.
    - `align.py` ink ratios are within 1.00 ± 0.03.
  - Status: verified
  - Evidence (2026-11-05): FreeType LIGHT raster in `third_party/gpui-pre-wgpu/src/freetype_raster.rs`; Linux defaults gamma 1.0, grayscale contrast 0.
    - Sweep of the same capture: swash + gamma 1.8/contrast 1 (old) total 1.97%; FreeType + 1.8/1 1.97%; FreeType + 1.4/0 1.70%; swash + 1.0/0 1.26%; FreeType + 1.0/0 1.20%.
    - Ink ratio 1.00 on 15 of 21 runs; several runs now differ by 0.05–0.09 mean grey levels (list name 1, header status, message time, voice time), i.e. bit-identical glyphs.
    - Continuation: the low ink came from palette-role mismatches, not raster weight. The header now uses `dialogsNameFg` (`TopBarWidget::paintTopBar`) and ordinary previews use `dialogsTextFg` / `dialogsTextFgActive` (`MessageView::paint`). Fresh canonical ratios are 0.983416–1.005492, all within [0.97, 1.03]; header/preview glyphs are within 0.03/0.22/0.13 mean grayscale levels of the reference. LIGHT and gamma 1/contrast 0 remain unchanged.
  - Also: middle elision and file title widths use the semibold face and shaped widths (`elidew` ceiled), and rail labels centre as `QFixed::toInt` rounds (`td::label::centered_label`).
  - Region diff (tolerance 24, masks on): rail 0.72, list 0.63, header 0.09, pinned 0.87, history 1.58, composer 0.46, total 1.18.
- **R4: Telegram emoji set.**
  - Source: plan, Phase 3 (user: bundle the sheets, default).
  - Acceptance:
    - The tdesktop sheets `emoji_1..8.webp` are bundled as the default "Telegram" set, indexed as `codegen_emoji` indexes `emoji.txt`.
    - Inline emoji use 18 px plus 1 px padding, vertically centred.
    - 🤖 draws in the chat list.
    - Saved choices are kept.
  - Status: verified
  - Evidence (2026-11-05):
    - `codegen` and `lib_base` submodules fetched; `scripts/tdesktop_parity/emoji_index` builds tdesktop's emoji data code against Qt 6 and lists 3900 emoji in sprite order (🤖 = 118: sheet 0, row 3, column 22). `assets/emoji/tdesktop/` holds the sheets (renumbered from 0), the preview and `config.json`.
    - `EmojiSetId::Tdesktop` ("Telegram Desktop") is the default; `bundled::ensure_installed` writes it once as an ordinary pack. A saved choice (System, Noto, Twemoji) is kept; the Noto fallback for systems without a colour emoji font no longer triggers.
    - Inline emoji in chat titles and previews: 18 px with 1 px padding (`emoji_sprites::inline_glyph`), shrunk on the CPU by area averaging as `Qt::SmoothTransformation` does (`emoji_inline.rs`); against the reference the robot differs by 1.97 mean grey levels (GPU minification gave 5.3).
    - The title row has no gap: the verified badge starts at `nameWidth` and the row's top (`drawVerifyCheck`, `rectForName.y()`); badge diff 0.1.
    - Message text keeps its em-space placeholder width for sprite emoji (the reference has none); tdesktop's 20 px advance there is not done.
    - Left in "Denis Sexy IT": the reference has ~0.45 px more between "I" and "T" than HarfBuzz's advances; not explained from the sources.
- **R5: Fixture from the reference.**
  - Source: plan, Phase 4 (user: crop from the reference).
  - Acceptance:
    - With `QUILL_PARITY_DONOR` set, the demo crops avatars, mini thumbnails, the photo, the reply thumbnail, the GIF frame and the reply pattern emoji from the reference.
    - Nothing is committed.
    - The photo is 322 px wide.
    - `capture.sh` sets the variable.
    - `diff.py` masks only the reference's mouse pointer.
  - Status: verified
  - Evidence (2026-11-05): `src/ui/tdesktop_parity_donor.rs` cuts the list avatars, both history avatars (Andrey, Lain), the photo's visible part (declared 320x430 so `photo_current` gives the reference's 320 px), the reply thumbnail, the GIF frame (the badge and time pill taken back out by undoing the `msgDateImgBg` blend, plus the same frame as a one-frame GIF clip) and the six mini previews into a temp folder the demo's media roots allow. `capture.sh` sets `QUILL_PARITY_DONOR`; `diff.py` compares everything unless `--masks`.
    - Userpics drop the kit avatar's border (tdesktop paints the photo in a circle, no frame).
    - Per picture, mean grey-level difference: photo 0.00, reply thumb 0.15, GIF 0.46 (GIF palette), avatars 0.18–0.87.
    - Diff without masks: total 0.46% (tolerance 24), was 7.26% at the start of this goal; history 0.54, list 0.32.
    - The reply pattern: Andrey gets a background custom emoji whose sticker is the alpha read back from the reference's first 20 px copy (`FillBackgroundEmoji` entry 0, opacity 0.32, tint from its strongest pixel).
    - Diff without masks after the pattern: total 0.37%, history 0.39%.
- **R6: Delivery.**
  - Source: user instruction "коммит пуш".
  - Acceptance: committed and pushed to `origin/main`, with this document recording progress.
  - Status: verified
  - Evidence: earlier phases (a069fa33b, a2447124e, 5b1314ec2, bb651d408, 318014906) were pushed. The approved continuation, including permanent test-notification silence and documented progress, is committed as `217ecf20467036bab67db0c30ab3eddc62c6cfe7` and pushed to `origin/main`; `git ls-remote origin refs/heads/main` confirmed that exact SHA. This closure record is a docs-only follow-up.
- **R7: Silent notification audio in tests.**
  - Source: user addition 2026-11-05: disable notification sounds during tests permanently, not through a forgotten manual mute/environment override; document and commit the rule.
  - Acceptance: ordinary test builds cannot reach real notification audio playback; production notifications and saved/system volume settings are unchanged.
  - Primary evidence: notification playback regression using the default device-free `SharedOutput` test backend, plus the UI test suite; rule in `docs/build.md`.
  - Status: verified
  - Evidence: test defaults reuse `SharedOutput::for_test(None)`; explicit software mixers/simulated devices remain usable. Native OS message and incoming-call delivery are both skipped in test builds because their daemon can sound outside Quill's audio output. The new output regression passes (1 test), and the existing incoming-call test deactivates the actual active window and verifies that notification bookkeeping still advances without OS workers (1 passed). The post-fix full UI suite passes 2730 tests (2 ignored). No production audio/notification or saved/system volume settings changed; `docs/build.md` records the permanent rule.

### Constraints

- C1: Geometry and colours from the tdesktop sources, not from measuring the picture (the picture is the check).
- C2: The narrow layout (history pane under 880px) keeps working.
- C3: Existing user settings stay switchable; a saved emoji set or other choice is not overwritten.
- C4: No committed screenshots or crops of the reference.

### Non-goals

- Other screens beyond what the rasteriser and emoji change touch globally.
- The light theme.
- macOS and Windows glyph rasterisation (R3 is Linux only).

## Change Envelope

- Target: main window rendering, its demo fixture, the Linux glyph rasteriser and the emoji set.
- Expected paths:
  - `scripts/tdesktop_parity/*`
  - `src/local_time.rs::chat_list_stamp` (continuation: preserve the locale's narrow separator before AM/PM instead of deleting its advance; direct consumers are chat-list, saved-sublist and search dates, and agent settings timestamps).
  - `src/ui/audio/output.rs::SharedOutput::default` and its existing tests, `src/ui/notifications/spawn_call_notification.rs::{show_notification_as,spawn_call_notification_as}`, existing `ringtone_tests.rs`, `docs/build.md` (user-authorized test-only notification silence; reuse the existing test backend and suppress external OS delivery, no audio refactor or production-setting changes).
  - `src/ui/{search_ui/chat_search_bar.rs,conversation/*,chat_row/*,synthetic.rs,history/*,reply_pattern.rs,td/*,tdesktop_parity_demo.rs,app.rs,emoji_sprites.rs,emoji_style_ui.rs,chatlist_style.rs,message_text/*}`
  - `src/state/{chat_types.rs,session_apply.rs}`
  - `src/emoji_sets_style/*`
  - `third_party/gpui-pre-wgpu/src/{cosmic_text_system.rs,wgpu_renderer.rs}`
  - `assets/emoji/tdesktop/`
  - `THIRD_PARTY.md`
- Forbidden: committed screenshots or reference crops.

## Current Checkpoint

- Closed: R1–R7.
- Next: none. Publish this delivery-only closure record and stop; no further implementation, cache/shaping work or adjacent audit is part of this goal.
- Evidence: current qualified checks/review and implementation/progress commit `217ecf20467036bab67db0c30ab3eddc62c6cfe7` reachable from `origin/main`. The canonical 22-run, ink and notification-silence gates pass. Known unrelated failures are explicitly qualified, not called PASS. Global diff is reported, not a mandatory 0.3% gate.

## Current State

- Resolved: R1–R7. Implementation/progress is published to `origin/main`; the unrelated original user WIP remains outside the commits.
- Current primary artifacts: `/home/stfu/.cache/opencode-tmp/opencode/pixel-exact.2loeuz/{separator.png,align-separator.txt,ink-final.json,diff-final/}`. Raw capture is fresh (2026-11-05 12:50 +0300), 1920×1036, compared against the 1919×1034 reference with top-left cropping; no shaping/raster overrides were set.
  - All 22 canonical runs at (0,0); exact ink range 0.983416–1.005492.
  - No content masks; only the reference pointer excluded. Total 0.36% at tolerance 24, 0.57% at 8, 3.16% at 0; largest delta 255.
  - Regions at tolerance 24: titlebar 0.00, rail 0.59, list 0.30, header 0.00, pinned 0.60, history 0.39, composer 0.46. Heatmap inspected; remaining differences are not a bit-identical-window failure criterion.
- Diagnostic residuals (not additional gates): internal date phase/advance differences, previously documented Wi-/Denis/Edit/elision and waveform differences. The continuation does not change the global shaper/phase quantizer or chase an advisory global percentage.
- `cargo test --lib` is **not PASS** (7139 passed, 1 failed, 23 ignored). `app_theme::tests::custom_themes_are_stored_by_name` expects `lightButtonBgRipple` to be unknown, but the untouched preexisting `src/app_theme/keys.rs` WIP adds it. `ThemeFile::sets_known_keys` therefore accepts that fixture. The failure also reproduces alone and in the external HEAD-plus-original-WIP probe (`test-head-wip-ready.log`, 0 passed / 1 failed); a disposable rustc probe confirms HEAD None versus WIP Some (`wip-key-probe.log`). This unrelated failure is not repaired or swept into the commit.
- Current post-fix `LC_TIME=C.UTF-8 cargo test --features demo-capture --locked --bin quill -- --test-threads=1`: **2730 passed, 0 failed, 2 ignored** (`test-ui-postfix.log`, 1077.25s). It includes the settings-arrival, narrow/wide layout, pull and paragraph-cache tests. Before R7, the targeted `paragraph_cache_scroll` filter passed 6 tests; `local_time::tests::chat_list_stamps` passed 1.
- The actual targeted pull-module filter is `ui::pull_next_ui::tests` (the supplied filename filter selected zero tests). Its isolated run is **not PASS** (5 passed / 1 failed: one unexpected conversation render). An external HEAD-plus-original-theme-WIP probe reproduces the identical assertion and render-count map, also 5 passed / 1 failed (`test-head-pull-ready.log`). All task source changes are absent there and hashes match HEAD (`head-probe-provenance.txt`); only external workspace/dependency paths are adjusted, with no new dependency versions. Both baseline narrow/wide layout tests pass. The current full suite passes this assertion too; no deterministic regression or reason to edit neighboring cache code is established.
- The post-fix notification-output regression and unfocused incoming-call regression each pass 1 test (`test-notification-final.log`, `test-call-postfix2.log`). Permanent silence comes from test-only code, not environment flags. Local verification additionally uses process-local ALSA Null PCM and an isolated session-bus address; user system settings are untouched.
- `cargo clippy --features demo-capture --all-targets --locked -p quill -p quill-calls-sys -p quill-webview-protocol -- -D warnings`: PASS (`clippy-postfix.log`). `cargo fmt --all -- --check`, `bash scripts/check-file-size.sh`, `git diff --check`: PASS. Cargo uses the existing native root's ALSA pkg-config files; no dependencies are added.
- Sol's actual-code review corrected the raw-capture dimension note and found the separate incoming-call OS dispatcher bypassing the first message guard. That P2 is fixed and re-reviewed. Final verdict: approved for the scoped commit/push, with the explicit preexisting library/pull qualifications above; no blocking or other open-scope findings remain. The interrupted verification was not called PASS; the subsequent full run above completed. Source code has not changed since that review; this later closure edit changes documentation only.

## Material Decisions

- 2026-11-05: user additionally requires permanently silent notification tests. Reuse the already-present device-free `SharedOutput::for_test(None)` as test default, rather than muting saved preferences or removing notification decisions. Incoming-call tests also use this output (their tone bypasses `NotificationSounds`); native OS delivery is skipped under `cfg(test)` because daemon sounds bypass the app output. Explicit in-memory mixer/gate tests remain available. Remaining local runs additionally use an external ALSA Null PCM config, verified by `aplay -v` (`null-audio.log`); this process-local belt-and-braces measure is not the permanent invariant.
- 2026-11-05: reviewed continuation approved by the user ("Утверждаю, приступай к реализации, потом коммит пуш"). Preserve preexisting WIP in parity tooling, intro theme keys and the disconnected intro module; it is not part of this continuation. R3's frozen ink gate is ±3%, not the older plan's ±1%; no bit-identical whole-window requirement or mandatory ≤0.3% global diff.
- 2026-11-04: fixture pictures are cropped from the reference at run time, never committed (user).
- 2026-11-04: Telegram Desktop's emoji sheets are bundled and become the default set (user).
- 2026-11-04: Linux glyphs go through FreeType with light hinting (user). Historical inspection found hintslight and no colour fringes in the reference. Current fontconfig has RGB subpixel settings, so it does not establish the old donor's grayscale backend/version; the source-derived LIGHT/blend implementation and direct reference comparison are the evidence, not an invented provenance claim.

## Checkpoint History

- 2026-11-05 closure: scoped implementation/progress commit `217ecf20467036bab67db0c30ab3eddc62c6cfe7` pushed to `origin/main`, remote SHA verified. Closure check resolves R1–R7, preserves the frozen tolerances and constraints, and excludes the original WIP. No further substantive work remains.
- 2026-11-05 R7: reused the device-free output backend and guarded both message and incoming-call OS dispatchers under `cfg(test)`, preserving notification decisions/bookkeeping and production settings. Targeted regressions pass; post-fix full UI 2730 passed / 2 ignored, clippy/format/file-size clean. The known parallel settings failure recurred in an intermediate run; final serialized verification passes it. The isolated pull render-count failure reproduces without this task's changes; no adjacent fix is made.
- 2026-11-05 continuation: preserving U+202F closes the canonical date alignment gate without a shared text-system change. The installed Qt probe (`locale.txt`) returns `10:13 U+202F PM` with that separator (no ASCII spaces) for en_US short time, confirming the source-derived string. Fresh capture: 22/22 zero shifts, ink 0.983416–1.005492, unmasked total 0.36%/0.57%/3.16% at 24/8/0. `cargo test --lib local_time::tests::chat_list_stamps --locked`: 1 passed. The Qt probe confirms that subpixel-flooring and 26.6 truncation differ from GPUI; they do not need to be ported to satisfy the frozen gates. No guessed quarter-space, side-bearing compensation, screenshot offset or gamma resweep was used.
- 2026-11-05 continuation: fresh native-root build/capture reproduced 21/22 zero shifts and unmasked diff 0.37%/0.76%/3.27% at 24/8/0. Scratch evidence: `/home/stfu/.cache/opencode-tmp/opencode/pixel-exact.2loeuz/`. Source-derived palette corrections (`dialogsNameFg` in the header, `dialogsTextFg` for previews) bring all 22 ink ratios into [0.97, 1.03] (previews/header now 1.00); date shift remains +1. A same-face Open Sans 13px Qt/cosmic/FreeType probe rejects LIGHT side-bearing-delta compensation: Qt uses design advances truncated to 26.6. It also exposes `chat_list_stamp` deleting the locale separator entirely; `FormatDialogsDate` instead returns `QLocale::ShortFormat`. Preserve that separator next, then recapture before deciding whether a shared shaping/phase change is necessary. The old main donor's backend/version remains unattributed; the separately downloaded 7.3.5 Pango binary is not proof of its provenance.
- 2026-11-05: captures froze on the first frame because the nested KWin locked with the desktop (logind) and stopped sending frame callbacks; `capture.sh` passes `--no-lockscreen`. `build.sh` builds without `demo-capture` into `target/parity` (`QUILL_NATIVE_ROOT` for a local alsa-lib RPM root).
- 2026-11-05: Phases 2–4 committed (FreeType LIGHT raster with gamma 1.0; the bundled Telegram Desktop emoji set; the fixture cut from the reference with the reply pattern). `cargo test --lib` 7140 passed; `cargo test --features demo-capture --bin quill` (with `LC_TIME=C.UTF-8`) 2728 passed, 1 failed (`every_setting_has_a_place_to_land`, which passes alone 3 of 3, as before); clippy clean; file sizes OK.
- 2026-11-05: Phase 1 committed. `cargo test --lib` 7139 passed; `cargo test --features demo-capture --bin quill` 2726 passed, 1 failed (`every_setting_has_a_place_to_land`, load-dependent as before). Two clock tests (`*_reads_the_pickers_clock`) need a 24-hour `LC_TIME`; they fail under `en_US` before and after this change.

- 2026-11-04: plan approved (`/home/stfu/.claude/plans/linear-snuggling-dahl.md`). Open Sans in `assets/fonts` is byte-identical to `lib_ui/fonts`.

## Completion

- Resolved outcomes: R1–R7, with the current reference capture and permanent notification-silence regressions above.
- Evidence: scratch `/home/stfu/.cache/opencode-tmp/opencode/pixel-exact.2loeuz/`; `align-separator.txt`, `ink-final.json`, `diff-final/`, `test-ui-postfix.log`, `clippy-postfix.log`, and qualified baseline failure logs. This is not a claim of a bit-identical whole window, live-account readiness or release-package validation.
- Constraint/scope check: source-derived palette roles and locale separator; narrow/wide layout and settings checks pass in the final UI suite; production audio/settings and the existing LIGHT/blend/emoji/media behavior are unchanged. Exactly eight scoped code/test/docs files were delivered; no dependencies, screenshots, crops or unrelated WIP were committed. Original WIP hashes remain intact.
- Final status: complete. The implementation/progress push is verified; the follow-up commit only publishes this closure record, with no runtime change or reason to rerun behavior gates/review.
