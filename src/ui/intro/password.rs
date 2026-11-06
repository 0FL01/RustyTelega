use super::*;
use gpui_kit::component::input::{Input, InputContentType};

impl QuillApp {
    pub(in crate::ui) fn intro_password(
        &mut self,
        busy: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let recovery = self.auth_ui.recovery_mode;
        let (errors, locked) = self.signin_error_elements(cx);
        let disabled = busy || locked;
        let field_top = if recovery {
            st::STEP_FIELD_TOP
        } else {
            st::PASSWORD_TOP
        };
        let mut form = div().relative().w_full().h(px(st::HEIGHT));
        if recovery {
            let active = self
                .auth_ui
                .recovery_code_input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window);
            form = form.child(
                div()
                    .absolute()
                    .top(px(field_top))
                    .w_full()
                    .h(px(st::FIELD_HEIGHT))
                    .child(
                        super::phone::field(
                            &self.auth_ui.recovery_code_input,
                            tr(keys::Q_MISC_PASSWORD_RECOVERY_CODE),
                            st::FIELD_MARGIN_LEFT,
                        )
                        .disabled(disabled),
                    )
                    .child(super::phone::underline(active, !errors.is_empty())),
            );
        } else {
            let active = self
                .auth_ui
                .password_input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window);
            form = form.child(
                div()
                    .absolute()
                    .top(px(field_top))
                    .w_full()
                    .h(px(st::FIELD_HEIGHT))
                    .child(
                        div()
                            .absolute()
                            .top(px(st::FIELD_MARGIN_TOP))
                            .left(px(st::FIELD_MARGIN_LEFT))
                            .right(px(st::FIELD_MARGIN_LEFT))
                            .h(px(st::FIELD_LINE))
                            .child(
                                Input::new(&self.auth_ui.password_input)
                                    .aria_label(tr(keys::Q_MISC_TWO_STEP_VERIFICATION_PASSWORD))
                                    .content_type(InputContentType::Password)
                                    .appearance(false)
                                    .bordered(false)
                                    .p_0()
                                    .min_h(px(st::FIELD_LINE))
                                    .max_h(px(st::FIELD_LINE))
                                    .text_size(px(st::FIELD_FONT))
                                    .line_height(px(st::FIELD_LINE))
                                    .text_color(td(K::WindowFg))
                                    .disabled(disabled),
                            ),
                    )
                    .child(super::phone::underline(active, !errors.is_empty())),
            );
        }
        form =
            form.child(
                div()
                    .absolute()
                    .left(px(st::BUTTON_RADIUS))
                    .top(px(st::STEP_FIELD_TOP + st::FIELD_HEIGHT + st::LINK_TOP))
                    .w_full()
                    .child(if recovery {
                        intro_link("cancel-recovery", tr(keys::INTRO_TRY_PASSWORD), cx)
                            .disabled(disabled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cancel_password_recovery(window, cx)
                            }))
                    } else {
                        intro_link("forgot-password", tr(keys::Q_MISC_FORGOT_PASSWORD), cx)
                            .disabled(disabled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.intro_forgot_password(window, cx)
                            }))
                    }),
            );
        if recovery {
            form = form.child(
                div()
                    .absolute()
                    .top(px(st::ERROR_BELOW_LINK_TOP))
                    .w_full()
                    .child(
                        intro_link("resend-recovery-code", tr(keys::Q_MISC_RESEND_CODE), cx)
                            .disabled(disabled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.request_password_recovery(window, cx)
                            })),
                    ),
            );
        }
        form.child(
            div()
                .absolute()
                .top(px(st::ERROR_TOP))
                .w_full()
                .children(errors),
        )
        .child(
            div().absolute().top(px(st::NEXT_TOP)).child(
                next_button(
                    if recovery {
                        "submit-recovery-code"
                    } else {
                        "submit-password"
                    },
                    tr(keys::INTRO_SUBMIT),
                    disabled,
                    cx,
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    if recovery {
                        this.submit_recovery_code(window, cx)
                    } else {
                        this.submit_password(window, cx)
                    }
                })),
            ),
        )
        .into_any_element()
    }

    fn intro_forgot_password(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(
            self.current_auth(),
            AuthorizationState::WaitPassword {
                has_recovery_email: true
            }
        ) {
            self.request_password_recovery(window, cx);
        } else {
            self.connection
                .set_note(crate::ui::status_note::StatusNote::error(tr(
                    keys::INTRO_NO_RECOVERY_EMAIL,
                )));
            cx.notify();
        }
    }
}
