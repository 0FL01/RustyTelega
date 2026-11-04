#![cfg(test)]
#![cfg(feature = "demo-capture")]
//! The side panel column sliding through GPUI on a stepped clock: an
//! opened profile slides in beside the chat, and a closed one keeps being
//! drawn, without taking clicks, until it is out.

use crate::ui::app::QuillApp;
use crate::ui::screenshot_demo::ScreenshotDemo;
use crate::ui::shell::QuillShell;
use crate::ui::shell_layout::INFO_PANEL_WIDTH;
use crate::ui::sidebar_reveal::SLIDE_DURATION;
use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, Entity, TestAppContext, VisualTestContext, px, size};
use quill::state::InfoPanelTarget;
use std::time::{Duration, Instant};

const HALF: Duration = Duration::from_millis(100);

fn frames(vcx: &mut VisualTestContext) {
    vcx.run_until_parked();
    for _ in 0..3 {
        vcx.update(|window, cx| window.render_frame(cx));
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<QuillApp>, VisualTestContext) {
    cx.update(gpui_kit::init);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = slot.clone();
    let demo = ScreenshotDemo::named("ready-chats");
    let handle = cx.open_window(size(px(1400.), px(800.)), move |window, cx| {
        let view = cx.new(|cx| QuillApp::new_with_demo(window, cx, None, Some(demo)));
        *slot_in.borrow_mut() = Some(view.clone());
        let shell = cx.new(|_| QuillShell::new(view));
        Root::new(shell, window, cx)
    });
    let app: Entity<QuillApp> = slot.borrow().clone().expect("app");
    let mut vcx = VisualTestContext::from_window(handle.into(), cx);
    frames(&mut vcx);
    (app, vcx)
}

fn at(app: &Entity<QuillApp>, vcx: &mut VisualTestContext, now: Instant) {
    app.update(vcx, |app, cx| {
        app.frame.panels.clock = Some(now);
        cx.notify();
    });
    frames(vcx);
}

fn panel_column(app: &Entity<QuillApp>, vcx: &mut VisualTestContext) -> f32 {
    app.read_with(vcx, |app, _| app.shell_layout().panel_column())
}

fn shown(vcx: &mut VisualTestContext, id: &'static str) -> bool {
    vcx.update(|window, _| window.try_find(id).is_some())
}

#[gpui_kit::test]
fn a_profile_slides_in_and_out_beside_the_chat(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    let vcx = &mut vcx;
    let t0 = Instant::now();
    at(&app, vcx, t0);
    app.update_in(vcx, |app, window, cx| {
        app.open_info_panel_target(InfoPanelTarget::User(11), window, cx)
    });
    frames(vcx);
    at(&app, vcx, t0 + HALF);
    let mid = panel_column(&app, vcx);
    assert!(mid > 0. && mid < INFO_PANEL_WIDTH, "{mid}");
    assert!(shown(vcx, "side-panels-sliding"));
    assert!(shown(vcx, "info-panel-close"), "drawn at full width inside");

    let t1 = t0 + SLIDE_DURATION;
    at(&app, vcx, t1);
    assert_eq!(panel_column(&app, vcx), INFO_PANEL_WIDTH);
    assert!(!shown(vcx, "side-panels-sliding"));

    app.update(vcx, |app, cx| app.close_info_panel(cx));
    frames(vcx);
    at(&app, vcx, t1 + HALF);
    let mid = panel_column(&app, vcx);
    assert!(mid > 0. && mid < INFO_PANEL_WIDTH, "{mid}");
    assert!(
        shown(vcx, "info-panel-close"),
        "the closed profile slides out"
    );

    at(&app, vcx, t1 + SLIDE_DURATION);
    assert_eq!(panel_column(&app, vcx), 0.);
    assert!(!shown(vcx, "info-panel-close"));
    assert!(!shown(vcx, "side-panels-sliding"));
}

#[gpui_kit::test]
fn without_a_clock_the_column_snaps(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    let vcx = &mut vcx;
    app.update_in(vcx, |app, window, cx| {
        app.open_info_panel_target(InfoPanelTarget::User(11), window, cx)
    });
    frames(vcx);
    assert_eq!(panel_column(&app, vcx), INFO_PANEL_WIDTH);
    app.update(vcx, |app, cx| app.close_info_panel(cx));
    frames(vcx);
    assert_eq!(panel_column(&app, vcx), 0.);
}
