# Gem Reporter Count And Weight (`OM9-L3-04-13-008`)

## Command

`BuilderGemReporter / gvGemReport` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_edits::gem_report(records, Option<density>)` | aggregates by shape, cut, girdle and material; each row carries count, carat, grams, the material density and the trade-formula cross-check, and the report carries the totals, the worst delta and the out-of-tolerance count |
| `gem_edits::MATERIAL_DENSITIES` / `material_density(name)` | gemological reference densities (diamond 3.52, corundum 4.00, beryl 2.72, …); an unknown material is flagged as an assumption instead of silently using diamond |
| `gem_edits::weight_formula(spec)` | published per-shape trade formula `L × W × depth × factor` on a diamond basis, scaled by the row density; no formula is applied to a cabochon or custom profile |
| `gem_edits::gem_weight(spec, density)` | primary mass from the closed procedural mesh volume × density; one carat = 0.2 g |
| `gem_edits::validate_density` | density must be finite and inside 0.1..=30 g/cm³ |
| `BuilderGemReporter` session | `Density=` recomputes the report, `File=` exports the JSON; the document is never modified |

## Effects

- A preview is pure session data: planned records and, for a report, computed rows. Nothing is
  written to the document before Apply.
- Apply is exactly one document transaction for every mutating command; a style save writes the
  versioned store atomically. Undo restores the previous records, geometry and label.
- Refusals are typed `om9_core::Error` values (`Invalid`, `Missing`, `Stale`, `Busy`,
  `Budget`); an attribute, stone or row is never silently dropped or replaced.
- Every edit keeps the `GemId`, the object id and the `om9.gem.placement` link; only the
  `om9.gem` record and the regenerated geometry change.

## Verification

- Positive: count and computed mass grouped by shape/cut/size/material, cross-checked against the trade formula and labelled as a computed value, read-only
- Negative: an impossible density is a typed refusal; an unknown material density is reported as an assumption instead of a silent diamond basis
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
