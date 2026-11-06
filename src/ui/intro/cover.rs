use super::*;

impl QuillApp {
    pub(super) fn intro_cover(&self, width: f32, top: f32, cx: &mut Context<Self>) -> AnyElement {
        let left_size = crate::ui::td::icon::icon_size(TdIcon::IntroLeft);
        let right_size = crate::ui::td::icon::icon_size(TdIcon::IntroRight);
        let plane_size = crate::ui::td::icon::icon_size(TdIcon::IntroPlaneTop);
        let lw = f32::from(left_size.width);
        let rw = f32::from(right_size.width);
        let (left, right) = if width < st::COVER_MAX_WIDTH {
            let max_skip = st::COVER_MAX_WIDTH - lw - rw;
            let skip = st::COVER_ICONS_MIN_SKIP
                + (max_skip - st::COVER_ICONS_MIN_SKIP) * (width - st::STEP_WIDTH)
                    / (st::COVER_MAX_WIDTH - st::STEP_WIDTH);
            let outside = (skip + lw + rw - width).trunc();
            let left = (-outside / 2.).trunc();
            (left, -outside - left)
        } else {
            (0., 0.)
        };
        // The external surface uses tdesktop's 208 scanlines, including both
        // gradient endpoints, rather than a GPU gradient's half-pixel samples.
        let cover = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(st::COVER_HEIGHT))
            .overflow_hidden()
            .children((0..st::COVER_HEIGHT as u32).map(|y| {
                let a = td(K::IntroCoverTopBg);
                let b = td(K::IntroCoverBottomBg);
                let t = y as f32 / (st::COVER_HEIGHT - 1.);
                let mix = |x: f32, y: f32| ((x + (y - x) * t) * 255.).round() / 255.;
                let color = Rgba {
                    r: mix(a.r, b.r),
                    g: mix(a.g, b.g),
                    b: mix(a.b, b.b),
                    a: 1.,
                };
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(y as f32))
                    .h(px(1.))
                    .bg(color)
            }))
            .child(
                div()
                    .absolute()
                    .left(px(left))
                    .bottom_0()
                    .child(td_icon(TdIcon::IntroLeft, td(K::IntroCoverIconsFg))),
            )
            .child(
                div()
                    .absolute()
                    .right(px(right))
                    .bottom_0()
                    .child(td_icon(TdIcon::IntroRight, td(K::IntroCoverIconsFg))),
            )
            .children(
                [
                    (TdIcon::IntroPlaneTrace, K::IntroCoverPlaneTrace),
                    (TdIcon::IntroPlaneInner, K::IntroCoverPlaneInner),
                    (TdIcon::IntroPlaneOuter, K::IntroCoverPlaneOuter),
                    (TdIcon::IntroPlaneTop, K::IntroCoverPlaneTop),
                ]
                .into_iter()
                .map(|(icon, color)| {
                    div()
                        .absolute()
                        .left(px(((width - f32::from(plane_size.width)) / 2.).floor()
                            - st::COVER_ICON_LEFT))
                        .top(px(st::COVER_ICON_TOP))
                        .child(td_icon(icon, td(color)))
                }),
            );
        let description = tr(keys::INTRO_ABOUT);
        div()
            .size_full()
            .child(cover)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(top + st::COVER_TITLE_TOP))
                    .text_size(px(st::COVER_TITLE_FONT))
                    .font_semibold()
                    .line_height(px(30.))
                    .text_color(td(K::IntroTitleFg))
                    .child(crate::ui::td::label::centered_label(
                        "Telegram Desktop",
                        width,
                        div().text_center().child("Telegram Desktop"),
                    )),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(top + st::COVER_DESCRIPTION_TOP))
                    .flex()
                    .flex_col()
                    .text_size(px(st::COVER_DESCRIPTION_FONT))
                    .line_height(px(st::COVER_DESCRIPTION_LINE))
                    .text_color(td(K::IntroDescriptionFg))
                    .children(description.lines().map(|line| {
                        crate::ui::td::label::centered_label(
                            line.to_owned(),
                            width,
                            div().text_center().child(line.to_owned()),
                        )
                    })),
            )
            .child(
                div()
                    .absolute()
                    .left(px(st::content_left(width)))
                    .top(px(top + st::NEXT_TOP))
                    .child(
                        next_button(
                            "intro-start",
                            tr(keys::START_MSGS),
                            !matches!(
                                self.current_auth(),
                                AuthorizationState::WaitPhoneNumber
                                    | AuthorizationState::WaitOtherDeviceConfirmation { .. }
                            ),
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.intro_start(cx))),
                    ),
            )
            .into_any_element()
    }
}
