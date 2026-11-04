# Sliding the side panel column

Owner request (Idan, 2026-11-03): animate the right side panel (the chat
info / profile column, and with it the downloads and shared media panels)
when it opens and closes. It should feel like Telegram Desktop, stay
smooth with no relayout jank, keep the chat content from jumping, and
follow reduced motion.

## What tdesktop does

Telegram Desktop shows its third column at once. `MainWidget` creates or
destroys `_thirdSection` and lays the columns out again
(`mainwidget.cpp`, `updateControlsGeometry`); the only slides are inside a
column (`Window::SlideAnimation` between sections) and the chat list's
width (`_a_dialogsWidth`, `anim::easeOutCirc`). So there is no tdesktop
motion to copy for the column itself. Quill uses the motion it already has
for the chat list column (`sidebar_reveal`, docs/decisions/sidebar-hide.md),
mirrored to the right edge.

## What it does

- **Open.** The column's share of its width in the layout glides from 0 to
  1 over 200 ms with ease-out cubic (`panel_reveal`, `SLIDE_DURATION`).
  The panels keep their full width inside a clipped column and lead in
  with their left edge, so nothing in them reflows while it slides. The
  conversation narrows by the same amount each frame, the way it does when
  the chat list hides.
- **Close.** The share glides back to 0. A closed panel is gone from
  state, so the column draws the panels it last had while it slides out
  (`PanelReveal::drawn`, `QuillApp::shown_info_target`). The leaving
  column takes no clicks. Shared media and Earn pages drop or move their
  data on close and cannot be drawn again; their column closes at once.
- **Switching** between panels while the column stays open (another
  profile, info to shared media) does not slide; the width changes at
  once, as in tdesktop.
- **No room.** When the panel fills the window in place of the chat
  (narrow windows, one column) it does not slide.
- **Reduced motion.** The OS reduce-motion setting and the "Interface
  animations" power-saving switch snap the column (`Glide::go`).

## State

Which panels are open is session state. `PanelReveal::settle` runs once per
frame before the layout and follows it; the shell reads the share
(`ShellInput::panels_shown`) and the panels to draw, and nothing else
mirrors the open panels.

## Checked

Unit tests in `panel_reveal.rs` and `shell_layout/tests.rs`; `panel_slide_tests.rs` steps a test clock through an open and a close in a real window (tests without a clock snap, so nothing waits on the wall clock). Frame captures
with `QUILL_MOTION_HOLD_MS` of `ready-info-user-bio`: mid-open (light) and
mid-close (dark) show the panel clipped at full width beside a narrowed
chat. Not checked: Linux and Windows, a live account.
