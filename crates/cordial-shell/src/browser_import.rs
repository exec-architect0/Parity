//! Turn cookies gathered by Cordial's own login WebView into a saved session.
//!
//! This is the hand-off between the browser-looking sign-in window and the
//! game: the WebKit view owns the real Roblox sign-in and its device cookies,
//! and this module writes what it collected into the same secret store the
//! engine reads on startup. No value is logged, including on failure.

use cordial_shell::profile;
use cordial_shell::secrets::{self, Kind, Store};
use std::collections::BTreeMap;
use std::time::Duration;

/// One cookie as read from the login WebView's cookie manager.
#[derive(Clone, Debug)]
pub struct CookieRecord {
    pub domain: String,
    pub name: String,
    pub value: String,
}

/// Whether this Cordial profile already has a stored Roblox session.
///
/// Both backends are checked rather than the configured one: an explicit
/// keyring machine can still have a plaintext file waiting to be adopted, and
/// asking would add a D-Bus round trip to a decision that only needs a name.
pub fn has_saved_session(target: &str) -> bool {
    let Ok(dir) = profile::dir(target) else {
        return false;
    };
    [Store::File, Store::Keyring].iter().any(|store| {
        secrets::load(*store, &dir, Kind::Cookies)
            .is_some_and(|jar| jar.contains(".ROBLOSECURITY="))
    })
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[derive(serde::Deserialize)]
struct AuthenticatedUser {
    id: u64,
    name: String,
    #[serde(default)]
    display_name: String,
}

/// Validate the session with Roblox and write both halves of a Cordial
/// sign-in. The cookie jar keeps every Roblox cookie the WebView collected,
/// because `RBXEventTrackerV2` and the PerimeterX cookies are part of why a
/// browser sign-in is trusted where the engine's login form is challenged.
pub fn save_session(cookies: &[CookieRecord], target: &str) -> Result<(), String> {
    let session = cookies
        .iter()
        .find(|c| c.name == ".ROBLOSECURITY")
        .map(|c| c.value.clone())
        .ok_or("the login window has no Roblox session cookie yet")?;
    let user = authenticated(&session)?;
    if user.id == 0 || user.name.is_empty() {
        return Err("Roblox did not return a usable identity for this session".into());
    }

    let mut by_host: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for cookie in cookies {
        if cookie.name.is_empty() || cookie.value.is_empty() {
            continue;
        }
        let host = cookie.domain.trim_start_matches('.');
        if host.is_empty() || host.contains('\t') || host.contains('\n') {
            continue;
        }
        by_host
            .entry(host.to_string())
            .or_default()
            .push(format!("{}={}", cookie.name, escape(&cookie.value)));
    }
    let mut jar = String::from(
        "# cordial cookie store v1 -- a live Roblox session. Treat it as a password.\n",
    );
    for (host, pairs) in by_host {
        jar.push_str(&host);
        jar.push('\t');
        jar.push_str(&pairs.join("; "));
        jar.push('\n');
    }
    let identity = serde_json::json!({
        "schema": 1,
        "warning": "a live Roblox identity; treat it as personal data and keep it out of any bug report",
        "userId": user.id,
        "username": user.name,
        "displayName": user.display_name,
        "membershipType": 0,
        "isUnder13": false,
        "hasRobloxSubscription": false,
        "countryCode": "",
    })
    .to_string();

    let dir = profile::dir(target)?;
    // Default `auto` gives a plaintext file precedence over an older keyring
    // value; explicit keyring mode still gets its copy.
    let _ = secrets::save(Store::Keyring, &dir, Kind::Cookies, &jar);
    secrets::save(Store::File, &dir, Kind::Cookies, &jar)
        .map_err(|e| format!("could not save cookies: {e}"))?;
    let _ = secrets::save(Store::Keyring, &dir, Kind::Identity, &identity);
    secrets::save(Store::File, &dir, Kind::Identity, &identity)
        .map_err(|e| format!("could not save identity: {e}"))?;
    Ok(())
}

fn authenticated(session: &str) -> Result<AuthenticatedUser, String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .max_redirects(0)
        .build()
        .into();
    let cookie = format!(".ROBLOSECURITY={session}");
    let mut response = agent
        .get("https://users.roblox.com/v1/users/authenticated")
        .header("Cookie", &cookie)
        .call()
        .map_err(|_| "Roblox did not accept this sign-in yet".to_string())?;
    let body = response
        .body_mut()
        .with_config()
        .limit(16 * 1024)
        .read_to_string()
        .map_err(|_| "Roblox's identity reply was unreadable".to_string())?;
    serde_json::from_str(&body)
        .map_err(|_| "Roblox's identity reply was not in the expected shape".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escaping_keeps_one_cookie_on_one_store_line() {
        assert_eq!(escape("a\\b\tc\nd\re"), "a\\\\b\\tc\\nd\\re");
    }
}
