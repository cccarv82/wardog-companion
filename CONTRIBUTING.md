# Contributing to WARDOG Companion

Thank you for helping make loadout decisions easier. We welcome focused fixes, better explanations, validated data corrections and thoughtful improvements to the Windows experience.

## Before you start

- Search existing issues and PRs. For a larger feature or modeling change, open an issue first with the player problem and the smallest useful solution.
- Read the [README](README.md), [modeling limits](MODELING_LIMITS.md) and [source notes](SOURCES.md). Unknown data must stay unknown.
- Keep all interface text and project documentation in English.
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md). Security reports follow [SECURITY.md](SECURITY.md).

## Set up a development environment

Use Windows x64, Rust/MSVC, Visual Studio C++ build tools and a Windows SDK. Python 3 is needed for importer tests. The app itself does not require Python. Installer work requires Inno Setup 7.

1. Fork this repository on GitHub.
2. Clone your fork and add the original repository as `upstream`.
3. Create a descriptive branch from the latest `upstream/main`.

```powershell
git clone https://github.com/YOUR-USERNAME/wardog-companion.git
cd wardog-companion
git remote add upstream https://github.com/cccarv82/wardog-companion.git
git fetch upstream
git switch -c fix/describe-the-change upstream/main
New-Item -ItemType Directory -Force .local | Out-Null
cargo build --release --locked
```

Use an isolated profile when testing UI changes:

```powershell
$env:WARDOG_PROFILE_PATH = "$PWD\.local\development-profile.json"
$env:WARDOG_DATA_PATH = "$PWD\.local\development-catalog.json"
Copy-Item data\catalog.json $env:WARDOG_DATA_PATH
.\target\release\wardog-companion.exe
```

These environment variables affect that shell session and its child processes. Close other companion instances before testing global shortcuts. Never commit your player profile, capture images, credentials or local tooling.

## Project map

| Area | Responsibility |
| --- | --- |
| `src/main.rs` | Iced application, views and user actions |
| `src/lab.rs`, `engine.rs`, `spending.rs` | Eligible setups, handling model and spending alternatives |
| `src/ranking.rs`, `range.rs` | Class comparisons, explicit scenarios and range recommendations |
| `src/catalog.rs`, `reference.rs` | Catalogue validation and source-backed records |
| `src/updates.rs`, `patch.rs` | Game-data updates, rollback and personal impact |
| `src/app_updates.rs` | Stable app releases, bounded downloads and installer verification |
| `src/storage.rs`, `hotkey.rs`, `recognition.rs` | Profile persistence, shortcuts and experimental OCR |
| `tools/` | Data maintenance, tests, artwork and release packaging |
| `data/`, `research/wardogs-data/` | Derived catalogue and minimal licensed pinned test fixtures |
| `installer/`, `assets/` | Windows Setup, original artwork and embedded icons |

## Design principles

**KISS is a product requirement.** The player is using the game, not studying a dashboard. Lead with one actionable decision. Put method, caveats and supporting details in secondary views. Preserve the compact play experience.

Use native Rust and the existing Iced components where practical. Avoid background services, startup network polling and new dependencies unless they solve a concrete problem. Keep blocking I/O outside the UI thread. Failures must preserve the current profile and active data.

Do not infer compatibility from caliber, turn missing values into zero, or describe model scores as measured game performance. Source-backed modeling changes should cite a pinned source and explain the effect on recommendations. Conditional mechanics need explicit support before entering automatic suggestions.

## Validate your change

```powershell
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --release --locked
python tools/test_update_data.py
cargo build --release --locked
```

Add regression coverage for changed behavior and failure paths; avoid tests that only restate the implementation. For UI changes, manually check the affected flow, resizing and compact mode. Include real screenshots when useful, with personal information removed. Explain what you did and did not validate.

Data changes require additional provenance, checksum/schema checks and coverage review; see [UPDATE_GUIDE.md](UPDATE_GUIDE.md). The optional website compatibility refresher expects local research inputs that are deliberately not redistributed; use the pinned reviewed mapping for ordinary builds and tests. Do not run `--apply` casually or refresh live data as part of an unrelated PR.

Installer/updater changes require a clean installation, upgrade over an existing installation, uninstall, profile retention and corrupt/interrupted-download checks. See [release engineering](docs/RELEASING.md).

## Submit a pull request

1. Keep the PR focused. Separate unrelated cleanup and generated-data changes.
2. Write a clear title and describe the player-visible problem, final behavior and relevant trade-offs.
3. Link the issue when there is one. Include validation results and limitations.
4. Update documentation when behavior, assumptions or setup requirements change.
5. Push your branch and open a PR against `cccarv82/wardog-companion:main`.
6. Respond to review and keep CI passing. A maintainer will review and merge accepted work.

Draft PRs are welcome for early feedback. Small, reviewable changes are easier to merge. A passing test suite is necessary; it does not guarantee a feature belongs in the product.

By contributing, you agree that your original contributions can be distributed under the [MIT license](LICENSE). Retain third-party licenses and attribution. There is no CLA requirement.
