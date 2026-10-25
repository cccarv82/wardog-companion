# WARDOG Companion 1.0.0

The first public open-source release: a focused Windows companion for source-backed weapon and attachment decisions.

## Included

- Native Rust + Iced desktop app with an English interface, pinned panel and global hide/show shortcut.
- Eligible loadout suggestions for Control, Balanced and Agility; class/range recommendations and supported ideal TTK comparisons.
- Spending alternatives, remaining purchase/unlock costs and useful next unlocks.
- Personal dataset-impact reports, validated game-data updates and data rollback.
- Branded per-user Windows installer, Start menu shortcut, optional desktop shortcut and standard uninstaller.
- On-demand app updates: review release notes, download a size/SHA-256-verified installer, then explicitly choose when to install.
- Original W/shield icon and embedded Windows version information.
- MIT-licensed application source, contribution guide, issue/PR templates and Windows CI.

- Single-instance protection avoids competing copies and shortcuts; Setup/Uninstall require the companion to be closed.

## Choose your download

- **WARDOG-Companion-1.0.0-Setup.exe** — recommended installation for your Windows user; no administrator rights required.
- **WARDOG-Companion-1.0.0-windows-x64.zip** — portable; extract the entire folder before opening the app.
- **SHA256SUMS.txt** — download integrity checksums.

Close previous companion copies before installing or opening this release. Existing local profiles are preserved. App updates and game-data updates are separate. A portable copy is not overwritten by the installer.

## Release boundaries

Version 1.0 is not digitally code-signed. Windows may display an unknown-publisher/SmartScreen notice; never disable security features to run it. SHA-256 verification is an integrity check, not publisher signing.

OCR is experimental and requires confirmation. Recommendations use pinned community game-file data; ideal TTK and handling scores are estimates, not measured in-game performance. Unknown or unsupported combinations remain excluded. See README.md, MODELING_LIMITS.md and VALIDATION.md.
