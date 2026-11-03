# Packaging

Where Quill is distributed besides GitHub Releases, and how each channel is
set up. Why it is built this way: `docs/decisions/distribution-channels.md`.

| Folder | What it is |
|---|---|
| `homebrew-tap/` | The contents of the `shinycake/homebrew-tap` repository (cask `quill-app`) |
| `chocolatey/` | `make-package.py`, which writes the Chocolatey package for a release |
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

2. **Flatpak.** Quill is not on Flathub (the owner's decision); it has its own
   repo on the site. Add the signing key, kept outside the repo, as the
   `FLATPAK_GPG_KEY` secret: `gh secret set FLATPAK_GPG_KEY -R shinycake/quill
   < ~/.config/quill-release/flatpak-signing.asc`. See `flatpak/README.md`.

3. **Chocolatey.** Sign up at community.chocolatey.org, copy the API key
   from the account page and add it as the `CHOCO_API_KEY` secret of this
   repository. `chocolatey.yml` then pushes every stable release; run it by
   hand with `push` checked for the current one. Chocolatey's moderators
   review each version before it is listed.

Windows package managers: Chocolatey only, by the owner's decision of
3 November 2026. Quill is not offered on winget or Scoop.
