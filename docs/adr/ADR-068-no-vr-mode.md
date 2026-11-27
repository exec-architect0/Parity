# ADR-068: No VR mode

**Status:** accepted
**Supersedes:** [ADR-053](ADR-053-vr-is-a-mode-of-the-android-runtime.md)

## Context

Cordial's VR mode ran the Quest (arm64) build through dynarmic and an OpenXR runtime. It needed Boost headers, a translator submodule, patched OpenXR headers, a second APK store and a settings page for runtime readiness. The feature was experimental, untested on real hardware, and had known gaps in typing, haptics and leaving games (`docs/vr.md:128-137`). For a fork focused on the desktop client it is most of the build complexity and none of the value.

## Decision

Parity does not build or expose VR.

- The `cordial-runtime/vr` feature is removed from CI, justfile, flake and package build recipes.
- Settings no longer adds the VR page, the launcher no longer registers `win.launch-vr`, and `--import-quest-apk` is gone from the CLI.
- The Quest store, guest translator and OpenXR modules remain in the tree for now but are unreachable; they can be deleted in a later cleanup without changing behaviour.

This supersedes ADR-053's decision that VR is a mode of the Android runtime. The runtime code is still Android; its second, emulated arm64 mode is simply not offered.

## Verification

`cargo test --workspace` and the webview-feature run in `.github/workflows/test.yml` are the gate. A successful build without Boost proves the feature is no longer pulled in.
