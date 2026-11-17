# Trimmed standalone surface in geometry-only 3DM export

## Large trim metadata investigation — 2026-11-13

**Historical:** the 16 KiB default limitation described below has since been
expanded for the trim slot. See [current scoped support](2026-11-13-large-trim-support.md).

A 28×28 planar NURBS surface with a rectangular trim and hole was prepared to
investigate the export bridge's 16 KiB metadata limit. The original fixture
failed during controller/document validation with `Budget("extension payload")`,
not in `set_trimmed_surface` as previously stated. Before this fix the setter
could install the oversized extension and report success; it now validates the
extension before replacing metadata and reports the budget error without
changing the entity. The trim extension duplicates the surface data and the
shared extension validation uses `max_name_bytes` (default 16 KiB). The
original fixture never reached the native adapter or 3DM output.
Removing the export-specific limit alone would not make this default workflow
work. Raising the shared name/extension bound globally also affects unrelated
data and admission, so it has not been changed. Supporting larger sources
needs a separately budgeted trim representation or a safely scoped limit and
save/load compatibility; the existing seven passing file fixtures do not prove
this case. The temporary failed export test was removed. Core regressions now
cover the 28×28 source under default limits, retention of an existing trim when
the limit rejects replacement, and checking the extension size before parsing.
The document-graph and semantic suites passed (32 and 13 tests at the initial
run); the added large-source regression passed separately. ExtractWireframe
regression passed 76 tests with four diagnostics ignored, and seven real 3DM
fixtures passed again with the P1 Release adapter.

The setter now also serializes the trim through a writer bounded by the existing
extension byte limit: it stops on an oversized payload instead of first building
the complete JSON string. The exact limit and one byte below were tested; the
accepted output is byte-identical to the existing representation and a rejected
replacement retains the entity. The 14-test semantic suite, four archive
roundtrip tests and all seven native 3DM file fixtures passed after this change.
This bound applies to the setter's serialization buffer; it is not a native
peak measurement and does not make larger trims supported. Other callers of the
unbounded `TrimmedSurface::to_extension` were not changed.

## Multiple trimmed surfaces and solid — 2026-11-13

The production 3DM host exported and reimported two separately placed planar
trimmed surfaces, each with a hole, together with a translated editable box.
Six rays checked material, hole and outside for the surfaces; source volume and
world bounds checked the box. Document and Redo stayed unchanged. The new
fixture passed separately and in the seven-case batch using the P1 Release
adapter. This exercises multiple simultaneously retained conversion outputs;
it does not measure native transient peak, prove arbitrary topologies or
establish desktop / Rhino interoperability.

## Mixed trimmed surface and editable solid — 2026-11-13

The earlier mixed-projection refusal below has been replaced by shared admitted
conversion for standalone trims and editable B-reps. The existing native
writer exports a plane with a hole and a translated box in one 3DM file; on
reimport the material and hole rays, source volume (8 mm³), and source-based
world bounds match. Document and Redo remain unchanged. The converter uses the
canonical B-rep source, not its viewport display, and the temporary export
copy has no display mesh. The original model is untouched.

The first two attempts reached export/reimport but used inappropriate test
oracles: the single-face ray operation cannot measure a box, and import may
bake placement into source. The corrected source-volume and world-bounds oracle
passed, followed by all six file cases. Mock admission tests also verify that
late pressure after a prior trim conversion discards the private projection,
releases its reservation and retains the original document; a separate test
checks late cancellation. They do not simulate actual OS allocation failure.
The test used the compatible P1 Release adapter, not the desktop debug layout.
Native temporary peak, arbitrary geometry and desktop remain unverified.

## Scoped admitted 3DM export — 2026-11-13

The staged export now converts supported standalone trimmed surfaces from their
source UV boundaries to temporary native B-reps, without using viewport mesh as
the geometric source. The conversion reserves managed projection, source and
payload memory; native OCCT/openNURBS transient peaks remain outside that bound.
The original document and its trim metadata are not changed.

With the compatible P1 Release adapter, the real 3DM file export/reimport batch
passed six fixtures: plane, partial cylinder, full-seam cylinder and selected
block with a plane, each with a hole; an unsupported later trim; and a mixed
trim/editable B-rep. Source-ray checks cover material, hole and outside on the
four positive fixtures. Selection and placement were checked in the relevant
fixtures. The plane fixture checks simulated admission pressure; failure,
pre-cancellation and unsupported sources preserve the existing destination,
document and Redo in the exercised cases. No desktop or independent Rhino test
was performed. Synchronous native calls cannot be interrupted mid-call.

The bridge refuses reversed surface/trim metadata, unverified seams, trim data
over 16 KiB or over 128 edges. Direct geometry-only packet and
retained merge input remain guarded. Other trims are not certified by the
fixtures. The existing debug native exchange executable rejects the packet
schema (`missing/unknown record fields`); this batch instead used
`L:/om9-target/p1-native-exchange/Release/om9-exchange-native.exe`.
Default desktop packaging must be reconciled and tested before claiming the
installed application can use this export.

The following sections record the earlier guard and bridge probe, before this
integration; their statements about the next step are historical.

The geometry-only 3DM packet serializes a standalone surface's underlying
NURBS geometry but omits entity extensions. A planar trimmed fixture with a
hole was accepted by the packet encoder before the guard, even though the
packet could not carry its outer boundary or hole. This establishes a source
projection defect; the test did not write or measure an incorrect native 3DM.

The 3DM packet and staged export paths now explicitly refuse a captured
standalone trim extension, including in a block definition. The retained
merge-input preparation is guarded as well. An unrelated selected point remains
exportable in the packet fixture, and the separate OM9 clipboard projection
is not changed. An existing native-adapter test confirms that the explicit
refusal leaves a pre-existing destination, document snapshot and one Redo
entry unchanged; no output file or staging residue was created. This is an
intentional temporary loss of 3DM export for these surfaces, not full support.

Next, construct a verified trimmed native face from source UV loops and export
it as an appropriate 3DM B-rep, retaining placement, holes, independent error
handling, admission and atomic output. Check both direct and nested/selected
exports, source geometry after native re-import, one Undo/Redo context, and
failure/pressure before enabling it. Arbitrary trims, reversed metadata,
closed seams, native peak and desktop remain separate evidence requirements.

## Native B-rep bridge probe

The existing source-trimmed OCCT face and B-rep conversion bridge were tested
directly with one planar and one cylindrical patch, each with a hole. Both
converted to an openNURBS B-rep payload and back; a material ray hit while
hole and outside rays missed in each fixture. The named batch had four attempts:
one fixture, then both fixtures in each of three runs; two unique fixtures passed. This supports reuse
of the bridge for these shapes, but does not yet exercise an exported 3DM file,
document projection, placement, UI or Undo/Redo.

The current export bridge accepts a `Brep` and produces a native payload, but
the standalone-trim path also needs a temporary face and display before that
call. The present projection reservation only covers a document clone; it does
not independently reserve all converted payloads and temporary source/display
owners. Adding a conversion call without accounting for simultaneous old,
new and scratch allocations would bypass the project's memory contract. The
temporary refusal remains in place while an admitted projection is integrated.
