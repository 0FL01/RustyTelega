# Packaging

The release workflow publishes packages only to
[GitHub Releases](https://github.com/0FL01/RustyTelega/releases). It does not
publish to AUR, Chocolatey, Homebrew or a separate Flatpak repository.

| Folder | Purpose |
|---|---|
| `linux/` | Linux x86_64 distribution packages built from the release bundle |
| `aur/` | Arch package generator used by the release build; no AUR upload |
| `windows/` | Inno Setup script for the Windows setup EXE |
| `flatpak/` | Flatpak manifest and local build instructions |

The older `homebrew-tap/` and `chocolatey/` directories are not used by the
release workflow. Flatpak bundles in Releases are installed directly; they do
not provide the automatic updates of a Flatpak repository.
