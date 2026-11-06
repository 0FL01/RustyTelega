use super::*;
use crate::ui::background_accounts::closing::CloseWait;
use crate::ui::status_note::StatusNote;
use quill::connect::{ConnectBlocker, LiveConnect, prepare_connect, start_prepared_live_connect};
use quill::diagnostics::DiscardSink;
use quill::ids::AccountKey;
use std::sync::Arc;
use std::time::{Duration, Instant};

impl QuillApp {
    pub(in crate::ui) fn cancel_intro_reset(&mut self) {
        self.auth_ui.registration_photo = None;
        self.auth_ui.registration_submitted = false;
        self.auth_ui.registration_terms_open = false;
        self.intro.reset_generation = None;
        self.intro.reset_error = None;
        self.intro.qr_requested = false;
        self.intro.qr_image = None;
    }

    pub(super) fn intro_switch_to_phone(&mut self, cx: &mut Context<Self>) {
        // Apply an already-arrived QR completion before deciding to discard.
        if self.poll_live(cx) {
            self.intro_reset_failed(tr(keys::Q_INTRO_RESET_BUSY), cx);
            return;
        }
        if self.intro.started
            && self.intro.qr_requested
            && matches!(self.current_auth(), AuthorizationState::WaitPhoneNumber)
            && self
                .session()
                .is_some_and(|s| !s.requests.has_auth_submit())
        {
            self.intro.qr_requested = false;
            self.intro.reset_error = None;
            cx.notify();
            return;
        }
        if !self.intro.started
            || self.disk.shows_screen()
            || self.intro.reset_generation.is_some()
            || self.passkey_login_busy()
            || !matches!(
                self.current_auth(),
                AuthorizationState::WaitOtherDeviceConfirmation { .. }
            )
        {
            self.intro_reset_failed(tr(keys::Q_INTRO_RESET_BUSY), cx);
            return;
        }
        let (Some(credentials), Some(root)) = (self.credentials.clone(), self.accounts_root())
        else {
            self.intro_reset_failed(tr(keys::Q_INTRO_RESET_FAILED), cx);
            return;
        };
        let Some(mut old) = self.live.take() else {
            // Demo fixtures never run or imitate a production reset.
            self.intro_reset_failed(tr(keys::Q_INTRO_RESET_FAILED), cx);
            return;
        };
        let account = old.driver.session.account.clone();
        // Invalidate the screen poll before sending destroy or awaiting work.
        self.connection.generation += 1;
        let generation = self.connection.generation;
        if old.begin_qr_phone_reset().is_err() {
            self.live = Some(old);
            self.spawn_poll_loop(cx);
            self.intro_reset_failed(tr(keys::Q_INTRO_RESET_BUSY), cx);
            return;
        }
        self.intro.reset_generation = Some(generation);
        self.intro.reset_error = None;
        self.intro.qr_image = None;
        self.intro.qr_requested = true;
        self.close_live_later(old, false, cx);
        let key = account.clone();
        let secrets = self.secrets();
        self.after_closing(
            cx,
            move |this| {
                if this.connection.generation != generation {
                    CloseWait::Never
                } else {
                    this.open_wait(&key)
                }
            },
            move |this, confirmed, cx| {
                if this.connection.generation != generation
                    || quill::settings::active_account(&root) != account
                {
                    return;
                }
                if !confirmed {
                    this.intro.reset_generation = None;
                    this.intro_reset_failed(tr(keys::Q_INTRO_RESET_UNCONFIRMED), cx);
                    return;
                }
                this.background.begin_intro_open(&account);
                let key = account.clone();
                let prepare_credentials = credentials.clone();
                let prepare = cx.background_executor().spawn(async move {
                    prepare_connect(&root, account, secrets().as_ref(), &prepare_credentials)
                });
                Self::spawn_main(cx, async move |this, cx| {
                    let prepared = prepare.await;
                    // The native client/call engine stay on the foreground executor.
                    let opened = this
                        .update(cx, |this, cx| {
                            if this.connection.generation != generation {
                                this.background.end_intro_open(&key);
                                return None;
                            }
                            if this.background.wiping
                                || this.disk.is_full()
                                || this.background.deleting.contains(&key)
                            {
                                this.background.end_intro_open(&key);
                                this.finish_intro_reset(
                                    &key,
                                    generation,
                                    Err(ConnectBlocker::DataFolder),
                                    cx,
                                );
                                return None;
                            }
                            Some(prepared.and_then(|prepared| {
                                start_prepared_live_connect(
                                    credentials,
                                    prepared,
                                    Arc::new(DiscardSink),
                                )
                            }))
                        })
                        .ok()
                        .flatten();
                    let Some(opened) = opened else {
                        return;
                    };
                    let opened = match opened {
                        Ok(mut fresh) => {
                            let deadline = Instant::now() + Duration::from_secs(30);
                            let phone = loop {
                                let current = this
                                    .update(cx, |this, _| {
                                        this.connection.generation == generation
                                            && !this.background.wiping
                                            && !this.disk.is_full()
                                            && !this.background.deleting.contains(&key)
                                    })
                                    .unwrap_or(false);
                                if !current || Instant::now() >= deadline {
                                    fresh.discard_reset_client();
                                    break false;
                                }
                                if let Some(phone) = fresh.poll_reset_phone() {
                                    break phone;
                                }
                                cx.background_executor()
                                    .timer(Duration::from_millis(16))
                                    .await;
                            };
                            Ok((fresh, phone))
                        }
                        Err(error) => Err(error),
                    };
                    let _ = this.update(cx, |this, cx| {
                        this.background.end_intro_open(&key);
                        this.finish_intro_reset(&key, generation, opened, cx);
                    });
                })
                .detach();
            },
        );
        cx.notify();
    }

    fn finish_intro_reset(
        &mut self,
        account: &AccountKey,
        generation: u64,
        opened: Result<(LiveConnect, bool), ConnectBlocker>,
        cx: &mut Context<Self>,
    ) {
        // A replaced account/flow can never receive this client.
        if self.connection.generation == generation {
            self.intro.reset_generation = None;
        }
        if self.connection.generation != generation
            || self
                .accounts_root()
                .is_none_or(|root| quill::settings::active_account(&root) != *account)
            || self.background.wiping
            || self.background.deleting.contains(account)
            || self.disk.is_full()
        {
            if let Ok((mut fresh, _)) = opened {
                fresh.discard_reset_client();
                self.close_live_later(fresh, false, cx);
            }
            return;
        }
        match opened {
            Ok((fresh, true)) if self.live.is_none() => {
                self.intro.qr_requested = false;
                self.intro.reset_error = None;
                self.open_client(Ok(fresh), cx);
            }
            Ok((mut fresh, _)) => {
                fresh.discard_reset_client();
                self.close_live_later(fresh, false, cx);
                self.intro_reset_failed(tr(keys::Q_INTRO_RESET_PHONE_FAILED), cx);
            }
            Err(_) => self.intro_reset_failed(tr(keys::Q_INTRO_RESET_FAILED), cx),
        }
        cx.notify();
    }

    fn intro_reset_failed(&mut self, error: &'static str, cx: &mut Context<Self>) {
        self.intro.reset_error = Some(error);
        self.connection.set_note(StatusNote::error(error));
        cx.notify();
    }
}

#[cfg(test)]
#[cfg(feature = "demo-capture")]
mod tests {
    use super::{AccountKey, AuthorizationState, ConnectBlocker, QuillApp};
    use crate::ui::screenshot_demo::ScreenshotDemo;
    use gpui_kit::{AppContext, TestAppContext};

    #[gpui_kit::test]
    fn intro_reset_stale_completion_cannot_change_the_replacement_flow(cx: &mut TestAppContext) {
        let root = quill::scratch::process_dir("quill-intro-reset");
        let _ = std::fs::create_dir_all(&root);
        quill::settings::use_isolated_app_root(root);
        cx.update(gpui_kit::init);
        let vcx = cx.add_empty_window();
        let app = vcx.update(|window, cx| {
            cx.new(|cx| {
                QuillApp::new_with_demo(window, cx, None, Some(ScreenshotDemo::named("intro-qr")))
            })
        });
        app.update(vcx, |app, cx| {
            app.connection.generation = 42;
            app.intro.reset_generation = Some(42);
            app.finish_intro_reset(
                &AccountKey::primary(),
                41,
                Err(ConnectBlocker::MissingCredentials),
                cx,
            );
            assert_eq!(app.intro.reset_generation, Some(42));
            assert!(app.intro.reset_error.is_none());
            assert!(matches!(
                app.current_auth(),
                AuthorizationState::WaitOtherDeviceConfirmation { .. }
            ));
            app.cancel_intro_reset();
            assert!(app.intro.reset_generation.is_none());
        });
    }
}
