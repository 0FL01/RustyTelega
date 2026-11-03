//! A channel's direct messages as its admins see them: one conversation per
//! user (tdesktop `Data::SavedMessages` owned by a monoforum, its
//! `SavedSublist`s).
#![deny(clippy::indexing_slicing)]

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::telegram::envelope::DirectMessagesTopic;

/// The conversations of every administered direct messages chat.
#[derive(Debug, Default)]
pub struct DirectMessagesTopics {
    /// Conversations by chat id, then topic id.
    pub topics: HashMap<i64, BTreeMap<i64, DirectMessagesTopic>>,
    /// The conversation shown in each chat; none = the All view. It stays
    /// after the chat is left, so opening the chat again lands on it
    /// (tdesktop `activeSubsectionThread`).
    pub open: HashMap<i64, i64>,
    /// `loadDirectMessagesChatTopics` answered 404: all are loaded.
    pub exhausted: HashSet<i64>,
    /// Chats whose first `loadDirectMessagesChatTopics` was sent.
    pub requested: HashSet<i64>,
    /// `true` = banned from the channel, by `(channel chat id, user id)`
    /// (tdesktop `kickedValue`). Missing = not asked yet.
    pub kicked: HashMap<(i64, i64), bool>,
    /// Stars a conversation has paid, by `(chat id, topic id)`
    /// (`getDirectMessagesChatTopicRevenue`).
    pub revenue: HashMap<(i64, i64), i64>,
    /// `getDirectMessagesChatTopicHistory`'s `total_count` per open
    /// conversation, for the header's "N messages".
    pub message_counts: HashMap<(i64, i64), i32>,
}

impl DirectMessagesTopics {
    /// The conversations of one chat in list order: `order` descending.
    pub fn ordered(&self, chat_id: i64) -> Vec<&DirectMessagesTopic> {
        let mut topics: Vec<&DirectMessagesTopic> = self
            .topics
            .get(&chat_id)
            .map(|topics| topics.values().filter(|t| t.order != 0).collect())
            .unwrap_or_default();
        topics.sort_by(|a, b| b.order.cmp(&a.order).then(a.id.cmp(&b.id)));
        topics
    }

    pub fn get(&self, chat_id: i64, topic_id: i64) -> Option<&DirectMessagesTopic> {
        self.topics.get(&chat_id)?.get(&topic_id)
    }

    pub fn open_topic(&self, chat_id: i64) -> Option<i64> {
        self.open.get(&chat_id).copied()
    }

    /// Conversations with unread messages or marked unread: the chat-list
    /// badge counts these (tdesktop `CountInBadge::Chats`).
    pub fn unread_topics(&self, chat_id: i64) -> usize {
        self.topics.get(&chat_id).map_or(0, |topics| {
            topics
                .values()
                .filter(|t| t.order != 0 && t.is_unread())
                .count()
        })
    }

    /// Unread messages across every conversation (the All view's
    /// jump-down counter, tdesktop `chatListUnreadState().messages`).
    pub fn unread_messages(&self, chat_id: i64) -> i32 {
        self.topics.get(&chat_id).map_or(0, |topics| {
            topics
                .values()
                .map(|t| t.unread_count.max(0))
                .fold(0i32, i32::saturating_add)
        })
    }
}
