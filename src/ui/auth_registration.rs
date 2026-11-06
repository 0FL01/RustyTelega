#![deny(clippy::indexing_slicing)]
use super::app::QuillApp;
use crate::ui::status_note::StatusNote;
use gpui_kit::component::button::*;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use quill::lang::{keys, tr};
use quill::state::RequestPurpose;
use quill::telegram::envelope::AuthorizationState;
use zeroize::Zeroize;

impl QuillApp {
    pub(super) fn registration_form(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let AuthorizationState::WaitRegistration { terms } = self.current_auth() else {
            return div().into_any_element();
        };
        let accepted = terms.as_ref() == self.auth_ui.accepted_registration_terms.as_ref();
        let pending = self.session().is_some_and(|s| s.requests.has_auth_submit());
        use super::td::{TdIcon, st::intro as st, td, td_icon};
        use quill::app_theme::PaletteKey as K;
        let mut form = div().relative().w_full().h(px(st::HEIGHT));
        for (input, label, top) in [
            (
                &self.auth_ui.registration_first_input,
                tr(keys::Q_MISC_FIRST_NAME),
                st::STEP_FIELD_TOP,
            ),
            (
                &self.auth_ui.registration_last_input,
                tr(keys::Q_MISC_LAST_NAME),
                st::STEP_FIELD_TOP + st::FIELD_HEIGHT + st::PHONE_TOP,
            ),
        ] {
            let active = input.read(cx).focus_handle(cx).is_focused(window);
            form = form.child(
                div()
                    .absolute()
                    .top(px(top))
                    .w_full()
                    .h(px(st::FIELD_HEIGHT))
                    .child(
                        super::intro::phone::field(input, label, st::FIELD_MARGIN_LEFT)
                            .disabled(pending),
                    )
                    .child(super::intro::phone::underline(active, false)),
            );
        }
        let photo = self
            .auth_ui
            .registration_photo
            .as_ref()
            .and_then(|photo| photo.preview.clone());
        form = form.child(
            Button::new("registration-photo")
                .ghost()
                .absolute()
                .right_0()
                .top(px(st::PHOTO_TOP))
                .size(px(st::PHOTO_SIZE))
                .rounded_full()
                .p_0()
                .border_0()
                .bg(td(K::ActiveButtonBg))
                .accessibility_label(tr(keys::PROFILE_SET_PHOTO_BUTTON))
                .disabled(pending)
                .child(match photo {
                    Some(super::profile_photo_sources::PhotoPreview::Image(image)) => div()
                        .size_full()
                        .rounded_full()
                        .overflow_hidden()
                        .child(img(image).size_full())
                        .into_any_element(),
                    _ => div()
                        .child(td_icon(TdIcon::IntroPhoto, td(K::ActiveButtonFg)))
                        .into_any_element(),
                })
                .on_click(cx.listener(|this, _, window, cx| {
                    this.choose_photo_or_video(
                        super::profile_photo_sources::PhotoTarget::Registration(
                            this.connection.generation,
                        ),
                        window,
                        cx,
                    );
                })),
        );
        form.child(
            div().absolute().top(px(st::ERROR_TOP)).w_full().child(
                Checkbox::new("registration-notify-contacts")
                    .label(tr(keys::Q_MISC_NOTIFY_MY_CONTACTS_THAT_I))
                    .checked(self.auth_ui.registration_notify_contacts)
                    .disabled(pending)
                    .on_click(cx.listener(|this, &on, _, cx| {
                        this.auth_ui.registration_notify_contacts = on;
                        cx.notify();
                    })),
            ),
        )
        .child(
            div().absolute().top(px(st::NEXT_TOP)).child(
                super::intro::next_button(
                    "create-telegram-account",
                    tr(keys::INTRO_FINISH),
                    !accepted || pending,
                    cx,
                )
                .on_click(cx.listener(|this, _, window, cx| this.submit_registration(window, cx))),
            ),
        )
        .when(terms.is_some(), |form| {
            form.child(
                div()
                    .absolute()
                    .top(px(st::NEXT_TOP + st::NEXT_HEIGHT + st::TERMS_BOTTOM))
                    .w_full()
                    .child(
                        Button::new("registration-terms")
                            .ghost()
                            .w_full()
                            .label(tr(keys::Q_MISC_I_ACCEPT_THESE_TERMS))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.auth_ui.registration_terms_open = true;
                                cx.notify();
                            })),
                    ),
            )
        })
        .into_any_element()
    }

    fn submit_registration(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let mut first = self
            .auth_ui
            .registration_first_input
            .read(cx)
            .value()
            .to_string();
        let mut last = self
            .auth_ui
            .registration_last_input
            .read(cx)
            .value()
            .to_string();
        let result = live.driver.register_user(
            &first,
            &last,
            self.auth_ui.accepted_registration_terms.as_ref(),
            self.auth_ui.registration_notify_contacts,
        );
        first.zeroize();
        last.zeroize();
        self.connection.set_note(match result {
            Ok(_) => {
                self.auth_ui.registration_submitted = true;
                self.auth_ui
                    .registration_first_input
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.auth_ui
                    .registration_last_input
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.auth_ui.accepted_registration_terms = None;
                StatusNote::silent(tr(keys::Q_MISC_REGISTRATION_SUBMITTED_WAITING_FOR_TELEGRAM))
            }
            Err(_) => StatusNote::error(tr(keys::Q_MISC_COULD_NOT_REGISTER_CHECK_YOUR)),
        });
        cx.notify();
    }

    pub(super) fn finish_registration_photo(&mut self, cx: &mut Context<Self>) {
        if !self.auth_ui.registration_submitted {
            return;
        }
        if matches!(self.current_auth(), AuthorizationState::Ready) {
            self.auth_ui.registration_submitted = false;
            if let Some(photo) = self.auth_ui.registration_photo.take()
                && let Err(note) = self
                    .take_chosen_photo(super::profile_photo_sources::PhotoTarget::Profile, photo)
            {
                self.connection.set_note(StatusNote::error(note));
                cx.notify();
            }
        } else if self
            .session()
            .is_none_or(|s| !s.requests.has_purpose(RequestPurpose::RegisterUser))
        {
            self.auth_ui.registration_submitted = false;
        }
    }
}
