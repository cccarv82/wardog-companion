# WARDOGS Companion 1.2.0 — optional Click assist

- Enable Click assist in Settings for the current session; position the cursor over the team button and press F8 to start or stop.
- Esc, cursor movement, a foreground-window change, disabling the feature or a 10-second attempt limit also stops input.
- Requested click interval: 2–100 ms, default 2 ms, using a Windows high-resolution waitable timer when available. Timing and game acceptance are not guaranteed.
- Input is restricted to the foreground WardogsClient-Win64-Shipping.exe process. No injection, input hooks, game memory access or game-file changes are used. Team entry is not detected; stop when accepted.
- The feature is disabled on every launch. Existing builds, game data and profile settings remain unchanged.
- Corrected an outdated explanation that still described missing prices as excluded.

# WARDOGS Companion 1.1.1 — missing prices no longer hide useful gear

- All compatible, eligible attachments with supported effects can enter automatic recommendations, regardless of whether a purchase price is published. This covers every slot, Keep selections and preferred sights.
- Incomplete costs remain unknown. The known subtotal is checked against the budget; the app does not claim that the complete build fits it.
- Class/range recommendations and future-unlock suggestions keep missing total costs explicit.
- Spending tiers compare fully priced setups only and explain that scope; missing prices never restrict the main recommendation.
- Saved builds stay intact. Choose **Update setup**, or change goal/range, to recalculate.

Regression coverage includes the reported PP-19 Vityaz case: Medium range, levels 86/14/19/10/22 and a $2000 parts budget now consider the unpriced Tricon 1.5x Compact Prism Scope. Price availability does not imply ownership, level eligibility or effect support.

# WARDOGS Companion 1.1.0 — builds with a purpose

Recommendations now consider what the weapon is for, not just which parts improve three stats.

- **Role-aware sights:** Auto uses the weapon role; optional Close / Medium / Long settings use explicit sight bands. Long-range snipers no longer automatically receive a reflex for its ADS bonus.
- **Protected magazine capacity:** the automatic model uses a published default capacity when available, otherwise documented class floors. Smaller magazines no longer win solely through recoil/aim-time bonuses.
- **Contextual handling:** recoil, aim time, spread, equip speed and aim sway contribute with priorities adjusted for precision weapons, range and published recoil-axis balance.
- **Conservative scope support:** one ordinary multiplier is supported; isolated additive aim time requires a known base. Ambiguous stacking still fails closed.
- **Consistent recommendations:** class/range builds, spending alternatives and useful unlocks follow the same context. Precision weapons display relevant handling metrics.
- **Your choices stay yours:** saved parts are preserved on upgrade. Choose **Update setup**, or change the goal/range, to apply the new rules. Keep, ownership, explicit sight/capacity preferences and budget remain enforced.

The interface adds one compact range selector, defaulting to Auto. Method and policy details stay behind Why this setup? and in the documentation. Identical parts can still be correct for several weapons; recommendations are never randomized for variety.

Download **WARDOG-Companion-1.1.0-Setup.exe**, the portable ZIP, and SHA256SUMS.txt from this release. Existing updater identifiers and profile paths remain compatible. The original profile is backed up before saving the new context fields.

These are source-backed models and explicit product policies, not measured in-game effectiveness. Scope optical quality, full variable zoom ranges, armor, skills, carried weight and sustained fire/reload performance remain outside the model. OCR is experimental. This release remains unsigned. See docs/RECOMMENDATIONS.md and VALIDATION.md.
