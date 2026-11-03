# channel-direct-messages: writing to a channel's direct messages (`parity:chrome-channel-direct-messages`)

## tdesktop
- A channel's direct messages group (a monoforum) takes messages from anyone. The writer never
  joins it, so the composer shows even though TDLib reports the viewer as `chatMemberStatusLeft`.
- The top bar shows a back arrow to the channel, the channel's name with its badges and the line
  "Direct messages" (`lng_profile_direct_messages`).
- An empty chat shows `AboutView::makeStarsPerMessage`: a round service card with the direct
  messages icon, then `lng_send_charges_stars_channel` ("{channel} charges ★N per message to its
  admin.") and a "Buy Stars" button (`EmptyChatLockedBox::Type::StarsCharged`), or
  `lng_send_free_channel` with no button when the messages are free. A private chat with a user
  who charges gets the same card with `lng_send_charges_stars_text`.
- The field reads "Message for N Stars" (`lng_message_stars_ph`). The "Suggest a post" toggle sits
  by the field for writers only (`refreshSuggestPostToggle`), and Join never shows
  (`isJoinChannel` leaves monoforums out).
- The monoforum takes the channel's name, photo and profile (`monoforumBroadcast()` in
  `PeerData::name`, `paintUserpic`, `NameValue`, the profile status label). TDLib already gives the
  direct messages chat the channel's title and photo; the header's info opens the channel's.
- The card is for writers. The channel's admins see the chats list of the direct messages instead.
- The channel's own bar is one full-width button (Join or Mute/Unmute) with the direct messages
  icon on its left edge (`HistoryWidget::setupDirectMessageButton`).

## What changed in Quill
- `quill::chat_bottom_bar` gains `ChannelFacts::direct_messages`: a direct messages group never
  gets the Join bar, which was hiding the composer.
- The header's info panel opens the channel, and the card is left out for the channel's admins.
- `quill::locked_intro` decides the card and splits its sentence around the name and the price.
  `src/ui/direct_messages_chat.rs` draws it and the header's back arrow and line. The header takes
  the channel's badges (the verified check).
- The "charges N for each message" line over the composer stays for private chats and is left
  out in direct messages, where the card and the placeholder say it.
- The channel bar is three slots: the direct messages button, the action, Discuss. The sides are
  the same width so the action stays centred. The direct messages button used to sit on top of
  the full-width Mute button, and one click pressed both.
- Demo `ready-direct-messages` (`QUILL_DEMO_DIRECT=channel|free`) and a UI test cover the flow.
  Buy Stars opens the Stars page; nothing is bought or sent by itself.

# The admin side (`parity:chrome-channel-direct-messages-admin`)

## tdesktop
- Admins see one sublist per person writing (`Data::SavedSublist` under the monoforum). The
  subsection tabs list them after "All", each with the person's userpic, short name and unread
  badge (a dot when the chat was marked unread by hand). Scrolling to the end loads more.
- Opening the direct messages chat opens the sublist that was open last
  (`saveActiveSubsectionThread`); "All" saves none.
- Header (`TopBarWidget::paintTopBar`): in All, the channel's name and "N chats" or "No chats"
  (`lng_filters_chats_count`, the server's full count). In a sublist, the person's name and "N
  messages" (`lng_forum_messages`), or loading.
- In All, while the channel still has direct messages, the composer gives way to "Choose a
  message to reply." (`lng_monoforum_choose_to_reply`). Replying to a message lifts it, and the
  reply goes to that message's sublist.
- A sublist's "..." menu (`Filler::fillMonoforumPeerActions`) exists only while messages cost
  Stars: Ban or Unban the person in the channel, Remove Fee or Charge Fee, a separator, and a
  disabled line saying what the person pays now (`lng_context_fee_now` / `lng_context_fee_free`).
  Remove Fee opens the same box as a private chat (refund checkbox when the person paid);
  Charge Fee runs at once.
- A tab's context menu (`fillContextMenuActions` for a sublist): Mark as read or unread, Ban or
  Unban in the channel (only with the right to restrict members), Delete chat
  (`DeleteSublistBox`: userpic, "Delete chat", `lng_sure_delete_history`, Delete). Ban asks
  first (`lng_profile_sure_kick_channel`, Remove); Unban runs at once.
- Clear History, auto-delete, Boost and Statistics are not offered for a monoforum
  (`addClearHistory`, `addTTLSubmenu`, `addBoostChat`, `addViewStatistics`).

## What changed in Quill
- TDLib 1.8.69 calls: `loadDirectMessagesChatTopics`, `getDirectMessagesChatTopic`,
  `getDirectMessagesChatTopicHistory`, `deleteDirectMessagesChatTopicHistory`, `setDirectMessagesChatTopicIsMarkedAsUnread`,
  `toggleDirectMessagesChatTopicCanSendUnpaidMessages`, `getDirectMessagesChatTopicRevenue`, and
  `setChatMemberStatus` on the channel for bans. `updateDirectMessagesChatTopic` lands in
  `threads.direct` (`src/state/domains/threads/direct_messages.rs`).
- One source of truth: `threads.direct.open` maps the chat to the open sublist and stays after
  the chat is left. The chat's main history shows that sublist: every page request goes through
  `main_history_request`, live messages of other sublists are dropped, and the unread anchor and
  "has newer" come from the sublist. Sends, drafts and `viewMessages` carry the sublist.
- The tabs reuse the forum subsection tabs (`Tab::Direct`), so Top, Bottom and Left all work.
- The jump-down counter shows the open sublist's unread messages, or all of them in All.
- The writer's direct messages button on the channel bar follows
  `supergroupFullInfo.direct_messages_chat_id`, so it goes away live when the channel turns
  direct messages off.
- Demo: `QUILL_DEMO_DIRECT=admin` (Maya's chat open) and `admin-all` (the reply lock). UI tests
  cover the lock and its release, opening a tab, the menus' entries and the channel button.

## Differences and follow-ups
- "N chats" counts the loaded conversations; tdesktop shows the server's full count, which
  TDLib does not expose.
- "N messages" is `total_count` of the first page of the open sublist.
- The Ban/Unban label asks `getChatMember` when the menu opens and reads "Ban" until the answer;
  tdesktop already knows the state.
- Not built: the chat-list row in forum style with jump chips per sublist, the thread bars over
  each sublist's messages in All, Open in New Window for one sublist, and forwarding into a
  person's sublist.
