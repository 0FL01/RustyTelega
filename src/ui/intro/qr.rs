use super::*;
use crate::ui::owned_image::OwnedImage;
use crate::ui::peer_qr::qr_path::QrGrid;
use std::sync::Arc;

pub(super) struct QrImage {
    link: String,
    scale: f32,
    side: f32,
    image: OwnedImage,
}

impl QuillApp {
    fn intro_qr_image(
        &mut self,
        link: &str,
        window: &Window,
        cx: &App,
    ) -> Option<(Arc<RenderImage>, f32)> {
        if link.is_empty() {
            return None;
        }
        let scale = window.scale_factor();
        if let Some(cached) = &self.intro.qr_image
            && cached.link == link
            && cached.scale == scale
        {
            return Some((cached.image.clone(), cached.side));
        }
        let grid = QrGrid::encode_with_level(link, qrcode::EcLevel::Q)?;
        let side = st::qr_side(grid.size);
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {n} {n}"><path d="{path}"/></svg>"#,
            n = grid.size,
            path = grid.path()
        );
        let renderer = cx.svg_renderer();
        let parsed = renderer.parse_svg(svg.as_bytes()).ok()?;
        let physical = DevicePixels((side * scale).round() as i32);
        let image = renderer
            .render_parsed(&parsed, SvgSize::ExactSize(size(physical, physical)))
            .ok()?;
        self.intro.qr_image = Some(QrImage {
            link: link.into(),
            scale,
            side,
            image: image.clone().into(),
        });
        Some((image, side))
    }

    pub(super) fn intro_qr(
        &mut self,
        width: f32,
        top: f32,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.current_auth();
        let resetting = self.intro.reset_generation.is_some();
        let link = match &state {
            AuthorizationState::WaitOtherDeviceConfirmation { link } if !resetting => link.as_str(),
            _ => "",
        };
        let image = self.intro_qr_image(link, window, cx);
        let waiting = image.is_none();
        let matrix = image
            .as_ref()
            .map_or(st::qr_side(st::QR_USUAL_MODULES), |(_, side)| *side);
        let outer = st::QR_MAX_SIZE + 2. * st::QR_BACKGROUND_SKIP;
        let plate = matrix + 2. * st::QR_BACKGROUND_SKIP;
        let inset = ((outer - plate) / 2.).floor();
        let mut code = div()
            .id("intro-qr-code")
            .role(Role::Image)
            .aria_label(tr(keys::INTRO_QR_TITLE))
            .absolute()
            .left(px(((width - outer) / 2.).floor()))
            .top(px(top + st::QR_TOP))
            .size(px(outer))
            .child(
                div()
                    .absolute()
                    .left(px(inset))
                    .top(px(inset))
                    .size(px(plate))
                    .rounded(px(st::QR_BACKGROUND_RADIUS))
                    .bg(gpui_kit::white()),
            );
        if let Some((image, side)) = image {
            let skip = ((outer - side) / 2.).floor();
            code = code.child(
                div()
                    .absolute()
                    .left(px(skip))
                    .top(px(skip))
                    .child(img(image).size(px(side))),
            );
        }
        let center = ((outer - st::QR_CENTER_SIZE) / 2.).floor();
        let plane = crate::ui::td::icon::icon_size(TdIcon::IntroQrPlane);
        code = code.child(
            div()
                .absolute()
                .left(px(center))
                .top(px(center))
                .size(px(st::QR_CENTER_SIZE))
                .rounded_full()
                .bg(rgb(st::QR_DISC_COLOR))
                .child(
                    div()
                        .absolute()
                        .left(px(
                            ((st::QR_CENTER_SIZE - f32::from(plane.width)) / 2.).floor()
                        ))
                        .top(px(
                            ((st::QR_CENTER_SIZE - f32::from(plane.height)) / 2.).floor()
                        ))
                        .child(td_icon(TdIcon::IntroQrPlane, gpui_kit::white())),
                ),
        );
        if waiting {
            let side = st::QR_CENTER_SIZE + 2. * st::RADIAL_LINE;
            let skip = ((outer - side) / 2.).floor();
            code = code.child(
                div().absolute().left(px(skip)).top(px(skip)).child(
                    gpui_kit::component::spinner::Spinner::new()
                        .with_size(px(side))
                        .color(rgb(st::QR_DISC_COLOR).into()),
                ),
            );
        }
        let mut page = div().size_full().child(code).child(
            div()
                .absolute()
                .left(px(((width - st::QR_TITLE_WIDTH) / 2.).floor()))
                .w(px(st::QR_TITLE_WIDTH))
                .top(px(top + st::QR_TITLE_TOP))
                .text_size(px(st::QR_TITLE_FONT))
                .font_semibold()
                .line_height(px(27.))
                .text_color(td(K::IntroTitleFg))
                .child(crate::ui::td::label::centered_label(
                    tr(keys::INTRO_QR_TITLE),
                    st::QR_TITLE_WIDTH,
                    div().text_center().child(tr(keys::INTRO_QR_TITLE)),
                )),
        );
        for (index, key) in [
            keys::INTRO_QR_STEP1,
            keys::INTRO_QR_STEP2,
            keys::INTRO_QR_STEP3,
        ]
        .into_iter()
        .enumerate()
        {
            let (plain, bold) = crate::ui::screen_chooser::split_bold(tr(key));
            let highlights = bold
                .into_iter()
                .map(|range| {
                    (
                        range,
                        HighlightStyle {
                            font_weight: Some(FontWeight::SEMIBOLD),
                            ..Default::default()
                        },
                    )
                })
                .collect::<Vec<_>>();
            page = page.child(
                div()
                    .absolute()
                    .left(px(((width - st::QR_LABELS_WIDTH) / 2.).floor()))
                    .top(px(top
                        + st::QR_STEPS_TOP
                        + st::QR_STEP_MARGIN_TOP
                        + index as f32 * 26.))
                    .w(px(st::QR_LABELS_WIDTH))
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(td(K::WindowFg))
                    .child(
                        div()
                            .absolute()
                            .right(px(st::QR_LABELS_WIDTH - st::QR_STEP_MARGIN_LEFT + 4.))
                            .font_semibold()
                            .child(crate::ui::td::label::label(format!("{}.", index + 1))),
                    )
                    .child(
                        div()
                            .ml(px(st::QR_STEP_MARGIN_LEFT))
                            .child(StyledText::new(plain).with_highlights(highlights)),
                    ),
            );
        }
        let busy = resetting
            || self.passkey_login_busy()
            || self.session().is_some_and(|s| s.requests.has_auth_submit());
        page = page.child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(px(top + st::QR_SKIP_TOP))
                .flex()
                .justify_center()
                .child(
                    intro_link("intro-qr-phone", tr(keys::INTRO_QR_PHONE), cx)
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| this.intro_switch_to_phone(cx))),
                ),
        );
        if self.passkey_login_link_shown() {
            page = page.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(top + st::QR_SKIP_TOP + 27.))
                    .flex()
                    .justify_center()
                    .child(
                        intro_link("passkey-login", tr(keys::INTRO_QR_PASSKEY), cx)
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.start_passkey_login(window, cx)
                            })),
                    ),
            );
        }
        let error = self
            .intro
            .reset_error
            .map(str::to_owned)
            .or_else(|| {
                self.session()
                    .and_then(|s| s.settings.passkeys.login.error.clone())
            })
            .or_else(|| self.privacy.passkeys.demo_login_error.clone());
        if let Some(error) = error {
            page = page.child(
                div()
                    .id(if self.intro.reset_error.is_some() {
                        "intro-reset-error"
                    } else {
                        "passkey-login-error"
                    })
                    .test_support()
                    .role(Role::Alert)
                    .aria_label(error.clone())
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(top + st::QR_ERROR_TOP))
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_center()
                    .text_color(td(K::IntroDescriptionFg))
                    .child(error),
            );
        }
        page.into_any_element()
    }
}
