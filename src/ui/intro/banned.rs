use crate::ui::app::{PaneMode, QuillApp};
use crate::ui::shell::{DialogKind, QuillShell};
use gpui_kit::component::dialog::Dialog;
use gpui_kit::*;
use quill::telegram::envelope::ErrorClass;
use std::cell::RefCell;
use std::rc::Rc;

impl QuillApp {
    fn signin_banned_open(&self) -> bool {
        self.pane_mode() == PaneMode::Connecting
            && !self.auth_ui.signin.banned_dismissed
            && self
                .session()
                .and_then(|s| s.auth_state.last_auth_error)
                .or(self.auth_ui.signin.demo_error)
                .is_some_and(|error| error.class == ErrorClass::PhoneBanned)
    }

    fn close_phone_banned(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.auth_ui.signin.banned_dismissed = true;
        cx.notify();
    }

    fn build_phone_banned(
        app: &Entity<Self>,
        shell: &Entity<QuillShell>,
        dialog: Dialog,
        cx: &mut App,
    ) -> Dialog {
        let close = QuillShell::on_close_kind(
            app,
            shell,
            DialogKind::PhoneBanned,
            Self::close_phone_banned,
        );
        let body = Rc::new(RefCell::new(Some(
            app.update(cx, |this, cx| this.banned_box(cx)),
        )));
        dialog
            .close_button(false)
            .content(move |content, _, _| content.children(body.borrow_mut().take()))
            .on_close(close)
    }
}

crate::ui::shell::register_dialogs! {
    PhoneBanned => DialogSpec::new(
        190,
        QuillApp::signin_banned_open,
        QuillApp::build_phone_banned,
    ).closes(QuillApp::close_phone_banned),
}
