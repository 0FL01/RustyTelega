use super::settings_controls::{checkbox, divider, row, toggle};
use super::*;
use gpui_kit::base::slider::{
    Slider, SliderEvent, SliderIndicator, SliderState, SliderThumb, SliderTrack,
};
use quill::settings::{INTERFACE_SCALE_MAX, INTERFACE_SCALE_MIN};

impl QuillApp {
    pub(super) fn intro_settings(
        &mut self,
        width: f32,
        height: f32,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.intro.scale.is_none() {
            let value = self.appearance.interface_scale_pct as f32;
            let scale = cx.new(|_| {
                SliderState::new()
                    .min(INTERFACE_SCALE_MIN as f32)
                    .max(INTERFACE_SCALE_MAX as f32)
                    .step(5.)
                    .default_value(value)
            });
            cx.subscribe(&scale, |this, _, event: &SliderEvent, cx| {
                if let SliderEvent::Change(value) = event {
                    this.set_appearance(cx, |prefs| {
                        prefs.interface_scale_pct = value.end() as u16;
                        prefs.interface_scale_auto = false;
                    });
                }
            })
            .detach();
            self.intro.scale = Some(scale);
        }
        let panel_width = if width < 480. {
            width
        } else {
            (width - 96.).min(st::SETTINGS_WIDTH)
        };
        let top = if panel_width == width {
            0.
        } else {
            (height / 24.).clamp(20., 40.)
        };
        let prefs = self.settings.update_preferences;
        let update_options = prefs.automatic
            && !matches!(
                self.settings.update_state,
                quill::updater::UpdateState::Downloading(..)
            );
        let installing = matches!(
            self.settings.update_state,
            quill::updater::UpdateState::Installing(_)
        );
        let update_skip = if update_options { 0. } else { 80. };
        let frame = crate::ui::window_frame::supported();
        let frame_skip = if frame { 0. } else { 62. };
        let themes_top = 410. - update_skip - frame_skip;
        let accent_skip = if self.settings.system_accent.is_some() {
            0.
        } else {
            40.
        };
        let animations = crate::ui::motion::interface_animations_off();
        let extra = if animations { 62. } else { 0. };
        let body_height = st::SETTINGS_HEIGHT - update_skip - frame_skip - accent_skip + extra;
        let panel_height = body_height.min(height - top * 2.);
        let language = quill::lang::current_pack_id();
        let connection = if self
            .session()
            .is_some_and(|s| s.settings.proxy.enabled().is_some())
        {
            tr(keys::PROXY_USE_CUSTOM)
        } else {
            tr(keys::INTRO_CONNECTION_DEFAULT)
        };
        let language_open = self.settings.page == Some(quill::settings_catalog::Page::Language);
        let connection_open = self.settings.page == Some(quill::settings_catalog::Page::Connection);
        let mut content = div()
            .relative()
            .w_full()
            .h(px(body_height - st::SETTINGS_TOP_BAR));
        content =
            content
                .child(divider(0.))
                .child(
                    div().absolute().top(px(13.)).w_full().child(
                        row("intro-language", "")
                            .accessibility_label(tr(keys::SETTINGS_LANGUAGE))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(20.))
                                    .child(td_icon(TdIcon::IntroLanguage, td(K::WindowBoldFg))),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .left(px(60.))
                                    .child(crate::ui::td::label::label(tr(
                                        keys::SETTINGS_LANGUAGE,
                                    ))),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .right(px(23.))
                                    .text_color(td(K::WindowActiveTextFg))
                                    .child(crate::ui::td::label::label(if language == "en" {
                                        "English".to_string()
                                    } else {
                                        language
                                    })),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_language_box(window, cx);
                                window.focus(&this.settings.focus, cx);
                            })),
                    ),
                )
                .child(
                    div().absolute().top(px(53.)).w_full().child(
                        row("intro-connection", "")
                            .accessibility_label(tr(keys::Q_PREFS_CONNECTION_TYPE_2))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(20.))
                                    .child(td_icon(TdIcon::IntroConnection, td(K::WindowBoldFg))),
                            )
                            .child(div().absolute().left(px(60.)).child(
                                crate::ui::td::label::label(tr(keys::Q_PREFS_CONNECTION_TYPE_2)),
                            ))
                            .child(
                                div()
                                    .absolute()
                                    .right(px(23.))
                                    .text_color(td(K::WindowActiveTextFg))
                                    .child(crate::ui::td::label::label(connection)),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_proxy_list(cx);
                                window.focus(&this.settings.focus, cx);
                            })),
                    ),
                )
                .child(divider(99.))
                .child(
                    div()
                        .absolute()
                        .top(px(111.))
                        .w_full()
                        .h(px(58.))
                        .child(
                            toggle(
                                "intro-update-automatic",
                                tr(keys::INTRO_UPDATE_AUTOMATICALLY),
                                prefs.automatic,
                                cx,
                                |this, on, _, cx| {
                                    this.set_update_preferences(
                                        on,
                                        this.settings.update_preferences.beta,
                                        cx,
                                    )
                                },
                            )
                            .disabled(installing)
                            .h(px(58.)),
                        )
                        .child(
                            div()
                                .absolute()
                                .left(px(22.))
                                .top(px(29.))
                                .text_size(px(13.))
                                .text_color(td(K::WindowSubTextFg))
                                .child(self.settings.update_state.label()),
                        ),
                )
                .when(update_options, |content| {
                    content
                        .child(
                            div().absolute().top(px(169.)).w_full().child(
                                toggle(
                                    "intro-update-beta",
                                    tr(keys::INTRO_INSTALL_BETA),
                                    prefs.beta,
                                    cx,
                                    |this, on, _, cx| {
                                        this.set_update_preferences(
                                            this.settings.update_preferences.automatic,
                                            on,
                                            cx,
                                        )
                                    },
                                )
                                .disabled(installing),
                            ),
                        )
                        .child(
                            div().absolute().top(px(207.)).w_full().child(
                                row("intro-update-check", "")
                                    .accessibility_label(Self::update_control_label(
                                        &self.update_control(),
                                    ))
                                    .disabled(matches!(
                                        self.update_control(),
                                        quill::updater::UpdateControl::Check { busy: true }
                                            | quill::updater::UpdateControl::Restarting
                                    ))
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(st::SETTINGS_TEXT_LEFT))
                                            .top(px(st::SETTINGS_TEXT_TOP))
                                            .child(crate::ui::td::label::label(
                                                Self::update_control_label(&self.update_control()),
                                            )),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.press_update_control(window, cx)
                                    })),
                            ),
                        )
                })
                .child(divider(251. - update_skip));
        if frame {
            content = content
                .child(
                    div()
                        .absolute()
                        .top(px(266. - update_skip))
                        .w_full()
                        .child(checkbox(
                            "intro-window-frame",
                            tr(if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                                keys::INTRO_WINDOW_FRAME
                            } else {
                                keys::SETTINGS_NATIVE_FRAME
                            }),
                            self.appearance.native_window_frame,
                            cx,
                            |this, on, _, cx| {
                                this.set_appearance(cx, |prefs| prefs.native_window_frame = on)
                            },
                        )),
                )
                .child(divider(313. - update_skip));
        }
        let scale_top = 327. - update_skip - frame_skip;
        content = content
            .child(div().absolute().top(px(scale_top)).w_full().child(toggle(
                "intro-scale-default",
                tr(keys::INTRO_DEFAULT_SCALE),
                self.appearance.interface_scale_auto,
                cx,
                |this, on, window, cx| {
                    if on && let Some(scale) = &this.intro.scale {
                        scale.update(cx, |scale, cx| scale.set_value(100., window, cx));
                    }
                    this.set_appearance(cx, |prefs| {
                        prefs.interface_scale_auto = on;
                        if on {
                            prefs.interface_scale_pct = 100;
                        }
                    });
                },
            )))
            .child(
                div()
                    .absolute()
                    .left(px(21.))
                    .right(px(21.))
                    .top(px(scale_top + 45.))
                    .h(px(30.))
                    .child(self.intro_scale()),
            )
            .child(
                div()
                    .absolute()
                    .top(px(themes_top))
                    .w_full()
                    .child(self.intro_themes(panel_width, cx)),
            )
            .child(divider(623. - update_skip - frame_skip - accent_skip))
            .when(animations, |content| {
                content
                    .child(
                        div()
                            .absolute()
                            .top(px(637. - update_skip - frame_skip - accent_skip))
                            .w_full()
                            .child(toggle(
                                "intro-enable-animations",
                                tr(keys::INTRO_ANIMATIONS),
                                false,
                                cx,
                                |this, on, _, cx| {
                                    if on {
                                        this.set_appearance(cx, |prefs| {
                                            prefs.power_saving &=
                                                !quill::power_saving::Flag::Animations.bit()
                                        });
                                    }
                                },
                            )),
                    )
                    .child(divider(685. - update_skip - frame_skip - accent_skip))
            })
            .child(
                div()
                    .absolute()
                    .top(px(637. - update_skip - frame_skip - accent_skip + extra))
                    .w_full()
                    .child(
                        row("intro-faq", tr(keys::SETTINGS_FAQ)).on_click(cx.listener(
                            |this, _, _, cx| this.open_link(quill::settings_account::FAQ_URL, cx),
                        )),
                    ),
            );
        let content = if language_open {
            self.language_page(cx)
        } else if connection_open {
            div()
                .p(px(22.))
                .child(self.proxy_settings_body(cx))
                .into_any_element()
        } else {
            content.into_any_element()
        };
        div()
            .id("intro-settings-backdrop")
            .occlude()
            .absolute()
            .inset_0()
            .bg(td(K::IntroLayerBg))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.close_intro_settings(window, cx)),
            )
            .child(
                div()
                    .id("intro-settings-panel")
                    .test_support()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .absolute()
                    .left(px((width - panel_width) / 2.))
                    .top(px(top))
                    .w(px(panel_width))
                    .h(px(panel_height))
                    .rounded(px(st::SETTINGS_RADIUS))
                    .overflow_hidden()
                    .bg(td(K::WindowBg))
                    .font_family(crate::ui::td::fonts::INTRO_FAMILY)
                    .text_size(px(st::SETTINGS_FONT))
                    .line_height(px(st::SETTINGS_LINE))
                    .text_color(td(K::WindowBoldFg))
                    .track_focus(&self.settings.focus)
                    .focus_trap("intro-settings-focus", &self.settings.focus)
                    .child(
                        div()
                            .relative()
                            .h(px(st::SETTINGS_TOP_BAR))
                            .w_full()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(24.))
                                    .top(px(16.))
                                    .text_size(px(16.))
                                    .text_color(td(K::IntroSettingsTitleFg))
                                    .font_semibold()
                                    .line_height(px(23.))
                                    .child(crate::ui::td::label::label(tr(if language_open {
                                        keys::SETTINGS_LANGUAGE
                                    } else if connection_open {
                                        keys::Q_PREFS_CONNECTION_TYPE_2
                                    } else {
                                        keys::MENU_SETTINGS
                                    }))),
                            )
                            .child(
                                Button::new("settings-close")
                                    .ghost()
                                    .absolute()
                                    .right_0()
                                    .top_0()
                                    .w(px(48.))
                                    .h(px(56.))
                                    .p_0()
                                    .border_0()
                                    .rounded_none()
                                    .child(div().absolute().left(px(4.)).top(px(12.)).child(
                                        td_icon(
                                            TdIcon::IntroSettingsClose,
                                            td(K::IntroSettingsCloseFg),
                                        ),
                                    ))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.close_intro_settings(window, cx)
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("intro-settings-scroll")
                            .absolute()
                            .top(px(st::SETTINGS_TOP_BAR))
                            .bottom_0()
                            .w_full()
                            .overflow_y_scroll()
                            .child(content),
                    ),
            )
            .into_any_element()
    }

    fn intro_scale(&self) -> AnyElement {
        let Some(scale) = &self.intro.scale else {
            return div().into_any_element();
        };
        let fraction = (self.appearance.interface_scale_pct as f32 - 50.) / 250.;
        div()
            .relative()
            .w_full()
            .h(px(30.))
            .child(
                div()
                    .absolute()
                    .right_0()
                    .top_0()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(td(K::WindowActiveTextFg))
                    .child(crate::ui::td::label::label(format!(
                        "{}%",
                        self.appearance.interface_scale_pct
                    ))),
            )
            .child(
                div()
                    .id("intro-scale-slider")
                    .test_support()
                    .absolute()
                    .left_0()
                    .right(px(42.))
                    .h(px(18.))
                    .child(
                        Slider::new(scale)
                            .accessibility_label(tr(keys::Q_PREFS_INTERFACE_SCALE))
                            .w_full()
                            .h(px(18.))
                            .child(
                                SliderTrack::new(scale)
                                    .relative()
                                    .w_full()
                                    .h(px(18.))
                                    .child(
                                        div()
                                            .absolute()
                                            .top(px(7.5))
                                            .w_full()
                                            .h(px(3.))
                                            .bg(td(K::IntroSliderInactiveFg)),
                                    )
                                    .child(
                                        SliderIndicator::new(scale)
                                            .absolute()
                                            .top(px(7.5))
                                            .left(px(7.5))
                                            .right(px(7.5))
                                            .h(px(3.))
                                            .child(
                                                div()
                                                    .absolute()
                                                    .left(px(-7.5))
                                                    .w(px(7.5))
                                                    .h_full()
                                                    .bg(td(K::WindowBgActive)),
                                            )
                                            .child(
                                                div()
                                                    .h_full()
                                                    .w(relative(fraction))
                                                    .bg(td(K::WindowBgActive)),
                                            )
                                            .child(
                                                SliderThumb::new(scale)
                                                    .absolute()
                                                    .top(px(-6.))
                                                    .left(relative(fraction))
                                                    .ml(px(-7.5))
                                                    .size(px(15.))
                                                    .rounded_full()
                                                    .bg(td(K::WindowBgActive)),
                                            ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}
