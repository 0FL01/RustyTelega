//! The player's clock.

use std::time::Instant;

/// A wall clock that can be paused, seeked and sped up. It holds still
/// (`held`) after a seek until the first picture is shown, so decoding the
/// first frame doesn't eat into playback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    base: f64,
    anchor: Option<Instant>,
    pub(super) rate: f64,
    playing: bool,
    pub(super) held: bool,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            base: 0.0,
            anchor: None,
            rate: 1.0,
            playing: false,
            held: true,
        }
    }

    pub fn position(&self, now: Instant) -> f64 {
        match self.anchor {
            Some(anchor) if self.playing && !self.held => {
                self.base + now.saturating_duration_since(anchor).as_secs_f64() * self.rate
            }
            _ => self.base,
        }
    }

    fn rebase(&mut self, now: Instant) {
        self.base = self.position(now);
        self.anchor = Some(now);
    }

    pub fn play(&mut self, now: Instant) {
        self.rebase(now);
        self.playing = true;
    }

    pub fn pause(&mut self, now: Instant) {
        self.rebase(now);
        self.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Jump to `secs` and hold there until [`Clock::release`].
    pub fn seek(&mut self, secs: f64, now: Instant) {
        self.base = secs.max(0.0);
        self.anchor = Some(now);
        self.held = true;
    }

    /// Follow an external clock (the sound): jump without holding.
    pub fn follow(&mut self, secs: f64, now: Instant) {
        self.base = secs.max(0.0);
        self.anchor = Some(now);
        self.held = false;
    }

    /// The first picture after a seek is up: time starts running.
    pub fn release(&mut self, now: Instant) {
        if self.held {
            self.anchor = Some(now);
            self.held = false;
        }
    }

    pub fn set_rate(&mut self, rate: f64, now: Instant) {
        self.rebase(now);
        self.rate = if rate.is_finite() && rate > 0.0 {
            rate
        } else {
            1.0
        };
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
