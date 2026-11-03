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
