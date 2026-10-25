# Release engineering

## Release contract

Publish stable tags as `vMAJOR.MINOR.PATCH`, matching `Cargo.toml` and the embedded Windows product version. The application updater accepts only a newer numeric stable version from the official repository. Drafts and prereleases never enter that channel.

Each public release should provide:

- `WARDOG-Companion-VERSION-Setup.exe`: per-user Windows x64 installer.
- `WARDOG-Companion-VERSION-windows-x64.zip`: portable folder.
- `SHA256SUMS.txt`: checksums for both downloadable files.

The installer name is a compatibility contract with `src/app_updates.rs`. GitHub must expose the asset's `sha256:` digest; updates fail closed if it is absent. Do not replace a published asset in place. Fix an issue in a new version and tag.

## Build and validate

Use a clean Windows checkout. Keep the bundled catalogue and pinned fixtures under review. Verify formatting, Clippy, Rust tests and Python importer tests as described in CONTRIBUTING.md. Run `cargo build --release --locked`.

Install Inno Setup **7 or later** from its official distribution. The branded installer uses its modern dark wizard. To regenerate original artwork, run `tools/build_icon.ps1` and `tools/build_installer_art.ps1`; committed assets allow ordinary builds without these tools.

Package using a fresh output directory:

```powershell
.\tools\package_release.ps1 -ISCC 'C:\Path\To\Inno Setup 7\ISCC.exe'
```

The script checks embedded product version, builds the portable folder and its offline model audit, creates the installer, then writes asset checksums. `dist/` is ignored by Git. Pass `-OutputDirectory` to choose a fresh location. Packaging without `-ISCC` produces only the portable ZIP, as CI does.

## Installation behavior

Setup uses a stable AppId and defaults to `%LOCALAPPDATA%\Programs\WARDOG Companion`. It runs per user without requesting administrator rights. It creates a Start menu entry, an optional desktop shortcut and a standard uninstaller. The native executable includes an icon and product version information.

Setup can ask to close a running companion before replacing its executable. It does not force-close unrelated apps. Launching the installed app is offered at completion. Existing external `Data/catalog.json` is installed only if absent and is retained on uninstall. Setup does not touch `%LOCALAPPDATA%\WardogCompanion\profile.json`.

The app updater checks metadata on demand, displays release notes, downloads into `%LOCALAPPDATA%\WardogCompanion\Updates\VERSION`, verifies size and SHA-256, and rechecks the file when the player chooses **Install & close app**. The regular installer opens and the companion exits. Setup manages the installed location. A portable copy is not overwritten; players should close it and use the installed shortcut afterward.

## Required manual checks

Use an isolated installation directory and profile. Check the branded wizard, clean install, executable metadata, shortcuts, upgrade over existing files, profile/external-data retention and uninstall. Check current-version behavior against the published release API. Regression tests cover numeric versions, release/asset restrictions, corrupt/truncated downloads, failed HTTP responses and cached-file revalidation.

A production “newer version” upgrade can only be completed when a newer stable release exists. Local fixtures exercise that branch without publishing a fake release. Record that distinction in VALIDATION.md. Do not claim a test of a future public release.

Before publishing, review staged paths for personal data, credentials, raw captures and unrelated research. Create a draft release at the reviewed commit, upload all assets, verify their GitHub digests against local hashes, and publish once checks pass. Confirm the repository, README images, download URLs and latest-release update check afterward.

## Signing

Version 1.0 is unsigned. No signing keys are stored in this repository or workflow. To add Authenticode signing later, use a trusted certificate/signing service, protect credentials outside source control, sign executable and installer before generating hashes, and timestamp signatures. Do not use a self-signed certificate to imply public publisher trust.

## Rollback

Game-data rollback is available in the app. App binary rollback is a separate manual operation using an earlier release; automatic rollback and background replacement are not implemented. Retain a profile backup before trying an older application schema. Never overwrite or retag published releases to roll back.

The interactive app holds a session-local named mutex. Uninstall checks the same mutex and refuses to proceed while the companion is running. Diagnostic CLI modes do not acquire it. Close the app before uninstalling; a hidden panel still counts as running.

Branding uses WARDOGS Companion from 1.0.1 onward. The crate/executable name, existing installation/profile directories, AppId, mutex, repository URL and WARDOG-Companion release asset prefix remain stable so 1.0.0 can discover and install the update. Setup migrates the original generated shortcuts to the corrected display name.
