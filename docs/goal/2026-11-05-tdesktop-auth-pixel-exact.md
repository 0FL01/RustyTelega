# Goal: Pixel-exact sign-in flow after Telegram Desktop 7.3.5

Status: blocked
Source: user instruction 2026-11-05 ("Надо допилить pixel perfect ... процесс экрана входа не соответствует tdesktop"), then "Приступить к реализации и коммит пуш потом"; reference `artifacts/intro-ref/{start,qr,phone,phone-filled,phone-error,settings,qr-back}.png`; follows `2026-11-04-tdesktop-pixel-exact.md`
Last updated: 2026-11-05

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
  - Status: pending
  - Evidence:

- **R3: QR screen and Start -> QR.**
  - Acceptance:
    - Start submits a live QR request and shows the QR step.
    - QR has the 204x204 white plate (12px skip, radius 8), integer module size, rounded modules and finder patterns, and the 44px `#40a7e3` disc with the tdesktop plane centred.
    - Title, three numbered steps, "Log in using phone number", conditional "Log in using passkey", 56x56 back and SETTINGS are at the tdesktop positions.
    - The dynamic matrix (inside the white plate) is the only masked QR area; plate, disc, plane, title and links are compared.
    - QR -> Phone is verified on live TDLib. TDLib does not allow `setAuthenticationPhoneNumber` from `WaitOtherDeviceConfirmation`; the implementation must use a verified client reset (close/reopen, reusing `background_accounts/closing.rs`) and must not fake the state. If no supported reset exists, stop with a proven blocker.
  - Status: blocked
  - Evidence: Live installed `/usr/bin/quill`, isolated HOME/XDG directories and the same disposable database: initial `--connect-smoke` -> `SMOKE_OK wait-phone`; live UI QR -> `status: Confirm the sign-in on another device.`; normal `/usr/bin/quill --quit`, wait for its PID, reopen `--connect-smoke` -> **`SMOKE_OK other-device`**. Direct `td_execute(getOption)` confirms library version 1.8.69 and commit `4c6377aa7386c39217878a9a56d581922ad37ca4`. Redacted evidence and the exact temporary scenario: `/home/stfu/.cache/opencode-tmp/opencode/intro-live.nlFQ1Y/{initial-smoke.log,qr-status.txt,reopen-smoke.log,tdlib-version.txt,inner.sh}`. The disposable auth database/key and raw logs were deleted after exit. This is a negative reset result across processes, not a successful in-process `ClosedClient.confirmed` proof. Pinned `AuthManager.cpp:1680–1793` persists/restores QR state and retains it during Closing; `:640–650` rejects a number from QR. No production auth code was changed.

- **R4: Phone and error.**
  - Acceptance:
    - Country field 300x61, code field 64 and phone field 225 at +6, 16px text, 300/20 description, title at contentLeft+4/contentTop+1, Next at +266, QR link, back and SETTINGS follow the source.
    - Existing phone formatting and country search are reused; paste, caret, focus, IME and secrets are preserved.
    - Error text, red underline and shake match; flood and banned-number handling remain.
    - Demo fixtures reproduce the reference's Sweden/+46 and 1234 states.
  - Status: pending
  - Evidence:

- **R5: Back to phone from later steps.**
  - Acceptance:
    - Presentation history lets Code/Email/Password/Registration return to the Phone form where tdesktop does; the phone form is not shown while the underlying auth state cannot accept a number.
    - `setAuthenticationPhoneNumber` is allowed from the later states listed in `schema/td_api.tl:12187-12192` only when no auth query is pending.
    - A production-handler test covers Password -> Back -> Phone -> submit and proves the request is sent with no stale auth query.
  - Status: pending
  - Evidence:

- **R6: Code and Email.**
  - Acceptance:
    - Code uses one hidden `TextareaState` for input/IME/paste and painted cells 40x50, gap 10, 4px border, radius 8; the count comes from TDLib and a complete code submits.
    - Telegram, SMS, call, Fragment, resend countdown, email setup and email code remain supported.
    - The shell geometry matches the source; there is no reference capture, so the evidence is source-derived geometry and behaviour, not a pixel claim.
  - Status: pending
  - Evidence:

- **R7: Password, recovery and Signup.**
  - Acceptance:
    - Password/recovery use the same step shell; existing submit, zeroize, recovery, reset and terms paths remain.
    - Signup keeps names, terms and photo handling with intro geometry; no terms acceptance means no registration request.
    - Back follows R5.
    - No reference captures exist; evidence is source-derived geometry and behaviour, not a pixel claim.
  - Status: pending
  - Evidence:

- **R8: Settings::Intro.**
  - Acceptance:
    - SETTINGS opens a 392px-max layer with a 56px top bar and radius 8.
    - Language, Connection type, update block, system/window frame, Interface scale, themes/accents and FAQ follow `settings_intro.cpp`, with its conditions.
    - Existing Quill language, proxy, zoom, theme, updater and FAQ owners are reused; pre-auth availability is proved.
    - Dynamic version/update text is excluded; the rest is captured as `intro-settings`.
  - Status: pending
  - Evidence:

- **R9: Regression and delivery.**
  - Acceptance:
    - `ready-tdesktop-parity` stays at no worse than 0.37% at tolerance 24, all 22 runs at (0,0) and ink in [0.97, 1.03].
    - `cargo fmt --all -- --check`, core and UI clippy with `-D warnings`, `cargo test --lib`, `cargo test --features demo-capture --bin quill` with `LC_TIME=C.UTF-8`, `pull_next_ui_tests`, `paragraph_cache_scroll`, `scripts/check-file-size.sh` and `scripts/check-hotspots.sh` are recorded.
    - Scoped changes are committed and pushed to `origin/main`, the goal is updated, and the original WIP is preserved.
  - Status: pending
  - Evidence:

### Constraints

- C1: Geometry, colours, strings and assets come from the tdesktop sources and the 7.3.5 reference, not from screenshot nudges. Profile boxes and masks are test fixtures only.
- C2: The narrow main-window layout (history pane under 880px) keeps working.
- C3: Existing user settings stay switchable; saved emoji, theme, language and window settings are not overwritten.
- C4: No committed screenshots, reference crops, QR tokens, official binary or workdir.
- C5: TDLib remains the source of truth for auth; no new MTProto implementation and no fake production auth path.
- C6: The owner chose a full tdesktop visual copy, including the "Telegram Desktop" title, the "official Telegram Desktop app" description, the official paper-plane and removal of the Quill third-party notice. This overrides the previous 2.2/2.4 compliance claims in `docs/decisions/codex-legal-compliance.md` and `PRODUCT.md`; the override and the remaining risk are recorded, and no continued compliance is claimed.
- C7: No new dependencies; QR uses the existing `qrcode` and `peer_qr::qr_path`, input uses existing `TextareaState`, and settings reuse existing owners.
- C8: Settings::Intro must work before authorization; if an owner requires a session, only the minimum seam is added.

### Non-goals

- The main window beyond the regression gate.
- The light theme and other screens.
- The full post-auth Settings.
- macOS/Windows-specific glyph or window behaviour.
- A fake/scripted auth backend; demos inject states, and production handlers plus a live check prove the process.
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
  - `src/ui/{auth_ui.rs,signin_ui.rs,auth_state.rs,auth_recovery.rs,auth_registration.rs}`
  - `src/connect/auth.rs`, `src/telegram/requests/auth.rs`
  - `src/ui/peer_qr/qr_path.rs` (shared QR path only; no unrelated peer-QR changes)
   - `scripts/tdesktop_parity/*`, `lang/en/*.strings`, `docs/decisions/tdesktop-intro.md`, `docs/decisions/codex-legal-compliance.md`, `PRODUCT.md`, `THIRD_PARTY.md`
   - `docs/decisions/tdesktop-pixel-contract.md` (direct tooling consumer: explicit main-reference crop after strict size validation)
  - `assets/tdesktop-icons/**`, `assets/onboarding/**` (retired)
- Forbidden: committed screenshots, crops, QR tokens, the official binary or its workdir; unrelated user WIP.

## Current Checkpoint

- Closes: R1; records the failed QR -> Phone spike that gates R3 and the remaining UI work.
- Smallest next action: owner/parent decision on the reset protocol; ordinary close/reopen preserves QR and cannot implement the approved immediate Phone transition.
- Expected evidence after unlock: a live, confirmed-close reset to a phone-capable TDLib state before any QR/Phone UI implementation.
- Stop or replan if: reset requires destructive local TDLib state removal. `destroy` is public and allowed before authorization (`schema/td_api.tl:12295–12299`), but destroys all TDLib-local data; it has not been approved or tested as this flow's replacement. Do not silently substitute it or fake Phone.

## Current State

- Resolved: R1 tooling; R2/R4–R8 remain pending, R3 is blocked, R9 is incomplete.
- Existing WIP: parity tooling is preserved and completed in the checkpoint commit. The original `src/app_theme/{embedded,keys}.rs` modifications and untracked `src/ui/td/st/intro.rs` remain untouched and outside that commit. `__pycache__/` is ignored. No first-run or auth UI implementation has begun.
- Baseline: current phone capture vs `artifacts/intro-ref/phone.png` is 1.92% total at tolerance 24; text 18.44%, country 11.87%, fields 6.28%, error 91.18%, button 58.56%, link 12.12%; 7 of 11 runs in the current profile are shifted. Reproduced with final tooling at `/home/stfu/.cache/opencode-tmp/opencode/intro-r1.0qFbjW/{phone-tooling.png,phone-tooling-align.txt,phone-tooling-diff.txt}` (totals 1.92% @24 / 1.96% @8 / 1.99% @0; expected baseline failure, not parity).
- Latest main regression: fresh build with `QUILL_NATIVE_ROOT=/home/stfu/.cache/opencode-tmp/quill-native-build/root scripts/tdesktop_parity/build.sh`, `capture.sh <bin> <main-full.png>`, explicit historical donor crop `magick <main-full.png> -crop 1919x1034+0+0 +repage <main-parity.png>`, then align/diff. All 22 runs (0,0), ink [0.98,1.01]; total **0.34% @24 / 0.48% @8 / 1.83% @0**. Artifacts: `/home/stfu/.cache/opencode-tmp/opencode/intro-r1.0qFbjW/{main-parity.png,main-final-align.txt,main-final-diff.txt,main-final-diff/heat.png}`. The original 1920x1036 viewport must remain: reducing it to the donor's cropped dimensions shifts bottom-anchored history by two pixels. No unexpected layout structure was observed in the final heatmap.
- Checks: `cargo fmt --all -- --check` and `git diff --check` pass. Initial build without `QUILL_NATIVE_ROOT` failed for missing system `alsa.pc`; using the existing native sysroot passed. One capture attempt used nonexistent demo kind `phone` and failed; the valid `wait-phone` capture passed. Capture now aborts if the demo exits or never reaches its ready marker instead of taking a misleading screenshot. Core/UI tests, clippy, file-size/hotspot checks and pull/paragraph tests have not been run: no Rust implementation was changed and the gated UI work is blocked; no whole-goal PASS is claimed.
- Blocker: pinned TDLib preserves QR on close/reopen (live `SMOKE_OK other-device`). Credentials were not extracted; the installed binary's embedded credentials were used through its existing UI/CLI only.
- Next: approve/test the smallest supported reset (potentially destructive `destroy` on an explicitly unauthenticated client, preserving Quill settings and handling a concurrent successful QR scan), or revise the transition requirement. Do not start R2–R8 on a fake reset assumption.

## Material Decisions

- 2026-11-05: full tdesktop branding and text, including the official plane and removal of the Quill intro notice (owner). This overrides the older legal/product statements; no compliance with those clauses is claimed.
- 2026-11-05: source-only qualification for Code/Email/Password/Signup because no reference captures exist.
- 2026-11-05: no fake auth backend; injected demo states for screenshots, production handlers and live TDLib for the process.
- 2026-11-05: Settings::Intro is part of the goal but implemented last; QR matrix and settings version/update text are the only dynamic masks.
- 2026-11-05: the 7.3.5 reference's visible QR string is “Add Device”; the local tdesktop checkout says “Link Desktop Device”. The reference version's text remains authoritative.

## Checkpoint History

- 2026-11-05: plan reviewed by Review-Sol-Global; the Back-to-phone policy, first-run consumers and empty Settings runs were added. Implementation starts with R1 and the QR -> Phone gate.
- 2026-11-05: R1 verified; live QR close/reopen returned other-device, matching pinned TDLib persistence code. Stopped at the frozen gate before changing the intro UI; checkpoint review and scoped delivery follow.
- 2026-11-05: final-code Review-Sol-Global `ses_e53ea2aefffewG4le0UPrg5z1l` found no blocking R1 defects. Its two documentation findings were corrected: the main checking instructions now explicitly crop the historical donor dimensions, and the phone baseline reports 7/11 runs. No UI completion or destructive reset approval is implied.

## Completion

- Resolved outcomes: R1 only; not a completed intro implementation.
- Commands and artifacts: recorded above under R1/R3 and Current State; no reference screenshots, binaries, QR tokens or account data are committed.
- Constraint and diff-scope check: checkpoint changes only parity tooling, its direct checking instructions, Python bytecode ignore and this execution document; original palette/metric WIP is preserved outside the checkpoint commit. No dependencies, production auth state, settings, narrow layout or test-notification policy were changed.
- Final status: blocked on the approved QR -> Phone reset assumption; R2–R9 are not delivered.
