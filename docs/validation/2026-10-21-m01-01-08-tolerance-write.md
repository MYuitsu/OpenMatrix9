# Document tolerance write through SDK v1 — 2026-10-21

A plugin can set the document tolerance policy; previously the units owner exposed tolerance
as read-only. Owner: `OM9-L2-01-08` (`OM9-L3-01-08-003`).

## Changed

| Layer | Change |
|---|---|
| Public SDK | `Host::set_document_tolerance(absolute_mm, relative, angular_rad)`; one request and capability entry. |
| Host | Validates through `Tolerance::try_new` (finite, positive absolute/angular, non-negative relative) and stages `Mutation::Tolerance` at the base revision. |
| Tests | Host test that a valid policy commits (`effective_tolerance` = 0.01 mm), Undo restores the document, and a zero absolute value is a `tolerance` refusal. Native suite unchanged (35) — this slice is host-tested only. |

## Semantics fixed by this slice

- Admission is the document's own rule (`Tolerance::try_new`), so SDK and host share one
  definition of a valid policy.
- The write is a transaction mutation with `expected_revision`, so it composes with other
  mutations and one Undo restores the prior policy (or removes it when none existed).
- Context stamps/revisions used by existing recipes are not rewritten.
- Reading stays unchanged: `document_info`/`document_tolerance` continue to report the state.

## Measured batch

| Batch | Result |
|---|---|
| `cargo test -p om9-sdk -j 2` | 13 passed, 3 suites |
| `cargo test -p om9-plugin-host -j 2` | 65 passed, 5 suites |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered |
| `validate_plugin_sdk.py --output .../20261021-plugin-sdk-tolerance-b` | PASS, 10 commands, 35 native tests |
| `cargo check -p om9-app -p om9-plugin-host -j 2` | exit 0 |
| public source audit / rustfmt / `git diff --check` | 8863 files, 0 errors; PASS |

The [JSON receipt](2026-10-21-m01-01-08-tolerance-write.json) records hashes and artifacts.

## Not proved here

- No native DLL command exercises `set_document_tolerance` yet (host-tested only).
- Per-object tolerance overrides are not exposed.
- macOS host and plugin execution are unverified; no manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: add a native tolerance command, or return to M02 for subobject/pick services.
