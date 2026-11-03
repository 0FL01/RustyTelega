//! The card an empty chat shows when writing there costs Stars, or when it
//! is a channel's direct messages (tdesktop `AboutView::makeStarsPerMessage`
//! with `EmptyChatLockedBox::Type::StarsCharged` or `FreeDirect`).
#![deny(clippy::indexing_slicing)]

use crate::lang::{Key, keys, tr};

/// Which card the empty chat shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockedIntro {
    /// Each message costs `stars`; the card offers to buy Stars.
    StarsCharged { stars: i64, channel: bool },
    /// A channel's direct messages that are free to write.
    FreeDirect,
}

/// One run of the card's sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntroPiece {
    Text(String),
    /// The user's or channel's name, drawn bold.
    Name(String),
    /// The price, drawn as a star and the bold number.
    Stars(i64),
}

impl LockedIntro {
    /// The card for an empty chat, if one applies: a channel's direct
    /// messages always have one, a private chat only when it charges.
    pub fn for_chat(direct_messages: bool, stars: i64) -> Option<Self> {
        match (direct_messages, stars) {
            (true, 0) => Some(Self::FreeDirect),
            (channel, stars) if stars > 0 => Some(Self::StarsCharged { stars, channel }),
            _ => None,
        }
    }

    /// Whether the card carries the "Buy Stars" button.
    pub fn buys_stars(self) -> bool {
        matches!(self, Self::StarsCharged { .. })
    }

    /// The sentence, split where the name and the price go.
    pub fn pieces(self, name: &str) -> Vec<IntroPiece> {
        let (key, name_tag): (Key, &str) = match self {
            Self::StarsCharged { channel: false, .. } => (keys::SEND_CHARGES_STARS_TEXT, "user"),
            Self::StarsCharged { channel: true, .. } => {
                (keys::SEND_CHARGES_STARS_CHANNEL, "channel")
            }
            Self::FreeDirect => (keys::SEND_FREE_CHANNEL, "channel"),
        };
        let stars = match self {
            Self::StarsCharged { stars, .. } => stars,
            Self::FreeDirect => 0,
        };
        split_template(tr(key), name_tag, name, stars)
    }
}

/// `template` with `{name_tag}` and `{amount}` turned into their pieces.
fn split_template(template: &str, name_tag: &str, name: &str, stars: i64) -> Vec<IntroPiece> {
    let name_mark = format!("{{{name_tag}}}");
    let mut pieces = Vec::new();
    let mut rest = template;
    loop {
        let name_at = rest.find(&name_mark).map(|at| (at, name_mark.len(), true));
        let amount_at = rest
            .find("{amount}")
            .map(|at| (at, "{amount}".len(), false));
        let next = match (name_at, amount_at) {
            (Some(a), Some(b)) => Some(if a.0 < b.0 { a } else { b }),
            (a, b) => a.or(b),
        };
        let Some((at, len, is_name)) = next else {
            if !rest.is_empty() {
                pieces.push(IntroPiece::Text(rest.to_string()));
            }
            return pieces;
        };
        let (before, from) = rest.split_at(at);
        if !before.is_empty() {
            pieces.push(IntroPiece::Text(before.to_string()));
        }
        pieces.push(if is_name {
            IntroPiece::Name(name.to_string())
        } else {
            IntroPiece::Stars(stars)
        });
        rest = from.get(len..).unwrap_or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::{IntroPiece, LockedIntro};

    #[test]
    fn direct_messages_always_get_a_card() {
        assert_eq!(
            LockedIntro::for_chat(true, 0),
            Some(LockedIntro::FreeDirect)
        );
        assert_eq!(
            LockedIntro::for_chat(true, 50),
            Some(LockedIntro::StarsCharged {
                stars: 50,
                channel: true
            })
        );
        assert_eq!(LockedIntro::for_chat(false, 0), None);
        assert!(!LockedIntro::FreeDirect.buys_stars());
    }

    #[test]
    fn the_channel_sentence_names_the_channel_and_the_price() {
        let pieces = LockedIntro::StarsCharged {
            stars: 50,
            channel: true,
        }
        .pieces("Quill Weekly");
        assert_eq!(
            pieces,
            vec![
                IntroPiece::Name("Quill Weekly".into()),
                IntroPiece::Text(" charges ".into()),
                IntroPiece::Stars(50),
                IntroPiece::Text(" per message to its admin.".into()),
            ]
        );
    }

    #[test]
    fn a_user_who_charges_reads_for_each_message() {
        let pieces = LockedIntro::StarsCharged {
            stars: 9,
            channel: false,
        }
        .pieces("Dana");
        assert_eq!(pieces.first(), Some(&IntroPiece::Name("Dana".into())));
        assert_eq!(
            pieces.last(),
            Some(&IntroPiece::Text(" for each message.".into()))
        );
    }

    #[test]
    fn free_direct_messages_end_with_the_channel() {
        let pieces = LockedIntro::FreeDirect.pieces("Quill Weekly");
        assert_eq!(
            pieces,
            vec![
                IntroPiece::Text("Send a direct message to the administrator of ".into()),
                IntroPiece::Name("Quill Weekly".into()),
                IntroPiece::Text(".".into()),
            ]
        );
    }
}
