use super::*;
use gpui_kit::base::{Checkbox, CheckboxState, Switch};

pub(super) fn row(id: &'static str, label: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .w_full()
        .h(px(st::SETTINGS_ROW))
        .p_0()
        .border_0()
        .rounded_none()
        .relative()
        .justify_start()
        .text_size(px(st::SETTINGS_FONT))
        .line_height(px(st::SETTINGS_LINE))
        .text_color(td(K::WindowBoldFg))
        .bg(td(K::WindowBg))
        .child(
            div()
                .absolute()
                .left(px(st::SETTINGS_TEXT_LEFT))
                .top(px(st::SETTINGS_TEXT_TOP))
                .child(crate::ui::td::label::label(label)),
        )
}

pub(super) fn toggle(
    id: &'static str,
    label: &'static str,
    checked: bool,
    cx: &Context<QuillApp>,
    change: impl Fn(&mut QuillApp, bool, &mut Window, &mut Context<QuillApp>) + 'static,
) -> Switch {
    let listener = cx.listener(move |this, on: &bool, window, cx| change(this, *on, window, cx));
    let fg = td(if checked {
        K::WindowBgActive
    } else {
        K::MenuIconFg
    });
    Switch::new(id)
        .checked(checked)
        .accessibility_label(label)
        .relative()
        .w_full()
        .h(px(38.))
        .text_size(px(st::SETTINGS_FONT))
        .line_height(px(st::SETTINGS_LINE))
        .text_color(td(K::WindowBoldFg))
        .hover(|style| style.bg(td(K::WindowBgOver)))
        .focus_visible(|style| style.bg(td(K::WindowBgOver)))
        .child(
            div()
                .absolute()
                .left(px(st::SETTINGS_TEXT_LEFT))
                .top(px(st::SETTINGS_TEXT_TOP))
                .child(crate::ui::td::label::label(label)),
        )
        .child(
            div()
                .absolute()
                .right(px(23.))
                .top(relative(0.5))
                .mt(px(-7.))
                .w(px(28.))
                .h(px(14.))
                .rounded_full()
                .bg(fg),
        )
        .child(
            div()
                .absolute()
                .right(px(if checked { 21. } else { 35. }))
                .top(relative(0.5))
                .mt(px(-9.))
                .size(px(18.))
                .rounded_full()
                .border_2()
                .border_color(fg)
                .bg(td(K::WindowBg)),
        )
        .on_change(move |on, _, window, cx| listener(&on, window, cx))
}

pub(super) fn checkbox(
    id: &'static str,
    label: &'static str,
    checked: bool,
    cx: &Context<QuillApp>,
    change: impl Fn(&mut QuillApp, bool, &mut Window, &mut Context<QuillApp>) + 'static,
) -> Checkbox {
    let listener = cx.listener(move |this, on: &bool, window, cx| change(this, *on, window, cx));
    Checkbox::new(id)
        .checked(checked)
        .accessibility_label(label)
        .relative()
        .w_full()
        .h(px(st::SETTINGS_ROW))
        .text_size(px(st::SETTINGS_FONT))
        .line_height(px(st::SETTINGS_LINE))
        .text_color(td(K::WindowFg))
        .hover(|style| style.bg(td(K::WindowBgOver)))
        .focus_visible(|style| style.bg(td(K::WindowBgOver)))
        .child(
            div()
                .absolute()
                .left(px(22.))
                .top(px(9.))
                .size(px(22.))
                .rounded(px(2.))
                .border_2()
                .border_color(td(if checked {
                    K::WindowBgActive
                } else {
                    K::CheckboxFg
                }))
                .when(checked, |this| {
                    this.bg(td(K::WindowBgActive))
                        .child(td_icon(TdIcon::IntroCheckboxCheck, td(K::WindowFgActive)))
                }),
        )
        .child(
            div()
                .absolute()
                .left(px(59.))
                .top(px(10.))
                .child(crate::ui::td::label::label(label)),
        )
        .on_change(move |on, _, window, cx| listener(&(on == CheckboxState::Checked), window, cx))
}

pub(super) fn divider(top: f32) -> Div {
    div()
        .absolute()
        .top(px(top))
        .w_full()
        .h(px(st::SETTINGS_DIVIDER))
        .bg(td(K::IntroDividerBg))
}
