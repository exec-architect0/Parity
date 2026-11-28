# ADR-069: Reader-facing defaults

**Status:** accepted
**Related:** [ADR-033](ADR-033-a-version-picker.md), [ADR-044](ADR-044-settings-are-a-live-control-surface.md), [ADR-066](ADR-066-sign-in-starts-inside-the-client.md), [ADR-067](ADR-067-profile-labels-and-duplication.md)

## Context

Three complaints had the same shape: the header showed an unreadable `name [username]` at every width, Preferences asked the reader to choose among seven tabs, and the render resolution and UI density were numbers picked on one machine. Fullscreen requests from the game were made from the engine thread and did not reliably reach GTK.

## Decision

- The header profile button shows only the chosen display name. The full `name [username]` label is in the tooltip, the menu and the accessible name, so long accounts never ellipsise inside a crowded header.
- Preferences has one **Roblox** page folding the former Roblox, Updates and Version pages together. Groups are moved from their existing builders, not recreated. The launcher's version button opens that page. Plugins, FastFlags and App remain separate destinations.
- Render resolution defaults to the primary monitor's geometry, capped at 2560x1440 to bound the workload. `dpiScale` defaults to the monitor's physical DPI divided by 96, including GDK's scale factor, clamped to 0.75–2.5. Explicit `CORDIAL_RESOLUTION` / `CORDIAL_DPI_SCALE` still win, so this is a default, not a forced value.
- `HostWindow::set_fullscreen` marshals the GTK call onto the default main context with `MainContext::invoke`, because the caller is the engine's looper thread.

These are rules rather than tuned numbers: no display model, username length or window size is privileged.

## Verification

`cargo test --workspace` and the webview-feature run in `.github/workflows/test.yml` are the gate. Visual verification needs at least two monitors with different DPI and one long username; a single machine cannot prove the label rule.
