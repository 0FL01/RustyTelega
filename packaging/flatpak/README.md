# Quill's Flatpak

`io.github.shinycake.Quill.yml` builds the Flatpak. It does not compile
Quill: it unpacks the Linux release tarball for each CPU
(`quill-linux-<arch>-bundle.tar.gz`, made by `scripts/linux-package.sh`) into
`/app/lib/quill`, links `/app/bin/quill` to it, and renames the desktop file and
icons to the app id. The AppStream file lives in
`assets/linux/io.github.shinycake.Quill.metainfo.xml`; the copy here is a
symlink to it.

Why a repackaged tarball and the reasons for each permission:
[docs/decisions/flatpak.md](../../docs/decisions/flatpak.md).

## Build and run locally (Linux)

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub org.flatpak.Builder
cd packaging/flatpak
flatpak run org.flatpak.Builder --user --install --force-clean \
  --install-deps-from=flathub build-dir io.github.shinycake.Quill.yml
flatpak run io.github.shinycake.Quill
```

Lint like Flathub does:

```sh
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest io.github.shinycake.Quill.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo repo   # after a build with --repo=repo
```

CI (`.github/workflows/flatpak.yml`) does all of this for x86_64 and aarch64 on
every pull request that touches these files, on demand, and on every published
release. It also runs `quill --version` and `ldd` on every binary inside the
sandbox. The build lint skips the screenshot errors listed in
`lint-exceptions.json`, which only Flathub's own screenshot mirror satisfies;
pull requests also skip the screenshot URL check, because the screenshots are
linked from `main` and a new one exists there only after the merge.

## Quill's own repository

Quill is not on Flathub. Each published release runs `flatpak.yml`, which
builds the manifest for both CPUs, attaches `quill-linux-<arch>.flatpak` to the
release and asks the site to rebuild. The Pages job runs
`scripts/site-flatpak-repo.sh`: it imports the latest release's bundles into a
fresh OSTree repo signed with `FLATPAK_GPG_KEY`, and publishes

- `https://shinycake.github.io/quill/flatpak/repo/`, the repo,
- `quill.flatpakref`, which installs Quill and pulls the GNOME runtime from
  Flathub,
- `quill.flatpakrepo`, which adds the repo only,
- `quill.gpg`, the public key (also in `quill-repo.gpg` here).

The private key lives outside the repo, in
`~/.config/quill-release/flatpak-signing.asc` on the owner's Mac. Losing it
means a new key, and every user re-adding the remote. For a release from before
this workflow, run `flatpak.yml` by hand with its tag.

## A new release

Nothing to do: `flatpak.yml` runs `bash packaging/flatpak/bump.sh <tag>` on
the published release before it builds. Run the same command and commit it
when the manifest in the repo should name a newer release, for example before
changing the manifest.
