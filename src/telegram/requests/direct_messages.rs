//! Requests for a channel's direct messages as its admins see them
//! (schema 1.8.69, lines 12612–12676).
#![deny(clippy::indexing_slicing)]

use crate::ids::{ChatId, MessageId, RequestId, TopicId};
use serde_json::{Value, json};

/// `loadDirectMessagesChatTopics`.
pub fn load_direct_messages_chat_topics(extra: RequestId, chat_id: ChatId, limit: i32) -> String {
    json!({
        "@type": "loadDirectMessagesChatTopics",
        "@extra": extra.as_extra(),
        "chat_id": chat_id.0,
        "limit": limit,
    })
    .to_string()
}

/// `getDirectMessagesChatTopic`.
pub fn get_direct_messages_chat_topic(extra: RequestId, chat_id: ChatId, topic_id: i64) -> String {
    topic_request("getDirectMessagesChatTopic", extra, chat_id, topic_id)
}

/// `getDirectMessagesChatTopicHistory`: the same `from_message_id` /
/// `offset` / `limit` rules as `getChatHistory`.
pub fn get_direct_messages_chat_topic_history(
    extra: RequestId,
    chat_id: ChatId,
    topic_id: i64,
    from_message_id: MessageId,
    offset: i32,
    limit: i32,
) -> String {
    json!({
        "@type": "getDirectMessagesChatTopicHistory",
        "@extra": extra.as_extra(),
        "chat_id": chat_id.0,
        "topic_id": topic_id,
        "from_message_id": from_message_id.0,
        "offset": offset,
        "limit": limit,
    })
    .to_string()
}

/// `deleteDirectMessagesChatTopicHistory`.
pub fn delete_direct_messages_chat_topic_history(
    extra: RequestId,
    chat_id: ChatId,
    topic_id: i64,
) -> String {
    topic_request(
        "deleteDirectMessagesChatTopicHistory",
        extra,
        chat_id,
        topic_id,
    )
}

/// `setDirectMessagesChatTopicIsMarkedAsUnread`.
pub fn set_direct_messages_chat_topic_is_marked_as_unread(
    extra: RequestId,
    chat_id: ChatId,
    topic_id: i64,
    is_marked_as_unread: bool,
) -> String {
    json!({
        "@type": "setDirectMessagesChatTopicIsMarkedAsUnread",
        "@extra": extra.as_extra(),
        "chat_id": chat_id.0,
        "topic_id": topic_id,
        "is_marked_as_unread": is_marked_as_unread,
    })
    .to_string()
}

/// `getDirectMessagesChatTopicRevenue`: answers `starCount`.
pub fn get_direct_messages_chat_topic_revenue(
    extra: RequestId,
    chat_id: ChatId,
    topic_id: i64,
) -> String {
    topic_request(
        "getDirectMessagesChatTopicRevenue",
        extra,
        chat_id,
        topic_id,
    )
}

/// `toggleDirectMessagesChatTopicCanSendUnpaidMessages`.
pub fn toggle_direct_messages_chat_topic_can_send_unpaid_messages(
    extra: RequestId,
    chat_id: ChatId,
    topic_id: i64,
    can_send_unpaid_messages: bool,
    refund_payments: bool,
) -> String {
    json!({
        "@type": "toggleDirectMessagesChatTopicCanSendUnpaidMessages",
        "@extra": extra.as_extra(),
        "chat_id": chat_id.0,
        "topic_id": topic_id,
        "can_send_unpaid_messages": can_send_unpaid_messages,
        "refund_payments": refund_payments,
    })
    .to_string()
}

/// A send with no topic that replies to a message of the same chat goes
/// to that message's conversation (`topic_of` maps a message id to it).
/// Anything else passes through unchanged.
pub fn route_into_direct_topic(json: &str, topic_of: impl Fn(i64) -> Option<TopicId>) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(json) else {
        return json.to_owned();
    };
    let Some(object) = value.as_object_mut() else {
        return json.to_owned();
    };
    if !object.get("topic_id").is_some_and(Value::is_null) {
        return json.to_owned();
    }
    let reply = object.get("reply_to").filter(|reply| {
        reply.get("@type").and_then(Value::as_str) == Some("inputMessageReplyToMessage")
    });
    let Some(topic) = reply
        .and_then(|reply| reply.get("message_id"))
        .and_then(Value::as_i64)
        .and_then(topic_of)
    else {
        return json.to_owned();
    };
    object.insert("topic_id".to_owned(), super::topic_id_json(&topic));
    value.to_string()
}

fn topic_request(kind: &str, extra: RequestId, chat_id: ChatId, topic_id: i64) -> String {
    json!({
        "@type": kind,
        "@extra": extra.as_extra(),
        "chat_id": chat_id.0,
        "topic_id": topic_id,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn parse(json: &str) -> Value {
        serde_json::from_str(json).unwrap_or(Value::Null)
    }

    #[test]
    fn topic_history_reads_like_chat_history() {
        let v = parse(&get_direct_messages_chat_topic_history(
            RequestId(3),
            ChatId(-1_000_000_000_082),
            501,
            MessageId(2_097_152),
            -24,
            50,
        ));
        assert_eq!(v["@type"], "getDirectMessagesChatTopicHistory");
        assert_eq!(v["@extra"], "3");
        assert_eq!(v["chat_id"], -1_000_000_000_082_i64);
        assert_eq!(v["topic_id"], 501);
        assert_eq!(v["from_message_id"], 2_097_152);
        assert_eq!(v["offset"], -24);
        assert_eq!(v["limit"], 50);
    }

    #[test]
    fn a_reply_in_the_all_view_goes_to_its_conversation() {
        let send = r#"{"@type":"sendMessage","chat_id":5,"topic_id":null,"reply_to":{"@type":"inputMessageReplyToMessage","message_id":77}}"#;
        let topic = TopicId::DirectMessages {
            direct_messages_chat_topic_id: 501,
        };
        let v = parse(&route_into_direct_topic(send, |id| {
            (id == 77).then_some(topic)
        }));
        assert_eq!(v["topic_id"]["@type"], "messageTopicDirectMessages");
        assert_eq!(v["topic_id"]["direct_messages_chat_topic_id"], 501);
        assert_eq!(route_into_direct_topic(send, |_| None), send);
        let plain = r#"{"@type":"sendMessage","chat_id":5,"topic_id":null,"reply_to":null}"#;
        assert_eq!(route_into_direct_topic(plain, |_| Some(topic)), plain);
        let set = r#"{"@type":"sendMessage","chat_id":5,"topic_id":{"@type":"messageTopicForum","forum_topic_id":2},"reply_to":{"@type":"inputMessageReplyToMessage","message_id":77}}"#;
        assert_eq!(route_into_direct_topic(set, |_| Some(topic)), set);
    }

    #[test]
    fn topic_requests_name_the_topic() {
        let v = parse(&toggle_direct_messages_chat_topic_can_send_unpaid_messages(
            RequestId(4),
            ChatId(5),
            6,
            true,
            false,
        ));
        assert_eq!(
            v["@type"],
            "toggleDirectMessagesChatTopicCanSendUnpaidMessages"
        );
        assert_eq!(v["can_send_unpaid_messages"], true);
        assert_eq!(v["refund_payments"], false);
        let v = parse(&set_direct_messages_chat_topic_is_marked_as_unread(
            RequestId(1),
            ChatId(5),
            6,
            true,
        ));
        assert_eq!(v["is_marked_as_unread"], true);
        let v = parse(&load_direct_messages_chat_topics(
            RequestId(1),
            ChatId(5),
            50,
        ));
        assert_eq!(v["limit"], 50);
        for (json, kind) in [
            (
                get_direct_messages_chat_topic(RequestId(1), ChatId(5), 6),
                "getDirectMessagesChatTopic",
            ),
            (
                delete_direct_messages_chat_topic_history(RequestId(1), ChatId(5), 6),
                "deleteDirectMessagesChatTopicHistory",
            ),
            (
                get_direct_messages_chat_topic_revenue(RequestId(1), ChatId(5), 6),
                "getDirectMessagesChatTopicRevenue",
            ),
        ] {
            let v = parse(&json);
            assert_eq!(v["@type"], kind);
            assert_eq!(v["topic_id"], 6);
        }
    }
}
