# M02 `OM9-L2-03-09` — Febe Matrix F6 Context Actions

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-09` (M02)

## Audit result

F6 existed only as a fixed Line/Point/File popup. Added the typed context adapter:

- Selection classification: General / Solid / Curve / Surface / Gem; mixed or unknown falls back to General.
- General actions come from the existing command registry, no duplicate dispatch.
- Curve/surface/solid/gem/report actions are **registered disabled** with explicit provider reasons when M03/M05 is absent.
- Custom user menu stores command IDs, preserves order, deduplicates and disables deleted IDs.
- Builder prefill emits a typed versioned request only when the M04 provider exists.
- Floating preference clamps to the active screen in logical points; pin state is retained.
- Disabled buttons use `add_enabled` and never call `invoke`.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Selected-class dispatch | `f6::classify`, `context` | f6 unit 7, `f6_context` 2 |
| `-002` | General mode | registry actions | f6 unit 7, `f6_context` 2 |
| `-003` | Curve/surface | registered disabled M03 | f6 unit 7 |
| `-004` | Gem/report | registered disabled M05 | f6 unit 7 |
| `-005` | Custom menu | `CustomMenu`, `om9.f6.v1` | f6 unit 7, `f6_context` 2 |
| `-006` | Builder prefill | `BuilderPrefill`, provider gate | f6 unit 7 |
| `-007` | Floating pin/position | `F6WindowPreference::clamp` | f6 unit 7 |
| `-008` | Disabled actions | availability + `add_enabled` | f6 unit 7 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `om9-app f6 unit` | **7 passed** | `implementation/evidence/M02/2026-10-22-f6-unit.log` |
| `om9-app f6_context` | **2 passed** | `implementation/evidence/M02/2026-10-22-f6-context.log` |
| `om9-app --features gpu-sphere --lib` | **168 passed** | `implementation/evidence/M02/2026-10-22-f6-app-lib.log` |
| `cargo fmt --all -- --check` | exit 0 | `implementation/evidence/M02/2026-10-22-app-fmt.log` |
| App/workspace clippy | exit 0 | `implementation/evidence/M02/2026-10-22-app-clippy-final.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-f6-public-source-audit.log` |

## Manual gate and limits

- **M02-UI human review is still pending.** Context grouping, click dispatch, pin and disabled tooltips are tested headlessly; visual placement/keyboard behavior needs human review.
- macOS runtime is unverified.
- M03/M04/M05 providers are absent; their F6 actions are registered disabled, not stubbed success.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-09-f6-context-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-09-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-09-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-09-00*/`
