# Unofficial-client detection vectors found in code

This is a record of signals with **code evidence**, not a list of theories. "Server" means the signal is sent to a Roblox/Google endpoint or the client contains a server-authored verdict string for it. "Local" means the client checks it without an observed network transmission. Confidence is about the evidence, not about whether a given ban wave used the signal.

Read `docs/analysis/macos-runtime.md` and `docs/HANDOVER.md` for the public-policy and investigation history behind several entries.

## Signals with code evidence

| Signal | Evidence | Scope | Confidence |
|---|---|---|---|
| Google Play Integrity / Standard & Express integrity tokens | APK dex strings: `IntegrityTokenRequest`, `StandardIntegrityTokenRequest`, `requestIntegrityToken`, `warmUpIntegrityToken`, `getPlayIntegrityProjectId`, `getPlayIntegrityUrl`, `PlayIntegrityApiData`, `BIND_INTEGRITY_SERVICE`, `BIND_EXPRESS_INTEGRITY_SERVICE`, `IntegrityTokenResponse`. Native bridge exports in `libroblox.so`: `Java_com_roblox_engine_jni_meta_NativeMetaInterface_getDeviceAttestationToken`, `Java_com_roblox_universalapp_account_JNIAccountProtocol_getGetIntegrityTokenMethodName`, `getDeviceIntegrityAvailableMethodName`. Live waydroid trace records token requests and Finsky attestation records (`docs/traces/waydroid-roblox-startup.log.gz:429,437,853,1080,1083,1202,1227,1409-1410`; `docs/HANDOVER.md:633-638`). Token minted by Google; Roblox verifies server-side. | Server | High that it exists and that Cordial cannot mint it (Java/Play Store layer is not run). |
| Remote attestation disconnect reasons | `libroblox.so`: `DisconnectRemoteAttestationFailureGeneral`, `DisconnectRemoteAttestationFailureBootValidation`, `DisconnectRemoteAttestationFailureOsOutOfDate`, `DisconnectRemoteAttestationFailureTimeout`, `DisconnectRemoteAttestationFailureUnsupported`, `DisconnectRemoteAttestationGeneralFailure`, `DisconnectRemoteAttestationOSOutOfDate`, `DisconnectRemoteAttestationTimeout`, `DisconnectRemoteAttestationUnsupported`, `RemoteAttestationDebugLog`, `QuestDeviceAttestationTimeoutMs`. These are server-authored disconnect reasons the client maps. | Server -> client | High that an attestation outcome can end a session. |
| Android emulator kick | `libroblox.so`: `AndroidEmulatorKick`, `DisconnectAndroidEmulatorKick`. Public policy recorded at `docs/analysis/macos-runtime.md:272-276` (“players detected on an emulator are kicked”). | Server -> client | High that the verdict exists; detection mechanism unknown. |
| Player integrity / `IntegrityCheckedProcessor` | `libroblox.so`: `IntegrityCheckedProcessor`, `[DFLog::IntegrityCheckedProcessor] onRemoteEventFromClient: player integrity for processor: {} userId: {} is untrusted, ignoring entry`, `player trusted, direct add`, `player integrity ... is unknown, allowing due to override`, `IntegrityStatus`, `ClientIntegrity`, `kIntegrityCheckedProcessorAllCounters`. Server sends per-player integrity state; client gates entries from untrusted players. | Server -> client | High that the trust system is real; not proven to be the unofficial-client verdict. |
| Sign-in bot challenge (PerimeterX/HUMAN) | A fresh password login returned HTTP 403 with body “Incorrect username or password” while the real verdict was challenge-required; the challenge page sent `challengePageLoaded`/`Parsed`/`Initialized`/`Displayed` and no integrity token (`docs/adr/ADR-066-sign-in-starts-inside-the-client.md:8,31-33`). Live getIntegrityToken requests are recorded in the waydroid trace. | Server | High as a server challenge; the webkit login path obtains browser-issued trust instead. |
| `RBXEventTrackerV2` / BrowserTrackerId device cookie | Endpoint `/browser-tracker-api/device/initialize` in both `libroblox.so` and APK dex; engine string `BrowserTrackerIdRequest: No RBXEventTrackerV2 in cookie.` (`docs/analysis/flag-init.md:1319-1336`, `webview-surface.md:255-260`). Minted in WebViewCookieHandler during a real browser sign-in; standalone GET is 404 and POST is 500 (`flag-init.md:1354-1386`). | Server (cookie) | High that the check and endpoint exist; low as a discriminator — mocktail logs failure and still plays (`flag-init.md:1630-1633`). |
| User-Agent / platform strings | Engine builds a Roblox User-Agent and a `getPlatformName()` answer (`native/init_params.cpp:149-169,353-486`; `docs/analysis/platform-identity.md`). roblox.com branches page behavior on UA (`webview-surface.md:556-571`). | Server (HTTP header) | High that the server sees it; no evidence it is a ban signal. |
| WebAuthn / passkey attestation | APK dex: `AuthenticatorAttestationResponse`, `attestationObject`, `setAttestationChallenge`, WebAuthn attestation JSON keys. WebKitGTK in Cordial has no WebAuthn (`webview-surface.md:419-508`). | Server | High that the API exists; unknown whether absence is scored as suspicious. |
| Persona device-integrity verification | APK dex: `/api/internal/verify/v1/current-inquiry-session/verify-device-integrity`, `VerifyDeviceIntegrityRequest`, `PlayIntegrityHelper`, `verifyDeviceIntegrity`. | Server | Real endpoint, but tied to identity verification, not ordinary gameplay. |

## Ruled out by code or measurement

- **App signature / installer source**: engine never asks for `getPackageCodePath`, `sourceDir`, `getApplicationInfo`, or a real `getInstallerPackageName` over JNI (`docs/HANDOVER.md:1032-1037`; `crates/cordial-runtime/src/unimplemented.rs:104-109,264-277`). Cordial's own signature pin only verifies Roblox's APK before it is loaded.
- **Flag signature**: `signatureSize: 0` on both Cordial and Sober; no gate on a signature Cordial fails to supply (`docs/analysis/flag-init.md:2002-2013`).
- **304 disconnect**: explicitly retracted as a load-time integrity check (`docs/HANDOVER.md:735-743,792-982`).
- **`onFlagsFailed`**: a local engine gate, not a server verdict (`docs/analysis/flag-init.md:676-717,2184-2207`).
- **Cordial's “Official build” stamp**: a report hint; a fork can set it and nothing behaves differently (`docs/doctor.md:30`).
- **`x-csrf-token`**: appears only in a log-redaction unit test (`crates/cordial-shell/src/log_export/tests.rs:25`).
- **`DEVICE_ID`**: only flag names and unrelated Vulkan local fields; no observed identifier sent.
- **MangoHud/vkBasalt injection**: known risk in Sober discussions (`ADR-041:105-111`), not an observed Cordial signal.

## Limits

No server code was read. Several entries are server-authored *verdict strings* in the client, which prove the verdict exists but not which signals caused it. Attestation token contents and server weighting remain unobserved. Any claim that a specific signal caused a specific ban needs a throwaway-account control run.
