# Dataset update workflow

For normal use, open **Data & updates** in the app, check for updates, review the candidate, then install. **Restore previous data** rolls back the last dataset. Failed checks and invalid files preserve active data. The instructions below are for source maintainers.

Run from the source checkout with Python 3. No downloaded repository code is executed. Network access is required only for download/supplement refresh. The Windows app runs offline.

## Review a pinned candidate

1. Identify the exact 40-character wardogs-data revision you intend to review.
2. Run `python tools/update_data.py --download --revision <commit>`.
3. Inspect `data/update-report.json`: source builds, changed/removed weapons and attachments, counts and coverage notes. Downloads live under `research/staged-<commit>` and do not replace the active catalogue.
4. Run `python tools/update_data.py --source-dir research/staged-<commit> --revision <commit> --apply` only after reviewing the candidate. The output catalogue is written atomically to `data/catalog.json`.
5. Run `python tools/test_update_data.py`, `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --release --locked`, and `cargo build --release --locked`. Review source coverage and changes in the native UI before publishing.
6. Ship the validated `data/catalog.json` as `Data/catalog.json` beside the executable, together with the updated source report/license. Keep the previous payload for rollback. The user can choose Reload dataset; invalid files leave active data intact.

A new dataset schema or modifier operation can require code changes; a JSON replacement cannot add unsupported semantics. Default source revision is pinned. Download checksums, matching builds and arithmetic agreement verify the pipeline, not live server overrides.

## Supplemental mounts

`python tools/refresh_compatibility.py` cross-checks publicly published per-weapon muzzle lists against the current pinned source records. This is optional and separate from core import. The current audit is `research/muzzle-compatibility-all.json`. Refresh is rate-limited/best effort; missing endpoints, 429 responses, ambiguous IDs and mismatched modifiers are recorded and excluded. Respect service limits and review gaps instead of inferring compatibility from caliber.

External supplement mappings are disabled for a different wardogs-data revision until revalidated. Core direct-slot compatibility still imports from the new source. The full attachment dictionary remains searchable even when fit or calculations are unknown.

## Validation behavior

Corrupted original downloads, mixed game builds, duplicate IDs and inconsistent published DPS stop generation. Check-only mode never replaces the active catalogue. Runtime reload checks structure, IDs, builds, finite values, source-manifest shape and compatibility references; it does not authenticate an arbitrary JSON payload. The update tool verifies original file hashes before deriving that payload.

Preview 0.9: use --retry-gaps with tools/refresh_compatibility.py to revisit unresolved named joins sequentially. Duplicate names require an exact weapon-specific source tag. Unsupported operations remain coverage limitations. After rebuilding, run wardog-companion.exe --audit-output <path> to export the bundled dataset model audit. This does not read the player profile or validate the running game.


## In-app updates (Preview 0.10)

Use Settings → Data & updates → Check for updates. A background Rust worker fetches repository HEAD and its pinned manifest; identical source-file hashes report up-to-date without changing the dataset. Otherwise it downloads all nine files, checks their SHA-256 hashes, schemas, joins, duplicate IDs and published DPS, then validates the generated catalogue. The summary includes changed/new and removed weapon counts, attachment counts and the removal of unreviewed community muzzle links. Install reviewed update writes Data/catalog.json and preserves the previous catalogue as Data/catalog.json.previous. Restore previous data validates that backup before replacing active data. Both operations require the selected weapon to exist in the candidate. Unknown or removed selected parts remain explicitly unavailable, rather than being fabricated; use Customize to replace them or restore previous data. A write failure preserves the active file. Keep the extracted app folder writable.

The runtime importer does not carry the community website supplement across new source revisions. Maintainers can still use the existing Python pipeline and revalidate that supplement before distributing a fuller payload. The application runtime itself does not require Python.

For OCR diagnostics, --ocr-image <absolute image path> prints recognized text without capturing a screen; --recognize-image <absolute image path> opens the normal confirmation view for a supplied image. These are explicit diagnostic arguments, not startup scanning. Normal recognition captures only a foreground WardogsClient-Win64-Shipping window, uses local OCR and asks for confirmation.

## Personal patch impact (Preview 0.11)

What changed for me? previews an available candidate before installation and compares active data with Data/catalog.json.previous after installation. Its scope follows the currently selected weapon and parts. It recalculates when relevant profile/setup constraints change. Restore previous data compares the restored snapshot against the previously active data during the current session; on restart the unchanged saved backup may equal the restored data, correctly producing no modeled difference. Manual Reload dataset keeps an in-memory baseline only when data actually changes. Identical reloads do not erase an existing comparison. An unavailable or invalid backup is reported; no history is inferred from the version number alone. No per-weapon preference archive or automatic part replacement is introduced.



## Optional compatibility research inputs

Ordinary builds and tests use the committed reviewed mapping and pinned licensed fixtures. The website refresher additionally expects local `research/public-catalog.json` and `research/catalog.js`; those third-party research inputs are not redistributed in this source repository. Revalidate any independently obtained inputs before refreshing supplemental mappings. The primary in-app data update path does not require them.
