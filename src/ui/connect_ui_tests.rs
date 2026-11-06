use crate::ui::app::{PaneMode, QuillApp};
use crate::ui::connect_ui::ConnectUiStatus;
use crate::ui::screenshot_demo::ScreenshotDemo;
use crate::ui::shell::QuillShell;
use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, Entity, Focusable, TestAppContext, VisualTestContext, px, size};
use quill::auth::StartupFailure;
use quill::connect::ConnectBlocker;
use quill::telegram::envelope::AuthorizationState;

const RETRY: &str = "retry-connection";
const PHONE_FORM: &str = "country-select";

fn open(cx: &mut TestAppContext) -> (Entity<QuillApp>, VisualTestContext) {
    let root = quill::scratch::process_dir("quill-failure-card");
    let _ = std::fs::create_dir_all(&root);
    quill::settings::use_isolated_app_root(root.clone());
    cx.update(gpui_kit::init);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = slot.clone();
    let handle = cx.open_window(size(px(1100.), px(800.)), move |window, cx| {
        let demo = ScreenshotDemo::named("need-tdjson");
        let view = cx.new(|cx| QuillApp::new_with_demo(window, cx, None, Some(demo)));
        *slot_in.borrow_mut() = Some(view.clone());
        let shell = cx.new(|_| QuillShell::new(view));
        Root::new(shell, window, cx)
    });
    let app = slot.borrow().clone().expect("app");
    let mut vcx = VisualTestContext::from_window(handle.into(), cx);
    app.update(&mut vcx, |app, _| {
        app.background.test_root = Some(root);
    });
    vcx.run_until_parked();
    (app, vcx)
}

fn shown(vcx: &mut VisualTestContext, id: &'static str) -> bool {
    vcx.update(|window, cx| window.render_frame(cx));
    vcx.update(|window, _| window.try_find(id).is_some())
}

fn reads(vcx: &mut VisualTestContext, text: &str) -> bool {
    let quoted = format!("{text:?}");
    vcx.update(|window, cx| {
        window.set_a11y_forced(true);
        window.render_frame(cx);
        let tree = window.a11y_tree_snapshot().expect("a tree");
        format!("{:?}", tree.nodes).contains(&quoted)
    })
}

#[gpui_kit::test]
fn each_blocker_shows_its_reason_and_retry_and_never_the_phone_form(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    for (blocker, failure) in [
        (ConnectBlocker::LockedStore, StartupFailure::KeyLocked),
        (ConnectBlocker::StoreError, StartupFailure::KeyUnreadable),
        (
            ConnectBlocker::MissingKeyAgainstExistingDb,
            StartupFailure::KeyMissing,
        ),
        (
            ConnectBlocker::DataFolder,
            StartupFailure::DataFolder { detail: None },
        ),
        (ConnectBlocker::MissingTdjson, StartupFailure::Library),
        (ConnectBlocker::TdjsonLoad, StartupFailure::Library),
        (ConnectBlocker::TdjsonVersion, StartupFailure::Library),
        (ConnectBlocker::ReceiveThread, StartupFailure::NoThread),
    ] {
        app.update(&mut vcx, |app, cx| {
            app.connection.status = ConnectUiStatus::Live;
            app.auth_ui.demo_state = AuthorizationState::WaitTdlibParameters;
            app.open_client(Err(blocker), cx);
        });
        assert!(reads(&mut vcx, failure.reason()), "{failure:?}");
        assert!(shown(&mut vcx, RETRY));
        assert!(!shown(&mut vcx, PHONE_FORM));
        assert!(app.read_with(&vcx, |app, _| app.connection.status_note().is_empty()));
    }
}

#[gpui_kit::test]
fn a_database_tdlib_refused_shows_its_reason_and_retry(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    for failure in [
        StartupFailure::WrongKey,
        StartupFailure::DataFolder { detail: None },
    ] {
        app.update(&mut vcx, |app, cx| {
            app.connection.status = ConnectUiStatus::Live;
            app.auth_ui.demo_state = AuthorizationState::StartupFailed(failure.clone());
            cx.notify();
        });
        assert!(reads(&mut vcx, failure.reason()), "{failure:?}");
        assert!(shown(&mut vcx, RETRY));
        assert!(!shown(&mut vcx, PHONE_FORM));
    }
}

#[gpui_kit::test]
fn a_build_without_credentials_keeps_its_note_and_offers_no_retry(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    app.update(&mut vcx, |app, cx| {
        app.open_client(Err(ConnectBlocker::MissingCredentials), cx);
    });
    assert!(app.read_with(&vcx, |app, _| {
        matches!(app.pane_mode(), PaneMode::Synthetic) && !app.connection.status_note().is_empty()
    }));
    assert!(!shown(&mut vcx, RETRY));
}

#[gpui_kit::test]
fn intro_settings_controls_work_without_an_authorized_account(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    vcx.update(|_, cx| crate::ui::keybindings::bind_keys(cx));
    let old = quill::settings::load_update_preferences();
    let old_appearance = app.read_with(&vcx, |app, _| app.appearance.clone());
    app.update(&mut vcx, |app, cx| {
        app.connection.status = ConnectUiStatus::DemoWaitPhone;
        app.settings.update_preferences = Default::default();
        cx.notify();
    });
    vcx.update(|window, cx| {
        window.render_frame(cx);
        window.click("intro-settings", cx);
    });
    assert!(shown(&mut vcx, "intro-update-beta"));
    vcx.update(|window, cx| window.click("intro-update-beta", cx));
    assert!(app.read_with(&vcx, |app, _| app.settings.update_preferences.beta));
    assert!(quill::settings::load_update_preferences().beta);
    vcx.update(|window, cx| window.click("intro-update-automatic", cx));
    assert!(!app.read_with(&vcx, |app, _| app.settings.update_preferences.automatic));
    assert!(!shown(&mut vcx, "intro-update-beta"));
    vcx.update(|window, cx| window.click("intro-scale-default", cx));
    assert!(shown(&mut vcx, "intro-scale-slider"));
    vcx.update(|window, cx| window.click("intro-scale-slider", cx));
    assert_eq!(
        app.read_with(&vcx, |app, _| app.appearance.interface_scale_pct),
        175
    );
    assert!(!app.read_with(&vcx, |app, _| app.appearance.interface_scale_auto));
    assert!(shown(&mut vcx, "intro-theme-classic"));
    vcx.update(|window, cx| window.click("intro-theme-classic", cx));
    vcx.run_until_parked();
    assert_eq!(
        app.read_with(&vcx, |app, _| app.applied_builtin_theme()),
        Some(quill::app_theme::BuiltinTheme::Classic)
    );
    assert!(shown(&mut vcx, "intro-accent-45bce7"));
    vcx.update(|window, cx| window.click("intro-accent-45bce7", cx));
    assert_eq!(
        app.read_with(&vcx, |app, _| app
            .theme_accent(quill::app_theme::BuiltinTheme::Classic)),
        Some(0x45bce7)
    );
    app.update(&mut vcx, |app, cx| {
        app.set_appearance(cx, |prefs| *prefs = old_appearance)
    });
    assert!(shown(&mut vcx, "intro-faq"));
    vcx.update(|window, cx| window.click("intro-language", cx));
    assert!(app.read_with(&vcx, |app, _| app.live.is_none()));
    vcx.update(|window, cx| window.press("escape", cx));
    vcx.run_until_parked();
    assert!(shown(&mut vcx, "intro-connection"));
    vcx.update(|window, cx| window.click("intro-connection", cx));
    vcx.run_until_parked();
    assert!(app.read_with(&vcx, |app, _| app.live.is_none()));
    assert!(shown(&mut vcx, "proxy-add"));
    vcx.update(|window, cx| window.click("proxy-add", cx));
    assert!(shown(&mut vcx, "proxy-edit-save"));
    vcx.update(|window, cx| window.click("proxy-edit-cancel", cx));
    assert!(shown(&mut vcx, "proxy-add"));
    vcx.update(|window, cx| {
        assert_eq!(
            window.focused(cx),
            Some(app.read(cx).settings.focus.clone())
        );
        window.click("proxy-add", cx);
    });
    assert!(shown(&mut vcx, "proxy-edit-save"));
    vcx.update(|window, cx| window.press("escape", cx));
    vcx.run_until_parked();
    assert!(shown(&mut vcx, "proxy-add"));
    vcx.update(|window, cx| window.press("escape", cx));
    assert!(shown(&mut vcx, "intro-connection"));
    quill::settings::save_update_preferences(&old);
}

#[gpui_kit::test]
fn intro_settings_pointer_close_dismisses_and_restores_focus(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    let previous = vcx.update(|window, cx| {
        window.render_frame(cx);
        window.click("intro-settings", cx);
        window.focused(cx)
    });
    assert!(app.read_with(&vcx, |app, _| app.intro.settings_open && app.settings.open));
    assert!(shown(&mut vcx, "settings-close"));
    vcx.update(|window, cx| window.click("settings-close", cx));
    assert!(app.read_with(&vcx, |app, _| !app.intro.settings_open
        && !app.settings.open));
    assert!(!shown(&mut vcx, "settings-close"));
    assert_eq!(vcx.update(|window, cx| window.focused(cx)), previous);
}

#[gpui_kit::test]
fn intro_reset_failure_keeps_qr_and_back_still_returns_to_start(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    app.update(&mut vcx, |app, cx| {
        app.connection.status = ConnectUiStatus::DemoWaitPhone;
        app.auth_ui.demo_state = AuthorizationState::WaitTdlibParameters;
        app.intro.started = true;
        app.intro.qr_requested = true;
        cx.notify();
    });
    assert!(shown(&mut vcx, "intro-qr-phone"));
    assert!(!shown(&mut vcx, PHONE_FORM));
    vcx.update(|window, cx| window.click("intro-qr-phone", cx));
    assert!(shown(&mut vcx, "intro-reset-error"));
    assert!(!shown(&mut vcx, PHONE_FORM));
    vcx.update(|window, cx| window.click("intro-back", cx));
    assert!(shown(&mut vcx, "intro-start"));
    assert!(!shown(&mut vcx, "intro-qr-phone"));
    assert!(!shown(&mut vcx, PHONE_FORM));

    let cache = quill::local_path::media_cache_base().join("intro-reset-back-preserved");
    std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
    std::fs::write(&cache, b"keep").unwrap();
    app.update(&mut vcx, |app, cx| {
        let store = quill::platform::MemorySecretStore::new();
        store.lock();
        app.background.secrets = std::sync::Arc::new(move || Box::new(store.clone()));
        app.credentials = Some(quill::credentials::TelegramCredentials {
            api_id: 99,
            api_hash: "unit-test-not-for-network".into(),
        });
        app.intro.started = true;
        app.intro.qr_requested = true;
        cx.notify();
    });
    assert!(shown(&mut vcx, "intro-back"));
    vcx.update(|window, cx| window.click("intro-back", cx));
    assert!(
        cache.is_file(),
        "Back is not logout and must not sweep media"
    );
    vcx.run_until_parked();
    assert!(cache.is_file());
    assert!(shown(&mut vcx, RETRY));
    vcx.update(|window, cx| window.click(RETRY, cx));
    assert!(
        cache.is_file(),
        "Retry after a failed reset is not logout either"
    );
    vcx.run_until_parked();
    assert!(cache.is_file());
    std::fs::remove_file(cache).unwrap();
}

#[gpui_kit::test]
fn intro_phone_paste_splits_the_code_and_preserves_the_digit_caret(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    vcx.update(|_, cx| crate::ui::keybindings::bind_keys(cx));
    vcx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.connection.status = ConnectUiStatus::DemoWaitPhone;
            app.auth_ui.demo_state = AuthorizationState::WaitPhoneNumber;
            app.auth_ui.demo_inputs = true;
            app.auth_ui.signin.demo_countries = crate::ui::signin_ui::demo_country_fixtures();
            app.signin_assign_phone("+46", window, cx);
            app.auth_ui.phone_input.update(cx, |input, cx| {
                input.set_value("+461234", window, cx);
                input.set_selected_range(7..7, cx);
            });
            app.signin_phone_changed(window, cx);
            assert_eq!(app.auth_ui.signin.country_code.read(cx).value(), "+46");
            assert_eq!(app.auth_ui.phone_input.read(cx).value(), " 12 34");
            assert_eq!(app.auth_ui.phone_input.read(cx).selected_range(), 6..6);
            assert_eq!(app.auth_ui.signin.phone_text, "+46 12 34");
            app.auth_ui
                .signin
                .country_code
                .update(cx, |input, cx| input.set_value("+4", window, cx));
            app.signin_code_changed(window, cx);
            assert_eq!(app.auth_ui.signin.country_code.read(cx).value(), "+4");
            assert_eq!(app.auth_ui.phone_input.read(cx).value(), "1234");
            assert!(app.signin_checked_phone(cx).is_none());
            app.auth_ui
                .signin
                .country_code
                .update(cx, |input, cx| input.set_value("+46", window, cx));
            app.signin_code_changed(window, cx);
            assert_eq!(app.auth_ui.phone_input.read(cx).value(), " 12 34");
            cx.notify();
        })
    });
    assert!(shown(&mut vcx, PHONE_FORM));
    assert!(reads(&mut vcx, "Sweden"));
    vcx.update(|window, cx| window.click("country-select", cx));
    assert!(
        app.read_with(&vcx, |app, _| app.auth_ui.signin.picker_open),
        "country-select bounds: {:?}",
        vcx.debug_bounds("country-select")
    );
    assert!(shown(&mut vcx, "country-picker"));
    vcx.simulate_keystrokes("escape");
    assert!(!shown(&mut vcx, "country-picker"));
    assert!(vcx.update(|window, cx| {
        app.read(cx)
            .auth_ui
            .phone_input
            .read(cx)
            .focus_handle(cx)
            .is_focused(window)
    }));
}

#[gpui_kit::test]
fn intro_code_cells_share_native_paste_composition_and_tdlib_length(cx: &mut TestAppContext) {
    use gpui_kit::test::TestInput;
    use gpui_kit::{ClipboardItem, ElementInputHandler, point};
    let (app, mut vcx) = open(cx);
    vcx.update(|window, cx| {
        crate::ui::keybindings::bind_keys(cx);
        app.update(cx, |app, cx| {
            app.connection.status = ConnectUiStatus::DemoWaitPhone;
            app.auth_ui.demo_state = AuthorizationState::WaitCode {
                code_length: Some(5),
                delivery: quill::telegram::envelope::CodeDelivery {
                    kind: quill::telegram::envelope::CodeKind::Sms,
                    ..Default::default()
                },
            };
            app.auth_ui.signin.submitted_phone = "+44 7700 900123".into();
            app.auth_ui.demo_inputs = true;
            app.auth_ui
                .code_input
                .update(cx, |input, cx| input.focus(window, cx));
            cx.notify();
        });
    });
    vcx.run_until_parked();
    assert!(reads(&mut vcx, "+44 7700 900123"));
    assert!(reads(
        &mut vcx,
        quill::lang::tr(quill::lang::keys::CODE_DESC)
    ));
    vcx.update(|window, cx| {
        window.render_frame(cx);
        let first = window.find(("intro-code-digit", 0usize)).bounds();
        let second = window.find(("intro-code-digit", 1usize)).bounds();
        assert_eq!(first.size, size(px(40.), px(50.)));
        assert_eq!(second.left() - first.right(), px(10.));
        assert!(window.try_find(("intro-code-digit", 5usize)).is_none());
        cx.write_to_clipboard(ClipboardItem::new_string("12 3-456".into()));
        window.press("secondary-v", cx);
    });
    vcx.run_until_parked();
    assert_eq!(
        app.read_with(&vcx, |app, cx| app.auth_ui.code_input.read(cx).value()),
        "12345"
    );
    vcx.update(|window, cx| {
        let input = app.read(cx).auth_ui.code_input.clone();
        input.update(cx, |input, cx| {
            input.set_value("", window, cx);
            input.set_selected_range(0..0, cx);
        });
        let bounds = gpui_kit::Bounds::new(point(px(0.), px(0.)), size(px(260.), px(50.)));
        let mut ime = TestInput::new(ElementInputHandler::new(bounds, input), window);
        ime.compose_text(None, "12", Some(2..2), cx);
        assert_eq!(ime.marked_text_range(cx), Some(0..2));
        ime.commit_text("１２３４５", cx);
        assert!(ime.marked_text_range(cx).is_none());
    });
    vcx.run_until_parked();
    assert_eq!(
        app.read_with(&vcx, |app, cx| app.auth_ui.code_input.read(cx).value()),
        "12345"
    );
    app.update(&mut vcx, |app, cx| {
        app.auth_ui.demo_state = AuthorizationState::WaitEmailCode {
            code_length: Some(6),
            email_pattern: "a***@example.org".into(),
            reset: Default::default(),
        };
        cx.notify();
    });
    vcx.update(|window, cx| {
        window.render_frame(cx);
        assert!(window.try_find(("intro-code-digit", 5usize)).is_some());
    });
}

#[gpui_kit::test]
fn intro_signup_requires_current_terms_and_back_discards_its_photo(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    let terms = quill::telegram::envelope::RegistrationTerms {
        text: "Terms for this registration".into(),
        min_user_age: 16,
    };
    app.update(&mut vcx, |app, cx| {
        app.connection.status = ConnectUiStatus::DemoWaitPhone;
        app.auth_ui.demo_inputs = true;
        app.auth_ui.demo_state = AuthorizationState::WaitRegistration {
            terms: Some(terms.clone()),
        };
        cx.notify();
    });
    assert!(shown(&mut vcx, "registration-photo"));
    assert!(reads(&mut vcx, "Sign Up"));
    vcx.update(|window, cx| window.click("registration-terms", cx));
    vcx.run_until_parked();
    assert!(shown(&mut vcx, "accept-registration-terms"));
    assert!(app.read_with(&vcx, |app, _| {
        app.auth_ui.accepted_registration_terms.is_none()
    }));
    vcx.update(|window, cx| window.click("accept-registration-terms", cx));
    assert_eq!(
        app.read_with(&vcx, |app, _| app
            .auth_ui
            .accepted_registration_terms
            .clone()),
        Some(terms)
    );
    vcx.update(|window, cx| window.click("registration-terms-done", cx));
    vcx.run_until_parked();
    let image_path =
        quill::scratch::process_dir("quill-intro-registration-picker").join("signup.png");
    image::RgbaImage::from_pixel(128, 128, image::Rgba([40, 100, 160, 255]))
        .save(&image_path)
        .unwrap();
    vcx.update(|window, cx| window.click("registration-photo", cx));
    assert!(vcx.did_prompt_for_paths());
    vcx.simulate_path_prompt_response(|_| Some(vec![image_path.clone()]));
    vcx.run_until_parked();
    assert!(shown(&mut vcx, "photo-editor-done"));
    vcx.update(|window, cx| window.click("photo-editor-cancel", cx));
    std::fs::remove_file(image_path).unwrap();
    app.update(&mut vcx, |app, _| {
        use crate::ui::profile_photo_sources::{ChosenPhoto, PhotoTarget};
        let photo = || ChosenPhoto {
            input: quill::profile_photo_sources::ChatPhotoInput::Static {
                path: "not-sent-before-ready".into(),
            },
            preview: None,
        };
        assert!(
            app.take_chosen_photo(
                PhotoTarget::Registration(app.connection.generation + 1),
                photo()
            )
            .is_err()
        );
        assert!(
            app.take_chosen_photo(
                PhotoTarget::Registration(app.connection.generation),
                photo()
            )
            .is_ok()
        );
    });
    vcx.update(|window, cx| window.click("intro-back", cx));
    assert!(
        app.read_with(&vcx, |app, _| app.auth_ui.registration_photo.is_none()
            && app.auth_ui.accepted_registration_terms.is_none())
    );
    assert!(shown(&mut vcx, PHONE_FORM));
    vcx.update(|window, cx| {
        app.update(cx, |app, cx| {
            use crate::ui::profile_photo_sources::ChosenPhoto;
            app.credentials = Some(quill::credentials::TelegramCredentials {
                api_id: 99,
                api_hash: "unit-test-not-for-network".into(),
            });
            assert!(quill::settings::safe_app_root().is_some());
            app.auth_ui.demo_state = AuthorizationState::WaitRegistration { terms: None };
            app.auth_ui.signin.editing_phone = false;
            app.auth_ui.registration_photo = Some(ChosenPhoto {
                input: quill::profile_photo_sources::ChatPhotoInput::Static {
                    path: "not-sent-before-ready".into(),
                },
                preview: None,
            });
            app.auth_ui.registration_submitted = true;
            app.auth_ui.registration_terms_open = true;
            let generation = app.connection.generation;
            app.switch_account(&quill::ids::AccountKey("missing-signup-test-account".into()), "", window, cx);
            assert_eq!(app.connection.generation, generation, "the missing account stops the real switch after cleanup, before a native client starts");
            assert!(app.auth_ui.registration_photo.is_none());
            assert!(!app.auth_ui.registration_submitted);
            assert!(!app.auth_ui.registration_terms_open);
        });
    });
}

#[gpui_kit::test]
fn intro_password_recovery_back_clears_the_native_secret(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    vcx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.connection.status = ConnectUiStatus::DemoWaitPhone;
            app.auth_ui.demo_inputs = true;
            app.auth_ui.demo_state = AuthorizationState::WaitPassword {
                has_recovery_email: false,
            };
            app.auth_ui.recovery_mode = true;
            app.auth_ui.recovery_code_input.update(cx, |input, cx| {
                input.set_value("secret-recovery", window, cx)
            });
            cx.notify();
        })
    });
    assert!(shown(&mut vcx, "cancel-recovery"));
    vcx.update(|window, cx| window.click("cancel-recovery", cx));
    assert!(app.read_with(&vcx, |app, cx| {
        app.auth_ui.recovery_code_input.read(cx).value().is_empty()
    }));
    assert!(shown(&mut vcx, "submit-password"));
}

#[gpui_kit::test]
fn intro_password_back_keeps_tdlib_state_and_presents_the_phone_form(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    app.update(&mut vcx, |app, cx| {
        app.connection.status = ConnectUiStatus::DemoWaitPhone;
        app.auth_ui.demo_state = AuthorizationState::WaitPassword {
            has_recovery_email: true,
        };
        app.auth_ui.demo_inputs = true;
        app.pending_deep_link = Some("tg://login?code=12345".into());
        cx.notify();
    });
    assert!(!shown(&mut vcx, PHONE_FORM));
    vcx.update(|window, cx| window.click("intro-back", cx));
    assert!(shown(&mut vcx, PHONE_FORM));
    assert!(shown(&mut vcx, "submit-phone"));
    assert!(app.read_with(&vcx, |app, _| matches!(
        app.current_auth(),
        AuthorizationState::WaitPassword { .. }
    ) && app.auth_ui.signin.editing_phone));
    assert!(app.read_with(&vcx, |app, _| app.pending_deep_link.is_none()));
    vcx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.auth_ui
                .phone_input
                .update(cx, |input, cx| input.set_value("+46701234567", window, cx));
            app.signin_phone_changed(window, cx);
        })
    });
    vcx.run_until_parked();
    app.update(&mut vcx, |app, cx| {
        app.auth_ui.signin.phone_submitted = true;
        app.auth_ui.signin.demo_error = Some(quill::state::AuthRequestError {
            purpose: quill::state::RequestPurpose::SetPhoneNumber,
            class: quill::telegram::envelope::ErrorClass::PhoneInvalid,
            flood_wait_secs: None,
        });
        cx.notify();
    });
    assert!(shown(&mut vcx, PHONE_FORM));
    assert!(
        app.read_with(&vcx, |app, _| app.auth_ui.signin.error_clock.is_some_and(
            |(error, _)| error.class == quill::telegram::envelope::ErrorClass::PhoneInvalid
        ))
    );
    assert!(reads(
        &mut vcx,
        quill::lang::tr(quill::lang::keys::BAD_PHONE)
    ));
    app.update(&mut vcx, |app, cx| {
        app.auth_ui.demo_state = AuthorizationState::WaitEmailAddress;
        app.auth_ui.signin.demo_error = None;
        app.auth_ui.signin.auth_updated();
        cx.notify();
    });
    assert!(!shown(&mut vcx, PHONE_FORM));
}

#[gpui_kit::test]
fn intro_phone_flood_survives_edits_and_banned_uses_a_dialog(cx: &mut TestAppContext) {
    use quill::state::{AuthRequestError, RequestPurpose};
    use quill::telegram::envelope::ErrorClass;
    let (app, mut vcx) = open(cx);
    vcx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.connection.status = ConnectUiStatus::DemoWaitPhone;
            app.auth_ui.demo_state = AuthorizationState::WaitPhoneNumber;
            app.auth_ui.demo_inputs = true;
            app.signin_assign_phone("+461234", window, cx);
            app.auth_ui.signin.demo_error = Some(AuthRequestError {
                purpose: RequestPurpose::SetPhoneNumber,
                class: ErrorClass::Flood,
                flood_wait_secs: Some(30),
            });
            assert!(app.signin_error_elements(cx).1);
            app.auth_ui
                .phone_input
                .update(cx, |input, cx| input.set_value(" 12 345", window, cx));
            app.signin_phone_changed(window, cx);
            assert!(
                app.signin_error_elements(cx).1,
                "editing must not cancel the flood clock"
            );
            app.auth_ui.signin.demo_error = Some(AuthRequestError {
                purpose: RequestPurpose::SetPhoneNumber,
                class: ErrorClass::PhoneBanned,
                flood_wait_secs: None,
            });
            cx.notify();
        })
    });
    vcx.run_until_parked();
    assert_eq!(
        app.read_with(&vcx, |app, _| QuillShell::open_dialog_name(app)),
        Some("PhoneBanned")
    );
    assert!(shown(&mut vcx, "phone-banned-box"));
    assert!(shown(&mut vcx, "banned-help"));
    assert!(shown(&mut vcx, "banned-ok"));
    vcx.update(|window, cx| window.click("banned-ok", cx));
    vcx.run_until_parked();
    assert!(app.read_with(&vcx, |app, _| app.auth_ui.signin.banned_dismissed));
    assert!(!shown(&mut vcx, "banned-help"));
}

#[gpui_kit::test]
fn trying_again_leaves_the_card_for_a_new_connection(cx: &mut TestAppContext) {
    let (app, mut vcx) = open(cx);
    app.update(&mut vcx, |app, cx| {
        app.open_client(Err(ConnectBlocker::LockedStore), cx);
    });
    assert!(shown(&mut vcx, RETRY));
    let generation = app.read_with(&vcx, |app, _| app.connection.generation);
    vcx.update(|window, cx| {
        window.render_frame(cx);
        window.click(RETRY, cx);
    });
    assert_eq!(
        app.read_with(&vcx, |app, _| app.current_auth()),
        AuthorizationState::WaitTdlibParameters,
        "the reason goes as soon as a new attempt starts"
    );
    assert!(!shown(&mut vcx, RETRY));
    assert_eq!(
        app.read_with(&vcx, |app, _| app.connection.generation),
        generation + 1
    );
    vcx.run_until_parked();
}
