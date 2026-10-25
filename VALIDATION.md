# Release validation — 1.0.0

2026-10-25 · Windows MSVC · Rust + Iced

Formatting and Clippy with warnings denied passed. All 46 Rust release tests and four Python importer tests passed. The catalogue contains 40 weapon records and 298 attachment records. No new runtime dependency was added.

## Native desktop checks

The final executable was exercised with an isolated profile and dataset. A fresh profile was configured through the interface with levels 49/11/12/5/14 and a $2000 budget. Settings and the selected setup survived closing and reopening with an identical profile checksum.

Spend smarter displayed distinct Economy and Sweet spot alternatives, collapsed the duplicate maximum tier, and applied the Economy setup correctly. Returning home now scrolls to the selected setup. Zero-gain wording is covered by a regression test.

Ctrl+Shift+Space hid and restored the panel with Notepad focused. Two older companion instances initially held the shortcut; closing them resolved the conflict. Close old copies before launching this release.

An invalid catalogue was rejected while the active setup remained available. A clearly labeled local QA simulation changed AK-74 damage and a selected foregrip price/modifier. The personal patch report displayed those differences without modifying the saved setup. The final executable showed the patch alert above the footer and prices with two decimal places. Restoring the previous dataset through the interface recovered the original real catalogue. The profile checksum remained unchanged. Included simulation screenshots are test evidence, not a published game patch.

The real update check completed with “Your data files are up to date.” No newly published dataset was installed during this session. Automated tests cover candidate installation, backup, rollback, invalid backup rejection, and connection failure preserving active data.

## Limits of this validation

The bundled ranking audit exercises 1,620 model scenarios. It verifies model invariants; it does not measure live-game damage, recoil or kill times. The user's earlier in-game functional checks are separate from this release's desktop checks. No new live-game numerical measurements were performed.

Weapon reading remains experimental local Windows OCR. Previous supplied-image checks do not establish reliable recognition across live game resolutions, languages or full-screen modes. Manual weapon selection remains available.

The earlier interrupted preview UI check was completed in this release. See MODELING_LIMITS.md for source coverage and calculation assumptions.

Icon update: added an original orange W/shield emblem, a seven-size Windows ICO (16–256 px), and an embedded window icon. Formatting and Clippy passed again after integration. The Windows executable resource was checked through native icon extraction. Gameplay code and data are unchanged; the preceding gameplay test results apply to the same logic.

## Public distribution and updater validation

The public source adds an opt-in application updater, separately from game-data updates, and a branded per-user Inno Setup 7 installer. Formatting, Clippy with warnings denied, all **50 Rust release tests**, four Python importer tests and the release build passed after these additions.

Updater tests cover numeric stable-version ordering, draft/prerelease rejection, fixed official asset URLs, missing digests, excessive sizes, duplicate assets, cached-file corruption, successful bounded download, truncated data, bad checksums and HTTP failures. Failed downloads leave no staged executable. Native review confirmed the update page and its pre-publication “no stable release” response without changing the profile.

An isolated clean installation and same-version upgrade both succeeded. Installed executable bytes matched the build. Start menu and uninstall shortcuts were created. A deliberately changed but valid external JSON file was retained on upgrade. The real existing profile's checksum remained unchanged. This is a same-version reinstall test, not a claim that a future public version has already been installed.

The installer uses original high-resolution artwork and a dark wizard. Native visual review found and corrected a clipped welcome heading. The 1.0 executable and installer are unsigned; no Authenticode certificate was available. SHA-256 checks do not substitute for publisher signing.
