# Wallpapers the other side sets

A wallpaper set by the other person showed only its service message. Two
things stood between TDLib and the painter, which already handled every kind.

- `background.id` is an `int64`, which TDLib's JSON writes as a string.
  `parse_background` read it as a number, so every real background parsed to
  nothing: the chat's own, the theme's and the installed list. Tests used
  numeric ids. They now use the string form.
- A photo's file was downloaded only while the chat look picker was open. The
  open chat's files now download with its thumbnails, for the main window and
  chat windows alike, and again on `updateChatBackground`, `updateChatTheme`
  and `updateEmojiChatThemes`.

`backgroundTypeChatTheme` (boosted channels and groups) names an emoji theme
and holds no picture, so `Session::chat_wallpaper` shows that theme's background
with the chat's dimming.

Like tdesktop, the chat keeps the account wallpaper until the file is ready,
and dimming applies to photos in dark mode only.

`ready-chat-wallpaper` feeds a real `updateChatBackground` through the reducer;
`QUILL_DEMO_WALLPAPER` and `QUILL_DEMO_DIMMING` pick the case.
