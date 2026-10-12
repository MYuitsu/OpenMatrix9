# RCORE-01 — Solid construction evidence, 2026-10-11T17:44:24+07:00

**Status: in_progress. Every original chapter requirement remains required.**

Rust now returns an independent, bounded UTF-8 record of completed Box/Sphere
construction through a generation-bound query/copy ABI. It preserves accepted
world points, normalized solver state, constraints, current session frame and the
15-value construction output. Accepted points have their own Undo-aware list:
the third 4Point sphere point and original AroundCurve pick survive solver reduction.
No native pointer or borrowed Rust allocation is returned to the host.

The native adapter freezes this record and the actual document unit snapshot,
stores both beside Shape in the existing preserving-Redo transaction, and rejects
changed/removed properties before publication. A unit revision changed during
the command rejects the stale construction; a fresh command still works.
Removing construction metadata now reports StaleGeometry with the original
Box/input/property context before attempting to set the editor mode.

This is **OM9SolidConstruction input evidence, not the required full GeometryResult**.
Its lifetime marker explicitly excludes current/live validity. It does not certify
source conversion, parameter/topology maps, original input-token fidelity, source
owners or all FitPoints provenance. Coordinates remain document-world mm; stored
unit context does not prove that every solver threshold follows that context.

## Verified evidence

- Matching Runtime35: **757 checks /8 actual host exits0**, run serially:
  construction67, cold restore21, callback150, publication86, commands70,
  options252, native binding93 and admission policy18.
- The cold host is a distinct process starting without a loaded model. It opens
  five actual FCStd files for BoxCenter, SphereCenter, AroundCurve, FitPoints and
  4Point. Shape validity/closure and independently expected volumes pass; owned
  input records and exact unit snapshots survive. The wrapper imports OM9 before
  the fixture; this is not a proof of opening without OM9 loaded.
- Identical final native fixture: Runtime33 **12/67** (missing metadata/context
  gate) → Runtime34 **66/67** (one missing-property diagnostic) → Runtime35 **67/67**.
  Both RED hosts themselves exited0; failing wrapper verdicts remain RED.
- Full Rust: **816 passed /0 failed /0 ignored** at final Runtime34. All221 Rust
  source files plus Cargo.toml/lock match Runtime35. The last repair is C++ only;
  no new full Rust execution is claimed for Runtime35.
- Historical Runtime32 caller evidence **703/8** is preserved separately.
  It is not a Runtime35 rerun or a current-source aggregate certificate.

Actual executable/plugin/App/GUI selections, frozen fixture hashes, reports,
FCStd hashes, build/full-test terminal receipts and source fingerprints are in
the [machine-readable checkpoint](2026-10-11-rhino-core-solid-construction.json).
The single FreeCAD SDK and standalone plugin cache were reused. SDK binaries
remain unchanged. Plugin SHA256: `c695720ecc0f0062143301ea835ddcd78f84816c53591692b28a733ff638b291`.

The three metadata fault cases exercise replacement, deletion and deleted unit
metadata. Each rolls back the output, preserves source/Undo/prior executable
Redo and retains exactly the test probe's one invalid output wrapper reference
after calibrated native attribute-owner cleanup. Existing150 callback assertions
and86 publication assertions pass on the same new plugin.

Independent source review found the reduced-point loss before the Rust repair
and the missing-property diagnostic before the C++ repair. A focused final source
review found no remaining issue in that guard; source review is separate from
native acceptance. Fixture errors and the earlier815-test Rust run are excluded
from the accepted cohort. Python-runtime location and native deprecation/startup
warnings are retained; this is not a claim of warning-free execution.

## Remaining full chapter work

- RCORE-01 remains in_progress. Preserve every original geometry family, enabled consumer, eight typed errors, adapters1–7, C01–C12 and T01–T11; these caller witnesses do not certify the whole chapter.
- Global conversion deviation and complete parameter/topology maps, remaining source attributes/full GeometryResult consumers, rational/periodic basis/trims/seam occurrences, solid validity and truthful representation/read/edit/retain/export remain required.
- Worker-to-GUI dispatch, arbitrary/custom callbacks and native document closure, all-family source/lock/owner gates, aggregate allocation/traversal/cancellation budgets, preview/cache cleanup in other producers and independent persistence/lifetime contracts remain required. Solid proofs cover only the listed synchronous native publication/callback paths; no actual Solid preview was constructed by these fixtures.
- Builder, Surface and Solid caller diagnostics/reference release are proven only for the exercised cases. Partial Python allocation failures, additional native exception classes, callbacks after the final gate and general GeometryResult integration remain unproven.
- Production deployment/ABI acceptance remains pending. Tests use the isolated App19/camera-guard GUI host; shared SDK binaries are unchanged. Current Runtime35 has757 assertions/8 actual host exits0. Historical Runtime32 caller703/8 remains a separate cohort, not a Runtime35 rerun.
- One prior owned test launcher attempt terminated with ntdll invalid-handle c0000008 before recorded bootstrap telemetry; identical-binary retry passes, underlying startup cause remains unproven. Early new camera fixtures lacked pivy initialization or the checked-action argument; these attempts are retained and excluded from final proof.
- The Solid owned solver fingerprint excludes only native Checked cache bits on an independent copy. This is a tested publication-integrity guard, not a global geometric equivalence, deviation, conversion or topology-map certificate. Full Solid GeometryResult/conversion/maps remain required.
- OM9SolidConstruction is Rust-owned numerical input evidence only. Full Solid GeometryResult, complete source attributes/master/provenance, selected FitPoints native source guards, conversion/deviation and parameter/topology maps remain required. It does not certify current/live validity or preserve every original numeric input token.
- The captured frame is the session input frame at capture, not a promised initial frame. Existing Solid math still uses document-world mm and existing solver tolerances; storing and latching the unit-context revision does not establish tolerance-driven solver behavior.
- Warm and cold archives cover the five named command recipes. No actual Solid preview was constructed; preview leases, worker dispatch, arbitrary later callbacks and every original family/adapter/C01–C12/T01–T11 still require evidence.

Next, reuse the shared prepared-result/native adapters to bind owned source
snapshots and output validation into a genuine Solid GeometryResult. Continue
the full closure and enabled-consumer audits, every adapter1–7, C01–C12 and
T01–T11. This checkpoint closes only the construction-capture prerequisite.
