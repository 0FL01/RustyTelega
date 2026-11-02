//! Tooltip timing, shared by every tooltip in the app.
//!
//! Telegram Desktop shows every tooltip after the pointer has rested on its
//! trigger for one second (`Ui::Tooltip::Show(1000, ...)` in
//! `lib_ui/ui/widgets/tooltip.cpp` `InstallTooltip`, and the same literal in
//! the history, media, emoji and bot-keyboard widgets) and hides it the
//! moment the pointer leaves (`Tooltip::Hide`). It has no "recently shown"
//! shortcut: a neighbouring trigger waits the full delay again.
//!
//! The kit's tooltips read [`TooltipDefaults`]; Quill's own pointer-anchored
//! link tooltip reads the same global through [`show_delay`], so both agree.

use gpui_base::TooltipDefaults;
use gpui_kit::App;
use std::time::Duration;

/// How long the pointer rests on a trigger before its tooltip shows.
pub(crate) const SHOW_DELAY: Duration = Duration::from_millis(1000);

/// How long a left tooltip lets the next one skip the delay: none, as in
/// Telegram Desktop.
pub(crate) const GRACE_PERIOD: Duration = Duration::ZERO;

/// The timing Quill installs at startup.
pub(crate) fn defaults() -> TooltipDefaults {
    TooltipDefaults::new()
        .with_show_delay(SHOW_DELAY)
        .with_grace_period(GRACE_PERIOD)
}

/// Installs the app-wide tooltip timing. `QuillApp::new_with_demo` calls it,
/// so the app and the screenshot demos both get it before any tooltip shows.
pub(crate) fn install(cx: &mut App) {
    defaults().install(cx);
}

/// The installed show delay (for tooltips the kit does not draw).
pub(crate) fn show_delay(cx: &App) -> Duration {
    TooltipDefaults::global(cx).show_delay()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_telegram_desktop() {
        let defaults = defaults();
        assert_eq!(defaults.show_delay(), Duration::from_millis(1000));
        assert_eq!(defaults.grace_period(), Duration::ZERO);
    }

    #[cfg(feature = "demo-capture")]
    #[gpui_kit::test]
    fn show_delay_reads_the_installed_global(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            assert_eq!(show_delay(cx), Duration::from_millis(500));
            install(cx);
            assert_eq!(show_delay(cx), SHOW_DELAY);
        });
    }

    #[cfg(feature = "demo-capture")]
    mod wheel {
        use gpui_kit::{
            AnyView, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
            Modifiers, ParentElement as _, Render, ScrollDelta, ScrollEvent, ScrollSource,
            StatefulInteractiveElement as _, StyleRefinement, Styled as _, TestAppContext,
            TouchPhase, VisualTestContext, Window, div, point, px, size,
        };

        struct Tip;

        impl Render for Tip {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().debug_selector(|| "tip".to_string()).child("hint")
            }
        }

        struct Owner;

        impl Render for Owner {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .id("owner")
                    .size_full()
                    .tooltip(|_, cx| cx.new(|_| Tip).into())
            }
        }

        struct Slices {
            owner: Entity<Owner>,
        }

        impl Render for Slices {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let slice = AnyView::from(self.owner.clone())
                    .cached(StyleRefinement::default().size_full());
                div().size_full().child(slice)
            }
        }

        /// Nothing notifies the slice: the frame after the wheel leaves the
        /// view cache out on its own.
        #[gpui_kit::test]
        fn a_wheel_drops_a_tooltip_shown_from_a_cached_view(cx: &mut TestAppContext) {
            let handle = cx.open_window(size(px(300.), px(300.)), |_, cx| {
                let owner = cx.new(|_| Owner);
                Slices { owner }
            });
            let mut vcx = VisualTestContext::from_window(handle.into(), cx);
            vcx.run_until_parked();
            let over = point(px(50.), px(50.));
            vcx.simulate_mouse_move(over, None, Modifiers::none());
            vcx.executor().advance_clock(super::SHOW_DELAY * 2);
            vcx.run_until_parked();
            assert!(vcx.debug_bounds("tip").is_some(), "the tooltip shows");
            vcx.simulate_event(ScrollEvent {
                position: over,
                delta: ScrollDelta::Pixels(point(px(0.), px(-20.))),
                modifiers: Modifiers::default(),
                touch_phase: TouchPhase::Moved,
                source: ScrollSource::Pointer,
            });
            vcx.run_until_parked();
            assert!(vcx.debug_bounds("tip").is_none(), "the wheel dropped it");
        }
    }
}
