use super::*;
use quill::auth::AuthAction;

impl QuillApp {
    pub(super) fn intro_auth_form(
        &mut self,
        auth: &AuthView,
        width: f32,
        top: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.current_auth();
        let disk_full = self.disk_full_view();
        let signin_open = disk_full.is_none() && (self.live.is_some() || self.auth_ui.demo_inputs);
        let auth = disk_full.unwrap_or_else(|| self.signin_view(auth, &state));
        let show_phone = signin_open && self.signin_phone_visible(&state);
        let editing = show_phone && self.auth_ui.signin.editing_phone;
        let show_code = signin_open && !editing && matches!(auth.action, AuthAction::EnterCode);
        let busy = self.session().is_some_and(|s| s.requests.has_auth_submit());
        let phone_title = show_code
            && !self.auth_ui.signin.submitted_phone.is_empty()
            && !matches!(&state, AuthorizationState::WaitCode { delivery, .. } if delivery.kind == quill::telegram::envelope::CodeKind::Fragment);
        let title: SharedString = if phone_title {
            self.auth_ui.signin.submitted_phone.clone().into()
        } else {
            auth.title.into()
        };
        let (description, bold) = if matches!(&state, AuthorizationState::WaitCode { delivery, .. } if delivery.kind == quill::telegram::envelope::CodeKind::TelegramMessage)
        {
            crate::ui::screen_chooser::split_bold(&auth.body)
        } else {
            (auth.body.clone(), Vec::new())
        };
        let highlights = bold
            .into_iter()
            .map(|range| {
                (
                    range,
                    HighlightStyle {
                        font_weight: Some(FontWeight::SEMIBOLD),
                        ..Default::default()
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut form = div().w_full().flex().flex_col().gap_3();
        if signin_open {
            if matches!(auth.action, AuthAction::Register) {
                form = form.child(self.registration_form(window, cx));
            }
            if matches!(auth.action, AuthAction::EnterEmail) {
                form = form.child(self.intro_email(busy, window, cx));
            }
            if show_phone {
                form = form.child(self.intro_phone_fields(busy, window, cx));
            }
            let (errors, locked) = if show_phone {
                (Vec::new(), false)
            } else {
                self.signin_error_elements(cx)
            };
            if show_code {
                form = form.child(self.intro_code(busy || locked, window, cx));
            }
            if show_phone {
                form = form.children(errors);
            }
            if matches!(auth.action, AuthAction::EnterPassword) && !editing {
                form = form.child(self.intro_password(busy, window, cx));
            }
            if !show_phone && quill::auth::can_request_qr_login(&state) && !self.intro.qr_requested
            {
                form = form.child(
                    Button::new("qr-login")
                        .ghost()
                        .w_full()
                        .label(tr(keys::Q_MISC_SIGN_IN_WITH_QR_CODE))
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.intro.qr_requested = true;
                            this.request_qr_login(cx);
                        })),
                );
            }
        }
        if matches!(auth.action, AuthAction::StartupFailed | AuthAction::Closed) {
            form = form.child(
                Button::new("retry-connection")
                    .primary()
                    .label(tr(keys::Q_MISC_RETRY))
                    .on_click(cx.listener(|this, _, _, cx| this.start_connection(cx))),
            );
        }
        if let AuthAction::UnsupportedHalt { reason } = &auth.action {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(td(K::IntroDescriptionFg))
                    .child(reason.to_string()),
            );
        }
        if let AuthorizationState::WaitPremiumPurchase {
            support_email_address,
            support_email_subject,
            ..
        } = &state
            && !show_phone
            && let Some(url) =
                quill::signin::premium_support_mailto(support_email_address, support_email_subject)
        {
            form = form.child(
                Button::new("premium-support")
                    .label(tr(keys::Q_MISC_EMAIL_TELEGRAM_SUPPORT))
                    .on_click(cx.listener(move |this, _, _, cx| this.open_link(&url, cx))),
            );
        }
        div()
            .absolute()
            .left(px(st::content_left(width)))
            .top(px(top))
            .w(px(st::FIELD_WIDTH))
            .child(
                div()
                    .id("auth-title")
                    .role(Role::Heading)
                    .aria_label(title.clone())
                    .absolute()
                    .left(px(st::BUTTON_RADIUS))
                    .top(px(st::TITLE_TOP))
                    .text_size(px(st::TITLE_FONT))
                    .font_semibold()
                    .line_height(px(23.))
                    .text_color(td(K::IntroTitleFg))
                    .child(crate::ui::td::label::label(title)),
            )
            .child(
                div()
                    .id("auth-explanation")
                    .role(Role::Label)
                    .aria_label(description.clone())
                    .absolute()
                    .left(px(st::BUTTON_RADIUS))
                    .top(px(st::DESCRIPTION_TOP))
                    .w(px(st::DESCRIPTION_WIDTH))
                    .text_size(px(13.))
                    .line_height(px(st::DESCRIPTION_LINE))
                    .text_color(td(K::IntroDescriptionFg))
                    .child(StyledText::new(description).with_highlights(highlights)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(
                        if show_phone
                            || show_code
                            || matches!(
                                auth.action,
                                AuthAction::EnterEmail
                                    | AuthAction::EnterPassword
                                    | AuthAction::Register
                            )
                        {
                            0.
                        } else {
                            st::STEP_FIELD_TOP
                        },
                    ))
                    .w_full()
                    .child(form),
            )
            .children(self.disk_full_reveal(cx))
            .into_any_element()
    }
}
