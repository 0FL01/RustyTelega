# Distribution channels: disk image, Windows installer, package managers

Until November 2026 Quill shipped zips, Linux tarballs and three one-line
installer scripts. People expect to find an app where they already look for
apps: a drag-to-Applications disk image on the Mac, a setup.exe and winget on
Windows, Homebrew, and the software center on Linux. This adds those without
replacing anything: the zips stay (the in-app updater and the scripts use
them), and every channel installs the same signed build.

## macOS disk image

`quill-macos-aarch64.dmg` is built by `scripts/macos-dmg.sh` with
[dmgbuild](https://github.com/dmgbuild/dmgbuild), which writes the Finder
window layout (`.DS_Store`) itself, so it runs on a headless CI runner with no
AppleScript and no Finder. `assets/macos/dmg/settings.py` holds the layout: a
660x400 window, 128 pt icons, Quill at (170, 190), the Applications link at
(490, 190), no toolbar or sidebar, APFS with LZMA compression (`ULMO`, 46 MB
against the zip's 64 MB).

The background is `assets/macos/dmg/background.html`, rendered to the
committed `background.png` and `background@2x.png` (both resolutions are
joined into one TIFF at build time, so Retina screens get the sharp one). It
uses the landing page's colors (`site/styles.css`) and the icon's blue and
swooshes. Finder draws icon labels on a custom background in black or white
depending on macOS version and appearance, and a background image can't adapt
to dark mode, so the labels sit on a mid-tone blue ribbon that both colors read
on (about 4.5:1 for black and 3.9:1 for white). The icons themselves sit on
the dark ground, where the blue icon stands out. To change the art, edit the
HTML and re-render:

    C="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    for s in 1 2; do f=background.png; [ $s = 2 ] && f=background@2x.png
      "$C" --headless --hide-scrollbars --window-size=660,400 --force-device-scale-factor=$s \
        --allow-file-access-from-files --screenshot="$PWD/assets/macos/dmg/$f" \
        "file://$PWD/assets/macos/dmg/background.html"; done

In `package-macos.yml` the image is made from the app after the app is
notarized and stapled, then signed, notarized and stapled itself
(`scripts/macos-notarize.sh` does both submissions), so Gatekeeper opens
both the image and the app inside it offline. Every CI build makes the image
and mounts it to check the layout files, the Applications link and the app's
signature; only Developer ID release builds are notarized.

## Windows installer

`quill-windows-x86_64-setup.exe` is an Inno Setup 6 installer
(`packaging/windows/quill.iss`). It installs for the current user without
administrator rights into `%LOCALAPPDATA%\Programs\Quill`, the folder the
PowerShell installer already uses, so the two are interchangeable and the
in-app updater keeps working (it replaces files one by one and leaves
`unins000.exe` alone). It adds the Start menu shortcut with Quill's
AppUserModelID, so notifications keep their name and icon. Its Settings >
Apps entry (`Quill_is1`) replaces the PowerShell installer's `Quill` entry,
and the PowerShell installer no longer writes its own entry when the setup's
is present. The uninstaller quits Quill first and removes the `tg:` handler
and the start-at-login entry only when they point at this install. Account
data in `%APPDATA%` is kept. `package-windows.yml` builds it, then installs,
runs and uninstalls it silently, the way winget drives it.

The installer is not code-signed. SmartScreen asks once for a browser
download, as it does for the zip. Signing needs a paid certificate or Azure
Artifact Signing (identity validation by the owner), which is the remaining
Windows gap.

## Package managers

| Channel | Where | Updates |
|---|---|---|
| Homebrew | `shinycake/homebrew-tap`, cask `quill-app` (the token `quill` is a homebrew/core formula) | the tap's scheduled `bump` workflow reads the latest release's `SHA256SUMS.txt`, audits and commits |
| Scoop | `shinycake/scoop-bucket`, manifest `quill` | Scoop's Excavator action, from `checkver`/`autoupdate` in the manifest |
| winget | `shinycake.Quill` in microsoft/winget-pkgs, the Inno installer | `release-published.yml` opens the PR with Komac (winget-releaser) when `WINGET_TOKEN` is set |
| Flathub | `io.github.shinycake.Quill` | flatpak-external-data-checker on Flathub (see `docs/decisions/flatpak.md`) |

The tap and the bucket are separate repositories because Homebrew and Scoop
clone the whole repository they are given, and this one is large. Their
contents live here under `packaging/homebrew-tap/` and
`packaging/scoop-bucket/` and are pushed from there once
(`packaging/README.md`); after that each repository updates itself on a
schedule with its own `GITHUB_TOKEN`, so no token for another repository is
stored anywhere. Homebrew's main cask repository needs 75 stars for a
self-submitted app, so the tap is the channel until Quill qualifies.

The in-app updater already stands aside for package managers
(`src/update_install/swap.rs` `distro_managed`, and Homebrew's Caskroom on
the Mac). The cask moves Quill.app into /Applications and sets
`auto_updates true`, so Quill keeps updating itself there and Homebrew does
not fight it.
