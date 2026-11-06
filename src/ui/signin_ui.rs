//! Sign-in polish (tdesktop `intro_phone.cpp` / `intro_code.cpp` /
//! `country_select_box.cpp`): country picker with search, as-you-type
//! international formatting, resend countdown, "Wrong number?", the
//! banned-number box and flood countdowns. The pure logic lives in
//! `quill::phone` and `quill::signin`.
#![deny(clippy::indexing_slicing)]

use super::app::QuillApp;
use crate::ui::status_note::StatusNote;
use gpui_kit::base::input::{InputBaseState, InputModeKind};
use gpui_kit::component::button::*;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::*;
use gpui_kit::*;
use quill::auth::AuthView;
use quill::lang::{keys, tr, tr_with};
use quill::phone::{self, Country};
use quill::signin;
use quill::state::AuthRequestError;
use quill::telegram::envelope::{AuthorizationState, CodeDelivery, EmailResetState};
use std::time::{Duration, Instant};

/// Per-screen UI state for the sign-in steps. Nothing here is a secret:
/// the phone is shown on screen anyway and the code is never kept.
pub(super) struct SignInUi {
    pub(super) search: Entity<InputState>,
    pub(super) country_code: Entity<InputState>,
    picker_focus: FocusHandle,
    pub(super) picker_open: bool,
    /// The text last written to (or accepted from) the phone field.
    pub(super) phone_text: String,
    /// Inline hint under the phone field after a rejected attempt.
    pub(super) hint: Option<&'static str>,
    /// The number the code was requested for, shown on the code screen.
    pub(super) submitted_phone: String,
    /// "Wrong number?" re-opens the phone form over the code step.
    pub(super) editing_phone: bool,
    pub(super) phone_submitted: bool,
    /// The user typed or picked a country: the IP-based default stays out.
    pub(super) touched: bool,
    pub(super) code_clock: Option<(CodeDelivery, Instant)>,
    pub(super) email_clock: Option<(EmailResetState, Instant)>,
    pub(super) error_clock: Option<(AuthRequestError, Instant)>,
    pub(super) banned_dismissed: bool,
    tick_armed: bool,
    /// Demo-only fixtures (no live TDLib to ask).
    pub(super) demo_countries: Vec<Country>,
    pub(super) demo_guess: Option<String>,
    pub(super) demo_error: Option<AuthRequestError>,
}

fn set_phone_part<M: InputModeKind>(
    state: &Entity<InputBaseState<M>>,
    text: String,
    removed_digits: usize,
    window: &mut Window,
    cx: &mut Context<QuillApp>,
) {
    state.update(cx, |input, cx| {
        let before = input.value().to_string();
        if before == text {
            return;
        }
        let selection = input.selected_range();
        let reversed = input.is_selection_reversed();
        let map = |offset| {
            let digits = phone::digits_only(before.get(..offset).unwrap_or_default())
                .len()
                .saturating_sub(removed_digits);
            if digits == 0 {
                return 0;
            }
            text.char_indices()
                .filter(|(_, ch)| ch.is_ascii_digit())
                .nth(digits - 1)
                .map_or(text.len(), |(at, _)| at + 1)
        };
        let mapped = map(selection.start)..map(selection.end);
        input.set_value(text, window, cx);
        input.set_selected_range_with_direction(mapped, reversed, cx);
    });
}

impl SignInUi {
    pub(super) fn auth_updated(&mut self) {
        if self.phone_submitted {
            self.phone_submitted = false;
            self.editing_phone = false;
        }
    }

    pub(super) fn new(window: &mut Window, cx: &mut Context<QuillApp>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(tr(keys::Q_OPS_SEARCH)));
        cx.subscribe_in(
            &search,
            window,
            |this, search, event: &InputEvent, window, cx| match event {
                InputEvent::Change => cx.notify(),
                InputEvent::PressEnter {
                    secondary: false,
                    shift: false,
                } if search
                    .update(cx, |input, cx| input.marked_text_range(window, cx))
                    .is_none() =>
                {
                    let countries = this.signin_countries();
                    let query = search.read(cx).value().to_string();
                    if let Some(country) = phone::search_countries(&countries, &query).first() {
                        this.signin_pick_country(&country.iso, window, cx);
                    }
                }
                _ => {}
            },
        )
        .detach();
        let country_code = cx.new(|cx| InputState::new(window, cx).placeholder("+"));
        cx.subscribe_in(
            &country_code,
            window,
            |this, state, event: &InputEvent, window, cx| match event {
                InputEvent::Change => this.signin_code_changed(window, cx),
                InputEvent::PressEnter {
                    secondary: false,
                    shift: false,
                } if state
                    .update(cx, |input, cx| input.marked_text_range(window, cx))
                    .is_none() =>
                {
                    this.auth_ui
                        .phone_input
                        .update(cx, |input, cx| input.focus(window, cx));
                }
                _ => {}
            },
        )
        .detach();
        Self {
            search,
            country_code,
            picker_focus: cx.focus_handle(),
            picker_open: false,
            phone_text: String::new(),
            hint: None,
            submitted_phone: String::new(),
            editing_phone: false,
            phone_submitted: false,
            touched: false,
            code_clock: None,
            email_clock: None,
            error_clock: None,
            banned_dismissed: false,
            tick_armed: false,
            demo_countries: Vec::new(),
            demo_guess: None,
            demo_error: None,
        }
    }
}

/// English fixture rows for the demo captures (no `getCountries` there).
pub(super) fn demo_country_fixtures() -> Vec<Country> {
    let row = |iso: &str, name: &str, flag: &str, code: &str| Country {
        iso: iso.into(),
        name: name.into(),
        english_name: name.into(),
        flag: flag.into(),
        calling_codes: vec![code.into()],
        hidden: false,
    };
    vec![
        row("AU", tr(keys::Q_OPS_AUSTRALIA), "\u{1F1E6}\u{1F1FA}", "61"),
        row("BR", tr(keys::Q_OPS_BRAZIL), "\u{1F1E7}\u{1F1F7}", "55"),
        row("CA", tr(keys::Q_OPS_CANADA), "\u{1F1E8}\u{1F1E6}", "1"),
        row("FR", tr(keys::Q_OPS_FRANCE), "\u{1F1EB}\u{1F1F7}", "33"),
        row("DE", tr(keys::Q_OPS_GERMANY), "\u{1F1E9}\u{1F1EA}", "49"),
        row("IN", tr(keys::Q_OPS_INDIA), "\u{1F1EE}\u{1F1F3}", "91"),
        row("IL", tr(keys::Q_OPS_ISRAEL), "\u{1F1EE}\u{1F1F1}", "972"),
        row("JP", tr(keys::Q_OPS_JAPAN), "\u{1F1EF}\u{1F1F5}", "81"),
        row(
            "NL",
            tr(keys::Q_OPS_NETHERLANDS),
            "\u{1F1F3}\u{1F1F1}",
            "31",
        ),
        row("ES", tr(keys::Q_OPS_SPAIN), "\u{1F1EA}\u{1F1F8}", "34"),
        row("SE", "Sweden", "\u{1F1F8}\u{1F1EA}", "46"),
        row("UA", tr(keys::Q_OPS_UKRAINE), "\u{1F1FA}\u{1F1E6}", "380"),
        row(
            "GB",
            tr(keys::Q_OPS_UNITED_KINGDOM),
            "\u{1F1EC}\u{1F1E7}",
            "44",
        ),
        row(
            "US",
            tr(keys::Q_OPS_UNITED_STATES),
            "\u{1F1FA}\u{1F1F8}",
            "1",
        ),
    ]
}

impl QuillApp {
    pub(super) fn signin_countries(&self) -> Vec<Country> {
        match self.live.as_ref() {
            Some(live) => live
                .driver
                .session
                .auth_state
                .countries
                .clone()
                .unwrap_or_default(),
            None => self.auth_ui.signin.demo_countries.clone(),
        }
    }

    pub(super) fn signin_guess_iso(&self) -> Option<String> {
        match self.live.as_ref() {
            Some(live) => live.driver.session.auth_state.guessed_country_iso.clone(),
            None => self.auth_ui.signin.demo_guess.clone(),
        }
    }

    /// Per-frame bookkeeping for the sign-in steps: ask TDLib for the
    /// country list and default, keep the countdown clocks in step with the
    /// auth state and prefill the default country code.
    pub(super) fn sync_signin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let auth = self.current_auth();
        let phone_screen = self.signin_phone_visible(&auth);
        if let Some(live) = self.live.as_mut()
            && phone_screen
        {
            let _ = live.driver.fetch_countries();
            let _ = live.driver.fetch_country_code();
        }
        match &auth {
            AuthorizationState::WaitCode { delivery, .. } => {
                if self.auth_ui.signin.code_clock.as_ref().map(|(d, _)| d) != Some(delivery) {
                    self.auth_ui.signin.code_clock = Some((delivery.clone(), Instant::now()));
                }
            }
            _ => self.auth_ui.signin.code_clock = None,
        }
        match &auth {
            AuthorizationState::WaitEmailCode { reset, .. } => {
                if self.auth_ui.signin.email_clock.as_ref().map(|(r, _)| r) != Some(reset) {
                    self.auth_ui.signin.email_clock = Some((*reset, Instant::now()));
                }
            }
            _ => self.auth_ui.signin.email_clock = None,
        }
        if !quill::auth::can_submit_phone(&auth) {
            self.auth_ui.signin.editing_phone = false;
            self.auth_ui.signin.phone_submitted = false;
        }
        self.apply_login_code_link(&auth, window, cx);
        let phone_screen = self.signin_phone_visible(&auth);
        if phone_screen
            && !self.auth_ui.signin.touched
            && self.auth_ui.signin.phone_text.is_empty()
            && let Some(iso) = self.signin_guess_iso()
        {
            let countries = self.signin_countries();
            if let Some(code) = countries
                .iter()
                .find(|country| country.iso == iso)
                .and_then(|country| country.calling_codes.first())
            {
                let text = format!("+{code}");
                self.auth_ui.signin.phone_text = text.clone();
                cx.defer_in(window, move |this, window, cx| {
                    this.signin_assign_phone(&text, window, cx);
                });
            }
        }
    }

    /// A `tg://login?code=` link that arrives while the code step is up
    /// fills the field and sends the code once it is complete (tdesktop
    /// `CodeWidget::setHandleLoginCode`). On any other step the link waits;
    /// once signed in it gets the "already signed in" answer.
    fn apply_login_code_link(
        &mut self,
        auth: &AuthorizationState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.auth_ui.signin.editing_phone {
            if self
                .pending_deep_link
                .as_deref()
                .and_then(quill::deep_link_types::login_code_from_link)
                .is_some()
            {
                self.pending_deep_link = None;
            }
            return;
        }
        let AuthorizationState::WaitCode { code_length, .. } = auth else {
            return;
        };
        let Some(code) = self
            .pending_deep_link
            .as_deref()
            .and_then(quill::deep_link_types::login_code_from_link)
        else {
            return;
        };
        self.pending_deep_link = None;
        let complete = code_length
            .and_then(|len| usize::try_from(len).ok())
            .is_some_and(|len| len == code.len());
        cx.defer_in(window, move |this, window, cx| {
            if this.auth_ui.signin.editing_phone {
                return;
            }
            this.auth_ui
                .code_input
                .update(cx, |input, cx| input.set_value(code, window, cx));
            if complete {
                this.submit_code(window, cx);
            }
        });
    }

    /// The phone field changed: re-group the digits, refresh the country.
    pub(super) fn signin_phone_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.signin_phone_edited(false, window, cx);
    }

    pub(super) fn signin_code_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.signin_phone_edited(true, window, cx);
    }

    fn signin_phone_edited(
        &mut self,
        code_edited: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .auth_ui
            .phone_input
            .update(cx, |input, cx| input.marked_text_range(window, cx))
            .is_some()
            || self
                .auth_ui
                .signin
                .country_code
                .update(cx, |input, cx| input.marked_text_range(window, cx))
                .is_some()
        {
            return;
        }
        let value = self.signin_phone_value(cx);
        if value == self.auth_ui.signin.phone_text {
            return;
        }
        let countries = self.signin_countries();
        self.auth_ui.signin.touched = true;
        self.auth_ui.signin.hint = None;
        self.signin_clear_value_error();
        let code = phone::format_international(
            &self.auth_ui.signin.country_code.read(cx).value(),
            &countries,
        );
        let national = self.auth_ui.phone_input.read(cx).value().to_string();
        let pasted =
            national.trim_start().starts_with('+') || national.trim_start().starts_with("00");
        if code.calling_code.is_none() && !pasted {
            let prefix = format!("+{}", code.digits);
            let digits = phone::digits_only(&national);
            self.auth_ui.signin.phone_text = format!("{prefix}{digits}");
            set_phone_part(&self.auth_ui.signin.country_code, prefix, 0, window, cx);
            set_phone_part(&self.auth_ui.phone_input, digits, 0, window, cx);
        } else {
            let edited = if code_edited && !pasted {
                format!("{}{national}", code.text)
            } else {
                value
            };
            let formatted =
                phone::reformat_after_edit(&self.auth_ui.signin.phone_text, &edited, &countries);
            self.signin_assign_phone(&formatted.text, window, cx);
        }
        cx.notify();
    }

    fn signin_phone_value(&self, cx: &App) -> String {
        let national = self.auth_ui.phone_input.read(cx).value().to_string();
        if national.trim_start().starts_with('+') || national.trim_start().starts_with("00") {
            national
        } else {
            format!(
                "{}{}",
                self.auth_ui.signin.country_code.read(cx).value(),
                national
            )
        }
    }

    pub(super) fn signin_assign_phone(
        &mut self,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let formatted = phone::format_international(value, &self.signin_countries());
        let code = formatted
            .calling_code
            .as_ref()
            .map_or_else(|| formatted.text.clone(), |code| format!("+{code}"));
        let national = formatted
            .text
            .strip_prefix(&code)
            .unwrap_or_default()
            .to_string();
        let pasted = self
            .auth_ui
            .phone_input
            .read(cx)
            .value()
            .trim_start()
            .starts_with('+')
            || self
                .auth_ui
                .phone_input
                .read(cx)
                .value()
                .trim_start()
                .starts_with("00");
        self.auth_ui.signin.phone_text = formatted.text;
        set_phone_part(
            &self.auth_ui.signin.country_code,
            code.clone(),
            0,
            window,
            cx,
        );
        set_phone_part(
            &self.auth_ui.phone_input,
            national,
            if pasted {
                phone::digits_only(&code).len()
            } else {
                0
            },
            window,
            cx,
        );
    }

    fn signin_pick_country(&mut self, iso: &str, window: &mut Window, cx: &mut Context<Self>) {
        let countries = self.signin_countries();
        let Some(code) = countries
            .iter()
            .find(|country| country.iso == iso)
            .and_then(|country| country.calling_codes.first())
        else {
            return;
        };
        let text = format!("+{code}{}", self.auth_ui.phone_input.read(cx).value());
        self.auth_ui.signin.touched = true;
        self.auth_ui.signin.hint = None;
        self.auth_ui.signin.picker_open = false;
        self.auth_ui
            .signin
            .search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.signin_assign_phone(&text, window, cx);
        self.auth_ui
            .phone_input
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    pub(super) fn signin_toggle_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.auth_ui.signin.picker_open = !self.auth_ui.signin.picker_open;
        if self.auth_ui.signin.picker_open {
            self.auth_ui
                .signin
                .search
                .update(cx, |input, cx| input.focus(window, cx));
        }
        cx.notify();
    }

    pub(super) fn signin_close_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.auth_ui.signin.picker_open = false;
        self.auth_ui
            .phone_input
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    /// Validate what is in the phone field. `Ok` carries the E.164 number
    /// for TDLib and the grouped text for display.
    pub(super) fn signin_checked_phone(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<(String, String)> {
        let countries = self.signin_countries();
        let code = phone::format_international(
            &self.auth_ui.signin.country_code.read(cx).value(),
            &countries,
        );
        if code.calling_code.as_deref() != Some(code.digits.as_str()) {
            self.auth_ui.signin.hint = Some(tr(keys::BAD_PHONE));
            cx.notify();
            return None;
        }
        let value = self.signin_phone_value(cx);
        let formatted = phone::format_international(&value, &countries);
        match phone::e164(&formatted) {
            Some(number) => {
                self.auth_ui.signin.hint = None;
                Some((number, formatted.text))
            }
            None => {
                self.auth_ui.signin.hint = phone::validate(&formatted)
                    .message()
                    .or(Some(tr(keys::BAD_PHONE)));
                cx.notify();
                None
            }
        }
    }

    /// "Wrong number?" on the code screen.
    pub(super) fn signin_back_to_phone(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.signin_can_edit_phone() {
            return;
        }
        self.auth_ui.signin.editing_phone = true;
        self.auth_ui.signin.phone_submitted = false;
        self.auth_ui.signin.hint = None;
        self.signin_clear_value_error();
        self.auth_ui.recovery_mode = false;
        self.auth_ui.registration_photo = None;
        self.auth_ui.accepted_registration_terms = None;
        self.auth_ui.registration_submitted = false;
        self.auth_ui.registration_terms_open = false;
        self.auth_ui
            .password_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.auth_ui
            .code_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.auth_ui
            .recovery_code_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.auth_ui
            .phone_input
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    fn signin_clear_value_error(&mut self) {
        use quill::telegram::envelope::ErrorClass;
        self.auth_ui.signin.demo_error = self
            .auth_ui
            .signin
            .demo_error
            .filter(|error| error.class == ErrorClass::Flood);
        if let Some(live) = self.live.as_mut() {
            live.driver.session.auth_state.last_auth_error = live
                .driver
                .session
                .auth_state
                .last_auth_error
                .filter(|error| error.class == ErrorClass::Flood);
        }
    }

    pub(super) fn signin_can_edit_phone(&self) -> bool {
        quill::auth::can_submit_phone(&self.current_auth())
            && !self.passkey_login_busy()
            && self.session().is_none_or(|s| {
                !s.requests.has_auth_submit()
                    && matches!(s.shutdown, quill::state::ShutdownPhase::Running)
            })
    }

    fn signin_error(&mut self) -> Option<(AuthRequestError, i64)> {
        let err = self
            .session()
            .and_then(|session| session.auth_state.last_auth_error)
            .or(self.auth_ui.signin.demo_error);
        let Some(err) = err else {
            self.auth_ui.signin.error_clock = None;
            self.auth_ui.signin.banned_dismissed = false;
            return None;
        };
        match &self.auth_ui.signin.error_clock {
            Some((seen, since)) if *seen == err => {
                Some((err, i64::try_from(since.elapsed().as_secs()).unwrap_or(0)))
            }
            _ => {
                self.auth_ui.signin.error_clock = Some((err, Instant::now()));
                self.auth_ui.signin.banned_dismissed = false;
                Some((err, 0))
            }
        }
    }

    /// Re-render once a second while any countdown on screen is running.
    fn arm_signin_tick(&mut self, cx: &mut Context<Self>) {
        if self.auth_ui.signin.tick_armed {
            return;
        }
        self.auth_ui.signin.tick_armed = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            let _ = this.update(cx, |this, cx| {
                this.auth_ui.signin.tick_armed = false;
                cx.notify();
            });
        })
        .detach();
    }

    /// Title/body overrides: delivery text on the code step, the phone
    /// prompt while correcting the number.
    pub(super) fn signin_view(&self, auth: &AuthView, state: &AuthorizationState) -> AuthView {
        let mut view = auth.clone();
        if self.auth_ui.signin.editing_phone && quill::auth::can_submit_phone(state) {
            view.title = tr(keys::INTRO_PHONE_TITLE);
            view.body = tr(keys::PHONE_DESC).to_string();
            view.action = quill::auth::AuthAction::EnterPhone;
            return view;
        }
        match state {
            AuthorizationState::WaitCode {
                code_length,
                delivery,
            } => {
                view.title = tr(keys::INTRO_FRAGMENT_TITLE);
                view.body = match delivery.kind {
                    quill::telegram::envelope::CodeKind::TelegramMessage => {
                        tr(keys::CODE_FROM_TELEGRAM).into()
                    }
                    quill::telegram::envelope::CodeKind::Sms
                    | quill::telegram::envelope::CodeKind::Call => tr(keys::CODE_DESC).into(),
                    _ => signin::code_step_body(
                        delivery,
                        *code_length,
                        &self.auth_ui.signin.submitted_phone,
                    ),
                };
            }
            AuthorizationState::WaitEmailCode { email_pattern, .. } => {
                view.title = tr(keys::INTRO_FRAGMENT_TITLE);
                view.body = tr_with(
                    keys::INTRO_EMAIL_CONFIRM_SUBTITLE,
                    &[("email", email_pattern)],
                );
            }
            AuthorizationState::WaitPhoneNumber => {
                view.title = tr(keys::INTRO_PHONE_TITLE);
                view.body = tr(keys::PHONE_DESC).to_string();
            }
            AuthorizationState::WaitEmailAddress => {
                view.title = tr(keys::INTRO_EMAIL_SETUP_TITLE);
                view.body = tr(keys::SETTINGS_CLOUD_LOGIN_EMAIL_ABOUT).into();
            }
            AuthorizationState::WaitPassword { .. } => {
                view.title = tr(keys::INTRO_PASSWORD_TITLE);
                view.body = if self.auth_ui.recovery_mode {
                    tr_with(keys::SIGNIN_RECOVER_DESC, &[("email", "")])
                        .trim_end()
                        .into()
                } else {
                    tr(keys::INTRO_PASSWORD_DESC).into()
                };
            }
            AuthorizationState::WaitRegistration { .. } => {
                view.title = tr(keys::INTRO_SIGNUP_TITLE);
                view.body = tr(keys::INTRO_SIGNUP_DESC).into();
            }
            _ => {}
        }
        view
    }

    pub(super) fn signin_phone_visible(&self, state: &AuthorizationState) -> bool {
        matches!(state, AuthorizationState::WaitPhoneNumber)
            || (self.auth_ui.signin.editing_phone && quill::auth::can_submit_phone(state))
    }

    /// Error line, banned box and flood lock for the current screen.
    /// Returns the elements to append and whether submits are locked.
    pub(super) fn signin_error_elements(
        &mut self,
        cx: &mut Context<Self>,
    ) -> (Vec<AnyElement>, bool) {
        let mut out = Vec::new();
        let Some((err, elapsed)) = self.signin_error() else {
            return (out, false);
        };
        let locked = signin::flood_remaining(&err, elapsed).is_some_and(|left| left > 0);
        if locked {
            self.arm_signin_tick(cx);
        }
        if err.class == quill::telegram::envelope::ErrorClass::PhoneBanned {
            return (out, locked);
        }
        let message = signin::auth_error_line(&err, elapsed);
        out.push(
            div()
                .id("auth-error")
                .role(Role::Alert)
                .aria_label(message.clone())
                .w_full()
                .text_size(px(13.))
                .line_height(px(20.))
                .text_center()
                .text_color(super::td::td(
                    quill::app_theme::PaletteKey::IntroDescriptionFg,
                ))
                .child(message)
                .into_any_element(),
        );
        (out, locked)
    }

    /// tdesktop's `ShowPhoneBannedError` box: the text, OK, and Help which
    /// opens a prefilled mail to Telegram's login support.
    pub(super) fn banned_box(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let phone = self.auth_ui.signin.submitted_phone.clone();
        div()
            .id("phone-banned-box")
            .test_support()
            .role(Role::AlertDialog)
            .aria_label(tr(keys::SIGNIN_BANNED_TEXT))
            .w_full()
            .p_4()
            .gap_3()
            .flex()
            .flex_col()
            .items_center()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().danger)
            .bg(cx.theme().secondary)
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(tr(keys::SIGNIN_BANNED_TEXT)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("banned-help")
                            .label(tr(keys::SIGNIN_BANNED_HELP))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let url = signin::banned_help_mailto(
                                    &phone,
                                    quill::version::APP,
                                    std::env::consts::OS,
                                );
                                this.open_link(&url, cx);
                            })),
                    )
                    .child(
                        Button::new("banned-ok")
                            .label(tr(keys::BOX_OK))
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.auth_ui.signin.banned_dismissed = true;
                                cx.notify();
                            })),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn country_picker(
        &mut self,
        countries: &[Country],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let query = self.auth_ui.signin.search.read(cx).value().to_string();
        let hits = phone::search_countries(countries, &query);
        let muted = cx.theme().muted_foreground;
        let mut list = div()
            .id("country-list")
            .w_full()
            .max_h(px(240.))
            .overflow_y_scroll()
            .flex()
            .flex_col();
        if hits.is_empty() {
            list = list.child(
                div()
                    .p_3()
                    .text_sm()
                    .text_color(muted)
                    .child(tr(keys::Q_OPS_COUNTRY_NOT_FOUND)),
            );
        }
        for country in &hits {
            let iso = country.iso.clone();
            list = list.child(
                Button::new(SharedString::from(format!("country-row-{}", country.iso)))
                    .ghost()
                    .w_full()
                    .h_auto()
                    .p_0()
                    .border_0()
                    .accessibility_label(country.name.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.signin_pick_country(&iso, window, cx);
                    }))
                    .child(
                        div()
                            .w_full()
                            .px_3()
                            .py_2()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(div().w(px(24.)).child(country.flag.clone()))
                            .child(div().flex_1().text_sm().child(country.name.clone()))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(muted)
                                    .child(country.display_code()),
                            ),
                    ),
            );
        }
        div()
            .id("country-picker")
            .test_support()
            .role(Role::Dialog)
            .aria_label(tr(keys::Q_OPS_SEARCH_COUNTRIES))
            .track_focus(&self.auth_ui.signin.picker_focus)
            .w_full()
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                Input::new(&self.auth_ui.signin.search)
                    .aria_label(tr(keys::Q_OPS_SEARCH_COUNTRIES)),
            )
            .child(list)
            .focus_trap("country-picker", &self.auth_ui.signin.picker_focus)
            .into_any_element()
    }

    /// Code entry, resend countdown, "Wrong number?" and the email reset.
    pub(super) fn signin_code_links(
        &mut self,
        busy: bool,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut out: Vec<AnyElement> = Vec::new();
        let auth = self.current_auth();
        let mut ticking = false;
        match &auth {
            AuthorizationState::WaitCode { delivery, .. } => {
                let elapsed = self
                    .auth_ui
                    .signin
                    .code_clock
                    .as_ref()
                    .map_or(0, |(_, since)| {
                        i64::try_from(since.elapsed().as_secs()).unwrap_or(0)
                    });
                if let Some(url) = signin::fragment_url(delivery) {
                    let url = url.to_string();
                    out.insert(
                        0,
                        Button::new("open-fragment")
                            .label(tr(keys::Q_OPS_OPEN_FRAGMENT))
                            .primary()
                            .w_full()
                            .on_click(cx.listener(move |this, _, _, cx| this.open_link(&url, cx)))
                            .into_any_element(),
                    );
                }
                if let Some(option) = signin::resend_option(delivery, elapsed) {
                    ticking |= !option.ready;
                    out.push(
                        Button::new("resend-code")
                            .label(option.label)
                            .ghost()
                            .w_full()
                            .disabled(!option.ready || busy)
                            .on_click(cx.listener(|this, _, _, cx| this.resend_code(cx)))
                            .into_any_element(),
                    );
                }
                out.push(
                    Button::new("wrong-number")
                        .label(tr(keys::Q_OPS_WRONG_NUMBER))
                        .ghost()
                        .w_full()
                        .on_click(
                            cx.listener(|this, _, window, cx| {
                                this.signin_back_to_phone(window, cx)
                            }),
                        )
                        .into_any_element(),
                );
            }
            AuthorizationState::WaitEmailCode { reset, .. } => {
                out.push(
                    Button::new("resend-code")
                        .label(tr(keys::Q_OPS_SEND_THE_CODE_AGAIN))
                        .ghost()
                        .w_full()
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| this.resend_code(cx)))
                        .into_any_element(),
                );
                let elapsed = self
                    .auth_ui
                    .signin
                    .email_clock
                    .as_ref()
                    .map_or(0, |(_, since)| {
                        i64::try_from(since.elapsed().as_secs()).unwrap_or(0)
                    });
                if let Some(label) = signin::email_reset_label(*reset, elapsed) {
                    match reset {
                        EmailResetState::Available { .. } => out.push(
                            Button::new("reset-login-email")
                                .label(label)
                                .ghost()
                                .w_full()
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| this.reset_login_email(cx)))
                                .into_any_element(),
                        ),
                        _ => {
                            ticking = true;
                            out.push(
                                div()
                                    .id("email-reset-pending")
                                    .text_sm()
                                    .text_center()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(label)
                                    .into_any_element(),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
        if ticking {
            self.arm_signin_tick(cx);
        }
        out
    }

    /// `resetAuthenticationEmailAddress` from the email-code step.
    fn reset_login_email(&mut self, cx: &mut Context<Self>) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        self.connection
            .set_note(match live.driver.reset_login_email() {
                Ok(_) => StatusNote::silent(tr(keys::Q_OPS_EMAIL_RESET_REQUESTED_WAITING_FOR)),
                Err(_) => StatusNote::error(tr(keys::Q_OPS_COULD_NOT_RESET_THE_EMAIL)),
            });
        cx.notify();
    }
}
