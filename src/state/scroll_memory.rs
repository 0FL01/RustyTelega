//! Where each history was left when it is not read to the tail, for going
//! back to it from a reply thread opened over it.
//!
//! Telegram Desktop keeps `History::scrollTopItem` / `scrollTopOffset` for
//! every history, counted on each scroll and forgotten while the view sits
//! at the bottom (`forgetScrollState`). Quill keeps the same rule over
//! message ids and pixel offsets, but a chat, topic or thread that is
//! opened always lands at its first unread message, else at the bottom
//! (`docs/decisions/sweep-scroll-restore.md`): opening drops the entry, and
//! only closing a thread brings its chat back to where it was left.
use super::*;
use std::collections::{HashMap, VecDeque};

/// Positions kept before the least recently saved one goes.
pub const SCROLL_MEMORY_CAP: usize = 300;

/// A history view: chat, forum topic, reply thread (`0` for none).
pub type ScrollKey = (i64, Option<i32>, i64);

/// The top visible row of a history: the message it starts with and how
/// far its top edge is above the viewport top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedScroll {
    pub anchor: MessageId,
    pub offset: f32,
}

#[derive(Debug, Default)]
pub struct ScrollMemory {
    saved: HashMap<ScrollKey, SavedScroll>,
    order: VecDeque<ScrollKey>,
}

impl ScrollMemory {
    /// Remember `scroll` for `key`; `None` is the view at the tail (or
    /// with nothing to anchor to) and forgets it.
    pub fn record(&mut self, key: ScrollKey, scroll: Option<SavedScroll>) {
        let Some(scroll) = scroll else {
            self.forget(key);
            return;
        };
        if self.saved.get(&key) == Some(&scroll) {
            return;
        }
        if self.saved.insert(key, scroll).is_some() {
            if self.order.back() == Some(&key) {
                return;
            }
            self.order.retain(|queued| *queued != key);
        }
        self.order.push_back(key);
        while self.order.len() > SCROLL_MEMORY_CAP {
            if let Some(oldest) = self.order.pop_front() {
                self.saved.remove(&oldest);
            }
        }
    }

    pub fn forget(&mut self, key: ScrollKey) {
        if self.saved.remove(&key).is_some() {
            self.order.retain(|queued| *queued != key);
        }
    }

    pub fn get(&self, key: ScrollKey) -> Option<SavedScroll> {
        self.saved.get(&key).copied()
    }

    pub fn clear(&mut self) {
        self.saved.clear();
        self.order.clear();
    }

    pub fn len(&self) -> usize {
        self.saved.len()
    }

    pub fn is_empty(&self) -> bool {
        self.saved.is_empty()
    }

    /// How the history `key` opens: at an explicit target (a search hit,
    /// a reply, a message link), else at its first unread message or the
    /// bottom. Either way the saved position is dropped.
    pub fn open(&mut self, key: ScrollKey, explicit: Option<MessageId>) -> HistoryOpen {
        self.forget(key);
        explicit.map_or(HistoryOpen::Default, HistoryOpen::Message)
    }
}

/// What the first history window of an opened chat is loaded around.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryOpen {
    /// The read boundary when there are unread messages, else the latest.
    Default,
    /// An explicit target: loaded around and highlighted.
    Message(MessageId),
}

/// What the first rows of an opened history scroll to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollOpen {
    /// The saved row is loaded: start there, `offset` pixels into it.
    Restore { row: usize, offset: f32 },
    /// A jump target is set: it wins and the saved position is dropped.
    Jump,
    /// A saved position whose message is no longer in the window (deleted,
    /// or trimmed): the default position, and the saved one is dropped.
    Missing,
    /// Nothing saved: the unread divider, else the bottom.
    Default,
}

impl ScrollOpen {
    /// Whether opening this way drops the saved position.
    pub fn forgets(self) -> bool {
        matches!(self, Self::Jump | Self::Missing)
    }
}

/// Decide where the first rows of a history start. `row_of` finds the row
/// holding a message in the loaded window.
pub fn scroll_open(
    saved: Option<SavedScroll>,
    explicit_jump: bool,
    row_of: impl Fn(MessageId) -> Option<usize>,
) -> ScrollOpen {
    if explicit_jump {
        return ScrollOpen::Jump;
    }
    let Some(saved) = saved else {
        return ScrollOpen::Default;
    };
    match row_of(saved.anchor) {
        Some(row) => ScrollOpen::Restore {
            row,
            offset: saved.offset,
        },
        None => ScrollOpen::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HistoryOpen, SCROLL_MEMORY_CAP, SavedScroll, ScrollMemory, ScrollOpen, scroll_open,
    };
    use crate::ids::MessageId;

    fn at(id: i64, offset: f32) -> Option<SavedScroll> {
        Some(SavedScroll {
            anchor: MessageId(id),
            offset,
        })
    }

    #[test]
    fn leaving_mid_history_is_remembered_and_the_tail_is_forgotten() {
        let mut memory = ScrollMemory::default();
        memory.record((1, None, 0), at(40, 12.));
        assert_eq!(memory.get((1, None, 0)), at(40, 12.));
        assert_eq!(memory.get((2, None, 0)), None);
        memory.record((1, None, 0), None);
        assert_eq!(memory.get((1, None, 0)), None);
        assert!(memory.is_empty());
    }

    #[test]
    fn topics_and_threads_are_separate_views() {
        let mut memory = ScrollMemory::default();
        memory.record((1, None, 0), at(1, 0.));
        memory.record((1, Some(5), 0), at(2, 0.));
        memory.record((1, None, 9), at(3, 0.));
        assert_eq!(memory.len(), 3);
        assert_eq!(memory.get((1, Some(5), 0)), at(2, 0.));
    }

    #[test]
    fn the_oldest_saved_position_goes_past_the_cap() {
        let mut memory = ScrollMemory::default();
        for chat in 0..SCROLL_MEMORY_CAP as i64 + 5 {
            memory.record((chat, None, 0), at(chat, 0.));
        }
        assert_eq!(memory.len(), SCROLL_MEMORY_CAP);
        assert_eq!(memory.get((0, None, 0)), None);
        assert_eq!(memory.get((4, None, 0)), None);
        assert!(memory.get((5, None, 0)).is_some());
    }

    #[test]
    fn saving_again_refreshes_instead_of_duplicating() {
        let mut memory = ScrollMemory::default();
        memory.record((1, None, 0), at(1, 0.));
        memory.record((2, None, 0), at(2, 0.));
        memory.record((1, None, 0), at(3, 4.));
        for chat in 10..10 + SCROLL_MEMORY_CAP as i64 - 1 {
            memory.record((chat, None, 0), at(chat, 0.));
        }
        assert_eq!(memory.get((2, None, 0)), None);
        assert_eq!(memory.get((1, None, 0)), at(3, 4.));
        memory.clear();
        assert!(memory.is_empty());
    }

    #[test]
    fn opening_lands_at_the_default_and_drops_the_saved_position() {
        let mut memory = ScrollMemory::default();
        memory.record((1, None, 0), at(40, 12.));
        assert_eq!(memory.open((1, None, 0), None), HistoryOpen::Default);
        assert_eq!(memory.get((1, None, 0)), None);
    }

    #[test]
    fn an_explicit_target_wins_and_drops_the_saved_position() {
        let mut memory = ScrollMemory::default();
        memory.record((1, None, 0), at(40, 12.));
        assert_eq!(
            memory.open((1, None, 0), Some(MessageId(90))),
            HistoryOpen::Message(MessageId(90))
        );
        assert_eq!(memory.get((1, None, 0)), None);
    }

    #[test]
    fn the_first_rows_restore_when_the_anchor_is_loaded() {
        let open = scroll_open(at(40, 12.), false, |id| (id == MessageId(40)).then_some(7));
        assert_eq!(
            open,
            ScrollOpen::Restore {
                row: 7,
                offset: 12.
            }
        );
        assert!(!open.forgets());
    }

    #[test]
    fn a_jump_target_beats_the_saved_position() {
        let open = scroll_open(at(40, 12.), true, |_| Some(7));
        assert_eq!(open, ScrollOpen::Jump);
        assert!(open.forgets());
    }

    #[test]
    fn a_missing_anchor_falls_back_and_forgets() {
        let open = scroll_open(at(40, 12.), false, |_| None);
        assert_eq!(open, ScrollOpen::Missing);
        assert!(open.forgets());
        let open = scroll_open(None, false, |_| Some(1));
        assert_eq!(open, ScrollOpen::Default);
        assert!(!open.forgets());
    }
}
