# RCORE-03/04 shared Cartesian input consumers

Continue all-P0 from `e6e08e5`; this increment covers Distance, Angle and
PictureFrame adoption of the existing Rust point/length grammar only.

Rust owns typed context, parsing, resolution, immutable provenance and bounded
serialization. Native code captures actual document/frame/unit values and calls
FreeCAD/Qt APIs. No parser fallback remains in individual controllers. Bare
coordinates mean CPlane; `w` means world; relative forms use the last accepted
point. Mouse/snap/F4 coordinates already in world space are never transformed twice.
Keep each command's existing start-latched frame (PictureFrame before its image
dialog); unit changes invalidate the session. Distance `Unit=` remains output-only
and its current mm default remains unchanged in this bounded migration.

Only accepted inputs update previous/provenance or consume Once. Measurements
remain document read-only and retain completed input reports in UI/session memory.
PictureFrame stores provenance with its existing one-transaction output, with
rollback on native or standard exceptions. Its scalar width uses captured model
scale and supports explicit length units. Invalid input preserves phase/options.

- [x] Establish actual old-module failure with mixed bare/world measurement points
  and PictureFrame model-unit width; preserve hashes and failure logs.
- [x] Implement/test pure Rust resolver, owned provenance and bounded atomic ABI.
- [x] Add thin CoreCoordinates adapter and real CPlane owner/slot/revision snapshot.
  Publish the shared interface before parallel consumer integration.
- [x] Integrate Distance/Angle, acceptance-only reports and document lifecycle.
- [x] Integrate PictureFrame point/width parsing, modal revalidation and atomic
  provenance. Update old non-XY fixtures to explicitly request world coordinates.
- [x] Verify native geometry against independent transformed-frame/unit oracles,
  stale contexts, malformed input, acceptance/Once, Undo/Redo and cold persistence.
- [x] Run matching SDK and affected Rust/native regression, independent review,
  synchronize bounded evidence/specs/ledger, commit and continue remaining P0.

Detailed read-only design: `H:/FreeCAD-src/build/p0-parser-migration-design.md`.
No new grammar or document-unit default is inferred. Polar/spherical/surveyor,
locale/fraction input, all-family common command reducers, associative snap source
identity, all layers and full P0 remain required outside this increment.

Verified increment: docs/validation/2026-10-09-rhino-core-point-consumers.md. Remaining all-P0 requirements stay open in the master plan.
