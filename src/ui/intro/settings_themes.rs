use super::settings_controls::checkbox;
use super::*;
use gpui_kit::component::color_picker::ColorPicker;
use quill::app_theme::{BuiltinTheme, ThemeId};

impl QuillApp {
    pub(super) fn intro_themes(&self, width: f32, cx: &mut Context<Self>) -> AnyElement {
        let applied = self.applied_builtin_theme();
        let mut content = div().relative().w_full().h(px(212.));
        for (index, theme) in [
            BuiltinTheme::Classic,
            BuiltinTheme::Day,
            BuiltinTheme::Tinted,
            BuiltinTheme::Night,
        ]
        .into_iter()
        .enumerate()
        {
            let [background, outgoing, incoming] =
                crate::ui::app_theme::builtin_preview(theme, self.theme_accent(theme));
            let selected = applied == Some(theme);
            let accent = match theme {
                BuiltinTheme::Tinted => 0x5ca7d4,
                BuiltinTheme::Night => 0x75bfb5,
                _ => 0xffffff,
            };
            let left = 22. + (index as f32 * (width - 44. - 80.) / 3.).round();
            content = content.child(
                Button::new(format!("intro-theme-{}", theme.id()))
                    .ghost()
                    .absolute()
                    .left(px(left))
                    .top_0()
                    .w(px(80.))
                    .h(px(120.))
                    .p_0()
                    .border_0()
                    .rounded_none()
                    .accessibility_label(theme.name())
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .w_full()
                            .h(px(92.))
                            .rounded(px(6.))
                            .bg(rgb(background))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(6.))
                                    .top(px(8.))
                                    .w(px(40.))
                                    .h(px(14.))
                                    .rounded(px(2.))
                                    .bg(rgb(incoming)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .right(px(6.))
                                    .top(px(28.))
                                    .w(px(40.))
                                    .h(px(14.))
                                    .rounded(px(2.))
                                    .bg(rgb(outgoing)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .left(px(30.))
                                    .bottom(px(13.))
                                    .size(px(20.))
                                    .rounded_full()
                                    .border_2()
                                    .border_color(rgb(accent).opacity(if selected {
                                        1.
                                    } else {
                                        192. / 255.
                                    }))
                                    .when(selected, |this| {
                                        this.child(
                                            div()
                                                .absolute()
                                                .inset(px(3.))
                                                .rounded_full()
                                                .bg(rgb(accent)),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(99.))
                            .w_full()
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .text_color(td(if selected {
                                K::WindowActiveTextFg
                            } else {
                                K::WindowSubTextFg
                            }))
                            .child(crate::ui::td::label::centered_label(
                                theme.name(),
                                80.,
                                div().child(theme.name()),
                            )),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.choose_app_theme(ThemeId::Builtin(theme), theme.is_dark(), cx)
                    })),
            );
        }
        if let Some(theme) = applied {
            let current = self.theme_accent(theme).unwrap_or(theme.accent());
            let mut colors = vec![theme.accent()];
            colors.extend_from_slice(theme.accent_choices());
            if !colors.contains(&current)
                && let Some(last) = colors.last_mut()
            {
                *last = current;
            }
            for (index, color) in colors.iter().copied().enumerate() {
                let selected = !self.appearance.system_accent && color == current;
                content = content.child(
                    Button::new(format!("intro-accent-{color:06x}"))
                        .ghost()
                        .absolute()
                        .left(px(22. + index as f32 * (width - 44. - 24.) / 9.))
                        .top(px(129.))
                        .size(px(24.))
                        .p_0()
                        .border_0()
                        .rounded_full()
                        .accessibility_label(format!("Accent #{color:06x}"))
                        .child(div().size_full().rounded_full().bg(rgb(color)).when(
                            selected,
                            |this| {
                                this.child(
                                    div()
                                        .absolute()
                                        .inset(px(2.5))
                                        .rounded_full()
                                        .border(px(3.))
                                        .border_color(td(K::WindowBg)),
                                )
                            },
                        ))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.set_theme_accent(theme, color, cx)
                        })),
                );
            }
            let mut custom = div()
                .id("intro-custom-accent")
                .track_focus(&self.settings.accent_picker.focus_handle(cx))
                .focus_visible(|style| {
                    style
                        .rounded_full()
                        .border_1()
                        .border_color(td(K::WindowBgActive))
                })
                .absolute()
                .left(px(width - 46.))
                .top(px(129.))
                .size(px(24.))
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .opacity(if self.settings.accent_picker.read(cx).is_open() {
                            1.
                        } else {
                            0.
                        })
                        .child(
                            ColorPicker::new(&self.settings.accent_picker)
                                .accessibility_label(tr(keys::Q_MISC_CUSTOM_ACCENT_COLOR))
                                .size(px(24.)),
                        ),
                );
            for (index, color) in colors.into_iter().take(7).enumerate() {
                let angle = (index as f32 - 1.) * std::f32::consts::PI / 3.;
                let (x, y) = if index == 0 {
                    (9., 9.)
                } else {
                    (9. + 9. * angle.cos(), 9. + 9. * angle.sin())
                };
                custom = custom.child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top(px(y))
                        .size(px(6.))
                        .rounded_full()
                        .bg(rgb(color)),
                );
            }
            content = content.child(custom);
        }
        if self.settings.system_accent.is_some() {
            content = content.child(div().absolute().top(px(160.)).w_full().child(checkbox(
                "intro-system-accent",
                tr(keys::SETTINGS_THEME_SYSTEM_ACCENT_COLOR),
                self.appearance.system_accent,
                cx,
                |this, on, _, cx| this.set_appearance(cx, |prefs| prefs.system_accent = on),
            )));
        }
        content.into_any_element()
    }
}
