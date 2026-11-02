use crate::ui::app::QuillApp;
use crate::ui::screenshot_demo::ScreenshotDemo;
use crate::ui::shell::QuillShell;
use gpui_kit::component::Root;
use gpui_kit::{AppContext as _, TestAppContext, VisualTestContext, px, size};

const FOLDER: i32 = 5;

#[gpui_kit::test]
fn marking_a_folder_read_drops_its_chats_from_the_title_badge(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = slot.clone();
    let handle = cx.open_window(size(px(1100.), px(700.)), move |window, cx| {
        let view = cx.new(|cx| {
            QuillApp::new_with_demo(window, cx, None, Some(ScreenshotDemo::named("ready-chats")))
        });
        *slot_in.borrow_mut() = Some(view.clone());
        let shell = cx.new(|_| QuillShell::new(view));
        Root::new(shell, window, cx)
    });
    let app = slot.borrow().clone().unwrap();
    let mut vcx = VisualTestContext::from_window(handle.into(), cx);
    let badge = |app: &mut QuillApp| {
        let session = app.demo_session.as_ref().unwrap();
        app.frame
            .title_badge
            .borrow_mut()
            .get(session, &session.settings.badge_prefs)
    };
    let before = app.update_in(&mut vcx, |app, _, _| {
        let session = app.demo_session.as_mut().unwrap();
        let chat = session
            .chats
            .values_mut()
            .find(|chat| chat.unread_count > 0 && chat.notification_settings.mute_for == 0)
            .expect("an unread chat in the demo");
        chat.folder_positions.insert(FOLDER, 1);
        session.revision = session.revision.wrapping_add(1);
        badge(app)
    });
    assert!(before > 0);
    let after = app.update_in(&mut vcx, |app, _, cx| {
        app.mark_folder_read(Some(FOLDER), cx);
        badge(app)
    });
    assert!(after < before, "the badge counted {before}, then {after}");
}
