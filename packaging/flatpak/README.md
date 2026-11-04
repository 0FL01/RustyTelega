# Quill's Flatpak

`io.github.shinycake.Quill.yml` repackages the Linux release tarballs; it does
not compile Quill. For each CPU it installs the relocatable
`quill-linux-<arch>-bundle.tar.gz` into `/app/lib/quill`, links
`/app/bin/quill` to it, and renames the desktop file and icons to the app ID.
The AppStream file here is a relative symlink to
`assets/linux/io.github.shinycake.Quill.metainfo.xml`. Keep generated manifests
in this directory so that Flatpak Builder can resolve it.

The rationale for repackaging and the permissions is in
[docs/decisions/flatpak.md](../../docs/decisions/flatpak.md).

## Releases and installation

On a release-relevant push to `main`, `.github/workflows/release.yml` builds the x86_64 and
aarch64 Flatpaks **before** creating the draft GitHub Release. It uses the
Linux tarballs produced by that same CI run, verifies their `.sha256` files,
generates a local-source manifest, and builds and lints both architectures.
The same release contains both `quill-linux-<arch>.flatpak` files and the other
release assets; it is not assembled by a post-publication workflow.

Download the matching `.flatpak` directly from GitHub Releases. For example,
after configuring Flathub for the GNOME runtime:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user ./quill-linux-x86_64.flatpak
flatpak run io.github.shinycake.Quill
```

There is no Pages-hosted Flatpak repository, `.flatpakref`, or automatic
Flatpak update channel for these bundles. To update, download and install the
new bundle manually.

## Build locally from the current Linux bundles

The checked-in manifest still contains historical `shinycake/quill` upstream
URLs. **Do not build it directly.** Stage both tarballs and their matching
`.sha256` files from the same build in `packaging/flatpak/_release/`, then run
from the repository root:

```sh
(cd packaging/flatpak/_release && \
  sha256sum -c quill-linux-x86_64-bundle.tar.gz.sha256 \
               quill-linux-aarch64-bundle.tar.gz.sha256)
bash packaging/flatpak/bump.sh --local packaging/flatpak/_release packaging/flatpak/ci.yml
flatpak run org.flatpak.Builder --user --install --force-clean \
  --install-deps-from=flathub packaging/flatpak/build-dir packaging/flatpak/ci.yml
flatpak run io.github.shinycake.Quill
```

Install `org.flatpak.Builder` from Flathub first if needed. `--local` requires
both regular files with the exact tarball names, reads neither a release nor
the network, calculates their SHA-256 hashes, and atomically writes the output
manifest next to the checked-in one. It does **not** check the sidecar hashes:
that is the responsibility of the caller, as shown above. Flatpak Builder does
not validate the `sha256` property against a local archive, so keep the
verified tarballs unchanged until the build finishes. The script exits nonzero
on missing or ambiguous sources and leaves any previous output unchanged; do
not build an old output after a failed invocation. `ci.yml` and `_release/` are
ignored by Git; other chosen output names should not be committed either.

To lint a generated manifest and a repo exported with `--repo=repo`:

```sh
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  --exceptions --user-exceptions packaging/flatpak/lint-exceptions.json manifest packaging/flatpak/ci.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  --exceptions --user-exceptions packaging/flatpak/lint-exceptions.json repo repo
```

The local `appid-filename-mismatch` lint exception is for the temporary
`ci.yml` name; the application ID is unchanged. The other local exceptions are
for screenshots that are not mirrored in a hosted Flatpak repository.

## Bump the checked-in manifest to an already published tag

For an explicit local update only, `bash packaging/flatpak/bump.sh
vYEAR.MONTHDAY.COMMITS` reads `SHA256SUMS.txt` from that tag in
`0FL01/RustyTelega`, checks for exactly one checksum per archive, and updates
the checked-in manifest to that repository's URLs. Review the resulting diff.
The command fails if the tag or checksums are unavailable; it never falls back
to `shinycake/quill`. This mode is not used to build the current CI release.

The script deliberately rejects unexpected source layouts, extra archive
sources, and changed architecture mappings. If the manifest's source structure
must change, update the checks in `bump.sh` and its tests as well.

Run the offline fixture tests from the repository root with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s packaging/flatpak -p 'test_*.py' -v
```
