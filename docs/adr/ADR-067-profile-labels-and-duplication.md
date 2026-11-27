# ADR-067: Profile labels and duplication

**Status:** accepted
**Related:** [ADR-012](ADR-012-profiles-and-instances.md), [ADR-066](ADR-066-sign-in-starts-inside-the-client.md)

## Context

Profiles were shown by their directory name, which is also the lock ID and the keyring attribute. That name is deliberately restricted to `A-Za-z0-9-_`, so it cannot carry spaces or the account it belongs to. A user with two accounts saw two opaque directory names.

New profiles also created a directory and nothing else: the user pressed Roblox, reached the engine's login screen, and only then got a session. Since ADR-066 moved sign-in into a launcher WebView, that delay is no longer needed.

## Decision

A profile keeps its directory name as an immutable internal ID. A new `profile.json` in that directory holds `display_name`, the human label. Switcher labels are `ChosenName [username]` once a saved identity names an account; otherwise just the chosen name. Missing or damaged metadata falls back to the directory name, so a bad label never blocks a launch.

Creating a profile now writes the chosen name as its display name and immediately opens the ADR-066 sign-in window. Cancel leaves a signed-out profile; the next launch still prompts.

Two new menu actions:

- **Duplicate Profile** copies settings, flags, plugin grants, network config and version pin, **including the saved sign-in**. Because keyring items are keyed by the profile's full directory path, cookies and identity are copied through the secret-store API and re-keyed to the destination directory; a raw directory copy would look configured and be signed out. Runtime state, locks, data directories, and the WebKit login caches are left behind. The new profile's label is `Copy of <name>` and an unused directory name is derived from it.
- **Edit Profile Name** changes only `display_name`. Renaming the directory would require moving it, updating `ShellConfig.profile`, and re-keying secrets in three places; that risk buys only a prettier filesystem path, so it is not offered.

## Verification

The gate is the same as ADR-066: after duplicating, launching the copy must reach `APP_READY Home` without a sign-in prompt or a `ChallengeHybridWebView` line. Full workspace tests remain in `.github/workflows/test.yml`.
