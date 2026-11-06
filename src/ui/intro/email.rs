use super::*;
use std::sync::{Arc, OnceLock};

fn email_icon(window: &Window) -> Arc<Image> {
    static IMAGES: [OnceLock<Arc<Image>>; 3] = [const { OnceLock::new() }; 3];
    let (cache, bytes): (_, &[u8]) = if window.scale_factor() <= 1. {
        (
            IMAGES.first(),
            include_bytes!("../../../assets/tdesktop-intro/email.png"),
        )
    } else if window.scale_factor() <= 2. {
        (
            IMAGES.get(1),
            include_bytes!("../../../assets/tdesktop-intro/email@2x.png"),
        )
    } else {
        (
            IMAGES.last(),
            include_bytes!("../../../assets/tdesktop-intro/email@3x.png"),
        )
    };
    let image = || Arc::new(Image::from_bytes(ImageFormat::Png, bytes.to_vec()));
    cache.map_or_else(image, |cache| cache.get_or_init(image).clone())
}

impl QuillApp {
    pub(super) fn intro_email(
        &mut self,
        busy: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (errors, locked) = self.signin_error_elements(cx);
        // EmailWidget: title, 2px skip, description, 100 / 3 * 2 icon,
        // then AddWrappedField. The bundled still is the source's final frame.
        div()
            .relative()
            .w_full()
            .h(px(st::HEIGHT))
            .child(
                div()
                    .absolute()
                    .top(px(st::EMAIL_ICON_TOP))
                    .w_full()
                    .flex()
                    .justify_center()
                    .child(img(email_icon(window)).size(px(st::EMAIL_ICON_SIZE))),
            )
            .child(
                div()
                    .absolute()
                    .top(px(st::EMAIL_FIELD_TOP))
                    .w_full()
                    .child(
                        super::phone::field(
                            &self.auth_ui.email_input,
                            tr(keys::SETTINGS_CLOUD_LOGIN_EMAIL_PLACEHOLDER),
                            st::FIELD_MARGIN_LEFT,
                        )
                        .disabled(busy || locked),
                    ),
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
                    next_button(
                        "submit-login-email",
                        tr(keys::INTRO_NEXT),
                        busy || locked,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.submit_email(window, cx))),
                ),
            )
            .into_any_element()
    }
}
