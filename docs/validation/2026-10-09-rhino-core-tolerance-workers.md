# Rhino core P0 — Join tolerance and worker policy — 2026-10-09

Status: **validated bounded increment; all P0 remains in progress**.
Parent commit: `22fecb2`. No new publication is part of this continuation.

Rust resolves actual Join cutoff from explicit override, explicit document absolute
tolerance, then legacy operation default. Legacy curve Join remains 1e-7 mm and
surface Join remains 1e-6 mm. Document-derived surface Join uses twice absolute;
an explicit override is already the actual cutoff. Relative/angular values and
display precision do not alter this rule. These choices follow the documented
[Rhino 5 Join threshold](https://docs.mcneel.com/rhino/5/help/en-us/commands/join.htm)
and distinguish [unit/tolerance settings](https://docs.mcneel.com/rhino/5/help/en-us/documentproperties/units.htm)
from OM9-specific precedence, range and numerical guards.

The native adapter captures unit context at command start and rejects context
changes before committing. Preview and commit use the same resolved rule. The
surface path now calls OCCT sewing with the resolved cutoff instead of the host
Python sewShape wrapper that ignores its parsed argument. It rejects any backend
cutoff clamp. Valid output records the actual cutoff, rule/source/context revision
and explicit unit context in its creation transaction. Undo/Redo and cold FCStd
restore retain that provenance. The arithmetic lower bound is an OM9 conservative
32*epsilon*max(1 mm, world coordinate bound), not a certified OCCT error bound.

Rust also owns import process count: max(1, floor(60% of logical CPUs)), bounded
by available tasks and a validated override ceiling, with no fixed four-worker cap.
Native Qt/openNURBS code owns process execution and geometry. Existing small-root
and large-file serial heuristics remain. Invalid overrides fail before document
mutation. The real host run exercised 11 top-level and 8 definition partitions;
both default and requested-64 modes exactly matched serial records and ordering.
This does not establish an aggregate memory budget or asynchronous cancellation.

## Verification

- Matching SDK GUI and worker build passed; standalone native geometry build and test passed.
- Full Rust suite: 347 passed. Clippy exit 0 with existing warnings. New files pass rustfmt.
- Python tooling: 49 passed, 1 skipped. Source audit is recorded in machine evidence.
- Native: 444 passed assertions in eight exit-0 processes, including four cold-restore checks.
- Every accepted macro/module hash matches the tested files. Native SHA256: `ea12455c1a6d4065dc02d29c5923a8148818b51e051b989bcc75037028f40e1d`.

| Suite | Assertions | Directory in build/merge_replay-1 |
| --- | ---: | --- |
| tolerance | 29 | `dde13a0e4de2408f88a4e0da1aef7713` |
| tolerance-cold | 4 | `4e7d7c12fc30466c8b3c4fa4a4974243` |
| edit | 115 | `6f89e136f98249c8af312619954c2d70` |
| edit-options | 57 | `23daded1d0b64807ac6055f427677a9d` |
| history | 83 | `8adfbb8717774dbc8c639512b3ab1ae1` |
| surface | 97 | `f78d25d73905410da9ec7c905f7d9a86` |
| units | 20 | `0b0d0c7afd0a418f92b3f0d358b88c24` |
| workers | 39 | `891ce9994810495686b5cc27a40ded01` |

The final tolerance fixture checks 0.5tau acceptance/1.5tau rejection, actual
surface 2tau, explicit curve/surface overrides, legacy defaults, backend clamp
rejection, precision 2 versus 8, stale context, numerical floor, Undo/Redo and
persistence. Worker tests inspect real staging requests before cleanup, test
invalid configuration, preserve objects/Undo/modified state and verify missing or
failed helper cleanup. The SDK test executable and module live in different bin
directories; a matching helper was copied beside the executable only for the
worker run, hash-checked and removed afterward. The fixture now uses the actual
application directory for its deliberate helper-failure checks.

Meaningful failed baselines and the missing-helper environment failure are excluded
from pass totals and retained in [machine evidence](2026-10-09-rhino-core-tolerance-workers-evidence.json).
Earlier tolerance runs using precision 1/8 were superseded by the exact 2/8 fixture;
their counts are not added to this checkpoint.

## Ownership and remaining work

Safe Rust owns numeric policy, validation and owned report strings. C++ accesses
native shape bounds, FreeCAD properties/transactions, OCCT sewing and QProcess;
these are required host API boundaries. ABI pointers are borrowed only for each
call, outputs stay untouched on failure, and neither policy stores native pointers.
Release panic=abort remains a limitation. No solver threshold outside ordinary
Join changed. This increment does not migrate all operation-specific tolerance,
History replay, shared command/input/snap/layer policy or asynchronous jobs.

The [full P0 matrix](rhino-core-p0-acceptance.json) and independent audits preserve
the remaining requirements. Only the bounded Join gap fixture RCORE-02.T03 is
closed here; precision/preview/persistence requirements with additional consumers
remain partial. Historical reports are unchanged.

Final source review found no remaining actionable defect in this scope. Source audit: 2040 files, 511 bindings, 510 SVGs, zero errors. Spec manifest: 664 entries; JSON/YAML equivalent.
