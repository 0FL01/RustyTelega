# Flatpak package

## What tdesktop does
Telegram Desktop ships a Flatpak, `org.telegram.desktop`, built from source on
the GNOME runtime with `--device=all`, `--system-talk-name=org.bluez`, and
talk names for notifications, Mutter's idle monitor and the tray watcher.

## Shape
- **App id `io.github.shinycake.Quill`.** A reverse-DNS id on a domain the
  developer controls; the GitHub account `shinycake` proves this one. Outside Flatpak the
  desktop id stays `org.shinycake.quill`, so existing tarball installs keep
  their launcher, `tg:` registration and pinned docks.
- **Repackaged release tarball, not a source build.** The manifest unpacks
  `quill-linux-<arch>-bundle.tar.gz` (the same files CI already checks with
  `check-bundle-elf.sh` and the tarball smoke) into `/app/lib/quill`. A source
  build would have to vendor about 1,080 crates and the git-pinned GPUI family
  for an offline build, plus TDLib, tgcalls and FFmpeg as modules, and would
  have to embed the Telegram API credentials, which live in a CI secret and
  must not be in a public manifest. A source build would be a separate
  project.
- **Runtime `org.gnome.Platform` 51.** It still ships WebKitGTK 4.1 and
  libsoup 3 for `quill-webview`, and the release binaries need glibc 2.35, far
  below the runtime's. CI runs `ldd` on every binary in the sandbox.
- **`/app/bin/quill` is a symlink.** Quill finds `lib/`, `ffmpeg` and
  `quill-webview` from its resolved path (`$ORIGIN` and `current_exe`), which is
  `/app/lib/quill/quill`.
- **Updates** come from Quill's own repo on the site, rebuilt from each
  release's bundles (`scripts/site-flatpak-repo.sh`). The in-app updater already reports
  "managed by your package manager" under `/app`.

## Permissions
| Permission | Why |
| --- | --- |
| `--share=network` | Telegram, link previews, calls |
| `--share=ipc`, `--socket=wayland`, `--socket=fallback-x11` | the window |
| `--socket=pulseaudio` | sounds, playback, the microphone in calls and voice messages |
| `--device=all` | cameras: calls and round videos open `/dev/video*` through V4L2; also the GPU |
| `--filesystem=xdg-download` | the default folder for saved files; anything else goes through the file chooser portal |
| `--talk-name=org.freedesktop.Notifications` | notifications with inline reply and actions (the portal has no inline reply) |
| `--talk-name=org.kde.StatusNotifierWatcher` | tray icon with the unread count |
| `--talk-name=org.gnome.Mutter.IdleMonitor`, `--talk-name=org.freedesktop.ScreenSaver` | idle time for passcode auto-lock and away status |
| `--own-name=org.mpris.MediaPlayer2.quill` | media keys and desktop media controls |
| `--system-talk-name=org.freedesktop.GeoClue2` | sending the current location; GeoClue asks the user |
| `--system-talk-name=org.bluez` | passkey sign-in with a phone (FIDO hybrid over Bluetooth LE) |

Not requested: `org.freedesktop.secrets` (Linux keeps the database key in a
0600 file in the data folder, which is inside `~/.var/app`), home or host
filesystem access, and `com.canonical.Unity` (the launcher badge is a signal
Quill emits, not a call; whether the bus proxy passes it to the dock is
untested).

## What changes inside the sandbox (`src/desktop_app_id.rs`)
- The window app_id, notification `desktop-entry`, launcher badge URI, GeoClue
  `DesktopId` and tray id use `$FLATPAK_ID` (checked for Flatpak's id rules,
  only when `/.flatpak-info` exists). The exported desktop file keeps
  `StartupWMClass=org.shinycake.quill`, so builds from before this change also
  group with their launcher.
- "Open Telegram links with Quill" says to pick the app in the desktop
  settings: Flatpak exports the desktop file with `x-scheme-handler/tg`, and
  `xdg-mime` inside the sandbox would only change the sandbox's own list.
- The launch-at-login switch is replaced by a note: the sandbox's XDG autostart
  folder is not one the session reads. The Background portal's autostart
  request is the follow-up.
- Agent access setup lines and `quill agent describe` name the launcher
  Flatpak exports (`<installation>/exports/bin/io.github.shinycake.Quill`)
  instead of `/app/...`. `agent connect claude-desktop|cursor --yes` and
  `install-skill` without `--dir` refuse with a pointer to `--print`, because
  they cannot write outside the sandbox.

## Not covered yet
- System proxy settings come from `gsettings`, which reads the sandbox's own
  defaults; the proxy portal is not used.
- Spell-check dictionaries: only those in the runtime and `/app/share/hunspell`.
- Nothing has run on a real Linux desktop from the Flatpak; CI checks the build,
  flatpak-builder-lint, `--version` and library loading.
