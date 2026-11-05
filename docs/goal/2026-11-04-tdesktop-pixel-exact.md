# Goal: Pixel-exact main window after Telegram Desktop (Night)

Status: in progress
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
  - Status: in progress
  - Evidence (2026-11-05 capture): 21 of 22 runs at (0,0). Fixed from tdesktop sources:
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
  - Left: "list date 1" shows +1 from sub-pixel glyph placement inside "10:13PM" (the string's end matches, its start does not); revisit after R3.
  - Region diff (tolerance 24, masks on): rail 1.43, list 1.45, header 0.55, pinned 2.69, history 2.46, composer 0.47, total 2.00 (was 2.11).
- **R3: Glyph rasterisation.**
  - Source: plan, Phase 2 (user: FreeType LIGHT on Linux).
  - Acceptance:
    - On Linux, grayscale glyphs are rasterised by FreeType with light (hintslight) hinting.
    - Blending matches Qt's.
    - `align.py` ink ratios are within 1.00 ± 0.03.
  - Status: in progress
  - Evidence (2026-11-05): FreeType LIGHT raster in `third_party/gpui-pre-wgpu/src/freetype_raster.rs`; Linux defaults gamma 1.0, grayscale contrast 0.
    - Sweep of the same capture: swash + gamma 1.8/contrast 1 (old) total 1.97%; FreeType + 1.8/1 1.97%; FreeType + 1.4/0 1.70%; swash + 1.0/0 1.26%; FreeType + 1.0/0 1.20%.
    - Ink ratio 1.00 on 15 of 21 runs; several runs now differ by 0.05–0.09 mean grey levels (list name 1, header status, message time, voice time), i.e. bit-identical glyphs.
    - Left at 0.91–0.94: list previews (next to fixture mini pictures), "Denis Sexy IT" (the 🤖 is missing, R4), header name.
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
  - Evidence: commits a069fa33b (R1, R2), a2447124e (R3), 5b1314ec2 (R4), bb651d408 and 318014906 (R5), and this document, pushed to `origin/main`.

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
  - `src/ui/{search_ui/chat_search_bar.rs,conversation/*,chat_row/*,synthetic.rs,history/*,reply_pattern.rs,td/*,tdesktop_parity_demo.rs,app.rs,emoji_sprites.rs,emoji_style_ui.rs,chatlist_style.rs,message_text/*}`
  - `src/state/{chat_types.rs,session_apply.rs}`
  - `src/emoji_sets_style/*`
  - `third_party/gpui-pre-wgpu/src/{cosmic_text_system.rs,wgpu_renderer.rs}`
  - `assets/emoji/tdesktop/`
  - `THIRD_PARTY.md`
- Forbidden: committed screenshots or reference crops.

## Current Checkpoint

- Closes: R2 and R3 (sub-pixel leftovers below).

## Current State

- Resolved: R1, R4, R5, R6. R2 and R3 are in progress, with the leftovers below.
- Last relevant evidence (2026-11-05 capture, `align.py` and `diff.py`):
  - 21 of 22 runs are at (0,0). Ink is 1.00 ± 0.01 on 18 of 22 runs.
  - Without masks: total 0.37% at tolerance 24, 0.76% at 8, 3.27% at 0. With `--masks`: 0.34%.
  - Regions: titlebar 0.00, rail 0.59, list 0.32, header 0.09, pinned 0.60, history 0.39, composer 0.46.
- Leftovers, none explained from the sources yet:
  - "10:13PM" in the list: glyphs drift from +1.25 px at the start to 0 at the end.
  - Text after "Wi-" in the DHCP message's third line sits about 0.5 px left of tdesktop's glyphs, while "DHCP-сервер" and "LAN-порта" match.
  - In "Denis Sexy IT", "I" sits +0.4 px and "T" +0.85 px off.
  - The rail's "Edit" label is 1 px right of the reference, though the same centring matches "All chats".
  - Preview runs show ink 0.93 next to the mini pictures; the header name shows 0.94.
  - Some waveform bars are 1 px shorter at the top, within tolerance.
  - The file title's ellipsis sits about 1 px off.
- Blocker: none. Next step: find Qt's glyph positioning rule from a run that differs (QTextEngine positions with FreeType light hinting, lsb/rsb deltas), since HarfBuzz's own advances already match on line 1.

## Material Decisions

- 2026-11-04: fixture pictures are cropped from the reference at run time, never committed (user).
- 2026-11-04: Telegram Desktop's emoji sheets are bundled and become the default set (user).
- 2026-11-04: Linux glyphs go through FreeType with light hinting (user). The donor's desktop uses hintslight, grayscale, according to `kdeglobals` and fontconfig on the capture machine, and the reference has no colour fringes.

## Checkpoint History

- 2026-11-05: captures froze on the first frame because the nested KWin locked with the desktop (logind) and stopped sending frame callbacks; `capture.sh` passes `--no-lockscreen`. `build.sh` builds without `demo-capture` into `target/parity` (`QUILL_NATIVE_ROOT` for a local alsa-lib RPM root).
- 2026-11-05: Phases 2–4 committed (FreeType LIGHT raster with gamma 1.0; the bundled Telegram Desktop emoji set; the fixture cut from the reference with the reply pattern). `cargo test --lib` 7140 passed; `cargo test --features demo-capture --bin quill` (with `LC_TIME=C.UTF-8`) 2728 passed, 1 failed (`every_setting_has_a_place_to_land`, which passes alone 3 of 3, as before); clippy clean; file sizes OK.
- 2026-11-05: Phase 1 committed. `cargo test --lib` 7139 passed; `cargo test --features demo-capture --bin quill` 2726 passed, 1 failed (`every_setting_has_a_place_to_land`, load-dependent as before). Two clock tests (`*_reads_the_pickers_clock`) need a 24-hour `LC_TIME`; they fail under `en_US` before and after this change.

- 2026-11-04: plan approved (`/home/stfu/.claude/plans/linear-snuggling-dahl.md`). Open Sans in `assets/fonts` is byte-identical to `lib_ui/fonts`.
