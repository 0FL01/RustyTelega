use super::*;
use crate::ui::connect_ui::ConnectUiStatus;
use crate::ui::screenshot_demo::{DemoSpec, register_demos};

fn start() -> (ConnectUiStatus, String, AuthorizationState) {
    (
        ConnectUiStatus::DemoWaitPhone,
        String::new(),
        AuthorizationState::WaitPhoneNumber,
    )
}

fn cover(app: &mut QuillApp, _: &mut Window, _: &mut Context<QuillApp>) {
    app.intro.started = false;
}

fn qr() -> (ConnectUiStatus, String, AuthorizationState) {
    (
        ConnectUiStatus::DemoWaitPhone,
        String::new(),
        AuthorizationState::WaitOtherDeviceConfirmation {
            link: "tg://login?token=abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz".into(),
        },
    )
}

fn qr_setup(app: &mut QuillApp, _: &mut Window, _: &mut Context<QuillApp>) {
    app.privacy.passkeys.demo_login_link = true;
}

fn phone_setup(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>, value: &str) {
    // Keep the reference's hidden-caret phase stable during the capture window.
    cx.defer(|cx| {
        let mut theme = gpui_kit::base::Theme::global(cx);
        theme.cursor_blink =
            gpui_kit::base::input::CursorBlink::blinking(std::time::Duration::from_secs(3));
        cx.set_global(theme);
    });
    app.auth_ui.signin.demo_countries = crate::ui::signin_ui::demo_country_fixtures();
    app.auth_ui.signin.demo_guess = Some("SE".into());
    app.auth_ui.signin.touched = true;
    app.signin_assign_phone(value, window, cx);
    app.auth_ui.phone_input.update(cx, |input, cx| {
        let end = input.value().len();
        input.set_selected_range(end..end, cx);
        input.focus(window, cx);
    });
}

fn phone(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    phone_setup(app, window, cx, "+46");
}

fn phone_filled(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    phone_setup(app, window, cx, "+461234");
}

fn phone_error(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    phone_filled(app, window, cx);
    app.auth_ui.signin.demo_error = Some(quill::state::AuthRequestError {
        purpose: quill::state::RequestPurpose::SetPhoneNumber,
        class: quill::telegram::envelope::ErrorClass::PhoneInvalid,
        flood_wait_secs: None,
    });
}

fn settings(app: &mut QuillApp, _: &mut Window, _: &mut Context<QuillApp>) {
    app.intro.settings_open = true;
    app.open_settings(None);
    app.settings.system_accent = Some(0x5288c1);
    app.settings.system_accent_probed = true;
    app.settings.update_preferences = Default::default();
    app.settings.update_state = quill::updater::UpdateState::UpToDate;
}

fn late_step(
    app: &mut QuillApp,
    _: &mut Window,
    _: &mut Context<QuillApp>,
    state: AuthorizationState,
) {
    app.auth_ui.demo_state = state;
    app.auth_ui.demo_inputs = true;
}

fn code(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    app.auth_ui.signin.submitted_phone = "+44 7700 900123".into();
    late_step(
        app,
        window,
        cx,
        AuthorizationState::WaitCode {
            code_length: Some(5),
            delivery: quill::telegram::envelope::CodeDelivery {
                kind: quill::telegram::envelope::CodeKind::Sms,
                ..Default::default()
            },
        },
    );
}

fn email(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    late_step(app, window, cx, AuthorizationState::WaitEmailAddress);
}

fn email_code(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    late_step(
        app,
        window,
        cx,
        AuthorizationState::WaitEmailCode {
            email_pattern: "a***@example.org".into(),
            code_length: Some(6),
            reset: Default::default(),
        },
    );
}

fn password(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    late_step(
        app,
        window,
        cx,
        AuthorizationState::WaitPassword {
            has_recovery_email: true,
        },
    );
}

fn recovery(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    password(app, window, cx);
    app.auth_ui.recovery_mode = true;
}

fn signup(app: &mut QuillApp, window: &mut Window, cx: &mut Context<QuillApp>) {
    late_step(
        app,
        window,
        cx,
        AuthorizationState::WaitRegistration {
            terms: Some(quill::telegram::envelope::RegistrationTerms {
                text: "Review Telegram's terms before creating an account.".into(),
                min_user_age: 16,
            }),
        },
    );
}

register_demos![
    DemoSpec::signed_out("intro-start", start).setup(cover),
    DemoSpec::signed_out("intro-qr", qr).setup(qr_setup),
    DemoSpec::signed_out("intro-phone", start).setup(phone),
    DemoSpec::signed_out("intro-phone-filled", start).setup(phone_filled),
    DemoSpec::signed_out("intro-phone-error", start).setup(phone_error),
    DemoSpec::signed_out("intro-settings", start).setup(settings),
    DemoSpec::signed_out("intro-code", start).setup(code),
    DemoSpec::signed_out("intro-email", start).setup(email),
    DemoSpec::signed_out("intro-email-code", start).setup(email_code),
    DemoSpec::signed_out("intro-password", start).setup(password),
    DemoSpec::signed_out("intro-recovery", start).setup(recovery),
    DemoSpec::signed_out("intro-signup", start).setup(signup),
];
