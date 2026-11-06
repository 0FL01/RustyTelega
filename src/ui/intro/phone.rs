use super::*;
use gpui_kit::component::input::{Input, Textarea, TextareaState};
use quill::phone;

pub(in crate::ui) fn field(
    input: &Entity<TextareaState>,
    label: &'static str,
    left: f32,
) -> Textarea {
    Textarea::new(input)
        .aria_label(label)
        .w_full()
        .h(px(st::FIELD_HEIGHT))
        .appearance(false)
        .bordered(false)
        .text_size(px(st::FIELD_FONT))
        .line_height(px(st::FIELD_LINE))
        .text_color(td(K::WindowFg))
        .editor_padding(gpui_kit::Edges {
            top: px(st::FIELD_MARGIN_TOP),
            right: px(left),
            bottom: px(st::FIELD_MARGIN_BOTTOM),
            left: px(left),
        })
}

pub(in crate::ui) fn underline(active: bool, error: bool) -> Div {
    div()
        .absolute()
        .bottom_0()
        .w_full()
        .h(px(if active || error {
            st::FIELD_BORDER_ACTIVE
        } else {
            st::FIELD_BORDER
        }))
        .bg(td(if error {
            K::ActiveLineFgError
        } else if active {
            K::ActiveLineFg
        } else {
            K::InputBorderFg
        }))
}

fn placeholder(typed: String, mask: String) -> impl IntoElement {
    canvas(
        move |_, window, _| {
            let style = window.text_style();
            let size = px(st::FIELD_FONT);
            let typed: SharedString = typed.into();
            let length = typed.chars().count();
            let line = window.text_system().shape_line(
                typed.clone(),
                size,
                &[style.to_run(typed.len())],
                None,
            );
            let rest: SharedString = mask.chars().skip(length).collect::<String>().into();
            let mut run = style.to_run(rest.len());
            run.color = td(K::WindowSubTextFg).into();
            (
                window.text_system().shape_line(rest, size, &[run], None),
                px(f32::from(line.width).ceil()),
            )
        },
        |bounds, (line, skip), window, cx| {
            let _ = line.paint(
                bounds.origin + point(skip, px(0.)),
                px(st::FIELD_LINE),
                Default::default(),
                None,
                window,
                cx,
            );
        },
    )
    .absolute()
    .left(px(st::PHONE_MARGIN_LEFT))
    .top(px(st::FIELD_MARGIN_TOP))
    .w(px(st::PHONE_WIDTH - 2. * st::PHONE_MARGIN_LEFT))
    .h(px(st::FIELD_LINE))
}

impl QuillApp {
    pub(super) fn intro_phone_fields(
        &mut self,
        busy: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let countries = self.signin_countries();
        let formatted = phone::format_international(
            &self.auth_ui.signin.country_code.read(cx).value(),
            &countries,
        );
        let selected = phone::country_for_digits(&formatted.digits, &countries);
        let country = selected.map_or_else(
            || tr(keys::Q_OPS_SELECT_COUNTRY).to_string(),
            |country| country.name.clone(),
        );
        let national = self.auth_ui.phone_input.read(cx).value().to_string();
        let active = self
            .auth_ui
            .phone_input
            .read(cx)
            .focus_handle(cx)
            .is_focused(window);
        let code_active = self
            .auth_ui
            .signin
            .country_code
            .read(cx)
            .focus_handle(cx)
            .is_focused(window);
        let (errors, locked) = self.signin_error_elements(cx);
        let hint = self.auth_ui.signin.hint;
        let error = hint.is_some() || !errors.is_empty();
        let phone_top = st::STEP_FIELD_TOP + st::FIELD_HEIGHT + st::PHONE_TOP;
        let mut page = div()
            .w_full()
            .h(px(st::HEIGHT))
            .relative()
            .text_size(px(st::FIELD_FONT))
            .line_height(px(st::FIELD_LINE))
            .child(
                Button::new("country-select")
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(transparent_black())
                            .foreground(td(K::WindowFg).into()),
                    )
                    .accessibility_label(country.clone())
                    .absolute()
                    .top(px(st::STEP_FIELD_TOP))
                    .w_full()
                    .h(px(st::FIELD_HEIGHT))
                    .p_0()
                    .border_0()
                    .rounded(px(0.))
                    .disabled(busy)
                    .child(
                        div()
                            .size_full()
                            .relative()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(st::FIELD_MARGIN_LEFT))
                                    .top(px(st::FIELD_MARGIN_TOP))
                                    .font_weight(FontWeight::NORMAL)
                                    .text_size(px(st::FIELD_FONT))
                                    .line_height(px(st::FIELD_LINE))
                                    .child(crate::ui::td::label::label(country)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .right(px(st::COUNTRY_ICON_LEFT))
                                    .top(px(st::COUNTRY_ICON_TOP))
                                    .child(td_icon(TdIcon::IntroCountry, td(K::MenuIconFg))),
                            )
                            .child(underline(false, false)),
                    )
                    .on_click(
                        cx.listener(|this, _, window, cx| this.signin_toggle_picker(window, cx)),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(phone_top))
                    .w(px(st::COUNTRY_CODE_WIDTH))
                    .h(px(st::FIELD_HEIGHT))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            if !busy {
                                this.auth_ui
                                    .signin
                                    .country_code
                                    .update(cx, |input, cx| input.focus(window, cx));
                            }
                        }),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(st::FIELD_MARGIN_TOP))
                            .left(px(st::FIELD_MARGIN_LEFT))
                            .right(px(st::FIELD_MARGIN_LEFT))
                            .h(px(st::FIELD_LINE))
                            .child(
                                Input::new(&self.auth_ui.signin.country_code)
                                    .aria_label(tr(keys::INTRO_COUNTRY_CODE))
                                    .appearance(false)
                                    .bordered(false)
                                    .p_0()
                                    .min_h(px(st::FIELD_LINE))
                                    .max_h(px(st::FIELD_LINE))
                                    .text_size(px(st::FIELD_FONT))
                                    .line_height(px(st::FIELD_LINE))
                                    .text_color(td(K::WindowFg))
                                    .text_center()
                                    .disabled(busy),
                            ),
                    )
                    .child(underline(code_active, false)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(phone_top))
                    .right_0()
                    .w(px(st::PHONE_WIDTH))
                    .h(px(st::FIELD_HEIGHT))
                    .child(placeholder(national, formatted.national_placeholder()))
                    .child(
                        field(
                            &self.auth_ui.phone_input,
                            tr(keys::Q_OPS_PHONE_NUMBER),
                            st::PHONE_MARGIN_LEFT,
                        )
                        .disabled(busy),
                    )
                    .child(underline(active, error)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(st::ERROR_TOP))
                    .w_full()
                    .text_size(px(13.))
                    .line_height(px(st::DESCRIPTION_LINE))
                    .text_color(td(K::IntroDescriptionFg))
                    .children(errors)
                    .when_some(hint, |this, hint| {
                        this.child(
                            div()
                                .id("phone-hint")
                                .role(Role::Alert)
                                .aria_label(hint)
                                .text_center()
                                .child(hint),
                        )
                    }),
            )
            .child(
                div().absolute().top(px(st::NEXT_TOP)).child(
                    next_button("submit-phone", tr(keys::INTRO_NEXT), busy || locked, cx)
                        .on_click(cx.listener(|this, _, window, cx| this.submit_phone(window, cx))),
                ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(st::QR_LOGIN_LINK_TOP))
                    .w_full()
                    .flex()
                    .justify_center()
                    .child(
                        intro_link("qr-login", tr(keys::INTRO_PHONE_QR), cx)
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.intro_start(cx);
                            })),
                    ),
            );
        if self.auth_ui.signin.picker_open {
            page = page.child(
                div()
                    .absolute()
                    .top(px(st::STEP_FIELD_TOP))
                    .w_full()
                    .bg(td(K::IntroBg))
                    .occlude()
                    .child(self.country_picker(&countries, cx)),
            );
        }
        page.into_any_element()
    }
}
