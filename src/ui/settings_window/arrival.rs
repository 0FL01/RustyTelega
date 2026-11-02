//! Landing on a setting from search: the page opens with the row centered
//! and a fading accent wash over it (tdesktop `HighlightWidget`). Which row
//! and how lit it is both follow from one `Arrival`, so a row that shows up
//! or moves late (a page still loading) is centered again and gets the rest
//! of the same highlight.
#![deny(clippy::indexing_slicing)]

use super::QuillApp;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use quill::settings_catalog::Page;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// tdesktop `HighlightArgs`.
const WAIT: Duration = Duration::from_millis(400);
const RISE: Duration = Duration::from_millis(600);
const HOLD: Duration = Duration::from_millis(400);
const FALL: Duration = Duration::from_millis(600);
const PEAK: f32 = 0.4;

const TOTAL: Duration = WAIT
    .saturating_add(RISE)
    .saturating_add(HOLD)
    .saturating_add(FALL);

/// The setting a search result opened.
pub(in crate::ui) struct Arrival {
    target: &'static str,
    /// Where the row sits, once it has been laid out.
    row: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// When the row was first centered: the wash runs from here.
    landed: Option<Instant>,
    /// The row's top in the page when last centered. It moves when what
    /// stood in for the row gives way to it, or content loads above it.
    centered_at: Option<Pixels>,
}

impl Arrival {
    pub(super) fn new(target: &'static str) -> Arrival {
        Arrival {
            target,
            row: Rc::default(),
            landed: None,
            centered_at: None,
        }
    }
}

/// How lit the row is `elapsed` after arriving. Reduced motion skips the
/// fades and keeps the row lit for as long as tdesktop's fades take.
fn wash_opacity(elapsed: Duration, reduce_motion: bool) -> f32 {
    if elapsed < WAIT || elapsed >= TOTAL {
        return 0.;
    }
    if reduce_motion {
        return PEAK;
    }
    let lit = elapsed.saturating_sub(WAIT);
    if lit < RISE {
        let rest = 1. - lit.as_secs_f32() / RISE.as_secs_f32();
        return PEAK * (1. - rest * rest * rest);
    }
    let fading = lit.saturating_sub(RISE + HOLD);
    let gone = fading.as_secs_f32() / FALL.as_secs_f32();
    PEAK * (1. - gone * gone * gone)
}

/// The scroll top that centers a row of `height` at `top` in the content,
/// as far as the page can scroll.
fn centered_top(top: Pixels, height: Pixels, viewport: Pixels, max: Pixels) -> Pixels {
    (top - (viewport - height) / 2.).clamp(px(0.), max.max(px(0.)))
}

impl Arrival {
    #[cfg(test)]
    #[cfg(feature = "demo-capture")]
    pub(super) fn landed_row(&self) -> Option<Bounds<Pixels>> {
        self.landed.and(self.row.get())
    }
}

impl QuillApp {
    /// What scrolls `page`, or the screen open over it. Two pages keep their
    /// own handle for the scrolls and paging they do themselves.
    pub(super) fn settings_scroll(&self, page: Page) -> &ScrollHandle {
        if self.settings.sub.is_some() {
            return &self.settings.sub_scroll;
        }
        match page {
            Page::AgentAccess => &self.agent_ui.page.scroll,
            Page::DataStorage => &self.settings.storage_scroll.page,
            _ => &self.settings.page_scroll,
        }
    }

    /// Before the page renders: scroll to the row once it has a place, and
    /// keep frames coming for the wash.
    pub(super) fn settle_arrival(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(arrival) = &self.settings.arrival else {
            return;
        };
        if arrival
            .landed
            .is_some_and(|landed| landed.elapsed() >= TOTAL)
        {
            self.settings.arrival = None;
            return;
        }
        let row = arrival.row.get();
        if let Some(row) = row
            && let Some(page) = self.settings.page
        {
            let scroll = self.settings_scroll(page);
            let viewport = scroll.bounds();
            let offset = scroll.offset();
            let top = row.top() - viewport.top() - offset.y;
            if arrival.centered_at != Some(top) {
                let max = scroll.max_offset().y;
                let y = centered_top(top, row.size.height, viewport.size.height, max);
                scroll.set_offset(point(offset.x, -y));
                if let Some(arrival) = &mut self.settings.arrival {
                    arrival.centered_at = Some(top);
                    arrival.landed.get_or_insert_with(Instant::now);
                }
                // The offset changes on the next frame and paints nothing
                // on its own.
                window.on_next_frame(|window, _| window.refresh());
            }
        }
        if row.is_some() {
            self.request_animation_tick(60, cx);
        }
    }

    /// A setting's place on its page: search lands here. `ids` are the
    /// catalog entries this element shows.
    pub(in crate::ui) fn setting_spot(
        &self,
        ids: &'static [&'static str],
        element: impl IntoElement,
        cx: &App,
    ) -> AnyElement {
        let arrival = self
            .settings
            .arrival
            .as_ref()
            .filter(|arrival| ids.contains(&arrival.target));
        let wash = arrival.map(|arrival| {
            let opacity = arrival.landed.map_or(0., |landed| {
                wash_opacity(landed.elapsed(), cx.reduce_motion())
            });
            (opacity, arrival.row.clone(), arrival.landed.is_none())
        });
        div()
            .relative()
            .w_full()
            .child(element)
            .when_some(wash, |this, (opacity, row, seeking)| {
                this.child(
                    canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| {
                            row.set(Some(bounds));
                            if seeking {
                                window.on_next_frame(|window, _| window.refresh());
                            }
                        },
                    )
                    .absolute()
                    .inset_0()
                    .size_full()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().primary.opacity(opacity)),
                )
            })
            .into_any_element()
    }

    /// `setting_spot` for entries whose own row is not on the page right
    /// now: `element` stands in for them while `absent` holds.
    pub(in crate::ui) fn setting_stand_in(
        &self,
        absent: bool,
        ids: &'static [&'static str],
        element: AnyElement,
        cx: &App,
    ) -> AnyElement {
        if absent {
            self.setting_spot(ids, element, cx)
        } else {
            element
        }
    }
}

#[cfg(test)]
#[cfg(feature = "demo-capture")]
#[path = "arrival_tests.rs"]
mod ui_tests;

#[cfg(test)]
mod tests {
    use super::{FALL, HOLD, PEAK, RISE, TOTAL, WAIT, centered_top, wash_opacity};
    use gpui_kit::px;
    use std::time::Duration;

    #[test]
    fn the_wash_waits_rises_holds_and_falls() {
        let at = |ms| wash_opacity(Duration::from_millis(ms), false);
        assert_eq!(at(0), 0.);
        assert_eq!(at(399), 0.);
        assert!(at(500) > 0. && at(500) < PEAK);
        assert_eq!(wash_opacity(WAIT + RISE, false), PEAK);
        assert_eq!(wash_opacity(WAIT + RISE + HOLD, false), PEAK);
        assert!(wash_opacity(TOTAL.saturating_sub(FALL / 2), false) < PEAK);
        assert_eq!(wash_opacity(TOTAL, false), 0.);
    }

    #[test]
    fn reduced_motion_shows_the_wash_without_fades() {
        assert_eq!(wash_opacity(WAIT, true), PEAK);
        assert_eq!(
            wash_opacity(TOTAL.saturating_sub(Duration::from_millis(1)), true),
            PEAK
        );
        assert_eq!(wash_opacity(TOTAL, true), 0.);
    }

    #[test]
    fn a_row_centers_unless_the_page_ends_first() {
        assert_eq!(
            centered_top(px(500.), px(40.), px(600.), px(1000.)),
            px(220.)
        );
        assert_eq!(centered_top(px(20.), px(40.), px(600.), px(1000.)), px(0.));
        assert_eq!(
            centered_top(px(1400.), px(40.), px(600.), px(1000.)),
            px(1000.)
        );
        assert_eq!(centered_top(px(300.), px(40.), px(800.), px(-1.)), px(0.));
    }
}
