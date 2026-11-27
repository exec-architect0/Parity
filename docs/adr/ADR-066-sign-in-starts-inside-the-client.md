# ADR-066: Sign-in starts inside the client

**Status:** accepted
**Related:** [ADR-035](ADR-035-browser-account-routing.md), [ADR-063](ADR-063-sign-in-challenge-windows-get-the-native-bridge.md), `docs/analysis/webview-surface.md`

## Context

A fresh profile opened Cordial straight into the engine's Lua `Login` screen. Password sign-in there gets an HTTP 403 whose body says "Incorrect username or password" while the real verdict is that a bot challenge is required. The challenge window opens, reports `challengePageLoaded`/`Parsed`/`Initialized`/`Displayed` through ADR-063's native bridge, and then produces no integrity token and no `challengeCompleted`. The user's hold is not what fails: Roblox does not trust the client that is asking.

What it wants is a session and device identity issued in a browser context. `RBXEventTrackerV2` is minted by `browser-tracker-api/device/initialize` through `WebViewCookieHandler` during a real browser sign-in, not by a standalone API call (`docs/analysis/flag-init.md` §13.1). The engine carries the check string `BrowserTrackerIdRequest: No RBXEventTrackerV2 in cookie.` and Cordial's `browser_tracker.rs` says it has never requested one.

Copying a session out of an installed browser was considered and rejected as the primary path. A Flatpak Cordial cannot see an apt browser's `~/.config` or `~/.mozilla` at all, and Chromium on Linux encrypts its cookies with the OS keyring. Asking the user to pick a browser profile would fail on the machine where the session actually exists.

## Decision

A Cordial profile with no stored `.ROBLOSECURITY` does not start the engine immediately. The launcher shows a WebKit view on `https://www.roblox.com/login` inside the app. The user signs in there as they would in a browser. When the cookie manager reports a `.ROBLOSECURITY` cookie, Cordial:

1. reads every `roblox.com` cookie from that WebView's jar — the session, the device tracker, and the risk cookies the site set;
2. validates the session against `users.roblox.com/v1/users/authenticated`;
3. writes the jar and identity snapshot to the chosen Cordial profile's secret store;
4. closes the sign-in dialog and starts the engine, which opens on Home instead of Login.

The login `NetworkSession` is persistent, bound to a per-profile `web-login` directory, with WebKit's persistent credential storage enabled. That is what makes the browser's own autofill available: on a later sign-in the saved credentials are offered, and the user can choose them or enter new ones. Cordial does not keep its own copy of the password.

Engine web windows remain ephemeral under ADR-063; only this launcher-side login view persists, because remembering the login is the feature.

`--play` keeps the old direct path: it is a programmatic request whose caller cannot answer a dialog.

This partially supersedes ADR-035's "the redeemed session is never copied into a saved profile". That rule was about a browser *join ticket*, which is single-use and belongs to the join. This change copies a session the user just created inside Cordial itself. It does not read any external browser.

## What is deliberately not done

No TEE or keybox simulation is attempted. Roblox verifies Play Integrity tokens server-side; a Linux host has no Android keystore from which one could be minted. No cookie value is logged, including on failure.

A logout in the engine removes the saved session. The next launch therefore returns to this same sign-in view rather than the Lua login form. Reopening it live on logout would need a shell/runtime message that does not yet exist.

## Verification

The verification gate is a launch, not a file parse: after sign-in, `DID_LOG_IN` and `APP_READY Home` appear, with no `ChallengeHybridWebView` line. The full workspace test run remains in `.github/workflows/test.yml`.
