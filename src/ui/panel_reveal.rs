//! The side panel column (info, downloads, shared media) sliding in from
//! the window's right edge when it opens, and back out when it closes.
//!
//! Which panels are open is session state; this only follows it. Once per
//! frame, before the layout, [`PanelReveal::settle`] compares the open
//! panels with the last ones and glides `shown` (the column's share of its
//! width in the layout, `0..=1`) toward 1 or 0. The shell narrows the
//! column to `shown` of its width and keeps the panels at their full width
//! inside it, so they slide in without reflowing.
//!
//! A closed panel is gone from state, so the column draws the panels it
//! last had while it slides out ([`PanelReveal::drawn`]). Panels that drop
//! their data on close (shared media, Earn pages) cannot be drawn again;
//! their column closes at once.
//!
//! Telegram Desktop shows its third column at once and slides only inside
//! it (docs/decisions/side-panel-slide.md); the motion here is the chat
//! list's (`sidebar_reveal`), mirrored.
#![deny(clippy::indexing_slicing)]

use super::motion::Glide;
use super::sidebar_reveal::{SLIDE_DURATION, ease_out_cubic};
use quill::state::InfoPanelTarget;
use std::time::Instant;

/// The panels in the side column.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PanelSet {
    pub(crate) info: Option<InfoPanelTarget>,
    pub(crate) downloads: bool,
    pub(crate) shared_media: bool,
}

impl PanelSet {
    pub(crate) fn is_empty(&self) -> bool {
        self.info.is_none() && !self.downloads && !self.shared_media
    }

    /// Whether these panels can still be drawn after their state closed.
    fn redrawable(&self) -> bool {
        !self.shared_media && !matches!(self.info, Some(InfoPanelTarget::Earn(_)))
    }
}

#[derive(Debug)]
pub(crate) struct PanelReveal {
    shown: Glide,
    /// The panels as of the last frame any were open.
    last: PanelSet,
    /// Tests step through a slide on this clock; without it they snap.
    #[cfg(test)]
    pub(crate) clock: Option<Instant>,
}

impl PanelReveal {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            shown: Glide::settled(0., now, ease_out_cubic),
            last: PanelSet::default(),
            #[cfg(test)]
            clock: None,
        }
    }

    /// The time the column is drawn at.
    pub(crate) fn now(&self) -> Instant {
        #[cfg(test)]
        if let Some(now) = self.clock {
            return now;
        }
        Instant::now()
    }

    /// Whether the column may slide: always, except in a test that set no
    /// clock (the wall clock would make it flaky).
    pub(crate) fn may_slide(&self) -> bool {
        #[cfg(test)]
        return self.clock.is_some();
        #[cfg(not(test))]
        true
    }

    /// Once per frame, before the layout: follow the open panels.
    /// `animate` is false where the column has no room to slide (it fills
    /// the window in place of the chat) or motion is reduced.
    pub(crate) fn settle(&mut self, open: PanelSet, animate: bool, now: Instant) {
        if !open.is_empty() {
            self.last = open;
            self.shown.go(1., SLIDE_DURATION, animate, now);
            return;
        }
        let animate = animate && self.last.redrawable();
        self.shown.go(0., SLIDE_DURATION, animate, now);
        if !animate {
            // A slide already under way when a panel that cannot be
            // redrawn was the last one: stop where it is.
            self.shown = Glide::settled(0., now, ease_out_cubic);
        }
        if self.shown.value(now) <= 0. {
            self.last = PanelSet::default();
        }
    }

    /// The panels the column draws: the open ones, else the ones sliding
    /// out.
    pub(crate) fn drawn(&self, open: PanelSet, now: Instant) -> PanelSet {
        if !open.is_empty() {
            open
        } else if self.shown.value(now) > 0. {
            self.last
        } else {
            PanelSet::default()
        }
    }

    /// Share of the column's width in the layout, `0..=1`.
    pub(crate) fn shown(&self, now: Instant) -> f32 {
        self.shown.value(now).clamp(0., 1.)
    }

    /// The column is sliding out: what it draws is no longer open.
    pub(crate) fn leaving(&self, open: PanelSet, now: Instant) -> bool {
        open.is_empty() && self.shown(now) > 0.
    }

    /// Whether the column is still moving (the view asks for frames).
    pub(crate) fn moving(&self, now: Instant) -> bool {
        self.shown.animating(now)
    }
}

#[cfg(test)]
mod tests {
    use super::{PanelReveal, PanelSet};
    use crate::ui::sidebar_reveal::SLIDE_DURATION;
    use quill::earn::EarnPanel;
    use quill::state::InfoPanelTarget;
    use std::time::{Duration, Instant};

    const HALF: Duration = Duration::from_millis(100);

    fn info(user_id: i64) -> PanelSet {
        PanelSet {
            info: Some(InfoPanelTarget::User(user_id)),
            ..PanelSet::default()
        }
    }

    #[test]
    fn opening_slides_the_column_in() {
        let t0 = Instant::now();
        let mut reveal = PanelReveal::new(t0);
        assert_eq!(reveal.shown(t0), 0.);
        reveal.settle(info(7), true, t0);
        let mid = reveal.shown(t0 + HALF);
        assert!(mid > 0. && mid < 1., "{mid}");
        assert!(reveal.moving(t0 + HALF));
        assert_eq!(reveal.shown(t0 + SLIDE_DURATION), 1.);
        assert!(!reveal.moving(t0 + SLIDE_DURATION));
        assert_eq!(reveal.drawn(info(7), t0 + HALF), info(7));
    }

    #[test]
    fn closing_keeps_drawing_the_last_panels_until_they_are_out() {
        let t0 = Instant::now();
        let mut reveal = PanelReveal::new(t0);
        reveal.settle(info(7), false, t0);
        assert_eq!(reveal.shown(t0), 1.);
        let closed = PanelSet::default();
        reveal.settle(closed, true, t0);
        assert!(reveal.leaving(closed, t0 + HALF));
        assert_eq!(reveal.drawn(closed, t0 + HALF), info(7));
        let end = t0 + SLIDE_DURATION;
        reveal.settle(closed, true, end);
        assert_eq!(reveal.shown(end), 0.);
        assert!(!reveal.leaving(closed, end));
        assert_eq!(reveal.drawn(closed, end), PanelSet::default());
    }

    #[test]
    fn reopening_mid_slide_turns_back_from_where_it_is() {
        let t0 = Instant::now();
        let mut reveal = PanelReveal::new(t0);
        reveal.settle(info(7), false, t0);
        reveal.settle(PanelSet::default(), true, t0);
        let mid = reveal.shown(t0 + HALF);
        reveal.settle(info(8), true, t0 + HALF);
        assert!((reveal.shown(t0 + HALF) - mid).abs() < 1e-4);
        assert!(reveal.shown(t0 + HALF + Duration::from_millis(16)) > mid);
        assert_eq!(reveal.drawn(info(8), t0 + HALF), info(8));
    }

    #[test]
    fn switching_panels_while_open_does_not_slide() {
        let t0 = Instant::now();
        let mut reveal = PanelReveal::new(t0);
        reveal.settle(info(7), false, t0);
        let downloads = PanelSet {
            downloads: true,
            ..PanelSet::default()
        };
        reveal.settle(downloads, true, t0);
        assert_eq!(reveal.shown(t0), 1.);
        assert!(!reveal.moving(t0));
    }

    #[test]
    fn panels_that_drop_their_data_close_at_once() {
        let t0 = Instant::now();
        for open in [
            PanelSet {
                shared_media: true,
                ..PanelSet::default()
            },
            PanelSet {
                info: Some(InfoPanelTarget::Earn(EarnPanel::Setup { bot_user_id: 3 })),
                ..PanelSet::default()
            },
        ] {
            let mut reveal = PanelReveal::new(t0);
            reveal.settle(open, false, t0);
            reveal.settle(PanelSet::default(), true, t0);
            assert_eq!(reveal.shown(t0), 0., "{open:?}");
            assert_eq!(reveal.drawn(PanelSet::default(), t0), PanelSet::default());
        }
    }

    #[test]
    fn without_animation_the_column_jumps() {
        let t0 = Instant::now();
        let mut reveal = PanelReveal::new(t0);
        reveal.settle(info(7), false, t0);
        assert_eq!(reveal.shown(t0), 1.);
        reveal.settle(PanelSet::default(), false, t0);
        assert_eq!(reveal.shown(t0), 0.);
        assert_eq!(reveal.drawn(PanelSet::default(), t0), PanelSet::default());
    }
}
