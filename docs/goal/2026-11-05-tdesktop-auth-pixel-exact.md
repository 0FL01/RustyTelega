# Goal: Pixel-exact sign-in flow after Telegram Desktop 7.3.5

Status: active
Source: user instruction 2026-11-05 ("Надо допилить pixel perfect ... процесс экрана входа не соответствует tdesktop"), then "Приступить к реализации и коммит пуш потом"; reference `artifacts/intro-ref/{start,qr,phone,phone-filled,phone-error,settings,qr-back}.png`; follows `2026-11-04-tdesktop-pixel-exact.md`
Last updated: 2026-11-06

## Objective

The whole signed-out flow matches Telegram Desktop 7.3.5 (Tinted, Linux, 100%) at 1920x1036: Start cover, QR, phone and error, code/email, password/recovery/signup, and the intro Settings layer. The custom Quill first-run video/notice/highlights and the centered sign-in card are gone. Geometry, colours, text, assets and transitions come from the tdesktop sources and the captured reference, except for the explicitly recorded owner branding decision below.

## Execution Directive

Complete the frozen Required Outcomes using the listed Change Envelope and Primary Evidence. Work on the smallest unresolved outcome. Do not add requirements from reviews, tests, tools, speculative risks, or optional source text. Finish when every required outcome is resolved and affected constraints remain satisfied.

## Frozen Contract

### Required Outcomes

Primary evidence for reference screens: a fresh `scripts/tdesktop_parity/build.sh` then `capture.sh` capture of the matching demo, checked with `align.py --profile intro-*` (text-run shift and ink) and `diff.py --profile intro-*` (tolerances 24, 8 and 0). A reference screen passes when every static run is at (0,0), ink is in [0.97, 1.03], the diff report is recorded, and the heatmap shows no unexplained structural difference. The QR matrix and the Settings version/update values are the only documented dynamic masks; they are not used to hide layout differences. Source-only steps are not called pixel-exact.

- **R1: Reference and tooling.**
  - Acceptance:
    - `profiles.py` keeps the existing `main` profile unchanged and has `intro-start`, `intro-qr`, `intro-phone`, `intro-phone-filled`, `intro-phone-error` and `intro-settings` profiles.
    - `intro-settings` has stable text runs for the title, Language, Connection type, Interface scale, theme labels and FAQ; version/update text is excluded as dynamic.
    - `align.py` and `diff.py` work through `--profile`; a reference/capture size mismatch is an error.
    - `capture_reference.sh`, `eis_input.py`, `fetch_reference_binary.sh` and the `reference/*.txt` scenarios are committed; capture output and the official binary stay untracked.
    - `capture.sh` accepts the demo kind and size instead of hard-coding `ready-tdesktop-parity` and 1919x1034.
  - Status: verified
  - Evidence: R1 tooling is included in the checkpoint commit. `python3 -I scripts/tdesktop_parity/test_profiles.py` passes (one test, four mismatch cases); `bash -n scripts/tdesktop_parity/{capture,capture_reference,fetch_reference_binary}.sh` passes. An AST comparison against HEAD `a4861bc18f0000c5572d2980a158042519c0fba9` confirms the main regions, 22 runs, masks and pointer are unchanged. All seven profiles self-compare successfully at (0,0), ink 1.00 and diff 0 at 24/8/0 (tool validation, not Quill pixel parity). `intro-settings` has 16 stable runs. Fresh official reference capture: `/home/stfu/.cache/opencode-tmp/opencode/intro-r1.0qFbjW/reference/`; archive SHA-256 validation and the full `capture_reference.sh intro` scenario succeeded. Fresh Quill captures at 1920x1036 and 1919x1034 prove parameterized size and kind. References, binaries and Python bytecode remain untracked/ignored.

- **R2: Intro shell and Start.**
  - Acceptance:
    - A new `src/ui/intro/` draws the signed-out flow; `first_run_*`, `onboarding.rs`, the replay entry, their demos, strings and assets are removed or retired.
    - All direct consumers are updated: `app_render/mod.rs`, `app.rs`, `app_demo.rs`, `esc_stack.rs`, `agent_driver_surface.rs`, `disk_state.rs`, `settings_catalog/entries/advanced.rs`; disk and startup-failure screens remain usable inside the intro shell and the agent UI still cannot sign in.
    - Start shows the 208px cover, the cover gradient and tdesktop `intro_left`, `intro_right` and composite plane assets, the exact tdesktop title/description, and a 300x42 radius-6 Next button; Settings is hidden on the cover.
    - `src/ui/td/st/intro.rs` is wired and has the source formulas and constants; new icons are added through `sync_icons.sh`; the WIP palette keys are wired.
    - The layout follows the source at 1920x1036, 800x600 and the 380x480 minimum.
  - Status: verified
   - Evidence: `IntroUi` replaces first-run/onboarding and their direct consumers; old modules/replay/string file are removed and the unreferenced onboarding video is retired (not included by the app). Original palette/metrics WIP is preserved and wired. Source masks/cover, disk/startup screens, agent sign-in exclusion and test-DC indication are retained. Fresh `intro-start`: all 4 runs (0,0), ink 1.00; total **0.02% @24 / 0.02% @8 / 15.29% @0**. The cover's strict differences are channel rounding (maximum 6), not structure. Capture/reports: `/home/stfu/.cache/opencode-tmp/opencode/intro-qr.6JwtgI/{start-source-font.png,align-start-source-font.txt,diff-start-source-font.txt}`. Geometry test covers the reference and the two required small sizes; remaining shared transitions/regression qualification follow with R9.

- **R3: QR screen and Start -> QR.**
  - Acceptance:
    - Start submits a live QR request and shows the QR step.
    - QR has a 204x204 outer widget centred at contentTop-18. The source-sized white plate is 188x188 for this reference (41 modules, floor(180/41)=4px modules, 12px skip on each side), radius 8; the matrix has a 180px maximum and integer module size, rounded modules/finder patterns, and the 44px `#40a7e3` disc with the tdesktop plane centred.
    - Title, three numbered steps, "Log in using phone number", conditional "Log in using passkey", 56x56 back and SETTINGS are at the tdesktop positions.
    - The dynamic mask covers only the matrix inside the plate; plate, disc, plane, title, steps and links remain static and compared.
    - TDLib does not allow `setAuthenticationPhoneNumber` from `WaitOtherDeviceConfirmation`; use the owner-authorized guarded pre-auth `destroy`/fresh-client reset, reusing the existing closing lifecycle, never a fake state. Show/submit Phone only after the new client reports `WaitPhoneNumber`. Attempt new-build live Start -> QR -> Phone -> submit without exposing/extracting credentials. Per the parent's continuation decision, if credentials are unavailable, record the exact limitation and use pinned TDLib source plus deterministic guard/stale tests; this is not a live PASS.
  - Status: verified
  - Geometry decision: parent explicitly adopts source/reference geometry: outer widget 204px, reference matrix 164px and painted plate 188px. `PrepareQrWidget` and reference white-pixel bounds at y=400 (x=866..1053) agree. This supersedes the mistaken planned plate size; no other acceptance, mask or tolerance change is authorized. The dimension issue is resolved, not a blocker.
   - Evidence: Production QR -> Phone now detaches/increments the screen generation before the guarded `destroy`, drains discarded clients without Ready side effects, waits for confirmed Closed and reserves the database through fresh-client preparation/confirmation. Only a real fresh `WaitPhoneNumber` is attached; stale/failing completions stay on QR with an error. Core `reset_qr_login` filter: **3 passed**, covering guards, discarded late Ready/Closed and fresh Phone vs rejected Ready. UI `intro_reset` filter: **3 passed**, covering database reservation/handoff, production failure/Back and stale completion. `test_profiles.py`: **2 passed**, including matrix masking with the static disc preserved. Evidence: `/home/stfu/.cache/opencode-tmp/opencode/intro-qr.6JwtgI/{core-reset-tests.log,ui-reset-final.log,profiles-tests.log}`. Fresh QR: all 8 runs (0,0), ink 1.00; **0.08% @24 / 0.10% @8 / 0.24% @0**. Reports/capture `qr-source-font.png`, `align-qr-source-font.txt`, `diff-qr-source-font.txt` there; the heatmap has only known raster/AA differences, not structure. Safe new-build attempt: `--embedded-credentials` -> `none`; isolated HOME/XDG `QUILL_TDJSON_PATH=/usr/lib/quill/lib/libtdjson.so target/parity/debug/quill --connect-smoke` -> exit 1, **`SMOKE_BLOCKED missing-credentials`** (`embedded-credentials.txt`, `new-build-smoke.log`). No credentials were extracted. This is source/deterministic qualification under the parent continuation, **not live reset or number-submit PASS**. Earlier installed-binary close/reopen returned other-device; that negative evidence remains at `/home/stfu/.cache/opencode-tmp/opencode/intro-live.nlFQ1Y/`. Pinned TDLib version/commit are 1.8.69 / `4c6377aa7386c39217878a9a56d581922ad37ca4`; destroy removes that client's SQLite/binlog, not Quill/account roots or media trees.

- **R4: Phone and error.**
  - Acceptance:
    - Country field 300x61, code field 64 and phone field 225 at +6, 16px text, 300/20 description, title at contentLeft+4/contentTop+1, Next at +266, QR link, back and SETTINGS follow the source.
    - Existing phone formatting and country search are reused; paste, caret, focus, IME and secrets are preserved.
    - Error text, red underline and shake match; flood and banned-number handling remain.
    - Demo fixtures reproduce the reference's Sweden/+46 and 1234 states.
  - Status: verified
   - Evidence: source phone fields, split native input, Sweden formatting, country search and error states are implemented. The multiline country-code editor wrapped the reference; replaced it with the existing centred single-line `InputState`, retaining `TextareaState` for the national number and a shared private caret remapper for these two native modes. Fresh post-fix captures in `/home/stfu/.cache/opencode-tmp/opencode/intro-phone.Y42kfY/{phone,phone-filled,phone-error}-reviewed.png`: phone **0.07% @24 / 0.08% @8 / 0.18% @0**, filled **0.08% / 0.09% / 0.19%**, error **0.12% / 0.14% / 0.25%**; all 11/11/12 runs (0,0), ink [1.00,1.01]. Corresponding `align-*-reviewed.txt`, `diff-*-reviewed.txt` and heatmaps are retained there. The images are byte-identical to the earlier `*-final.png`, whose heatmaps show glyph rasterization fringes, not unexplained structure. A deferred, demo-only blink policy reproduces the reference's hidden-caret phase after appearance initialization; production timing and masks are unchanged. The real end-caret is set in fixtures. Paste/caret/country-popup/Escape and incomplete +4 regression pass: national digits are not reinterpreted as +41. Review fixes preserve flood metadata on editing and move the banned box into the existing modal registry; the actual Edit -> flood locked -> banned Help/OK -> OK dismissal test passes. Full-goal review/regression remains.

- **R5: Back to phone from later steps.**
  - Acceptance:
    - Presentation history lets Code/Email/Password/Registration return to the Phone form where tdesktop does; the phone form is not shown while the underlying auth state cannot accept a number.
    - `setAuthenticationPhoneNumber` is allowed from the later states listed in `schema/td_api.tl:12187-12192` only when no auth query is pending.
    - A production-handler test covers Password -> Back -> Phone -> submit and proves the request is sent with no stale auth query.
  - Status: verified
   - Evidence: UI and driver share the source-state capability; later states enter Phone only when idle/running, never QR. Phone remains visible while its query is pending and on error; unsolicited auth progress or explicit exit releases it. Back clears obsolete secrets/value errors but preserves flood metadata and suppresses hidden old code links/submits. The final production-driver test rejects Phone during password submission, handles its error, sends the number with no stale password query, then verifies `PHONE_NUMBER_INVALID` keeps Password and clears pending: **1 passed, 7161 filtered**, `back-error-core-final.log`. The unrelated original WaitPhone driver test is restored unchanged and passes separately: **1 passed, 7161 filtered**, `wait-phone-core-unchanged.log`. Actual Back UI plus injected submitted/error/progression states retains Phone and its accessible error, then shows injected Email: **1 passed, 2731 filtered**, `back-error-ui-accessible.log`. These are deterministic driver/UI surfaces, not live end-to-end submission. The former `back-error-core.log` did not exercise the error block; that evidence claim is superseded by the corrected placement and final runs.

- **R6: Code and Email.**
  - Acceptance:
    - Code uses one hidden `TextareaState` for input/IME/paste and painted cells 40x50, gap 10, 4px border, radius 8; the count comes from TDLib and a complete code submits.
    - Telegram, SMS, call, Fragment, resend countdown, email setup and email code remain supported.
     - The shell geometry matches the source; there is no reference capture, so the evidence is source-derived geometry and behaviour, not a pixel claim.
  - Status: verified
  - Evidence: `src/ui/intro/{code,email}.rs` keeps the one native editor, TDLib-supplied 1–32 digit counts, fullwidth IME normalization, filtered paste/caret mapping and pending-submit guards; missing lengths retain manual native input. Source-derived email icon/field and code-cell geometry are implemented. `cargo test --features demo-capture --locked --bin quill failure_card_tests -- --test-threads=1`: **13 passed, 2723 filtered**, including actual native paste/composition and both code lengths. Existing email/Fragment/resend driver tests pass within the qualified core run. Six later-step captures in `/home/stfu/.cache/opencode-tmp/opencode/intro-delivery/captures/` are inspection evidence only: no Code/Email reference or pixel-exact claim exists.

- **R7: Password, recovery and Signup.**
  - Acceptance:
    - Password/recovery use the same step shell; existing submit, zeroize, recovery, reset and terms paths remain.
    - Signup keeps names, terms and photo handling with intro geometry; no terms acceptance means no registration request.
    - Back follows R5.
     - No reference captures exist; evidence is source-derived geometry and behaviour, not a pixel claim.
  - Status: verified
   - Evidence: Source-derived Password/recovery/Signup placement reuses existing submit/recovery/zeroize and login-email reset owners. Signup's required current terms are explicitly accepted in the modal before register; an optional generation-bound native picked/cropped photo is held until Ready and discarded on Back. The current **13/13** failure-card/UI tests include recovery secret cleanup, required-terms acceptance, a platform picker response decoded into the real crop, stale photo rejection and Back cleanup. Existing registration/secret driver tests pass in the core suite. No new pre-auth cloud-account deletion path was added. `intro-{password,recovery,signup}-delivery.png` in the delivery capture directory are source-derived inspection artifacts, not reference/pixel claims.

- **R8: Settings::Intro.**
  - Acceptance:
    - SETTINGS opens a 392px-max layer with a 56px top bar and radius 8.
    - Language, Connection type, update block, system/window frame, Interface scale, themes/accents and FAQ follow `settings_intro.cpp`, with its conditions.
    - Existing Quill language, proxy, zoom, theme, updater and FAQ owners are reused; pre-auth availability is proved.
     - Dynamic version/update text is excluded; the rest is captured as `intro-settings`.
  - Status: verified
  - Evidence: Parent continuation explicitly authorizes functional Auto-update and Install beta toggles and the minimum updater/installer/preferences expansion. Stable + automatic remains the default; beta defaults off and prereleases require explicit opt-in. Preserve signature verification, CI and release behavior. The earlier stable-only/no-switches policy is superseded only to this extent; update its decision document with implementation.
   - Evidence: Actual `Settings::Intro` capture `intro-delivery/captures/settings-delivery.png` (1920x1036): **16/16 runs (0,0), ink 1.00; total 0.06% @24 / 0.19% @8 / 1.57% @0**. Reports are `settings-delivery-{align,diff}.txt` and `settings-delivery-diff/heat.png`; only version/update text is excluded. Native pre-auth UI tests exercise persisted beta/automatic toggles, scale 175%, themes/accent, Language/Escape, visible proxy controls, Add Proxy editor and Cancel/Escape focus return; existing core language/proxy tests prove requests before authorization. A real scripted FAQ click with a temporary OS-helper shim receives the existing `https://telegram.org/faq#general-questions` URL (`faq-opened-url.txt`, `faq-boundary-result.txt`); this proves hand-off, not browser/network access. Beta policy and installer checks pass in the core suite; installer hand-off locks the switches, and a held download retains exclusive ownership until its worker exits.

- **R9: Regression and delivery.**
  - Acceptance:
    - `ready-tdesktop-parity` stays at no worse than 0.37% at tolerance 24, all 22 runs at (0,0) and ink in [0.97, 1.03].
    - `cargo fmt --all -- --check`, core and UI clippy with `-D warnings`, `cargo test --lib`, `cargo test --features demo-capture --bin quill` with `LC_TIME=C.UTF-8`, `pull_next_ui_tests`, `paragraph_cache_scroll`, `scripts/check-file-size.sh` and `scripts/check-hotspots.sh` are recorded.
    - Scoped changes are committed and pushed to `origin/main`, the goal is updated, and the original WIP is preserved.
    - The palette-fixture correction's code-bearing GitHub Actions run is watched to completion and reported by SHA/job; a docs-only skipped run is not CI evidence.
  - Status: in_progress
  - Evidence:
     - `cargo fmt --all -- --check`, core `cargo clippy --no-default-features --all-targets --locked -- -D warnings`, and UI `cargo clippy --features demo-capture --all-targets --locked -p quill -p quill-calls-sys -p quill-webview-protocol -- -D warnings` pass. Latest UI correction is checked in `signup-switch-clippy.log`; core has no subsequent changes.
     - The previous `cargo test --lib --locked` result was **7139 passed, 1 failed, 23 ignored** (`delivery-core-suite.log`). Its palette failure is **task-caused**, not inherited: the committed intro adds `lightButtonBgRipple`, invalidating the test's unknown-key fixture. CI run [42214599713](https://github.com/0FL01/RustyTelega/actions/runs/42214599713), job `127881414134`, `tests (core + replay)` confirms it. Replace only that fixture with `notARealPaletteKey`, retaining the refusal assertion/importer; current targeted/full results and a new code-bearing CI run follow.
     - Correction: only the unknown-key fixture now uses `notARealPaletteKey`; importer and assertion are unchanged. `cargo test --lib app_theme::tests::custom_themes_are_stored_by_name --locked`: **1 passed, 7162 filtered** (`palette-fixture-targeted.log`). `cargo test --lib --locked`: **7140 passed, 0 failed, 23 ignored**, aggregate PASS in 26.25s (`palette-fixture-library.log`). Both use the established native pkg-config/sysroot environment and `LC_TIME=C.UTF-8`; fmt and diff whitespace checks also pass.
     - `LC_TIME=C.UTF-8 cargo test --features demo-capture --locked --bin quill -- --test-threads=1`: **2734 passed, 0 failed, 2 ignored** in 1068.37s (`ui-suite-delivery.log`). That full binary predates only the final three-line account-switch cleanup; current `failure_card_tests` covers that owner correction: **13 passed, 2723 filtered**, `signup-switch-owner.log`.
     - `pull_next_ui_tests` filename filter selects zero (not coverage); actual `ui::pull_next_ui::tests`: **5 passed, 1 known preexisting isolated render-count failure**. `paragraph_cache_scroll`: **6 passed**. `bash scripts/check-file-size.sh`: PASS, limit 1000. Parity profile tests: **2 passed**. `bash scripts/check-hotspots.sh` passes under the documented `Hotspot-change` commit trailer for the required shared owners; it reports the palette match-size warning rather than requiring an unrelated palette refactor (`hotspots-delivery.log`).
     - Product commit `a05289e73bd6bf041a58a51b6eebf63c95fca8cd` is pushed to `origin/main`; `git ls-remote origin refs/heads/main` equals local HEAD (`product-remote-verification.txt`). The original scoped WIP is included; the staged-path audit excluded references, binaries, tokens, pycache, dependencies and CI changes.

### Constraints

- C1: Geometry, colours, strings and assets come from the tdesktop sources and the 7.3.5 reference, not from screenshot nudges. Profile boxes and masks are test fixtures only.
- C2: The narrow main-window layout (history pane under 880px) keeps working.
- C3: Existing user settings stay switchable; saved emoji, theme, language and window settings are not overwritten.
- C4: No committed screenshots, reference crops, QR tokens, official binary or workdir.
- C5: TDLib remains the source of truth for auth; no new MTProto implementation and no fake production auth path.
- C6: The owner chose a full tdesktop visual copy, including the "Telegram Desktop" title, the "official Telegram Desktop app" description, the official paper-plane and removal of the Quill third-party notice. This overrides the previous 2.2/2.4 compliance claims in `docs/decisions/codex-legal-compliance.md` and `PRODUCT.md`; the override and the remaining risk are recorded, and no continued compliance is claimed.
- C7: No new dependencies; QR uses the existing `qrcode` and `peer_qr::qr_path`, input uses existing `TextareaState`, and settings reuse existing owners.
- C8: Settings::Intro must work before authorization; if an owner requires a session, only the minimum seam is added.
- C9: Destructive reset is allowed only when the user switches from intro QR to Phone, with no authorized session/account, Ready/confirmed QR success, or pending auth submit. Destroy only that pre-auth TDLib instance/database; preserve Quill settings, saved accounts, credentials, media and other user data. Discard/invalidate the old client generation before reset so its late auth success cannot authorize the account. Wait for confirmed closure and a new `WaitPhoneNumber`; otherwise remain on QR with an honest error/state.

### Non-goals

- The main window beyond the regression gate.
- The light theme and other screens.
- The full post-auth Settings.
- macOS/Windows-specific glyph or window behaviour.
- A fake/scripted auth backend; demos inject states, production handlers and deterministic tests prove guards, and a live check is attempted subject to the recorded credential limitation.
- A full screenshot-crossfade transition; the source durations and reduced-motion behaviour are enough.

## Change Envelope

- Target: the signed-out intro shell, its demo fixtures and parity tooling.
- Expected paths:
  - `src/ui/intro/**` (new shell, cover, steps, QR, phone/country, code, email, password, signup, settings)
  - `src/ui/{app.rs,app_demo.rs,app_render/mod.rs,esc_stack.rs,agent_driver_surface.rs,disk_state.rs,mod.rs,quill_assets.rs}`
  - `src/ui/{onboarding.rs,first_run_ui.rs,first_run_state.rs,first_run_tests.rs,first_run_demo.rs}` (removed)
  - `src/settings/{mod.rs,first_run.rs}` (retired if no consumer remains)
  - `src/ui/td/{st/intro.rs,st/mod.rs,icon.rs,label.rs}`
   - `src/app_theme/{keys.rs,embedded.rs}`
   - `src/app_theme/tests.rs` (R9: the delivered palette invalidates the negative fixture's previously unknown key; replace only that fixture, preserving import refusal)
  - `src/ui/{auth_ui.rs,signin_ui.rs,auth_state.rs,auth_recovery.rs,auth_registration.rs}`
   - `src/connect/auth.rs`, `src/telegram/requests/auth.rs`
   - `src/ui/notifications/mod.rs` (R5: apply confirmed live auth progress to pending Phone presentation; no notification-policy change), existing dialog registry consumers for the banned-number modal
   - `src/auth.rs` (R5 shared source-state capability for the existing UI/driver boundary)
   - `src/phone.rs` (R4 source Sweden groups and national placeholder at the existing formatter owner), `src/quill_assets.rs` (actual root-level first-run icon consumer), `src/ui/appearance_power.rs` (internal font-family chooser filter)
  - `src/ui/peer_qr/qr_path.rs` (shared QR path only; no unrelated peer-QR changes)
  - `scripts/tdesktop_parity/*`, `lang/en/*.strings`, `docs/decisions/tdesktop-intro.md`, `docs/decisions/codex-legal-compliance.md`, `PRODUCT.md`, `THIRD_PARTY.md`
  - `docs/decisions/tdesktop-pixel-contract.md` (direct tooling consumer: explicit main-reference crop after strict size validation)
  - `src/connect/closing.rs`, `src/ui/{connect_ui.rs,background_accounts/closing.rs}` and nearest tests (minimum guarded destructive reset lifecycle seam)
  - `src/connect/live.rs` and nearest `connect` tests (discarded-auth drain and fresh-client Phone confirmation at the existing live-client boundary)
   - `src/ui/settings_window/sidebar.rs` and `src/ui/connect_ui_tests.rs` (intro fallback Settings close/focus direct consumer and regression test)
   - `src/ui/background_accounts/mod.rs` (reserve the reset's pending native open so switches/background clients/deletion cannot race its database ownership)
   - `src/ui/{peer_qr/mod.rs,security/passkey_login.rs}` (shared QR-path visibility and retirement of the replaced QR link presentation; existing passkey handlers remain)
   - `src/{updater,update_install,update_preferences}/**` and their existing direct UI owners (R8: explicit beta opt-in and automatic-update preferences only; preserve default stable and signature verification)
    - `docs/decisions/update-button.md` (authorized explicit opt-in policy)
    - `src/settings/{updates.rs,appearance.rs}` (device update consent; source 50–300% scale and the existing automatic/default scale choice), `src/ui/appearance/new_accent_picker.rs` and `src/ui/app_theme/chooser.rs` (direct scale/preview consumers)
     - `src/ui/{profile_photo_sources.rs,photo_editor_profile.rs,chat_list/sidebar.rs}` (existing photo picker/crop/upload and retained auth-form consumers), `assets/tdesktop-intro/email*.png` (source-animation final frame, not a screenshot)
     - `src/ui/accounts.rs` and its nearest switch regression test (R7: discard pending signup UI/photo ownership before adopting another account's Ready client)
  - `src/ui/td/{fonts.rs,fonts/intro.rs}` and the font-picker list consumer (R2/R3 source-font collision: an intro-only in-memory family alias of the existing bundled faces, without replacing main/user fonts)
  - `assets/tdesktop-icons/**`, `assets/onboarding/**` (retired)
- Forbidden: committed screenshots, crops, QR tokens, the official binary or its workdir; unrelated user WIP.

## Current Checkpoint

- Reopened R9: the task-caused negative fixture is corrected, targeted/full library tests pass, and narrow Sol review has no findings; commit/push and watch the new code-bearing CI run to completion. No importer, runtime, notification or release-workflow changes are required.

## Current State

- Resolved: R1–R8 retain their source/reference/deterministic evidence and review. R9 is reopened by the real code-bearing CI palette-test failure; the earlier inherited-failure classification and completion claim are superseded. No live new-build auth claim. Current native binary reports `--embedded-credentials`: **none**. Credentials are absent from the environment/local files; they were not extracted. Historical installed-binary live evidence is not new-build PASS.
- Current R9 main capture: `intro-delivery/captures/main-delivery-full.png` at **1920x1036**, then explicit historical donor crop `1919x1034+0+0` to `main-delivery.png`. **22/22 runs (0,0), ink [0.98,1.01], total 0.36% @24 / 0.57% @8 / 3.16% @0**. `main-delivery-{align,diff}.txt` and `main-delivery-diff/heat.png` are the reports. The fixture alone gives the real native caret a 3-second phase; production timing/focus and all main profile boxes/masks remain unchanged.
- Current checks use the recorded native pkg-config/sysroot and `LC_TIME=C.UTF-8`. The task-caused stale palette fixture is fixed without changing import validation. Targeted test: **1 passed, 7162 filtered**; full library: **7140 passed, 0 failed, 23 ignored**, aggregate PASS. Earlier strict clippy/fmt/file-size and UI evidence is retained; fmt also passes after this one-line test correction. Narrow Sol `ses_e4f986e9cffecPHM5ZZJshpm0V` reads the actual code/CI failure and local logs: no blocking or nonblocking findings, importer/assertion intent preserved. The correction's code-bearing CI is pending. UI evidence remains **2734 passed, 2 ignored**, with the subsequent **13-test** switch-owner correction; isolated pull retains its separately qualified preexisting render-count failure.
- The first full UI attempt was interrupted, not PASS: it had reached Settings tests, not hung in viewer cover (both cover tests passed). GDB showed active Taffy layout; no viewer runtime change was made. Its three in-scope regressions were corrected: accessible startup-failure heading/body and the old 100%-minimum clamp expectation. The complete final-caption UI run subsequently passed **2734/0/2**; the final nonvisual switch-owner cleanup is covered by the current **13/13** affected tests, not falsely attributed to that older full-suite binary.
- Preserved: all 124 original palette role declarations and all six original 124-value palette prefixes compare equal to the inherited WIP. Five source Settings palette roles were appended. Permanent test-notification silence, existing signatures/CI/release and user data remain untouched; no dependencies were added.
- Review corrections: the Connection subpage mounts its existing proxy owner, guarded signup picker completion reaches the real crop, held download workers retain exclusive ownership until completion, and source later-step copy/input captions replace legacy copy. Source recovery instructions omit the unavailable email pattern rather than inventing server metadata. Updater cancellation/installation and disk-held resume checks each pass **1 test**. `captures/late-steps-delivery.png` and the six full `intro-*-delivery.png` are source-only inspection, not pixel qualification.
- Intro runtime rereview: `ses_e4f986e9cffecPHM5ZZJshpm0V` closes all four P2 mechanisms and the cross-account signup-photo P1; those runtime closures remain valid. The common switch cleanup discards photo/submission/terms-layer state before both adoption and startup. The real-switch regression stops at the existing unknown-account failure after cleanup and, together with inspected branch dominance, proves owner cleanup, **not native adoption or sender-level/live execution**. This does not approve the subsequently identified palette-test failure or replace the new narrow correction review and code-bearing CI.

## Earlier checkpoint record (superseded by Current State)

- Resolved: R1 tooling (pushed `9de42cc17acd9d0fb2deeeaa6b9fc1a64c20e135`); R2–R5 retain their static/source/deterministic evidence. R6/R7 source-derived forms and tests are implemented. R8 layer and opt-in update policy are implemented; capture and full R9 qualification remain. No delivery/completion claim.
- Existing WIP: original palette/intro metrics are preserved and wired. Shell/cover, source masks, first-run retirement, production guarded reset and isolated source-font registration are uncommitted. Fresh confirmation and database-open ownership are wired; unavailable live credentials are explicitly qualified under R3.
- Targeted reset/font checks use `PKG_CONFIG_PATH=/home/stfu/.cache/opencode-tmp/quill-native-build/root/usr/lib64/pkgconfig PKG_CONFIG_SYSROOT_DIR=/home/stfu/.cache/opencode-tmp/quill-native-build/root LC_TIME=C.UTF-8`. `cargo test --lib reset_qr_login -- --test-threads=1`: **3 passed, 7159 filtered**. `cargo test --features demo-capture --locked --bin quill intro_reset -- --test-threads=1`: **3 passed, 2725 filtered**. The same UI command with `intro_alias_keeps_the_source_tables_and_valid_font_checksum`: **1 passed, 2728 filtered**. Logs are in `/home/stfu/.cache/opencode-tmp/opencode/intro-qr.6JwtgI/`. These are narrow checks, not full R9.
- Checkpoint review fixes: pointer-close of fallback Settings now routes through the intro close/focus owner; the agent hard-layer registry uses `intro-settings` instead of the retired first-run name. With the same `PKG_CONFIG_PATH`/`PKG_CONFIG_SYSROOT_DIR` above and `LC_TIME=C.UTF-8`, `cargo test --features demo-capture --locked --bin quill <filter> -- --test-threads=1` passes each of `intro_settings_pointer_close_dismisses_and_restores_focus`, `every_listed_layer_exists_and_no_layer_is_both`, and `source_geometry_at_reference_and_small_windows`: **1 test each, 2724 filtered out each**. Logs: `/home/stfu/.cache/opencode-tmp/opencode/intro-{settings-close-test,agent-layer-test,source-geometry-test}.log`. Fmt and diff whitespace checks pass after the fixes. These are targeted checks, not the full mandatory R9 suite.
- Historical pre-implementation phone baseline vs `artifacts/intro-ref/phone.png`: 1.92% total at tolerance 24; text 18.44%, country 11.87%, fields 6.28%, error 91.18%, button 58.56%, link 12.12%; 7 of 11 runs shifted. Artifacts: `/home/stfu/.cache/opencode-tmp/opencode/intro-r1.0qFbjW/{phone-tooling.png,phone-tooling-align.txt,phone-tooling-diff.txt}` (totals 1.92% @24 / 1.96% @8 / 1.99% @0; baseline failure, superseded by R4 captures).
- Historical R1 main regression, not qualification of the current uncommitted UI: build with `QUILL_NATIVE_ROOT=/home/stfu/.cache/opencode-tmp/quill-native-build/root scripts/tdesktop_parity/build.sh`, `capture.sh <bin> <main-full.png>`, explicit donor crop `magick <main-full.png> -crop 1919x1034+0+0 +repage <main-parity.png>`, then align/diff. All 22 runs (0,0), ink [0.98,1.01]; total **0.34% @24 / 0.48% @8 / 1.83% @0**. Artifacts: `/home/stfu/.cache/opencode-tmp/opencode/intro-r1.0qFbjW/{main-parity.png,main-final-align.txt,main-final-diff.txt,main-final-diff/heat.png}`. Keep the original 1920x1036 viewport; reducing it to cropped donor dimensions shifts bottom-anchored history by two pixels. Main recapture of the new UI remains R9 work.
- Current checks: with the recorded native pkg-config environment, final library filters `driver_submits_code_and_password_only_in_matching_states` and `driver_sends_parameters_then_reaches_wait_phone_via_injection` pass 1 each. UI `intro_` initially had 7 pass/1 fail, not aggregate PASS. After fixing the observed Alert accessibility label and correcting fixture setup, final Password/error/accessibility and flood/banned UI filters pass 1 each (2731 filtered each), with the original text assertion retained. Dialog priority/name test passes 1; `every_hard_dialog` passes 2. Native parity build and three R4 recaptures pass, with hashes unchanged from reviewed images; only final nonvisual Alert labels postdate that build/capture and are compiled by current UI tests. Logs are in the R4 directory. `cargo fmt --all -- --check` and `git diff --check` pass. The platform-dependent Linux `ErrorKind::Open` warning remains, so strict clippy is not claimed. Full core/UI suites, clippy, file-size/hotspot, main recapture and pull/paragraph gates remain unrun R9 work.
- Blocker: none currently; parent explicitly resolved the R8 policy conflict. No new commit/push beyond R1 yet.
- Next: qualify the current R8 capture, R6/R7 checks and full R9. Final-code review and delivery remain. New-build live reset/submit remains unverified; do not extract credentials or claim live PASS.

## Material Decisions

- 2026-11-06: R9 reuses the intro fixture's native-caret phase approach only in `ready-tdesktop-parity`, via the component Theme owner after appearance setup. This stabilizes the reference's hidden phase while preserving actual focus, production timing, all 22 runs and every main mask. The initial visible-caret captures failed ink 1.22 and are not qualification.
- 2026-11-06: R9's old tests are adapted only where the approved contract changed: Ready remains forbidden for Phone resubmission (Password is now allowed under R5), and interface zoom clamps to the source 50% minimum under R8. The startup-error accessibility regression is fixed in its intro presentation owner without weakening the original text assertions.
- 2026-11-06: R7 preserves existing login-email and post-auth password/account reset owners; it does not add a new pre-auth cloud-account deletion path. C9 authorizes only the local QR-to-Phone database reset, not deleting the user's cloud chats. Signup reuses the native picker/crop and holds a generation-bound photo until Ready; terms must match the current server terms before registration. Existing no-server-terms registration semantics remain unchanged.
- 2026-11-06: R8's existing scale owner now admits the source's 50–300% range in 5% steps and records the default/automatic choice. Old explicit non-100% saved scales stay explicit and unchanged. Theme previews reuse the existing recoloring owner. No dependency, new backend or signature/CI/release change.

- 2026-11-05 continuation: parent authorizes minimal functional Auto-update and Install beta preferences/installer support for R8. Automatic stable updates remain the default; beta is off unless explicitly enabled. This supersedes the earlier R8 blocker, not the signature/CI/release boundaries.
- 2026-11-05: full tdesktop branding and text, including the official plane and removal of the Quill intro notice (owner). This overrides the older legal/product statements; no compliance with those clauses is claimed.
- 2026-11-05: source-only qualification for Code/Email/Password/Signup because no reference captures exist.
- 2026-11-05: no fake auth backend; injected demo states for screenshots, production handlers and live TDLib for the process.
- 2026-11-05: Settings::Intro is part of the goal but implemented last; QR matrix and settings version/update text are the only dynamic masks.
- 2026-11-05: the 7.3.5 reference's visible QR string is “Add Device”; the local tdesktop checkout says “Link Desktop Device”. The reference version's text remains authoritative.
- 2026-11-05: owner explicitly authorizes destructive pre-auth TDLib reset only for intro QR -> Phone under C9, including stale-generation isolation and live/deterministic evidence. This supersedes the earlier close/reopen assumption and unblocks implementation; it does not authorize deleting the app root or authenticated/user data.
- 2026-11-05: parent explicitly corrects R3 to the source/reference geometry: 204px outer widget, 188px reference plate, integer-scaled matrix of at most 180px. Only the matrix is dynamic; plate/disc/plane/text stay compared. No other acceptance or mask changes.
- 2026-11-05: parent continuation authorizes source plus deterministic reset/stale evidence if new-build live credentials are unavailable, after a safe live attempt; record the limitation, never call it live PASS, and continue R4–R9 rather than blocking on credentials.
- 2026-11-05: a new R8 public-policy conflict is confirmed in `update-button.md`, the update checker and installer. It is not covered by the branding override. Pause after the in-flight R4/R5 checks/review under the user's stop-on-new-blocker directive; R6/R7 remain pending, not waived. No fake toggles or silent installer-policy expansion are authorized.
- 2026-11-05: native SETTINGS measurement is 64.73633px at Open Sans 14/600, versus Qt 65.4688px with the bundled source face (64.6875px with the installed face). The Linux font database loads system faces first and resolves equal-property collisions first; merely adding bundled Open Sans does not isolate the source face. Minimum R2/R3 envelope expansion: alias only the existing bundled font name records in memory for intro, keep original main/preference fonts untouched, and hide the internal alias from the font chooser. No dependency, backend/public API, reference mask or pixel nudge is added.

## Checkpoint History

- 2026-11-06: real code-bearing CI on `a05289e73bd6bf041a58a51b6eebf63c95fca8cd` proves the palette-test failure belongs to the delivered change. Reopen R9 and correct the negative fixture only; the earlier inherited-failure/completion classification was incorrect. Current tests, narrow review, fix delivery and its code-bearing CI evidence follow.
- 2026-11-06: R6–R8 implemented and source/reference qualified; final-code Sol's four P2s and newly reachable cross-account photo P1 corrected and closed. R9 records both qualified known failures, complete UI success plus the subsequent targeted owner regression, final main/Settings captures and committed hotspot override. Product `a05289e73bd6bf041a58a51b6eebf63c95fca8cd` pushed and remote SHA verified; goal complete, documentation-only closure follows.
- 2026-11-05: confirmed R4/R5 review findings require the minimal live-auth update hook for pending Phone presentation and the existing shell dialog registry for banned-number handling. The notification policy and updater/installer remain untouched; no new backend, dependency or generic state machine is added. Targeted fix checks and rereview follow.

- 2026-11-05: plan reviewed by Review-Sol-Global; the Back-to-phone policy, first-run consumers and empty Settings runs were added. Implementation starts with R1 and the QR -> Phone gate.
- 2026-11-05: R1 verified; live QR close/reopen returned other-device, matching pinned TDLib persistence code. Stopped at the frozen gate before changing the intro UI; checkpoint review and scoped delivery follow.
- 2026-11-05: final-code Review-Sol-Global `ses_e53ea2aefffewG4le0UPrg5z1l` found no blocking R1 defects. Its two documentation findings were corrected: the main checking instructions now explicitly crop the historical donor dimensions, and the phone baseline reports 7/11 runs. No UI completion or destructive reset approval is implied.
- 2026-11-05: parent conveys explicit owner approval of guarded destructive pre-auth reset; goal reactivated and R2–R9 implementation resumes.
- 2026-11-05: shell and core reset groundwork compile; 2 targeted reset tests pass. Source/reference measurement exposes the 204px-widget / 188px-painted-plate contradiction. Paused for the narrow parent decision; no new pixel or live auth claim.
- 2026-11-05: Review-Sol-Global `ses_e53ea2aefffewG4le0UPrg5z1l` independently confirms the geometry contradiction and recommends the same narrow correction. Its two P2 checkpoint findings (Settings pointer dismissal, obsolete agent layer registry) are corrected and the two affected tests plus source-geometry test pass. Latest fix-only review: both P2s resolved, no new findings; this qualifies only those fixes and the accurate pause report, not R2–R9 completion or delivery.
- 2026-11-05: R4 static phone/filled/error captures pass zero-shift/ink gates; R5 shared driver/UI capability and actual Back checks pass. The failed-reset Retry media-sweep regression is reproduced and corrected with sentinel and normal-Retry tests. R8's separate stable-only/no-switches owner decision conflicts with the source Settings block; updater and installer are untouched pending explicit parent resolution.
- 2026-11-05: Sol closes the three R4/R5 runtime findings (pending Phone presentation, flood preservation, banned modal) but catches a misplaced test block/evidence claim. The phone-error block is moved into the Password driver test, the original WaitPhone test is restored, and both targeted filters pass. Alert text is now accessible and its original UI assertion passes. Fix-only rereview follows; R8 remains blocked, no completion/delivery claim.
- 2026-11-05: latest Sol `ses_e53ea2aefffewG4le0UPrg5z1l` fix-only verdict: misplaced-test P2 closed, no blocking/nonblocking findings in the correction scope; the three runtime P2s remain closed. Runtime/UI hashes and original palette diff are unchanged. This is not live, whole-goal, R6–R9 or delivery approval. Stop at the documented R8 decision; no new commit/push beyond R1.

## Completion

- Resolved outcomes: R1–R8. R9 awaits the task-caused fixture correction's tests/review/delivery and actual code-bearing CI evidence; source-only and unavailable-live-evidence qualifications remain.
- Commands and artifacts: recorded above; delivery artifacts and exact logs are under `/home/stfu/.cache/opencode-tmp/opencode/intro-delivery/`. Product `a05289e73bd6bf041a58a51b6eebf63c95fca8cd` is verified on `origin/main`. No reference screenshots, binaries, QR tokens or account data are committed.
- Constraint and diff-scope check: delivered intro scope is unchanged. The current continuation changes only the negative fixture in `src/app_theme/tests.rs` and this goal's evidence/status. Importer, palette values, user data, permanent test-notification silence, dependencies, references, binaries, tokens, CI/release and signature verification are untouched. A narrow final-code review and actual code-bearing CI are required for this correction.
- Final status: active for the narrow R9 CI correction. Do not claim aggregate library or code-bearing CI PASS until current successful evidence exists.
