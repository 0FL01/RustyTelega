# Developer ID signing and notarization for macOS releases

Idan joined the paid Apple Developer Program on 2026-11-03. The membership is on the same team as the Apple Development identity releases were already signed with (`72D8U72V55`). A Developer ID Application certificate (G2 intermediate, expires 2027-12-03) now exists for that team. A Developer ID signature plus a notarization ticket lets Gatekeeper open a downloaded Quill without the workarounds in `macos-install-without-notarization.md`.

## The update problem

Every installed copy checks an update against its own designated requirement before installing it (`src/update_install/macos/codesign.rs`). Releases up to 2026.1102 carry the default requirement for an Apple Development signature, which pins the leaf certificate's common name:

```
identifier "org.shinycake.quill" and anchor apple generic and certificate leaf[subject.CN] = "Apple Development: idanbirman@icloud.com (RXQM374ZF7)" and certificate 1[field.1.2.840.113635.100.6.2.1]
```

A Developer ID build never satisfies it, so if the first Developer ID release shipped as-is, every installed copy would refuse it as "signed differently".

## Decision

`scripts/macos-package-smoke.sh` signs the app with an explicit requirement that names the team instead of the certificate:

```
identifier "org.shinycake.quill" and anchor apple generic and certificate leaf[subject.OU] = "<TEAM>" and (certificate leaf[field.1.2.840.113635.100.6.1.13] or certificate leaf[field.1.2.840.113635.100.6.1.12])
```

The team comes from the signing certificate. `6.1.13` marks a Developer ID Application leaf and `6.1.12` an Apple (Mac) Development leaf. Certificate signatures also get the hardened runtime and `assets/macos/Quill.entitlements` (camera, microphone, location), which notarization requires. The app and both helpers get the entitlements, and the dylibs get neither. Ad-hoc builds (CI, forks) are unchanged.

The switch happens in two releases:

1. **Bridge release**, still signed with the Apple Development identity. Installed copies accept it because the leaf is the same certificate. It carries the team requirement above.
2. **Swap the `MACOS_SIGNING_P12` secret** to the Developer ID identity and set the `MACOS_NOTARY_*` secrets. The next release is Developer ID signed, notarized and stapled. Copies that installed the bridge accept it.

A user who skips the bridge release and updates straight from an older release to a Developer ID one gets refused and has to reinstall from the website once. Keeping the bridge release as the latest for a few days makes this rare.

`package-macos.yml` notarizes only release builds whose signature names a Developer ID Application authority. It fails when such a build has no `MACOS_NOTARY_*` secrets, so a Developer ID release is never published unnotarized. A release still signed with Apple Development skips notarization with a warning.

The keychain partition for the database key is `teamid:72D8U72V55` for both identity types, so the switch raises no keychain prompt (`macos-release-signing.md`).

## Verified

On the owner's Mac (2026-11-03), using copies of the installed 2026.1102.4879 bundle:

- Apple Development signed with the team requirement: `codesign --verify --strict -R=<installed copy's requirement>` passes, so installed copies accept the bridge.
- Developer ID signed: passes `-R=<bridge requirement>`, so the bridge accepts the switch.
- Both run `--version` under the hardened runtime, and the bridge-signed copy stayed up after a full launch.
- The ad-hoc path still verifies.

Notarization itself is unverified until the API key exists. It needs the updated Developer Program License Agreement accepted first.

## Secrets

```bash
# Developer ID identity (after the bridge release is published)
base64 -i ~/.quill-signing/devid.p12 | gh secret set MACOS_SIGNING_P12 --repo shinycake/quill
gh secret set MACOS_SIGNING_P12_PASSWORD --repo shinycake/quill < ~/.quill-signing/devid-p12-password.txt
# App Store Connect API key for notarytool
base64 -i AuthKey_<KEYID>.p8 | gh secret set MACOS_NOTARY_KEY --repo shinycake/quill
gh secret set MACOS_NOTARY_KEY_ID --repo shinycake/quill --body <KEYID>
gh secret set MACOS_NOTARY_ISSUER_ID --repo shinycake/quill --body <ISSUER-UUID>
```
