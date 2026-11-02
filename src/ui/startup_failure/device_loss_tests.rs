//! A main window replaced after its renderer was lost, on the test platform:
//! the window closes and a new one opens around what it kept. The platform
//! side (a real device loss) does not run here.
use super::{reopen_main_window, watch_closed_windows};
use crate::ui::app::QuillApp;
use crate::ui::link_inbox::attach_main_window;
use crate::ui::screenshot_demo::ScreenshotDemo;
use crate::ui::shell::QuillShell;
use gpui_kit::component::input::{InputEvent, TextareaState};
use gpui_kit::component::{Root, WindowExt as _};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Context, Entity, Focusable as _, IntoElement, Render,
    TestAppContext, VisualTestContext, WeakEntity, Window, WindowOptions, div, px, size,
};
use quill::ids::ChatId;

/// Counts what reaches an entity that subscribed before the reset.
#[derive(Default)]
struct Probe {
    focus_events: usize,
    observed_focus: usize,
    activations: usize,
    bounds_changes: usize,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

fn watch(composer: &Entity<TextareaState>, window: &mut Window, cx: &mut Context<Probe>) -> Probe {
    cx.subscribe_in(composer, window, |probe, _, event: &InputEvent, _, _| {
        if matches!(event, InputEvent::Focus) {
            probe.focus_events += 1;
        }
    })
    .detach();
    cx.on_focus(&composer.focus_handle(cx), window, |probe, _, _| {
        probe.observed_focus += 1
    })
    .detach();
    cx.observe_window_activation(window, |probe, _, _| probe.activations += 1)
        .detach();
    cx.observe_window_bounds(window, |probe, _, _| probe.bounds_changes += 1)
        .detach();
    Probe::default()
}

struct Opened {
    app: WeakEntity<QuillApp>,
    probe: Entity<Probe>,
    window: AnyWindowHandle,
}

fn open(cx: &mut TestAppContext) -> Opened {
    let root = quill::scratch::process_dir("quill-device-loss");
    let _ = std::fs::create_dir_all(&root);
    quill::settings::use_isolated_app_root(root);
    cx.update(gpui_kit::init);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = slot.clone();
    let handle = cx.open_window(size(px(1100.), px(800.)), move |window, cx| {
        let demo = ScreenshotDemo::named("ready-chats");
        let view = cx.new(|cx| QuillApp::new_with_demo(window, cx, None, Some(demo)));
        attach_main_window(window, cx, &view);
        let composer = view.read(cx).composer.clone();
        let probe = cx.new(|cx| watch(&composer, window, cx));
        *slot_in.borrow_mut() = Some((view.downgrade(), probe));
        let shell = cx.new(|_| QuillShell::new(view));
        Root::new(shell, window, cx)
    });
    let Some((app, probe)) = slot.borrow_mut().take() else {
        panic!("the window builder did not run");
    };
    Opened {
        app,
        probe,
        window: handle.into(),
    }
}

/// Close the main window the way the platform does and let recovery run.
fn reset(opened: &Opened, cx: &mut TestAppContext) -> VisualTestContext {
    cx.update(|cx| {
        watch_closed_windows(
            |_| false,
            |cx, kept| reopen_main_window(WindowOptions::default(), kept, cx),
            cx,
        )
    });
    let closed = opened.window;
    let _ = cx.update_window(closed, |_, window, _| window.remove_window());
    cx.run_until_parked();
    let windows = cx.windows();
    let Some(new) = windows.first().copied() else {
        panic!("recovery opened no window");
    };
    assert_eq!(windows.len(), 1);
    assert_ne!(new.window_id(), closed.window_id());
    VisualTestContext::from_window(new, cx)
}

fn app_of(opened: &Opened) -> Entity<QuillApp> {
    let Some(app) = opened.app.upgrade() else {
        panic!("the app did not survive");
    };
    app
}

#[gpui_kit::test]
fn a_reset_keeps_the_open_chat_the_dialog_and_the_session(cx: &mut TestAppContext) {
    let opened = open(cx);
    let mut old = VisualTestContext::from_window(opened.window, cx);
    old.run_until_parked();
    let session_before = old.update(|_, cx| {
        let app = app_of(&opened);
        let app = app.read(cx);
        let session = app.session().map(|s| s as *const _);
        (app.open_chat_id(), session)
    });
    assert!(session_before.0.is_some(), "the demo has a chat open");
    old.update(|window, cx| window.open_dialog(cx, |dialog, _, _| dialog.title("Kept")));
    old.run_until_parked();

    let mut new = reset(&opened, cx);

    let chat: Option<ChatId> = new.update(|_, cx| app_of(&opened).read(cx).open_chat_id());
    assert_eq!(chat, session_before.0);
    let session = new.update(|_, cx| {
        let app = app_of(&opened);
        let app = app.read(cx);
        app.session().map(|s| s as *const _)
    });
    assert_eq!(session, session_before.1, "the same session, never rebuilt");
    assert!(new.update(|window, cx| window.has_active_dialog(cx)));
}

#[gpui_kit::test]
fn what_was_subscribed_before_a_reset_fires_once_in_the_new_window(cx: &mut TestAppContext) {
    let opened = open(cx);
    let mut new = reset(&opened, cx);
    new.run_until_parked();
    let app = app_of(&opened);
    let composer = new.update(|_, cx| app.read(cx).composer.clone());
    let fresh = new.update(|window, cx| cx.new(|cx| watch(&composer, window, cx)));
    new.run_until_parked();
    opened
        .probe
        .update(&mut new, |probe, _| *probe = Probe::default());
    new.update(|window, cx| composer.update(cx, |field, cx| field.focus(window, cx)));
    new.run_until_parked();
    new.update(|window, cx| window.render_frame(cx));
    new.deactivate_window();
    new.update(|window, _| window.activate_window());
    new.simulate_resize(size(px(900.), px(700.)));
    new.run_until_parked();

    let counts = |probe: &Entity<Probe>, new: &VisualTestContext| {
        probe.read_with(new, |probe, _| {
            [
                probe.observed_focus,
                probe.focus_events,
                probe.activations,
                probe.bounds_changes,
            ]
        })
    };
    let moved = counts(&opened.probe, &new);
    assert_eq!(
        moved,
        counts(&fresh, &new),
        "fires as often as a subscription made on the new window"
    );
    assert_eq!(moved.first(), Some(&1), "focus, once");
    assert_eq!(moved.get(1), Some(&1), "the field's own listener, once");
    assert!(moved.iter().all(|count| *count > 0), "{moved:?}");
    assert!(
        new.update(|_, cx| app.read(cx).frame.main_active.get()),
        "the app's own activation observer moved too"
    );
}

#[gpui_kit::test]
fn a_handle_to_the_closed_window_reaches_its_replacement(cx: &mut TestAppContext) {
    let opened = open(cx);
    let mut new = reset(&opened, cx);
    let replacement = new.update(|window, _| window.window_handle());
    let reached = new
        .cx
        .update_window(opened.window, |_, window, _| window.window_handle());
    assert_eq!(
        reached.map(|h| h.window_id()).ok(),
        Some(replacement.window_id())
    );
}

struct Pip;

impl Render for Pip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// A PiP left alone with no way back to the chats is the state to avoid.
#[gpui_kit::test]
fn a_main_window_lost_beside_a_pip_comes_back_and_the_pip_stays(cx: &mut TestAppContext) {
    let opened = open(cx);
    let pip: AnyWindowHandle = cx.open_window(size(px(200.), px(120.)), |_, _| Pip).into();
    cx.update(|cx| {
        watch_closed_windows(
            |_| false,
            |cx, kept| reopen_main_window(WindowOptions::default(), kept, cx),
            cx,
        )
    });
    let _ = cx.update_window(opened.window, |_, window, _| window.remove_window());
    cx.run_until_parked();
    let windows = cx.windows();
    assert_eq!(windows.len(), 2, "the new main window beside the PiP");
    assert!(windows.iter().any(|w| w.window_id() == pip.window_id()));
    assert!(
        !windows
            .iter()
            .any(|w| w.window_id() == opened.window.window_id())
    );
    app_of(&opened);
}
