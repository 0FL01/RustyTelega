# Quill on Flathub

`io.github.shinycake.Quill.yml` builds the Flathub package. It does not compile
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
sandbox. Pull requests skip the two screenshot checks, because the screenshots
are linked from `main` and a new one exists there only after the merge.

## A new release

`bash packaging/flatpak/bump.sh vYEAR.MONTHDAY.COMMITS` points both tarball URLs
and their sha256 at that release. On Flathub, flatpak-external-data-checker
does the same by itself: the `x-checker-data` blocks read the latest GitHub
release and it opens a pull request on the Flathub repo, adding a `<release>`
to the metainfo there.

## Submitting to Flathub

Do this once, with the first release that includes the Flatpak changes in
`src/desktop_app_id.rs` (anything after v2026.1103.4940). That release sets the
window app_id to `io.github.shinycake.Quill` inside Flatpak, hides the
autostart switch, and shows agent setup commands that work outside the
sandbox. v2026.1103.4940 builds and starts in the Flatpak, but those three
things are wrong in it.

1. `bash packaging/flatpak/bump.sh <that tag>` and merge the change here.
2. Fork https://github.com/flathub/flathub on GitHub. Keep "Copy the master
   branch only" unticked: the submission branches from `new-pr`.
3. Clone the fork and branch from `new-pr`:
   ```sh
   git clone --branch new-pr git@github.com:<you>/flathub.git
   cd flathub
   git checkout -b add-quill
   ```
4. Copy the files in, resolving the metainfo symlink:
   ```sh
   cp /path/to/quill/packaging/flatpak/io.github.shinycake.Quill.yml .
   cp -L /path/to/quill/packaging/flatpak/io.github.shinycake.Quill.metainfo.xml .
   ```
   No `flathub.json` is needed: the manifest builds for both x86_64 and aarch64,
   which is Flathub's default.
5. Commit, push, and open a pull request against `flathub/flathub` with base
   branch `new-pr` and the title `Add io.github.shinycake.Quill`. Fill in the
   template's checklist.
6. Reviewers comment on the pull request. A comment of `bot, build` from them
   (or you) starts a test build. Once accepted, Flathub creates
   `flathub/io.github.shinycake.Quill` and invites the submitter as maintainer.
   Later changes go to that repo, not to `flathub/flathub`.

The app id `io.github.shinycake.Quill` is verified through the GitHub account
`shinycake`: after the app is published, sign in to flathub.org with that
account to get the verified badge.

Reviewers may ask why Quill is not built from source; the decision doc has the
answer to give.
