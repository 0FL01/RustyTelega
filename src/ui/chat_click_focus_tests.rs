#![cfg(test)]
#![cfg(feature = "demo-capture")]
//! A click on a chat row opens the chat with the keys in its composer, as in
//! Telegram Desktop (`MainWidget::showHistory` → `setInnerFocus`).

use crate::ui::app::QuillApp;
use crate::ui::keybindings::bind_keys;
use crate::ui::screenshot_demo::ScreenshotDemo;
use crate::ui::shell::QuillShell;
use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Entity, Focusable, Modifiers, TestAppContext, VisualTestContext, px, size,
};
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
    let demo = ScreenshotDemo::named("ready-chats");
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

/// A chat that is not the open one, and the first that can be written to.
fn other_chat(app: &Entity<QuillApp>, vcx: &mut VisualTestContext) -> ChatId {
    app.read_with(vcx, |app, _| {
        let open = app.open_chat_id();
        app.session()
            .into_iter()
            .flat_map(|session| session.chats.values())
            .filter(|chat| chat.can_post() && !chat.is_forum_chat() && Some(chat.id) != open)
            .map(|chat| chat.id)
            .min_by_key(|id| id.0)
            .expect("a chat to click")
    })
}

fn click_row(vcx: &mut VisualTestContext, chat: ChatId, modifiers: Modifiers) {
    let row = vcx.update(|window, _| window.find(("chat-row", chat.0 as u64)).bounds());
    vcx.simulate_click(row.center(), modifiers);
    frames(vcx);
}

fn composer_focused(app: &Entity<QuillApp>, vcx: &mut VisualTestContext) -> bool {
    app.update_in(vcx, |app, window, cx| {
        app.composer.read(cx).focus_handle(cx).is_focused(window)
    })
}

#[gpui_kit::test]
fn a_row_click_opens_the_chat_with_the_composer_focused(cx: &mut TestAppContext) {
    let (app, mut vcx) = open_demo(cx);
    let vcx = &mut vcx;
    let chat = other_chat(&app, vcx);
    assert!(!composer_focused(&app, vcx));
    click_row(vcx, chat, Modifiers::none());
    assert_eq!(app.read_with(vcx, |app, _| app.open_chat_id()), Some(chat));
    assert!(composer_focused(&app, vcx));
}
