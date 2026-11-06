#![deny(clippy::indexing_slicing)]
use super::app::QuillApp;
use crate::ui::status_note::StatusNote;
use gpui_kit::component::input::{InputEvent, TextareaState};
use gpui_kit::*;
use quill::composer::should_send_on_enter;
use quill::lang::{keys, tr};
use quill::telegram::envelope::AuthorizationState;
use zeroize::Zeroize;
/// Slice A10: 2FA password recovery UI. "Forgot password?" on the auth
/// password screen sends `requestAuthenticationPasswordRecovery` (the code
/// is emailed); the screen switches to recovery-code entry, which submits
/// `recoverAuthenticationPassword`. The code is zeroized after the send and
/// never stored — the A2 rule (auth secrets ride the request JSON only).
///
/// Recovery removes 2FA (empty new password/hint); the status note points
/// at Settings → Two-Step Verification to re-enable it (slice A2).
/// `checkAuthenticationPasswordRecoveryCode` is deliberately not called:
/// `recoverAuthenticationPassword` validates the code itself, so the extra
/// round-trip adds nothing.
/// Slice A10: wire Enter in the recovery-code field to submit. Called
/// from `QuillApp::new_with_demo` next to the other auth-input
/// subscriptions.
pub(crate) fn subscribe_recovery_input(
    input: &Entity<TextareaState>,
    window: &mut Window,
    cx: &mut Context<QuillApp>,
) {
    cx.subscribe_in(
        input,
        window,
        |this, state, event: &InputEvent, window, cx| {
            if let InputEvent::PressEnter { secondary, shift } = event {
                let marked = state.update(cx, |input, cx| input.marked_text_range(window, cx));
                if should_send_on_enter(
                    quill::composer::enter_event_from_kit(*shift, *secondary, marked),
                    // Recovery-code form: Enter always submits (not a chat
                    // composer — the send-key setting does not apply).
                    quill::composer::SendKeyMode::Enter,
                ) {
                    this.submit_recovery_code(window, cx);
                }
            }
        },
    )
    .detach();
}

impl QuillApp {
    /// Slice A10: start 2FA password recovery. No local cooldown: a
    /// too-early request fails server-side (429) and surfaces via
    /// `last_auth_error`.
    pub(crate) fn request_password_recovery(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        if !matches!(
            live.driver.session.auth,
            AuthorizationState::WaitPassword {
                has_recovery_email: true
            }
        ) {
            return;
        }
        match live.driver.request_password_recovery() {
            Ok(_) => {
                self.auth_ui.recovery_mode = true;
                self.auth_ui
                    .password_input
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.auth_ui.recovery_code_input.update(cx, |input, cx| {
                    input.set_value("", window, cx);
                    input.focus(window, cx);
                });
                self.connection.set_note(StatusNote::silent(tr(
                    keys::Q_MISC_RECOVERY_CODE_SENT_CHECK_YOUR,
                )));
            }
            Err(_) => {
                self.connection.set_note(StatusNote::error(tr(
                    keys::Q_MISC_COULD_NOT_REQUEST_A_RECOVERY,
                )));
            }
        }
        cx.notify();
    }

    /// Slice A10: submit the emailed recovery code.
    pub(crate) fn submit_recovery_code(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        if !matches!(
            live.driver.session.auth,
            AuthorizationState::WaitPassword { .. }
        ) {
            return;
        }
        let mut code = self
            .auth_ui
            .recovery_code_input
            .read(cx)
            .value()
            .to_string();
        let result = live.driver.submit_recovery_code(&code);
        code.zeroize();
        match result {
            Ok(_) => {
                self.auth_ui
                    .recovery_code_input
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.connection.set_note(StatusNote::silent(tr(
                    keys::Q_MISC_RECOVERY_CODE_SUBMITTED_2FA_WILL,
                )));
            }
            Err(_) => {
                self.connection.set_note(StatusNote::error(tr(
                    keys::Q_MISC_COULD_NOT_SUBMIT_RECOVERY_CODE,
                )));
            }
        }
        cx.notify();
    }

    /// Slice A10: leave recovery-code entry, back to password entry.
    pub(crate) fn cancel_password_recovery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.auth_ui.recovery_mode = false;
        self.auth_ui
            .recovery_code_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.auth_ui
            .password_input
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }
}
