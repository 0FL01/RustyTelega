# Goal: Green code-bearing CI after tdesktop parity

Status: active
Source: user instruction 2026-11-05: "commit and push, убедиться что в ci всё окей"
Last updated: 2026-11-05

## Objective

Publish the smallest proven CI correction and verify the actual code-bearing GitHub Actions run, not a Markdown-only run that skips substantive jobs. The completed pixel-exact goal remains closed.

## Execution Directive

Complete the frozen Required Outcomes using the listed Change Envelope and Primary Evidence. Work on the smallest unresolved outcome. Do not add requirements from reviews, tests, tools, speculative risks, or optional source text. Finish when every required outcome is resolved and affected constraints remain satisfied. Record an exact external blocker and smallest unlock if no approved in-scope action remains.

## Frozen Contract

### Required Outcomes

- R1: Scoped CI correction committed and pushed to `origin/main`.
  - Source: user "commit and push".
  - Acceptance: only necessary CI fixes/progress are published; original WIP remains untouched and unstaged.
  - Primary evidence: scoped diff, relevant local regression, Sol review, commit and remote SHA verification.
  - Status: in_progress
  - Evidence: baseline HEAD/remote `fe544777849a7c3221b56900665a9cf8b9ffbae5`; two exact catalog-arm exceptions added, existing integration test now 3 passed. Sol reviewed the actual diff and approves scoped commit/push without blocking findings. Original WIP hashes match; publication pending.
- R2: Actual code-bearing CI is green.
  - Source: user "убедиться что в ci всё окей".
  - Acceptance: the published non-Markdown/code-bearing run completes successfully, with every applicable job successful and only legitimate condition-skips. Record run URL, tested SHA and job results. A docs-only successor is not tested-code evidence.
  - Primary evidence: GitHub Actions run/job conclusions and failed-step logs where needed.
  - Status: in_progress
  - Evidence: implementation run [42015073616](https://github.com/0FL01/RustyTelega/actions/runs/42015073616), SHA `217ecf20467036bab67db0c30ab3eddc62c6cfe7`, has failed Linux checks job `127203577204`: `app_name_check::repository_sources_pass` rejects two legitimate emoji-art references. Docs-only run `42015342551` on `fe5447778` is not substantive-test evidence.

### Constraints and non-goals

- Preserve the original tracked/untracked WIP (four tracked files, 71 insertions/67 deletions; existing snapshot hashes).
- Keep app-name validation fully enabled; retain the approved external pack name and art attribution.
- Preserve permanent test-notification silence and normal production notification settings.
- Do not reopen parity R2/R3, chase pixel residuals, weaken gates or introduce blind retries/no-op trigger commits.
- No unrelated audits, dependencies, runtime features, screenshots or reference crops.

## Change Envelope

- Initial owner: `scripts/app-name-allowlist.txt`, consumed by `scripts/check-app-name.sh` and `tests/app_name_check.rs`.
- Add only exact path/text exceptions for the two `EmojiSetId::Tdesktop` name/credit arms in `src/emoji_sets_style/catalog.rs`; no runtime rename or generic checker change.
- This goal records CI evidence. Expand the edit envelope only for another proven applicable CI failure, recording its exact job/step and minimal correction first.
- Temporary logs/probes: `/home/stfu/.cache/opencode-tmp/opencode/ci-after-parity/`; test projects remain outside the worktree.

## Current Checkpoint

- Closes: R1, then R2.
- Next: commit/push only the allowlist and this goal, then follow the resulting code-bearing run to completion. Sol approved the current fix; CI-green is not yet verified.
- Expected evidence: local test changes from 2 passed/1 failed to 3 passed; new Actions run has substantive jobs and final successful conclusions, or an exact blocker.

## Current State

- Local reproduction: `cargo test --no-default-features --locked --test app_name_check -- --nocapture` failed 2 passed/1 failed, matching CI. Both hits name the external emoji source app, not Quill.
- `scripts/check-app-name.sh` already specifies path/text allowlisting for legitimate other-app mentions. Its negative/allowlist fixtures pass and remain unchanged.
- Fix: two exact path/Rust-arm entries in the existing allowlist, preserving the pack name and attribution. The same local integration command now passes 3 tests; `git diff --check` passes. Negative and allowlist fixtures remain enabled. No runtime/workflow changes or broad exceptions.
- Sol's new actual-diff review approves scoped commit/push, no blocking findings; the prior parity review is not substituted for it. No CI success claimed. `gh` is absent; GitHub Actions read tools work. Old implementation-run package jobs are still pending and its Linux checks already failed; a new substantive run is required.

## Checkpoint History

- 2026-11-05: froze this separate CI objective after reproducing the actual failing implementation-run step. Prior pixel-exact objective stays complete.
- 2026-11-05: added only two exact allowlist entries; `cargo test --no-default-features --locked --test app_name_check -- --nocapture` changes from 2 passed/1 failed to 3 passed/0 failed. All original WIP hashes remain unchanged. Next: Sol review and scoped publication.
