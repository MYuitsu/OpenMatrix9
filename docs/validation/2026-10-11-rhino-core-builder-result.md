# RCORE-01 — Native Builder result and source evidence — 2026-10-11

**RCORE-01 remains in progress.** This checkpoint integrates the existing
GeometrySnapshot/GeometryResult producer into native affine Builder. It does
not certify the original all-family requirements, conversion exactness or maps.

Existing Create/restore/Match stage the full owned result before transaction.
History recompute prepares a result before assigning Shape/result together;
failed recompute clears both Shape and accepted evidence. The result uses actual
NativeBuilder operation, independent raw recipe/feature/slot/affine dimensions,
full-u64 source generation/revision/epoch, unit context and source world frame.
The native OCCT V3 CAD master retains topology and inspected native validation.
Output parent-to-world excludes the own Location already stored in the master.

TemplateAttributes stores immutable seed identity, labels, visibility/selectable
and inspected native ShapeAppearance captured by CoreSourceAttributes. Seed
deletion or subsequent label/style edits cannot change saved evidence. Match
copies original template metadata while identifying the actual target source.
Rust decodes persisted values through SourceAttributes::capture, checks slot
cardinality and bounds before native JSON parsing, and reuses NativeBuilder
registry/affine_plan for operation admission. Genuine legacy empty metadata is
explicitly unavailable; no source identity/appearance is fabricated.

These attributes describe the seed, not current generated output styling.
Historical visibility/style is not silently applied to output. Other application
properties and Rhino userdata remain uninspected. Native transforms are observed;
deviation, parameter/topology maps and conversion exactness remain unmeasured.
Import keeps its own policy/lifetime/conversion reports and is not relabeled.

C++ owns GUI document transactions, properties, native copies/transform/readers
and OCCT serialization because those APIs require native owners. Rust owns
independent values and validation. No native shape/document pointer is retained
by Rust. Serialized result/metadata budgets come from Rust; prepared results stay
within aggregate byte limits before publication. This does not prove all native
or process-wide allocation budgets or safe foreign-pointer provenance.

## Evidence

- Native RED: 0/14, no fixture exception, actual host exit0 on prior plugin.
- Native result GREEN: 20/20, including History dimensions/revision, seed
  independence, FCStd, explicit restore, Match, corrupted metadata/recovery,
  legacy metadata and grouped output world composition.
- Matching 297 checks/9 actual host exits0 on plugin
  `b120149e1efda088cc3220711a2cc8a31e38a37e17be15d652b43bc93bef769f`; Builder output/input/warm/groups/cold and CAD/discrete
  import result regressions pass. Cold fixtures use models from this run.
- Rust behavioral RED 5/7 to GREEN 8/8; full 747 tests/107 nonempty
  suites, actual exit0. Standalone plugin-only build2 exit0 reuses the shared SDK.
- Earlier GREEN1 passed20 on the first candidate binary; it is historical,
  excluded from the current matching total after conversion `from` became the
  actual template representation. No FreeCAD root rebuild or host clone.

Source/runtime/artifact hashes are recorded in the [receipt](2026-10-11-rhino-core-builder-result.json).

## Original remaining scope

- Measured conversion/deviation and genuine parameter/topology correspondence for Builder and original conversion fixtures; no identity or exactness is invented.
- Full GeometryResult consumers and uniform source/lock/generation/cancellation/cache/transaction contracts across every enabled family.
- Remaining source properties/userdata, attribute application policies and original representation/preservation requirements.
- Aggregate native geometry/worker/cache/getter budgets and independent bounded Surface History JSON admission.
- Original RCORE-01 adapter1–7, C01–C12 and T01–T11 requirement-by-requirement final acceptance and review.
