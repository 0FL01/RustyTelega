# Packaging

Where Quill is distributed besides GitHub Releases, and how each channel is
set up. Why it is built this way: `docs/decisions/distribution-channels.md`.

| Folder | What it is |
|---|---|
| `homebrew-tap/` | The contents of the `shinycake/homebrew-tap` repository (cask `quill-app`) |
| `scoop-bucket/` | The contents of the `shinycake/scoop-bucket` repository (manifest `quill`) |
| `chocolatey/` | `make-package.py`, which writes the Chocolatey package for a release |
| `winget/` | `make-manifests.py`, which writes the manifests for a release in winget-pkgs layout |
| `windows/` | `quill.iss`, the Inno Setup script for `quill-windows-x86_64-setup.exe` |
| `flatpak/` | The Flatpak manifest; CI builds it for every release and the site serves it as Quill's own Flatpak repo |

## One-time setup

These publish to places outside this repository, so they are done once by
hand. After that every channel follows new releases by itself.

1. **Homebrew tap.** Create the public repository and push the folder:

       cp -R packaging/homebrew-tap /tmp/homebrew-tap && cd /tmp/homebrew-tap
       git init -b main && git add -A && git commit -m "Add the Quill cask"
       gh repo create shinycake/homebrew-tap --public --source . --push

   Then `brew install --cask shinycake/tap/quill-app` works. The `bump`
   workflow runs every three hours; run it by hand after a release to skip
   the wait.

2. **Scoop bucket.** The same with `packaging/scoop-bucket` and
   `shinycake/scoop-bucket`. Then
   `scoop bucket add shinycake https://github.com/shinycake/scoop-bucket` and
   `scoop install shinycake/quill`.

3. **winget.** Needs a release that has `quill-windows-x86_64-setup.exe`.
   Write the manifests and open the first pull request:

       packaging/winget/make-manifests.py <version> /tmp/winget
       gh repo fork microsoft/winget-pkgs --clone=false
       # copy /tmp/winget/manifests/... into a branch of the fork, then
       gh pr create --repo microsoft/winget-pkgs --title "New package: shinycake.Quill version <version>"

   After it is merged, add a classic token with `public_repo` scope as the
   `WINGET_TOKEN` secret of this repository; `release-published.yml` then
   submits each stable release.

4. **Flatpak.** Quill is not on Flathub (the owner's decision); it has its own
   repo on the site. Add the signing key, kept outside the repo, as the
   `FLATPAK_GPG_KEY` secret: `gh secret set FLATPAK_GPG_KEY -R shinycake/quill
   < ~/.config/quill-release/flatpak-signing.asc`. See `flatpak/README.md`.

5. **Chocolatey.** Sign up at community.chocolatey.org, copy the API key
   from the account page and add it as the `CHOCO_API_KEY` secret of this
   repository. `chocolatey.yml` then pushes every stable release; run it by
   hand with `push` checked for the current one. Chocolatey's moderators
   review each version before it is listed.
