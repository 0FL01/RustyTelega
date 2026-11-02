//! `QUILL_DEMO_SHOWCASE=poll-gallery`: every state of the poll bubble, in
//! Hebrew and English, appended to the showcase's group chat.
#![deny(clippy::indexing_slicing)]

use super::showcase_demo::{LUNCH, poll_option};

const FIRST_MESSAGE: i64 = 64 * 1_048_576;

struct GalleryPoll<'a> {
    question: &'a str,
    options: Vec<String>,
    total: i32,
    outgoing: bool,
    anonymous: bool,
    multiple: bool,
    closed: bool,
    can_see_results: bool,
    poll_type: &'a str,
}

impl Default for GalleryPoll<'_> {
    fn default() -> Self {
        Self {
            question: "",
            options: Vec::new(),
            total: 0,
            outgoing: false,
            anonymous: true,
            multiple: false,
            closed: false,
            can_see_results: true,
            poll_type: r#"{"@type":"pollTypeRegular"}"#,
        }
    }
}

fn message_json(index: usize, date: i64, poll: &GalleryPoll) -> String {
    let id = FIRST_MESSAGE + index as i64 * 1_048_576;
    let poll_id = 9200 + index as i64;
    let question = serde_json::to_string(poll.question).unwrap_or_default();
    let options = poll.options.join(",");
    let GalleryPoll {
        total,
        outgoing,
        anonymous,
        multiple,
        closed,
        can_see_results,
        poll_type,
        ..
    } = *poll;
    format!(
        r#"{{"@type":"updateNewMessage","message":{{"id":{id},"chat_id":{LUNCH},"sender_id":{{"@type":"messageSenderUser","user_id":104}},"is_outgoing":{outgoing},"date":{date},"content":{{"@type":"messagePoll","poll":{{"@type":"poll","id":{poll_id},"question":{{"@type":"formattedText","text":{question},"entities":[]}},"options":[{options}],"total_voter_count":{total},"is_anonymous":{anonymous},"allows_multiple_answers":{multiple},"allows_revoting":true,"is_closed":{closed},"can_see_results":{can_see_results},"vote_restriction_reason":null,"type":{poll_type}}},"description":{{"@type":"formattedText","text":"","entities":[]}},"can_add_option":false}}}}}}"#
    )
}

/// The gallery's messages, oldest first.
pub(super) fn poll_gallery(now: i64) -> Vec<String> {
    let unvoted = |id, text| poll_option(id, text, 0, 0, false);
    let polls = [
        GalleryPoll {
            question: "שאלה לכל מי שהצביע בבחירות האחרונות:\nמה התשובה הנכונה לשאלה הזאת, לדעתכם, אחרי כל מה שקרה בשבוע האחרון בעיר הזאת?",
            options: vec![
                poll_option(
                    "a",
                    "התשובה הראשונה היא ארוכה מספיק כדי להתפרס על יותר משורה אחת בבועה",
                    6690,
                    44,
                    false,
                ),
                poll_option("b", "כן", 3045, 20, true),
                poll_option(
                    "c",
                    "לא, בשום אופן לא, וזה גם לא משנה מה יגידו על זה בסוף היום הזה",
                    2436,
                    16,
                    false,
                ),
                poll_option("d", "אולי", 1827, 12, false),
                poll_option("e", "ה", 762, 5, false),
                poll_option("f", "כל התשובות Telegram 2026 נכונות", 467, 3, false),
            ],
            total: 15227,
            ..Default::default()
        },
        GalleryPoll {
            question: "לאן נצא לאכול בשישי?",
            options: vec![
                unvoted("a", "סושי"),
                unvoted(
                    "b",
                    "פיצה עם הרבה תוספות, כי כולם רעבים אחרי שבוע ארוך במיוחד במשרד",
                ),
                unvoted("c", "2026"),
                unvoted("d", "Burger Bar"),
            ],
            total: 41,
            anonymous: false,
            ..Default::default()
        },
        GalleryPoll {
            question: "מהי עיר הבירה של אוסטרליה?",
            options: vec![
                poll_option("a", "סידני", 12, 48, true),
                poll_option("b", "קנברה", 9, 36, false),
                poll_option("c", "מלבורן", 4, 16, false),
            ],
            total: 25,
            poll_type: r#"{"@type":"pollTypeQuiz","correct_option_ids":[1],"explanation":{"@type":"formattedText","text":"קנברה נבחרה כפשרה בין סידני למלבורן.","entities":[]}}"#,
            ..Default::default()
        },
        GalleryPoll {
            question: "באילו ימים אתם פנויים? (אפשר לבחור כמה)",
            options: vec![
                poll_option("a", "ראשון", 8, 67, true),
                poll_option("b", "שני", 3, 25, false),
                poll_option("c", "שלישי", 12, 100, true),
            ],
            total: 12,
            multiple: true,
            closed: true,
            ..Default::default()
        },
        GalleryPoll {
            question: "Which editor do you use the most?",
            options: vec![
                unvoted("a", "Zed"),
                unvoted(
                    "b",
                    "Neovim, with far too many plugins that took a whole weekend to set up",
                ),
                unvoted("c", "Something else"),
            ],
            total: 128,
            ..Default::default()
        },
        GalleryPoll {
            question: "Pick every snack you want at the offsite",
            options: vec![
                unvoted("a", "Pretzels"),
                unvoted("b", "Fruit"),
                unvoted("c", "Trail mix"),
            ],
            multiple: true,
            ..Default::default()
        },
        GalleryPoll {
            question: "Which planet has the shortest day?",
            options: vec![
                poll_option("a", "Jupiter", 30, 60, true),
                poll_option("b", "Mercury", 15, 30, false),
                poll_option("c", "Venus", 5, 10, false),
            ],
            total: 50,
            outgoing: true,
            poll_type: r#"{"@type":"pollTypeQuiz","correct_option_ids":[0],"explanation":{"@type":"formattedText","text":"","entities":[]}}"#,
            ..Default::default()
        },
        GalleryPoll {
            question: "Ship on Tuesday or wait a week?",
            options: vec![
                poll_option("a", "Ship on Tuesday", 7, 70, true),
                poll_option("b", "Wait a week", 3, 30, false),
            ],
            total: 10,
            closed: true,
            outgoing: true,
            ..Default::default()
        },
        GalleryPoll {
            question: "Secret ballot: new team name?",
            options: vec![
                poll_option("a", "Night owls", 0, 0, true),
                poll_option("b", "Early birds", 0, 0, false),
            ],
            total: 9,
            can_see_results: false,
            ..Default::default()
        },
    ];
    // `QUILL_DEMO_POLLS=2,3`: only these, so a window can hold them.
    let only = std::env::var("QUILL_DEMO_POLLS").unwrap_or_default();
    let only: Vec<usize> = only
        .split(',')
        .filter_map(|part| part.parse().ok())
        .collect();
    polls
        .iter()
        .enumerate()
        .filter(|(index, _)| only.is_empty() || only.contains(index))
        .map(|(index, poll)| message_json(index, now - 3600 + index as i64 * 60, poll))
        .collect()
}
