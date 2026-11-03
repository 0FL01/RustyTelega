//! One conversation in a channel's direct messages, as the channel's
//! admins see it (`directMessagesChatTopic`, schema line 4400; tdesktop
//! `Data::SavedSublist` with a `parentChat`).
#![deny(clippy::indexing_slicing)]

use super::*;
use serde_json::Value;

/// `directMessagesChatTopic`: the messages one user (or chat) sent the
/// channel, and the channel's answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectMessagesTopic {
    /// The direct messages chat.
    pub chat_id: i64,
    pub id: i64,
    /// Who writes to the channel here.
    pub sender: MessageSender,
    /// Topics sort by `order` descending.
    pub order: i64,
    /// The admins let this user write without paying.
    pub can_send_unpaid_messages: bool,
    pub is_marked_as_unread: bool,
    pub unread_count: i32,
    pub last_read_inbox_message_id: i64,
    pub last_read_outbox_message_id: i64,
    pub unread_reaction_count: i32,
    pub last_message_id: i64,
    pub last_message_date: i32,
    /// `MessageContent::preview` of `last_message`; empty when none.
    pub last_message_preview: String,
    pub last_message_outgoing: bool,
    pub draft: Option<ChatDraft>,
}

impl DirectMessagesTopic {
    /// The user behind the topic, when a user writes (a channel can write
    /// too, as itself).
    pub fn user_id(&self) -> Option<i64> {
        match self.sender {
            MessageSender::User { user_id } => Some(user_id),
            MessageSender::Chat { .. } => None,
        }
    }

    /// Unread for its tab and row: messages the admins have not read, or
    /// marked unread by hand.
    pub fn is_unread(&self) -> bool {
        self.unread_count > 0 || self.is_marked_as_unread
    }
}

pub(crate) fn parse_direct_messages_topic(value: &Value) -> Option<DirectMessagesTopic> {
    let last_message = value
        .get("last_message")
        .filter(|message| !message.is_null())
        .and_then(|message| parse_message(message).ok());
    Some(DirectMessagesTopic {
        chat_id: int53(value.get("chat_id")).ok()?,
        id: int53(value.get("id")).ok()?,
        sender: parse_message_sender(value.get("sender_id")).ok()?,
        order: int53_or_zero(value.get("order")),
        can_send_unpaid_messages: value
            .get("can_send_unpaid_messages")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_marked_as_unread: value
            .get("is_marked_as_unread")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        unread_count: int53_or_zero(value.get("unread_count")).sat_i32(),
        last_read_inbox_message_id: int53_or_zero(value.get("last_read_inbox_message_id")),
        last_read_outbox_message_id: int53_or_zero(value.get("last_read_outbox_message_id")),
        unread_reaction_count: int53_or_zero(value.get("unread_reaction_count")).sat_i32(),
        last_message_id: last_message.as_ref().map_or(0, |m| m.id.0),
        last_message_date: last_message.as_ref().map_or(0, |m| m.date),
        last_message_outgoing: last_message.as_ref().is_some_and(|m| m.is_outgoing),
        last_message_preview: last_message
            .map(|m| effective_content(&m.content, m.ephemeral.as_ref()).preview())
            .unwrap_or_default(),
        draft: parse_chat_draft(value.get("draft_message")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_topic_parses_with_its_sender_and_counts() {
        let json = r#"{"@type":"directMessagesChatTopic","chat_id":-1000000000082,"id":"501","sender_id":{"@type":"messageSenderUser","user_id":501},"order":"9000","can_send_unpaid_messages":true,"is_marked_as_unread":false,"unread_count":3,"last_read_inbox_message_id":1048576,"last_read_outbox_message_id":0,"unread_reaction_count":0,"last_message":{"id":2097152,"chat_id":-1000000000082,"date":1700000000,"is_outgoing":false,"content":{"@type":"messageText","text":{"@type":"formattedText","text":"Hello there","entities":[]}}},"draft_message":null}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let topic = parse_direct_messages_topic(&value).unwrap();
        assert_eq!(topic.chat_id, -1_000_000_000_082);
        assert_eq!(topic.id, 501);
        assert_eq!(topic.user_id(), Some(501));
        assert_eq!(topic.order, 9000);
        assert!(topic.can_send_unpaid_messages);
        assert_eq!(topic.unread_count, 3);
        assert_eq!(topic.last_read_inbox_message_id, 1_048_576);
        assert_eq!(topic.last_message_id, 2_097_152);
        assert_eq!(topic.last_message_preview, "Hello there");
        assert!(!topic.last_message_outgoing);
        assert!(topic.is_unread());
    }

    #[test]
    fn a_topic_without_a_last_message_or_sender() {
        let empty: Value = serde_json::from_str(
            r#"{"chat_id":5,"id":"7","sender_id":{"@type":"messageSenderChat","chat_id":-100},"order":"1","is_marked_as_unread":true,"last_message":null}"#,
        )
        .unwrap();
        let topic = parse_direct_messages_topic(&empty).unwrap();
        assert_eq!(topic.user_id(), None);
        assert_eq!(topic.last_message_id, 0);
        assert!(topic.is_unread(), "marked unread by hand");
        let broken: Value = serde_json::from_str(r#"{"chat_id":5,"id":"7"}"#).unwrap();
        assert!(parse_direct_messages_topic(&broken).is_none());
    }
}
