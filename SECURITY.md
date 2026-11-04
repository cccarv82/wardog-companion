# Security policy

## Supported releases

Security fixes target the latest stable release. Older portable copies are not automatically patched; use Settings → App updates or the official GitHub Releases page.

## Report a vulnerability privately

Use this repository's **Security → Report a vulnerability** feature:

https://github.com/cccarv82/wardog-companion/security/advisories/new

Include the affected version, reproduction steps, impact and a minimal proof of concept. Do not include secrets, personal profiles or unredacted player screenshots. Do not open a public exploit report while a fix is being coordinated. Response times are best effort; no service-level commitment or bounty is offered.

## Trust boundaries

- Update metadata and installers come from the fixed official GitHub repository over HTTPS. The app checks installer size and GitHub's SHA-256 digest before staging and again before launch. Failed downloads do not replace the running executable.
- The 1.0 installer is **not code-signed**. SHA-256 is an integrity check, not independent publisher authentication or a substitute for Authenticode signing.
- Game-data downloads use a separate manifest/checksum and schema-validation pipeline. Arbitrary user-edited derived JSON is structurally validated, not cryptographically authenticated.
- The application does not accept an arbitrary update feed or run downloaded repository scripts. Installers execute only after an explicit user action.
- OCR is local and experimental. Captures and profiles should not be attached to reports without reviewing their contents.

Do not disable Windows security features to install the companion. If a download cannot be verified, stop and report the failure.

Click assist is opt-in per session and sends standard Windows left-button input only while the foreground process basename matches `WardogsClient-Win64-Shipping.exe`. Foreground process metadata is checked using limited query access; no game memory is read. The basename check is a scope restriction, not cryptographic executable identity. Focus can change between a foreground check and input dispatch, so this is not an atomic isolation guarantee. F8 toggles, Esc stops, cursor movement and focus changes stop, with no automatic time limit. Normal shutdown attempts to release any pending synthetic press. The app does not bypass input rejection or request administrator rights.
