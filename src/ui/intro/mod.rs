//! Telegram Desktop's signed-out shell. Presentation never substitutes an
//! authorization state; requests and all auth transitions still belong to TDLib.

mod banned;
mod code;
mod cover;
mod demo;
mod dialogs;
mod email;
mod password;
pub(super) mod phone;
mod qr;
mod reset;
mod settings;
mod settings_controls;
mod settings_themes;
mod steps;

use crate::ui::app::QuillApp;
use crate::ui::td::st::intro as st;
use crate::ui::td::{TdIcon, td, td_icon};
use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use quill::app_theme::PaletteKey as K;
use quill::auth::AuthView;
use quill::lang::{keys, tr};
use quill::telegram::envelope::AuthorizationState;

pub(super) struct IntroUi {
    pub(super) started: bool,
    pub(super) qr_requested: bool,
    pub(super) settings_open: bool,
    previous_focus: Option<FocusHandle>,
    reset_generation: Option<u64>,
    reset_error: Option<&'static str>,
    qr_image: Option<qr::QrImage>,
    scale: Option<Entity<gpui_kit::base::slider::SliderState>>,
}

impl IntroUi {
    fn shows_qr(&self, state: &AuthorizationState) -> bool {
        self.started
            && (matches!(
                state,
                AuthorizationState::WaitOtherDeviceConfirmation { .. }
            ) || self.qr_requested
                && matches!(
                    state,
                    AuthorizationState::WaitTdlibParameters | AuthorizationState::WaitPhoneNumber
                ))
    }

    pub(super) fn new(demo: bool) -> Self {
        Self {
            started: demo,
            qr_requested: false,
            settings_open: false,
            previous_focus: None,
            reset_generation: None,
            reset_error: None,
            qr_image: None,
            scale: None,
        }
    }
}

impl QuillApp {
    pub(in crate::ui) fn intro_view(
        &mut self,
        auth: &AuthView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.current_auth();
        let cover = !self.disk.shows_screen()
            && !self.intro.started
            && matches!(
                state,
                AuthorizationState::WaitTdlibParameters
                    | AuthorizationState::WaitPhoneNumber
                    | AuthorizationState::WaitOtherDeviceConfirmation { .. }
            );
        let width = f32::from(window.viewport_size().width);
        let height =
            f32::from(window.viewport_size().height) - crate::ui::td::st::window_title::HEIGHT;
        let top = st::content_top(height, cover);
        let mut page = div()
            .id("intro")
            .size_full()
            .relative()
            .bg(td(K::IntroBg))
            .font_family(crate::ui::td::fonts::INTRO_FAMILY);
        if cover {
            page = page.child(self.intro_cover(width, top, cx));
        } else {
            let qr = !self.disk.shows_screen() && self.intro.shows_qr(&state);
            let step = if qr {
                self.intro_qr(width, top, window, cx)
            } else {
                self.intro_auth_form(auth, width, top, window, cx)
            };
            page = page
                .child(step)
                .child(
                    Button::new("intro-back")
                        .ghost()
                        .accessibility_label(tr(keys::Q_OPS_BACK))
                        .absolute()
                        .left_0()
                        .top_0()
                        .size(px(st::BACK_SIZE))
                        .disabled(
                            self.intro.reset_generation.is_some()
                                || (!qr && !self.signin_can_edit_phone()),
                        )
                        .child(td_icon(TdIcon::Back, td(K::MenuIconFg)))
                        .on_click(cx.listener(|this, _, window, cx| this.intro_back(window, cx))),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(st::SETTINGS_SKIP))
                        .right(px(st::SETTINGS_SKIP))
                        .flex()
                        .items_center()
                        .gap(px(st::SETTINGS_SKIP))
                        .when(self.settings.advanced.test_environment(), |this| {
                            this.child(
                                div()
                                    .id("auth-test-mode-label")
                                    .debug_selector(|| "auth-test-mode-label".into())
                                    .role(Role::Label)
                                    .aria_label(tr(keys::Q_MISC_TEST_MODE))
                                    .text_size(px(13.))
                                    .line_height(px(18.))
                                    .text_color(td(K::WindowSubTextFg))
                                    .child(crate::ui::td::label::label(tr(keys::Q_MISC_TEST_MODE))),
                            )
                        })
                        .child(
                            Button::new("intro-settings")
                                .custom(
                                    ButtonCustomVariant::new(cx)
                                        .foreground(td(K::LightButtonFg).into())
                                        .hover(td(K::LightButtonBgOver).into())
                                        .active(td(K::LightButtonBgRipple).into()),
                                )
                                .accessibility_label(tr(keys::MENU_SETTINGS))
                                .h(px(st::BOX_BUTTON_HEIGHT))
                                .px(px(st::BOX_BUTTON_ADDED_WIDTH / 2.))
                                .child(
                                    div()
                                        .text_size(px(st::BOX_BUTTON_FONT))
                                        .font_semibold()
                                        .line_height(px(20.))
                                        .child(crate::ui::td::label::label(
                                            tr(keys::MENU_SETTINGS).to_uppercase(),
                                        )),
                                )
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.intro.previous_focus = window.focused(cx);
                                    this.intro.settings_open = true;
                                    this.open_settings(None);
                                    window.focus(&this.settings.focus, cx);
                                    cx.notify();
                                })),
                        ),
                );
        }
        if self.intro.settings_open {
            page = page.child(self.intro_settings(width, height, window, cx));
        }
        page.into_any_element()
    }

    pub(in crate::ui) fn close_intro_settings(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(
            self.settings.page,
            Some(
                quill::settings_catalog::Page::Language | quill::settings_catalog::Page::Connection
            )
        ) {
            if self.settings.page == Some(quill::settings_catalog::Page::Language) {
                self.leave_language_settings();
            }
            self.settings.page = None;
            window.focus(&self.settings.focus, cx);
            cx.notify();
            return;
        }
        self.intro.settings_open = false;
        self.close_settings();
        if let Some(focus) = self.intro.previous_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn intro_start(&mut self, cx: &mut Context<Self>) {
        self.auth_ui.signin.editing_phone = false;
        self.auth_ui.signin.phone_submitted = false;
        self.intro.reset_error = None;
        self.intro.started = true;
        self.intro.qr_requested = true;
        if !matches!(
            self.current_auth(),
            AuthorizationState::WaitOtherDeviceConfirmation { .. }
        ) {
            self.request_qr_login(cx);
        }
        cx.notify();
    }

    fn intro_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.intro.shows_qr(&self.current_auth()) {
            self.intro.started = false;
            self.intro.qr_requested = false;
            self.intro.reset_error = None;
            if self.live.is_none() && self.credentials.is_some() {
                self.start_connection(cx);
            }
        } else if self.signin_phone_visible(&self.current_auth()) {
            self.auth_ui.signin.editing_phone = false;
            self.auth_ui.signin.phone_submitted = false;
            self.intro.qr_requested = true;
            self.request_qr_login(cx);
        } else {
            self.signin_back_to_phone(window, cx);
        }
        cx.notify();
    }
}

fn intro_link(id: &'static str, label: &'static str, cx: &App) -> Button {
    Button::new(id)
        .accessibility_label(label)
        .custom(
            ButtonCustomVariant::new(cx)
                .foreground(td(K::WindowActiveTextFg).into())
                .hover(transparent_black())
                .active(transparent_black()),
        )
        .h(px(18.))
        .p_0()
        .border_0()
        .child(
            div()
                .text_size(px(13.))
                .line_height(px(18.))
                .child(crate::ui::td::label::label(label)),
        )
}

pub(super) fn next_button(id: &'static str, label: &'static str, busy: bool, cx: &App) -> Button {
    Button::new(id)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(td(K::ActiveButtonBg).into())
                .foreground(td(K::ActiveButtonFg).into())
                .hover(td(K::ActiveButtonBgOver).into())
                .active(td(K::ActiveButtonBgRipple).into()),
        )
        .accessibility_label(label)
        .w(px(st::NEXT_WIDTH))
        .h(px(st::NEXT_HEIGHT))
        .rounded(px(st::NEXT_RADIUS))
        .bg(td(K::ActiveButtonBg))
        .border_0()
        .p_0()
        .disabled(busy)
        .child(
            div()
                .text_size(px(st::NEXT_FONT))
                .font_semibold()
                .line_height(px(20.))
                .child(crate::ui::td::label::label(label)),
        )
}
