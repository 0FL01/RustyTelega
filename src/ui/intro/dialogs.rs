use super::*;
use crate::ui::shell::{DialogKind, QuillShell};
use gpui_kit::component::{checkbox::Checkbox, dialog::Dialog};
use quill::lang::tr_with;
use std::{cell::RefCell, rc::Rc};

impl QuillApp {
    fn close_intro_terms(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.auth_ui.registration_terms_open = false;
        cx.notify();
    }

    fn intro_terms_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let AuthorizationState::WaitRegistration { terms: Some(terms) } = self.current_auth()
        else {
            return div().into_any_element();
        };
        let accepted = self.auth_ui.accepted_registration_terms.as_ref() == Some(&terms);
        let label = if terms.min_user_age > 0 {
            tr_with(
                keys::Q_MISC_I_ACCEPT_THESE_TERMS_AND,
                &[("min_user_age", &terms.min_user_age.to_string())],
            )
        } else {
            tr(keys::Q_MISC_I_ACCEPT_THESE_TERMS).into()
        };
        div().flex().flex_col().gap_3()
            .child(div().id("registration-terms-text").max_h(px(240.)).overflow_y_scroll().child(terms.text.clone()))
            .child(Checkbox::new("accept-registration-terms").label(label).checked(accepted)
                .on_click(cx.listener(move |this, &on: &bool, _, cx| {
                    if matches!(this.current_auth(), AuthorizationState::WaitRegistration { terms: Some(ref current) } if *current == terms) {
                        this.auth_ui.accepted_registration_terms = on.then(|| terms.clone());
                    }
                    cx.notify();
                })))
            .child(Button::new("registration-terms-done").label(tr(keys::BOX_OK)).on_click(cx.listener(|this, _, window, cx| this.close_intro_terms(window, cx))))
            .into_any_element()
    }

    fn build_intro_terms(
        app: &Entity<Self>,
        shell: &Entity<QuillShell>,
        dialog: Dialog,
        cx: &mut App,
    ) -> Dialog {
        let close = QuillShell::on_close_kind(
            app,
            shell,
            DialogKind::IntroRegistrationTerms,
            Self::close_intro_terms,
        );
        let body = Rc::new(RefCell::new(Some(
            app.update(cx, |this, cx| this.intro_terms_body(cx)),
        )));
        dialog
            .content(move |content, _, _| content.children(body.borrow_mut().take()))
            .on_close(close)
    }
}

crate::ui::shell::register_dialogs! {
    IntroRegistrationTerms => DialogSpec::new(191,
        |app| app.auth_ui.registration_terms_open && !app.auth_ui.signin.editing_phone && matches!(app.current_auth(), AuthorizationState::WaitRegistration { terms: Some(_) }),
        QuillApp::build_intro_terms).closes(QuillApp::close_intro_terms),
}
