# RCORE-01 native Edit publication review

Scope: the CAD result producer and publication guard. Full RCORE-01 is open.

Independent read-only review identified three publication defects: inherited
layer metadata was applied before group insertion; a broad organization allowance
admitted unrelated History/group mutations; and NaN escaped a matrix comparison.
Actual native fixtures reproduced the defects. Repairs use exact group membership
and History detach boundaries, apply layer metadata after insertion, and validate
both native Shape and Placement matrices before destructive source changes.

Additional native investigation found Union false rejection from Checked cache
and root Location datum identity. Native master comparison strips the root
Location before private topology copy and normalizes Checked on that copy only.
Numerical frames, orientation, native partner and the remaining bounded BRep digest
still have separate checks. Temporary file-dump instrumentation was removed.

The original legacy command fixture then exposed equivalent source root Location
refresh during recompute. The guard allows this only in its own recompute, with
source Recompute status and exact original partner/orientation, frozen digest and
world signature. Other source writes remain sticky failures. An independent
focused follow-up review found no further substantive issue in either repair.
The reviewer did not execute host tests; native evidence is recorded separately
in [the validation report](../validation/2026-10-11-rhino-core-edit-result.md).

The earlier checkpoint used matching runtime9 result110/original command115 and
historical full Rust805/114. Native recompute9c subsequently passes all five
assertions with actual host exit0. Raw BRep comparison originally differed only
in identity Location serialization; explicit Part.Shape value wrappers now compare
normalized private master and numeric frames, plus the original native partners.
The immutable property-wrapper fixture exception is retained separately.

The native performance probe found1189 Dimension segments. Source checks now
collect the entire displayed primitive set and exact parent/text/angular data
without repeatedly creating/exporting Part edges. Actual output creation shares
that extraction and retains original world-once geometry. Archive/block/cage
dispatch precedence is preserved; no sampled or revision-cached source checks.

Follow-up review found NaN could disappear in ordered segment deduplication.
Actual Runtime13 finite13b/focused finite13d each fail five of ten assertions:
NaN publishes output and loses Redo; Infinity has an untyped diagnostic. The
collector now records a sticky finite failure before ordering/insertion and
throws NonFiniteGeometry/Explode/source only after Coin traversal. The focused
fixture attaches a real displayed SoLineSet after valid native primitives.
Read-only review found no new actionable issue in that repair and fixture.

Matching Runtime14 finite input10 and publication mutations12 pass with native
exit0; the latter changes actual displayed coords, font and an attached NaN line.
Its font/graphics error assertion did not require the operation on the actual
error line. A strengthened fixture, cleared independently for each case, failed
two of twelve assertions: these stale errors said Edit instead of Explode.
Runtime15 stores the validated operation name in the publication guard and uses
the existing Rust finite scalar validator before native segment ordering. The
read-only follow-up found no actionable issue in either change. Native Runtime15
finite10/10 and strengthened diagnostic12/12 now have actual exit0.

Full Rust808/114 has fresh unchanged before/after218-file fingerprints. Preserve
the older806 and805 reports as historical; unrelated Gumball edits changed the
Rust inputs. Runtime14 and15 each have separate scoped producer source proofs.
Original Runtime14 special25, CAD110, commands115, options57, policy20,
special-input10, Surface81, analytic8, Builder17, prepared47, recompute5,
source/group8, output8, layer7, budget4 and independent cold37 all have matching
terminal native exit0. These results remain scoped to that plugin hash;
Runtime15 changed-consumer regressions have separate evidence.

Prior executable Redo is still lost on failed publication because FreeCAD deletes
its Redo history when opening a transaction. Runtime14 confirms the original five
assertions still pass only3/5: objects, source master and Undo restore, but prior
Redo falls1 to0 and does not execute. Keep the original five assertions.
See [the host contract investigation](../superpowers/plans/2026-10-11-rcore01-abort-redo-contract.md).
No core implementation or shared SDK rebuild was performed in this review.
