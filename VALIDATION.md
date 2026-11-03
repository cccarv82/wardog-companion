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

The final lifecycle fix passed formatting, Clippy, **51 Rust release tests** and a release build. The app now holds a session-local single-instance mutex; its dedicated test confirms duplicate rejection and lock release. Setup/Uninstall checks that mutex. A silent uninstall attempted while the isolated companion was running exited with code 1 and left it intact. After closing the companion normally, uninstall exited successfully, removed its executable and Start menu shortcuts, retained external data, and left the real player profile unchanged.

The first public GitHub Windows CI run passed from the committed source, including all then-current tests, importer checks, release build, portable packaging and artifact upload. The minimal source fixtures also passed importer tests from a clean Git archive. No fake newer public release was created to test upgrades; stable-version and download branches use local fixtures. The final public release assets and production current-version response are checked after publication.

## Branding patch — 1.0.1

2026-10-25: corrected visible branding to WARDOGS Companion. Formatting, Clippy with warnings denied, all 51 Rust release tests, four Python importer tests and the release build passed. Native screenshots were refreshed for the corrected app title/header, spending screen and installer artwork.

A real 1.0.0 installer was installed into an isolated QA directory, then upgraded with the 1.0.1 installer. Windows product metadata reported WARDOGS Companion 1.0.1, the original generated Start menu shortcut was removed, and its corrected replacement existed. The existing external catalogue retained its checksum; the real player's profile checksum was unchanged. The QA installation was uninstalled successfully afterward.

The executable/crate name, profile directory, installation directory, AppId, mutex, GitHub repository URL and WARDOG-Companion download-asset prefix remain stable intentionally. This lets the unmodified 1.0.0 updater locate the newer installer. No recommendation-model or game-data changes were made.

## Contextual recommendations — 1.1.0

2026-11-02: rebuilt automatic suggestions around weapon role and engagement range. Added source-zoom sight bands, automatic capacity floors, six-dimensional handling priorities with source recoil-axis balance, conservative ordinary-multiplier/isolated-additive support, and range propagation into class/range builds. Capability-aware scope unlocks can be useful even when they increase aim time. See docs/RECOMMENDATIONS.md for exact policies and assumptions.

All 59 Rust release tests passed, including an exhaustive 38-weapon × 4-range × 3-goal matrix and targeted regressions for Galil, AK-74, M4, AMP-9, PP-19 Vityaz and AMR 50. Four importer tests passed. The audit exports 1,620 ranking scenarios plus 1,824 setup scenarios over two progression profiles, four range choices, three goals and zero/$2000 budgets. Every automatic selected sight/magazine satisfies its context and every selected item remains eligible, supported and inside budget. This is model validation, not in-game measurement.

With all levels at 100 and a $2000 Control budget, examples include a 2.5x Combat Optic and a 30-round magazine for M4; a reflex and 30-round magazine for Vityaz; and a 6-10x Scope MOA with the 5-round magazine for AMR 50. These examples are constrained results, not universal prescriptions.

Native UI checks used a separate synthetic legacy profile, initially saving AMR 50 with Kobra Reflex. Opening 1.1.0 left the profile byte-for-byte unchanged (SHA-256 BCFE26F2225AB5FF9027C693EB6F26E8371838218ADDDEE5AB910C6CD41A9BF3) and offered Update setup. Clicking it selected the precision optic and magazine and created an original-profile backup with the same hash. Keep preserved the scope when switching to Close and a contextual warning appeared. Releasing Keep and requesting a suggestion selected a close-range sight. The saved range/model fields were checked. A separate AK profile showed the updated rifle recommendation and spending tiers with the same sight/capacity constraints. Screenshots were refreshed.

The player's installed 1.0.1 app was temporarily closed for the single-instance UI test and reopened afterward; test edits used WARDOG_PROFILE_PATH in the workspace. The primary dataset, updater endpoint, installer identity and real profile path are unchanged. Saved builds are not replaced at startup. The historical setup remains until an explicit recommendation action, and older profiles receive a versioned backup on first save of new fields.

## 1.1.1 missing-price regression checks

61 Rust release tests and four Python importer tests pass. Formatting and Clippy with warnings denied pass. Regression tests cover the player's Vityaz Medium scenario, all-price-missing selections at zero known-cost budget, unknown-price Keep/preferred selections, known-subtotal limits and complete-price spending tiers. The exported setup audit retains unknown price names and labels its monetary subtotal as known_cost. No new in-game measurements or native installer interaction are claimed for this patch.

## 1.2.0 Click assist

64 Rust release tests cover click-cycle press/release spacing, immediate release on stop, fresh-key activation, toggle edges, disarming, focus/target rejection, Esc and attempt timeouts, alongside all existing recommendation regressions. Formatting and Clippy with warnings denied pass. No automatic clicks were sent during development tests. Actual WARDOGS menu acceptance, input timing and anti-cheat compatibility are not validated by these tests; use the requested interval control to test on the game's menu. The regular offline model audit remains separate from native input behavior.

## 1.2.1 click status and count

65 Rust release tests and four importer tests pass. Formatting and Clippy pass. The new regression simulates timer observations and completed button pairs, verifies that ticks are not counted as clicks, and checks attempt reset/last-count retention. No synthetic clicks were sent during validation; game acceptance and native input timing remain unverified.
