//! The launcher-side sign-in window.
//!
//! Roblox's own login form is rendered by the engine after it starts, and a
//! password there gets a bot challenge a fresh Cordial cannot answer. This
//! window is a real WebKit view on `roblox.com` instead. The user signs in the
//! way they would in a browser; the cookies Roblox issues there — including
//! the device tracker and the risk cookies — are then copied into Cordial's
//! session store before the engine starts, so it opens already signed in.
//!
//! The network session is persistent on purpose, unlike the engine's in-app
//! web windows. WebKit's own form and credential storage is what gives the
//! "use the saved login or enter a new one" choice a real browser has; Cordial
//! does not keep a second copy of the password.

#[cfg(feature = "webview")]
use crate::browser_import::{self, CookieRecord};
use adw::prelude::*;
use libadwaita as adw;
use libadwaita::gtk;

#[cfg(feature = "webview")]
mod in_app {
    use super::*;
    use gtk::prelude::*;
    use libadwaita::glib;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use webkit6::prelude::*;

    const LOGIN_URL: &str = "https://www.roblox.com/login";

    pub fn prompt(window: &gtk::Window, target: &str, proceed: impl Fn() + 'static) {
        let Some(dir) = cordial_shell::profile::dir(target).ok() else {
            show_error(window, "That profile name cannot be used for a browser sign-in.");
            return;
        };
        let data = dir.join("web-login");
        let cache = dir.join("web-login-cache");
        if let Err(e) = std::fs::create_dir_all(&data).and_then(|_| std::fs::create_dir_all(&cache)) {
            show_error(window, &format!("Could not create the login browser data area: {e}"));
            return;
        }
        let (Some(data), Some(cache)) = (data.to_str(), cache.to_str()) else {
            show_error(window, "The login browser data path is not valid UTF-8.");
            return;
        };

        let session = webkit6::NetworkSession::new(Some(data), Some(cache));
        // WebKit's password manager is what remembers the credentials and
        // offers them on the next sign-in. It is bound to this profile's data
        // directory, so two Cordial profiles never share a saved login.
        session.set_persistent_credential_storage_enabled(true);
        let Some(manager) = session.cookie_manager() else {
            show_error(window, "WebKit did not provide a cookie manager for the login window.");
            return;
        };

        let view = webkit6::WebView::builder()
            .network_session(&session)
            .hexpand(true)
            .vexpand(true)
            .build();
        view.load_uri(LOGIN_URL);

        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&adw::WindowTitle::new("Sign in to Roblox", "")));
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&view));

        let dialog = adw::Dialog::new();
        dialog.set_child(Some(&toolbar));
        dialog.set_content_width(1040);
        dialog.set_content_height(760);
        dialog.set_title("Sign in to Roblox");

        let captured = Arc::new(AtomicBool::new(false));
        let manager_for_capture = manager.clone();
        let view_for_capture = view.clone();
        let window_for_capture = window.clone();
        let dialog_for_capture = dialog.clone();
        let target_for_capture = target.to_string();
        let proceed_for_capture = Arc::new(proceed);
        manager.connect_changed(move |_| {
            if captured.load(Ordering::SeqCst) {
                return;
            }
            let manager = manager_for_capture.clone();
            let view = view_for_capture.clone();
            let window = window_for_capture.clone();
            let dialog = dialog_for_capture.clone();
            let target = target_for_capture.clone();
            let proceed = proceed_for_capture.clone();
            let captured = captured.clone();
            manager.cookies("https://www.roblox.com/", gtk::gio::Cancellable::NONE, move |result| {
                let Ok(cookies) = result else { return };
                let Some(session) = cookies
                    .iter()
                    .find(|c| c.name() == ".ROBLOSECURITY")
                    .map(|c| c.value().to_string())
                else {
                    return;
                };
                if session.is_empty() || !captured.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    return;
                }
                let records: Vec<CookieRecord> = cookies
                    .iter()
                    .filter(|c| c.domain().contains("roblox.com"))
                    .map(|c| CookieRecord {
                        domain: c.domain().to_string(),
                        name: c.name().to_string(),
                        value: c.value().to_string(),
                    })
                    .collect();
                let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
                std::thread::spawn(move || {
                    let result = browser_import::save_session(&records, &target);
                    let _ = sender.send(result);
                });
                glib::MainContext::default().attach(Some(&receiver), move |result| {
                    match result {
                        Ok(()) => {
                            dialog.close();
                            proceed();
                        }
                        Err(reason) => {
                            // A transient failure leaves the window usable:
                            // the next cookie change tries again.
                            captured.store(false, Ordering::SeqCst);
                            show_error(&window, &reason);
                        }
                    }
                    glib::ControlFlow::Continue
                });
                // `view` is held so the WebKit view outlives the callback that
                // just read its cookies while the store is written.
                let _ = &view;
            });
        });

        dialog.present(Some(window));
    }

    fn show_error(window: &gtk::Window, message: &str) {
        let dialog = adw::AlertDialog::builder()
            .heading("Could not use that sign-in")
            .body(message)
            .build();
        dialog.add_response("ok", "OK");
        dialog.set_default_response(Some("ok"));
        dialog.set_close_response("ok");
        dialog.present(Some(window));
    }
}

#[cfg(feature = "webview")]
pub use in_app::prompt;

/// A build without WebKitGTK has no way to offer the in-app sign-in. Say so
/// and keep the old path rather than putting up a dialog nobody can use.
#[cfg(not(feature = "webview"))]
pub fn prompt(
    window: &gtk::Window,
    _target: &str,
    proceed: impl Fn() + 'static,
) {
    let dialog = adw::AlertDialog::builder()
        .heading("Browser sign-in is unavailable")
        .body("This Cordial build has no WebKitGTK. The engine's login screen will be used.")
        .build();
    dialog.add_response("ok", "Continue");
    dialog.set_default_response(Some("ok"));
    dialog.set_close_response("ok");
    let proceed = std::cell::RefCell::new(Some(proceed));
    dialog.connect_response(None, move |d, _| {
        d.close();
        if let Some(proceed) = proceed.borrow_mut().take() {
            proceed();
        }
    });
    dialog.present(Some(window));
}
