//! A channel's paid direct messages as a subscriber who has not written:
//! the composer instead of a Join bar, the price card, the way back to the
//! channel, and the channel bar's direct messages button.
use super::{CHANNEL, DIRECT};
use crate::ui::app::QuillApp;
use crate::ui::keybindings::bind_keys;
use crate::ui::screenshot_demo::ScreenshotDemo;
use crate::ui::shell::QuillShell;
use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, Entity, Modifiers, TestAppContext, VisualTestContext, px, size};
use quill::ids::ChatId;

fn frames(vcx: &mut VisualTestContext) {
    vcx.run_until_parked();
    for _ in 0..3 {
        vcx.update(|window, cx| window.render_frame(cx));
    }
}

fn open_demo(cx: &mut TestAppContext) -> (Entity<QuillApp>, VisualTestContext) {
    cx.update(gpui_kit::init);
    cx.update(bind_keys);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = slot.clone();
    let demo = ScreenshotDemo::named("ready-direct-messages");
    let handle = cx.open_window(size(px(1100.), px(800.)), move |window, cx| {
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

fn click(vcx: &mut VisualTestContext, id: &'static str) {
    let bounds = vcx.update(|window, _| window.find(id).bounds());
    vcx.simulate_click(bounds.center(), Modifiers::default());
    frames(vcx);
}

fn open_chat(app: &Entity<QuillApp>, vcx: &mut VisualTestContext) -> Option<ChatId> {
    app.read_with(vcx, |app, _| app.open_chat_id())
}

#[gpui_kit::test]
fn direct_messages_write_without_joining_and_lead_back(cx: &mut TestAppContext) {
    let (app, mut vcx) = open_demo(cx);
    let vcx = &mut vcx;
    assert_eq!(open_chat(&app, vcx), Some(ChatId(DIRECT)));
    let (bar, composer) = app.read_with(vcx, |app, _| {
        (app.bottom_action(), app.composer_available(app.pane_mode()))
    });
    assert_eq!(bar, None, "no Join bar in a channel's direct messages");
    assert!(composer, "the composer is up");
    assert!(
        vcx.update(|window, _| window.try_find("locked-intro-buy-stars").is_some()),
        "the price card offers Buy Stars"
    );

    click(vcx, "direct-messages-back");
    assert_eq!(open_chat(&app, vcx), Some(ChatId(CHANNEL)));

    let muted = |app: &Entity<QuillApp>, vcx: &mut VisualTestContext| {
        app.read_with(vcx, |app, _| {
            app.session()
                .and_then(|s| s.chats.get(&CHANNEL))
                .is_some_and(|chat| chat.is_muted())
        })
    };
    let before = muted(&app, vcx);
    click(vcx, "channel-direct-messages");
    assert_eq!(open_chat(&app, vcx), Some(ChatId(DIRECT)));
    assert_eq!(muted(&app, vcx), before, "the button leaves Mute alone");
}
