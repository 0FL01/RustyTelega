use super::*;
use gpui_kit::component::input::Textarea;
use zeroize::Zeroize;

fn code_digit(ch: char) -> Option<char> {
    if ch.is_ascii_digit() {
        Some(ch)
    } else if ('０'..='９').contains(&ch) {
        char::from_u32('0' as u32 + ch as u32 - '０' as u32)
    } else {
        None
    }
}

fn digits_count(state: &AuthorizationState) -> Option<usize> {
    let length = match state {
        AuthorizationState::WaitCode { code_length, .. }
        | AuthorizationState::WaitEmailCode { code_length, .. } => *code_length,
        _ => None,
    }?;
    usize::try_from(length)
        .ok()
        .filter(|length| (1..=32).contains(length))
}

impl QuillApp {
    pub(in crate::ui) fn intro_code_changed(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = self.current_auth();
        if self.auth_ui.signin.editing_phone
            || !matches!(
                state,
                AuthorizationState::WaitCode { .. } | AuthorizationState::WaitEmailCode { .. }
            )
            || self
                .auth_ui
                .code_input
                .update(cx, |input, cx| input.marked_text_range(window, cx))
                .is_some()
        {
            return;
        }
        let count = digits_count(&state);
        let mut value = self.auth_ui.code_input.read(cx).value().to_string();
        let mut digits: String = value
            .chars()
            .filter_map(code_digit)
            .take(count.unwrap_or(usize::MAX))
            .collect();
        if digits != value {
            self.auth_ui.code_input.update(cx, |input, cx| {
                let caret = input.selected_range().end.min(value.len());
                let at = value
                    .get(..caret)
                    .unwrap_or_default()
                    .chars()
                    .filter_map(code_digit)
                    .count()
                    .min(digits.len());
                input.set_value(digits.clone(), window, cx);
                input.set_selected_range(at..at, cx);
            });
        }
        let complete = count.is_some_and(|count| digits.len() == count);
        digits.zeroize();
        value.zeroize();
        if complete && !self.session().is_some_and(|s| s.requests.has_auth_submit()) {
            self.submit_code(window, cx);
        }
        cx.notify();
    }

    pub(super) fn intro_code(
        &mut self,
        busy: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (errors, locked) = self.signin_error_elements(cx);
        let count = digits_count(&self.current_auth());
        let mut form = div().relative().w_full().h(px(st::HEIGHT));
        if let Some(count) = count {
            let input = self.auth_ui.code_input.read(cx);
            let value = input.value();
            let focused = input.focus_handle(cx).is_focused(window);
            let caret = input.selected_range().end.min(count.saturating_sub(1));
            let width =
                count as f32 * (st::CODE_DIGIT_WIDTH + st::CODE_DIGIT_SKIP) + st::CODE_DIGIT_SKIP;
            let mut cells = div()
                .absolute()
                .left(px(-st::SHAKE_SHIFT - 1.))
                .top(px(st::CODE_TOP))
                .w(px(width))
                .h(px(st::CODE_DIGIT_HEIGHT));
            // Keep the native editor mounted for keyboard, clipboard and IME;
            // the opaque cells are its presentation, not separate input states.
            cells = cells.child(
                div().absolute().size_full().opacity(0.).child(
                    Textarea::new(&self.auth_ui.code_input)
                        .aria_label(tr(keys::CODE_PH))
                        .appearance(false)
                        .bordered(false)
                        .disabled(busy || locked)
                        .h(px(st::CODE_DIGIT_HEIGHT)),
                ),
            );
            for ix in 0..count {
                let digit = value
                    .chars()
                    .nth(ix)
                    .filter(char::is_ascii_digit)
                    .map(|digit| digit.to_string())
                    .unwrap_or_default();
                cells = cells.child(
                    div()
                        .id(("intro-code-digit", ix))
                        .test_support()
                        .absolute()
                        .left(px(st::CODE_DIGIT_SKIP
                            + ix as f32 * (st::CODE_DIGIT_WIDTH + st::CODE_DIGIT_SKIP)))
                        .w(px(st::CODE_DIGIT_WIDTH))
                        .h(px(st::CODE_DIGIT_HEIGHT))
                        .rounded(px(st::CODE_DIGIT_RADIUS))
                        .bg(td(K::WindowBgOver))
                        .border(px(st::CODE_DIGIT_BORDER / 2.))
                        .border_color(td(if !errors.is_empty() {
                            K::ActiveLineFgError
                        } else if focused && ix == caret {
                            K::WindowActiveTextFg
                        } else {
                            K::WindowBgRipple
                        }))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(st::CODE_DIGIT_FONT))
                        .text_color(td(K::WindowFg))
                        .child(digit)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                if !busy && !locked {
                                    this.auth_ui.code_input.update(cx, |input, cx| {
                                        input.focus(window, cx);
                                        let at = ix.min(input.value().len());
                                        input.set_selected_range(at..at, cx);
                                    });
                                }
                            }),
                        ),
                );
            }
            form = form.child(cells);
        } else {
            // TDLib can omit the length. Keep a real field and manual submit
            // rather than inventing a server-provided cell count.
            form = form.child(
                div().absolute().top(px(st::CODE_TOP)).w_full().child(
                    super::phone::field(
                        &self.auth_ui.code_input,
                        tr(keys::CODE_PH),
                        st::FIELD_MARGIN_LEFT,
                    )
                    .disabled(busy || locked),
                ),
            );
        }
        let links = self.signin_code_links(busy || locked, cx);
        form.child(
            div()
                .absolute()
                .top(px(st::CODE_TOP + st::CODE_DIGIT_HEIGHT + st::LINK_TOP))
                .w_full()
                .flex()
                .flex_col()
                .children(links),
        )
        .child(
            div()
                .absolute()
                .top(px(st::ERROR_TOP))
                .w_full()
                .children(errors),
        )
        .child(
            div().absolute().top(px(st::NEXT_TOP)).child(
                next_button("submit-code", tr(keys::INTRO_NEXT), busy || locked, cx)
                    .on_click(cx.listener(|this, _, window, cx| this.submit_code(window, cx))),
            ),
        )
        .into_any_element()
    }
}
