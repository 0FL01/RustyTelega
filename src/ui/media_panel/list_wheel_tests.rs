//! A list pushed against its end renders nothing, and one whose rows are
//! measured lazily still reaches its last row
//! (`third_party/gpui-pre/src/elements/list.rs`).

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::{
    Context, IntoElement, ListAlignment, ListState, ParentElement, Pixels, Render, ScrollDelta,
    ScrollEvent, Styled, TestAppContext, VisualTestContext, Window, div, list, point, px,
};

struct CountedList {
    state: ListState,
    renders: Rc<Cell<usize>>,
}

impl Render for CountedList {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div().w(px(100.)).h(px(100.)).child(
            list(self.state.clone(), |_, _, _| {
                div().h(px(20.)).w_full().into_any_element()
            })
            .w_full()
            .h_full(),
        )
    }
}

fn drawn(
    cx: &mut TestAppContext,
    rows: usize,
) -> (ListState, Rc<Cell<usize>>, VisualTestContext) {
    let state = ListState::new(rows, ListAlignment::Top, px(10.));
    let renders = Rc::new(Cell::new(0));
    let (_, vcx) = cx.add_window_view(|_, _| CountedList {
        state: state.clone(),
        renders: renders.clone(),
    });
    vcx.run_until_parked();
    (state, renders, vcx.clone())
}

fn wheel(vcx: &mut VisualTestContext, y: Pixels) {
    vcx.simulate_event(ScrollEvent {
        position: point(px(50.), px(10.)),
        delta: ScrollDelta::Pixels(point(px(0.), y)),
        ..Default::default()
    });
    vcx.run_until_parked();
}

#[gpui_kit::test]
fn a_wheel_against_the_top_renders_nothing(cx: &mut TestAppContext) {
    let (state, renders, mut vcx) = drawn(cx, 10);

    wheel(&mut vcx, px(10.));
    let at_top = renders.get();
    wheel(&mut vcx, px(10.));
    wheel(&mut vcx, px(10.));
    assert_eq!(renders.get(), at_top, "pushing past the top draws nothing");

    wheel(&mut vcx, px(-15.));
    let top = state.logical_scroll_top();
    assert_eq!((top.item_ix, top.offset_in_item), (0, px(15.)));
    assert_eq!(renders.get(), at_top + 1, "a real move draws once");
}

#[gpui_kit::test]
fn a_wheel_past_the_measured_rows_reaches_the_last_row(cx: &mut TestAppContext) {
    let (state, renders, mut vcx) = drawn(cx, 50);

    for _ in 0..200 {
        wheel(&mut vcx, px(-1000.));
    }
    assert_eq!(state.logical_scroll_top().item_ix, 45);
    let at_end = renders.get();
    wheel(&mut vcx, px(-1000.));
    assert_eq!(renders.get(), at_end, "pushing past the end draws nothing");
}
