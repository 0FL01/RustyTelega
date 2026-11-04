# Opening a chat at its unread messages

## What tdesktop does

- `HistoryWidget::countInitialScrollTop` scrolls so the "Unread messages" bar
  is at the very top of the view: `unreadBarTop()` is the bar's top plus its
  margin, plus the date badge above it, so neither the last read message nor
  the date shows above the bar.
- `countAutomaticScrollTop` only creates the bar when the first unread
  message is above `scrollTopMax + bar height - margin`. When the whole
  unread run fits on the last screen it opens at the bottom with no bar.
- The bar reads `lng_unread_bar_some` ("Unread messages"), with no count
  (`History::addUnreadBar`).
- The down button (`CornerButtons::downClick`) shows the history at
  `ShowAtUnreadMsgId`: the first unread message under a new bar while any
  are left, the bottom otherwise. Its badge is the chat's unread counter.
- `cornerButtonsDownShown`: the button shows past `historyToDownShownAfter`
  (480px) from the bottom, while unread messages are below the view, and
  always while the history is not loaded at its bottom.
- The history stays hidden until the first requested page is in
  (`_firstLoadRequest`), so nothing that arrives earlier decides where it
  opens.
- Later loads keep the scroll in place: `updateHistoryGeometry` holds the
  item at the top of the view, or the bottom when the view was there.
- A message jumped to sits in the middle of the view when it fits
  (`itemTopForHighlight`), at its top otherwise.
- Sending scrolls to the end (`fastShowAtEnd`, and `newItemAdded` for an
  item that is still sending), and any new outgoing message destroys the
  bar (`History::newItemAdded`).
- Choosing the chat that is already open goes to the first unread message
  while it is below the view, else to the end
  (`insideJumpToEndInsteadOfToUnread`).

## What Quill does

All of it lives in two places. `src/ui/conversation/landing.rs` brings the
history to a goal, and `src/ui/conversation/view_hold.rs` keeps the view
still while rows change.

- **Waiting for the first page.** While a page that starts the window is in
  flight, the history shows the skeleton, even when live messages already
  joined the window (`Session::history_loading`). Before, one live message
  could paint first. The bar then had no room, so the landing dropped it
  and the chat opened at the bottom.
- **The landing.** A new window lands once, when its first rows are in:
  the saved position when closing a reply thread, else the jump target,
  else the bar, else the bottom (`anchor_opened_history`). The bar's row
  goes to the top with the offset found last time. After each paint the
  history measures where the target is (`landing_error`) and scrolls by
  the difference (`MessageScrollerState::scroll_by`, a vendored kit
  patch).
- **A bar held by the end.** When the bar can't reach the top because the
  list is at its end, the decision depends on what is still to load. While
  the window stops short of the chat's latest message, the landing waits.
  The next newer page joins below, and the next paint brings the bar up.
  Once the window reaches the latest, the run fits on the last screen. The
  bar goes (`Session::hide_unread_bar`), the history follows its tail, and
  reads stay held back (`opened_with_unread`). A short first page used to
  drop the bar while dozens of unread messages were still unloaded.
- **Jumps.** A search hit, a reply, a pinned message or a link puts the
  message in the middle of the view, through the same landing
  (`LandingGoal::Message`). Before, it sat at the top edge.
- **The reader takes over.** A wheel or trackpad scroll over the history,
  any click in it, a history key, or a jump (to the latest, to the first
  unread) ends a landing, so a page arriving later never pulls the reader
  back. The scroll is caught by a capture listener painted before the list
  (`landing_scroll_watch`): the list's bounce takes wheel events in the
  capture phase and stops them, so a listener on the history's own element
  never hears them. A jump target whose row goes ends the landing after a
  few paints.
- **Holding the view.** Before rows change (pages at either end, both in
  one tick, deletions, edits, albums growing), the history notes the
  message at the top of the view and how far into its row the view is.
  After, it scrolls that message's row back to the same offset. A history
  following its latest message keeps following. A window that stops short
  of the latest has no tail to follow, so a newer page joins below what
  the reader sees instead of skipping to its end. Before, the list's own
  splice moved the anchor to the first changed row, so two pages in one
  tick showed the oldest loaded messages, and a newer page put the old
  last row at the top. Media that finishes loading remeasures rows with
  their top edge kept (absolute), not proportionally.
- **Sending.** A message this app is sending (TDLib's
  `messageSendingStatePending`) that joins rows which reach the chat's
  latest brings the history to its end. A failed send does not, nor does a
  page of newer history that happens to hold a message still queued: pages
  come only to a window that stops short of the latest. A bot reply still
  streaming (a local row with a negative id) is skipped when finding the
  newest message. Any outgoing message ends the bar
  (`Session::route_new_message_into_window`).
- **Choosing the open chat again.** It goes to the first unread message
  while that is below the view, else to the latest
  (`QuillApp::reselect_open_chat`). Before the history has landed (its
  first page still loading) it does nothing: the landing already goes to
  the first unread.
- The jump-down button calls `ConnectDriver::jump_down`, which brings the
  bar back at the chat's read boundary (`Session::show_first_unread`). If
  the window holds that message, it re-anchors there. If not, it loads the
  window around the boundary. With nothing unread, it jumps to the latest
  messages as before. The End key still goes to the latest.
- The kit scroller gained `with_jump_button_shown_after` and
  `jump_button_always` (third_party/gpui-component/QUILL-CHANGES.md). Quill
  passes 480px, or zero while the chat has unread messages (they're below
  the view, since rows on screen are read), and `always` while the window
  stops short of the latest.

Reply threads keep their own bar and jump to their latest replies. The
landing and the view hold apply to them too, but the jump-down change
doesn't.

## Not done yet

- Forum topics open at their latest messages with no bar. tdesktop opens
  them at the first unread.
- A notification for a mention in a group opens the chat at its first
  unread message. tdesktop opens that exact message
  (`Manager::openNotificationMessage`).

## Tests

`src/ui/conversation/open_position_tests.rs` plays the driver by hand on
the demo session: each test asks for pages the way `ConnectDriver` does and
answers them the way TDLib does, late, short, or several in one tick, then
checks where the paint put the bar or the rows. The first seven failed
before this change; the others cover sending with the bar up, scrolling
while the bar waits for a page, and choosing the open chat again.
