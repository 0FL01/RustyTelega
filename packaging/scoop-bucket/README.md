# shinycake/scoop-bucket

[Scoop](https://scoop.sh) bucket for [Quill](https://shinycake.github.io/quill/),
an independent, unofficial Telegram client for Windows, macOS and Linux.

```powershell
scoop bucket add shinycake https://github.com/shinycake/scoop-bucket
scoop install shinycake/quill
```

`quill` on the command line is Quill's agent CLI (`quill agent ...`); the app
itself is in the Start menu under Scoop Apps. A scheduled workflow follows
[Quill's releases](https://github.com/shinycake/quill/releases), so
`scoop update quill` picks up each one.

Quill is not affiliated with, endorsed by, or sponsored by Telegram.
