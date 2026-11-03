# shinycake/homebrew-tap

Homebrew casks for [Quill](https://shinycake.github.io/quill/), an independent,
unofficial Telegram client for macOS, Windows and Linux.

```sh
brew install --cask shinycake/tap/quill-app
```

Quill for Mac is signed with a Developer ID and notarized by Apple. It updates
itself; `brew upgrade --cask --greedy quill-app` also works. The cask follows
[Quill's releases](https://github.com/shinycake/quill/releases) automatically:
a scheduled workflow points it at each new release and audits it.

Quill is not affiliated with, endorsed by, or sponsored by Telegram.
