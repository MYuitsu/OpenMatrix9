# RCORE-01 — Solid publication, 2026-10-11T17:17:29+07:00

**Status: in_progress. All original chapter requirements remain required.**

Box/Sphere reuse the existing FreeCAD Part solver. Publication now opens the
native preserving-Redo transaction, freezes operation/input identity and session
generation, verifies source snapshots again and wraps native commit failures in
typed diagnostics. A callback that starts another Solid command cannot make the
old command publish or cancel the replacement session.

Before commit, the adapter verifies the output wrapper, local/world/Shape frames,
native shared-shape identity, root orientation, an owned solver fingerprint and
command/feature/parameter metadata. Rust validates value-only frame matrices and
owns the session generation. The adapter rejects NaN and detects placed-parent,
shared native tolerance, orientation and post-recompute Shape-root mutations.
Deleting metadata reports StaleGeometry with the original operation/input/property.

## Native evidence

- Current Runtime33: **669 checks /6 actual host exits0** — callback150,
  publication86, commands70, options252, input binding93 and admission policy18.
- Runtime32 unaffected caller cohort: **703 checks /8 actual exits0** — Builder
  and Surface fault/publication, Edit, atomic API, actual Surface preview and camera.
  These are separate binary cohorts;1372/14 does not mean1372 Runtime33 checks.
- Full Rust suite ran at the source-identical Runtime31 checkpoint: **811 passed,
  0 failed,0 ignored**.220 Rust source files plus Cargo.toml/lock match Runtime33.
  No new Rust execution is claimed for the C++-only metadata correction.

Actual host/module/App/GUI paths and hashes, frozen fixtures, observations and
terminal exit receipts are listed in the [machine-readable evidence](2026-10-11-rhino-core-solid-publication.json).
The single SDK and existing standalone plugin cache were reused; SDK binaries
were not replaced. Plugin SHA256: `0763215778d3ec5e0c1fdff72955daaf49fa2d541235f8c78bb279b0d649f5cd`.

## Failing witnesses and calibration

Original Runtime25 native before/close and AroundCurve source rejection witnesses
fail before the repair. Current publication86 verifies rollback, typed source
diagnostics, strict output-wrapper release and successful Undo/Redo/native identity.
Failed native close disposes uncertified prior Redo, as required by the existing
checked-publication API. Ordinary stale/callback rejection and Cancel preserve
and execute the prior Redo. Stronger success controls were added subsequently;
the early RED5 and current86 are not claimed to be identical whole fixtures.

Historical matched frame fixture: RED64/78 → GREEN78/78. The strengthened
integrity fixture: RED110/114 → GREEN114/114, detecting four Box Shape-root
publication failures. Current metadata fixture: **RED144/150 → GREEN150/150**,
with exactly six missing-property diagnostic failures on Box/Sphere.
Callback GREEN covers25 cases, including replacement commands, shape, Placement,
metadata tampering/deletion, NaN, placed parent, native tolerance, orientation and
post-recompute Shape-root mutation. Outputs are invalidated and retain exactly
the probe's one owned wrapper reference after calibrated host-owner cleanup.

FreeCAD PyObjectBase.setAttributeOf keeps a parent reference in its native
attribute dictionary. Reading output.Document in an observer can therefore keep
DocumentPy's parent owner on the rolled-back output.12 calibration assertions
show that reading the surviving Source.Document rebinds this known host owner;
the final oracle still requires refcount1 and does not relax or drop an OM9 owner.

Raw output BRep comparisons rejected legitimate native placement serialization;
RemoveLocations also rejected a valid rotated Cube (Runtime31 options45/46 plus
fixture assertion). These candidates are excluded. The accepted approach checks
native IsPartner, frames and orientation separately and fingerprints an owned
deep copy of the original solver shape with only Checked cache bits cleared.
Eight calibration assertions cover placed Sphere/rotated Box and shared tolerance
mutation. It is an integrity guard, not a general geometric equivalence certificate.

These Solid fixtures check that no preview remains but construct no actual Solid
preview. They do not certify Solid preview leases, worker completion or all native
callbacks. Build/host deprecation and initialization warnings are retained; final
cohorts contain no escaped Base::Exception, not a claim of warning-free output.
Independent source review of the missing-metadata correction found no remaining
issue in that narrow change; runtime evidence is recorded separately.

## Remaining full chapter work

- RCORE-01 remains in_progress. Preserve every original geometry family, enabled consumer, eight typed errors, adapters1–7, C01–C12 and T01–T11; these caller witnesses do not certify the whole chapter.
- Global conversion deviation and complete parameter/topology maps, remaining source attributes/full GeometryResult consumers, rational/periodic basis/trims/seam occurrences, solid validity and truthful representation/read/edit/retain/export remain required.
- Worker-to-GUI dispatch, arbitrary/custom callbacks and native document closure, all-family source/lock/owner gates, aggregate allocation/traversal/cancellation budgets, preview/cache cleanup in other producers and independent persistence/lifetime contracts remain required. Solid proofs cover only the listed synchronous native publication/callback paths; no actual Solid preview was constructed by these fixtures.
- Builder, Surface and Solid caller diagnostics/reference release are proven only for the exercised cases. Partial Python allocation failures, additional native exception classes, callbacks after the final gate and general GeometryResult integration remain unproven.
- Production deployment/ABI acceptance remains pending: isolated App19/camera-guard GUI test host is used; shared SDK App12/GUI binaries are unchanged. Current Solid669/6 uses Runtime33; unaffected caller703/8 uses Runtime32 and is not a Runtime33 rerun.
- One prior owned test launcher attempt terminated with ntdll invalid-handle c0000008 before recorded bootstrap telemetry; identical-binary retry passes, underlying startup cause remains unproven. Early new camera fixtures lacked pivy initialization or the checked-action argument; these attempts are retained and excluded from final proof.
- The Solid owned solver fingerprint excludes only native Checked cache bits on an independent copy. This is a tested publication-integrity guard, not a global geometric equivalence, deviation, conversion or topology-map certificate. Full Solid GeometryResult/conversion/maps remain required.

Continue the [full closure audit](../reviews/2026-10-10-rcore01-closure-audit.md)
and [enabled-consumer audit](../reviews/2026-10-10-rcore01-enabled-consumer-audit.md).
This checkpoint does not close every family, adapter1–7, C01–C12 or T01–T11.
