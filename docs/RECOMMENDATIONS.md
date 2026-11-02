# Recommendation model — 1.1

The companion optimizes a **contextual preference score**, not real-world effectiveness or a universally best build. Identical parts can still win on several weapons when the source effects and constraints justify them. The app never randomizes a suggestion to create variety.

## Range and role

The main panel has one range choice: Auto, Close (25 m), Medium (100 m) or Long (300 m). Distances describe app scenarios, not validated limits of a sight or weapon. Auto chooses Close for SMGs, pistols and shotguns; Medium for assault rifles and LMGs; Long for DMRs and snipers. Other classes use Close as a conservative policy. Class/range comparisons pass their explicit distance into attachment selection, and applying a range pick retains that context.

Automatic sight bands use the published `Weapon.AdsZoomMultiplier` override:

| Context | Accepted source zoom setting |
| --- | --- |
| Close | 1–1.5 |
| Medium | 1.5–4 |
| Long, other classes | 3–12 |
| Long, snipers | 6–12 |

A band is a recommendation policy. This scalar is not guaranteed to be the real optical magnification or the full adjustable zoom range. The app does not infer a missing value from a product name. Unknown/multiple zoom overrides do not qualify automatically. A manually preferred or kept sight remains allowed and gets a contextual warning when outside the band.

A supported, eligible sight in the band is required whenever it can fit beside kept parts. Otherwise the sight is left unchanged and an explanation is shown; a reflex is not silently substituted for a long-range scope. Existing in-game sights are not detected. Optic quality, reticle visibility, eye relief, glint and alternate/variable zoom positions are not modeled.

## Magazine capacity

Automatic selection preserves the capacity of a unique published default magazine when that capacity is known. Without one, it uses explicit class-policy floors: assault rifles/SMGs 30, LMGs 50, DMRs 10, snipers 5, pistols 7 and shotguns 2. These fallback numbers are assumptions, not inferred game defaults.

A supported eligible magazine meeting the floor is selected when it fits after the required sight and kept parts. Otherwise the slot stays unchanged and capacity remains unconfirmed. This avoids shrinking an M4 to 20 rounds, a Vityaz to 10, or an M249 from its published 200-round default just to improve recoil/ADS numbers. Explicit minimums and kept magazines override the automatic floor; choosing a smaller magazine manually remains possible. The app explains the trade-off.

Capacity is a feasibility constraint, not an invented damage or sustained-DPS bonus. Reload time, ammunition supply and kills per magazine are not simulated.

## Handling priorities

The six handling dimensions are vertical recoil, horizontal recoil, aim time, spread, equip speed and aim sway. Reductions are beneficial except for equip-speed multipliers, where increases are beneficial.

Control/Balanced/Agility start with their existing three-factor base preferences, reserve weight for spread/equip/sway, and normalize after adjustments. Published first-ten-shot recoil-axis balance adjusts the two recoil priorities with bounded factors. DMRs and snipers reduce recoil priority and emphasize aim/sway/spread; Long increases spread/sway importance, while Close emphasizes aim/equip handling. Unknown recoil patterns use neutral axis weights. See `src/purpose.rs` for the exact deterministic policy.

These weights are product choices, not measured accuracy. They do not establish an ideal weapon for every map, player or engagement. Spending tiers compare handling scores **within the same contextual constraints**, retaining their 70%/95%/maximum policies. A newly unlocked suitable sight can be highlighted as a capability even when it slows aiming; that reason is shown separately from numerical handling gains.

## Supported modifier composition

- `multiplyCompound`: factors multiply together.
- One ordinary `multiply` per modeled attribute is supported as a scalar factor; several ordinary multipliers on the same attribute remain unsupported.
- An isolated additive aim-time effect is evaluated only with a known positive source base aim time and no other aim-time modifier. Mixed additive stacking is rejected instead of guessing order.
- Other handling overrides/operations and conditional effects remain excluded. Non-handling effects may be displayed without affecting the score.

## Existing profiles

Upgrading does not replace saved parts. A one-time **Update setup** action applies the new rules while respecting Keep, preferred sight, explicit capacity, levels, ownership and budget. Choosing a goal or range also requests a new suggestion. The original profile gets a `.json.v1.0.bak` backup before the new fields are first saved. Auto is the default range when loading an older profile.

## Missing prices

Missing purchase prices never exclude otherwise compatible, level-eligible, supported attachments from automatic recommendations, including sights, magazines, preferred items and Keep selections. Recommendations maximize the same contextual handling score. The budget constrains the **known subtotal only**; unknown prices are never treated as confirmed zero. Such builds show **Cost incomplete** and explain that their total and budget fit cannot be verified. Weapon comparisons also leave incomplete build costs unknown.

The “Worth spending more?” view compares fully priced setups only, because cost differences and value tiers cannot be calculated honestly for unpriced items. This does not restrict the main recommendation engine.
