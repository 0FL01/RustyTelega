# Goal: Green code-bearing CI after tdesktop parity

Status: complete
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
  - Status: verified
  - Evidence: two exact catalog-arm exceptions and this progress contract committed as `a08b0fc40d561a2f62dc56cc6ef07fbb4fb5ca3b`, pushed to `origin/main`; `git ls-remote` confirmed the same SHA. Existing integration test passes 3 tests. Sol approved the actual scoped diff without blocking findings. Original WIP hashes match and remain outside staging/commits.
- R2: Actual code-bearing CI is green.
  - Source: user "убедиться что в ci всё окей".
  - Acceptance: the published non-Markdown/code-bearing run completes successfully, with every applicable job successful and only legitimate condition-skips. Record run URL, tested SHA and job results. A docs-only successor is not tested-code evidence.
  - Primary evidence: GitHub Actions run/job conclusions and failed-step logs where needed.
  - Status: verified
  - Evidence: code-bearing run [42018626177](https://github.com/0FL01/RustyTelega/actions/runs/42018626177), SHA `a08b0fc40d561a2f62dc56cc6ef07fbb4fb5ca3b`, attempt 1, completed **success** at 2026-11-05 11:54:48 UTC: **11 successful jobs, 0 skipped, 0 failed**. Direct GitHub Actions run/job queries confirmed this after the watcher completed. The earlier implementation run `42015073616` failed the two legitimate emoji-art references; neither docs-only run `42015342551` nor this closure's successor is substituted for tested-code evidence.

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

- Closed: R1 and R2.
- Next: publish this docs-only closure record and stop. No further fixes, audits or parity work are part of this objective.
- Evidence: the real code-bearing run on `a08b0fc40d561a2f62dc56cc6ef07fbb4fb5ca3b` passed every applicable job; the closure commit does not change tested code or claim its own skipped run as substantive validation.

## Current State

- Local reproduction: `cargo test --no-default-features --locked --test app_name_check -- --nocapture` failed 2 passed/1 failed, matching CI. Both hits name the external emoji source app, not Quill.
- `scripts/check-app-name.sh` already specifies path/text allowlisting for legitimate other-app mentions. Its negative/allowlist fixtures pass and remain unchanged.
- Fix: two exact path/Rust-arm entries in the existing allowlist, preserving the pack name and attribution. The same local integration command now passes 3 tests; `git diff --check` passes. Negative and allowlist fixtures remain enabled. No runtime/workflow changes or broad exceptions.
- R1 and R2 verified. Sol's new actual-diff review approved the exact allowlist change, no blocking findings; the prior parity review was not substituted for it. Source/allowlist content did not change after review. Only CI evidence/closure documentation follows the reviewed commit.
- The temporary read-only watcher completed; the final snapshot is `ci-after-parity/run-42018626177.json` and observation log `watch-42018626177-final.log`. No repository process/service/dependency was added. Direct GitHub Actions tools independently returned the same completed-success SHA and all 11 successful jobs.

### Verified code-bearing jobs

All job links share [run 42018626177](https://github.com/0FL01/RustyTelega/actions/runs/42018626177); every conclusion below is `success`.

| Job | ID |
| --- | --- |
| prepare | 127215393701 |
| Linux fmt, clippy and tests | 127215477769 |
| quill-calls (linux-aarch64) | 127215477780 |
| quill-calls (linux-x86_64) | 127215477890 |
| linux-package (x86_64) | 127217516923 |
| linux-package (aarch64) | 127217516990 |
| flatpak (x86_64) | 127226545846 |
| flatpak (aarch64) | 127226545883 |
| linux-distro-packages | 127226545841 |
| lgpl-sources | 127215477641 |
| verify and publish GitHub Release | 127228813966 |

## Checkpoint History

- 2026-11-05: froze this separate CI objective after reproducing the actual failing implementation-run step. Prior pixel-exact objective stays complete.
- 2026-11-05: added only two exact allowlist entries; `cargo test --no-default-features --locked --test app_name_check -- --nocapture` changes from 2 passed/1 failed to 3 passed/0 failed. All original WIP hashes remain unchanged. Next: Sol review and scoped publication.
- 2026-11-05: Sol approved the actual narrow diff; pushed `a08b0fc40d561a2f62dc56cc6ef07fbb4fb5ca3b` and verified remote SHA. New run `42018626177` starts substantive jobs; awaiting actual CI results, not claiming completion.
- 2026-11-05 closure: run `42018626177` completed success, all 11 applicable jobs passed on the published code-bearing SHA. No retries, gate suppression or additional fixes were needed. Direct run/job queries verified the result; the final follow-up changes only this document.

## Completion

- R1/R2 resolved: scoped fix is published and actual code-bearing CI is green, with tested SHA/run/job evidence above.
- Local proof: `cargo test --no-default-features --locked --test app_name_check -- --nocapture` passes 3 tests, including unchanged negative/allowlist fixtures; `git diff --check` passes. Full applicable gates are evidenced by the real CI run, not an unnecessary local replay of unrelated suites.
- Scope/constraints: exactly two exact allowlist exceptions; checker, runtime names/attribution, workflow, permanent audio protection and completed parity goal are unchanged. No dependencies, screenshots, crops or unrelated WIP are committed. Original WIP hashes remain intact.
- Final status: complete. This docs-only closure publication is not passed off as a newly tested code commit; code-bearing CI evidence remains bound to `a08b0fc40d561a2f62dc56cc6ef07fbb4fb5ca3b`.
