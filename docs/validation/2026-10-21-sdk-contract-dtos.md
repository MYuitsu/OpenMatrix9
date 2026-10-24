# SDK read-DTO wire contract — 2026-10-21

The new read DTOs now have an explicit wire-shape contract test in
`crates/om9-sdk/tests/contract.rs`, so the versioned JSON ABI cannot drift silently as the
services grow.

## Covered

`ObjectBounds` (nested arrays), `CurveSegment` (kind/domain/angle_domain/reversed),
`ObjectSummary` (total/by_kind map), `MeasuredArea` (square_millimeters), `ProjectChange`
(from/to revision, cause, entries) and `ObjectExtension` (version/payload) each serialize to
the documented member names and round-trip where the type is used as a reply.

## Why

The SDK plan requires contract-first DTOs and a versioned ABI; a shape change (renamed member,
array vs object) must be a deliberate, reviewable break, not a side effect of a host change.
This test is the review gate for that.

## Measured batch

| Batch | Result |
|---|---|
| `cargo test -p om9-sdk -j 2` | 13 passed, 3 suites (one new contract test) |
| `cargo test -p om9-plugin-host -j 2` | 56 passed, 5 suites |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-contract` | PASS, 10 commands; 27 native tests |
| `cargo check -p om9-app -p om9-plugin-host -j 2` | exit 0 (default features) |
| `rustfmt --check` on the test file; `git diff --check` | PASS |

Artifacts are recorded in the [JSON receipt](2026-10-21-sdk-contract-dtos.json).

## Not proved here

- This is not a plan capability slice; no owner L3 status changes.
- macOS host and plugin execution remain unverified; no manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: return to the plan's remaining owner slices (`OM9-L2-01-03` groups/blocks/snapshot,
`M02` subobject/pick) or record the remaining host edit paths.
