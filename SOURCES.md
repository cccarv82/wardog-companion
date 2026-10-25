# Data sources — 1.0.0

## Primary pinned source

Repository: https://github.com/warpuppies/wardogs-data
Revision: e4ca3e9f1c48d980ba531202642884c358208304
Declared game build: 25937608
Import date: 2026-10-25
License: Unlicense; included as WARDOGS-DATA-LICENSE.txt. Game names/numbers remain the publisher's property. No game binaries, icons or textures are distributed.

The import checks nine files against the source manifest: items.json, weapons.json, attachments.json, ammo.json, damage.json, equipment.json, tags.json, csv/weapon-compatibility.csv and csv/weapon-damage.csv. JSON schemas/builds, case-insensitive IDs, joins and published DPS arithmetic are validated. The generated catalogue contains all **40 weapon records and 298 attachment records** in that revision. Two weapons are explicitly test/vehicle records and never enter rankings. Missing fields stay unknown; a matching checksum validates the downloaded bytes, not live server values.

The original game installation was inspected read-only and its Steam build read as 25937608 on 2026-10-24. The app labels that as a past check, not automatic current-build detection. It does not decrypt containers, read game memory or alter installation files.

## Compatibility and attachment scope

Direct slot compatibility comes from the source CSV, including named/default/hidden components. An item's compatible slots are kept per weapon; IDs are compared case-insensitively. Global attachment records remain inspectable even when no compatible weapon or unlock is established. The common four slots stay on the main panel; further explicitly linked barrel, stock, pistol-grip and handguard components are available in Customize. Chamber/ammunition is not treated as an attachment slot.

The source does not publish every nested barrel mount. Caliber alone is not used to infer muzzle fits. Separately, the public site JavaScript at https://wardogsweapons.com/catalog.js exposes per-weapon attachment URLs such as https://wardogsweapons.com/api/weapon/ak74/attachments/MUZZLE . Public muzzle lists were joined to unique source items and cross-checked for the four modeled weapon modifier attributes and their operation. The retained supplemental audit has **120 verified weapon–muzzle links across 24 weapons**, with 15 remaining excluded entries plus operation limitations. Missing endpoints, ambiguous IDs and modifier/operation disagreements are listed in muzzle-compatibility-all.json and selected-weapon coverage notes. The website does not declare a source game build, so these are separate community compatibility evidence, not official/runtime validation. Partial rate-limited requests were retried sequentially with pauses; unavailable requests are not fabricated.

Conditional bipod records are searchable and explicitly labeled. The automatic model does not assume deployment. Unknown prices/unlocks, unsupported modeled operations and invalid mounts do not become valid/free upgrades. The supported compound attachment factors multiply together; other effects remain visible as raw source descriptions and are outside the six-factor estimate. Automatic suggestions search supported, priced combinations under the user's constraints, with an explicit search cap; reaching the cap reports failure rather than silently claiming an optimum.

## Weapon comparison methodology

The CSV provides base damage per projectile hit and raw DPS before falloff, armor and body multipliers. Each ranking baseline requires exactly one weapon caliber and the corresponding unsuffixed source projectile. Multi-caliber, explosive, bow, shotgun and test records do not enter ideal raw TTK rankings; their published damage rows remain inspectable. Missing RPM prevents a TTK/DPS estimate. Raw DPS is recomputed as damage × RPM / 60, avoiding CSV rounding in ranking.

For a hypothetical test target (100, 150 or 200 HP): shots = ceil(HP / raw damage); ideal raw TTK = (shots − 1) × 60 / RPM. First hit occurs at time zero. This excludes miss probability, armor, body multipliers, ammo-type rules, burst/trigger gaps, windup, reloads, projectile travel time and unmodeled effects. The UI says raw and ideal; no actual in-game kill time is claimed. Source damage-rule application order is partly game-code-defined, so the app does not invent an armor/headshot model from incomplete semantics.

Recoil index is mean vector magnitude sqrt(x²+y²) over the first ten published pattern shots, requiring at least ten shots. The source does not publish a unit. This is a base-pattern comparison within class, not validated overall stability, camera response, spread, sway or ease of control. Lowest recoil sorts this index. Balanced averages within-class ordinal positions for ideal TTK and recoil equally, including tie positions. It is an application preference, not a physical/game formula. Test records are excluded; locked/unknown weapon unlocks are labeled and optionally filtered. The source dictionary defines null unlock as no unlock required or absent from the unlock tree. Non-developer items with null unlock are level-eligible; ownership is still separate.

## Updating and runtime validation

The source tool stages pinned downloads, verifies file hashes/builds, generates a change report and replaces the catalogue only with --apply. An actual download/check-only run was completed without replacing the active payload. Corrupt download, mixed-build and duplicate-ID tests stop generation. External supplement mappings are disabled for an unreviewed new source revision.

The app loads a bundled catalogue or a structurally validated Data/catalog.json. Runtime validation checks schema, build consistency, IDs, finite values, operations, source-manifest shape and compatibility references. It is not a signature or re-verification of the original raw source files. Invalid reloads preserve the active data. See UPDATE_GUIDE.md and update-report.json.

## Functional validation

The user reported earlier in-game AK attachment/overlay/shortcut checks passed and supplied matching levels 49 / 11 / 12 / 5 / 14. Numerical stats were not shown in the game UI. That report confirms the described functional checks, not the numerical model, official anti-cheat acceptance or measured match performance. Expansion to all weapons and class rankings has desktop/automated validation, not a new in-game measurement campaign.

## Preview 0.9 additions

Distance uses source damageFalloff keys. Linear and constant segments are supported, as are exact keys; cubic interiors are unknown because tangents are not published. A missing curve is accepted only at the raw 0 m baseline. No body/armor model or actual game HP is assumed. Build comparisons use eligible, priced, supported attachments within shared constraints, with no current setup preferences. Built recoil applies each axis factor before vector magnitude; selected handling effects do not change source damage/RPM. Equal model scores share competition rank.

The six resolved supplement joins use an explicit source name alias or weapon-specific attachment tags, never caliber inference. TopComp fits are retained but its noncompound multiply operation remains outside automatic suggestions; the audit records that limitation.

Class checkboxes record user observations for six representative weapons, keyed by build and revision. Pending new-class game checks remain pending. The automated ranking audit covers 1,620 scenarios and must not be interpreted as measured game behavior.

## Preview 0.10

The Windows app now has its own Rust importer and on-demand HTTPS updater. It pins the current repository HEAD, compares manifest hashes to avoid updates for documentation-only commits, validates all nine source files against that pinned manifest, imports direct compatibility and validates the generated catalogue before offering installation. Requests go only to api.github.com and raw.githubusercontent.com for this fixed repository; redirects are disabled. No executable source content is downloaded or executed. Data/profile corruption, network errors and unsupported source schemas preserve the active snapshot. Manifest checks establish byte consistency with that source, not publisher verification. New source revisions intentionally omit the unreviewed supplemental muzzle mapping; the review summary warns of that loss of coverage.

Range recommendations use eligible built rankings in the selected class at fixed reference distances (25/100/300 m) and hypothetical 100 HP. Handling goals are Agility/Balanced/Control; model ordering equally weights ideal TTK and per-shot recoil. It is not an engagement simulation, range-specific optic selector or measured accuracy estimate.

Completion costs deduplicate selected IDs, use only the exact-item vendor tag and count published unlock fees unless marked paid or owned/default. Owned does not mean that the game inventory has been detected. Weapon and unchanged equipment prices are excluded; incomplete information remains labeled incomplete.

OCR uses Windows.Media.Ocr.OcrEngine and SoftwareBitmap, as documented by Microsoft: https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr.ocrengine.recognizeasync . It reads only on explicit request and confirms whole-name matches against non-test catalogue weapons. The supplied AK arsenal image was recognized locally. This is not evidence of reliable capture in every live game mode or resolution.

## Preview 0.11 modeling

Spending alternatives use the same exhaustive capped compatible-part search as suggestions, retaining a nondominated cost/handling-score frontier. A setup is removed if another costs no more and scores at least as well. This preserves the existing optimum while exposing cheaper alternatives. The 70% and 95% gain thresholds use the difference between cheapest feasible and maximum feasible score under current constraints. Scores are weighted modeled recoil/aim-time preferences. They are not estimated percentages of real player performance. The search cap still rejects incomplete searches instead of claiming an optimum.

Patch impact compares parsed source fields between the active and previous/available datasets under identical current profile constraints. It distinguishes selected attachment effects and prices from missing mounts/coverage, reports unknown comparisons, and compares algorithm suggestions without changing player selections. Community source changes are not proof of a live server balance update. No update was fabricated or installed for release validation; unit tests use synthetic changes to verify positive and negative reporting paths.
