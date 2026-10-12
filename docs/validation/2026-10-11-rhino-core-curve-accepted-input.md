# RCORE-01 — Accepted Curve input, 2026-10-11T21:01:30+07:00

**Status: in_progress. Every original RCORE-01 requirement remains required.**

The six existing Curve commands reuse their current solvers and native adapters.
Safe Rust retains owned accepted values before normalization: exact text and unit,
frame/previous-point/pending-option context, original mouse point before Ortho,
native reference values, and ordered FitPoints batches. Circle checkpoints share
immutable owned log nodes; Undo restores the prior branch, removing an entire
accepted batch. Other commands retain explicit successful Undo events, matching
their existing semantics. Preview clones cannot append to the live branch.

The log has8192-event/8MiB/500000-node admission limits, evaluated before mutation;
raw text is limited to1024 bytes. Context-after reserves the full20-digit u64
identity before acceptance. Iterative chain destruction bounds Cancel stack use.
All retained numbers must be finite, including fields the solver ignores. Final
construction serialization retains the existing16MiB/1million-node/depth64 gate.
These gates are not proof of complete aggregate worker/native allocation bounds.

The role is explicitly `entrypoint_values_before_normalization_not_complete_native_sources`.
This is not a complete original GUI/native source record: OnCurve parameter text,
source object/subelement identity/revision/attributes/full snapshots and common
Curve GeometryResult/publication remain required. Constructor state still labels
normalized inputs and analytic display outlines separately.

## Evidence

- Runtime44 native RED16/24: geometry, one Undo/Redo and existing persistence
  already passed;8 failures demonstrated absent original values/context.
  True host exit0 and no fixture exception.
- Initial Rust RED0/5 to GREEN5/5 for dimensions, quantity text/frame/pending
  options, raw mouse/relative context, batch Undo and bounded repeated options.
- Review finding reproduced by real CurveSession methods: native Circle accepts
  a NaN ignored normal, RED5/6. Fix GREEN6/6 covers NaN and both infinities in
  Circle/Ellipse, ignored Ellipse point fields, unchanged prompt/log/context,
  subsequent valid geometry and strict JSON parsing. One intermediate radius
  assertion was corrected to tolerance for the existing fit solver's rounding.
- Full Rust844/0/0; standalone plugin Runtime45 build exit0. Pre-build source46
  inventory includes the finite fix and448 inputs. Non-Gumball/module-root inputs
  remain source-identical, including Curve consumers and fixtures. Parallel
  Gumball/Rust module-root edits invalidate full current-checkout/build matching;
  these are scoped Runtime45 results, not certification of the changed checkout.
  Shared SDK unchanged. Exact changed-source fingerprints are retained.
- Matching Runtime45:466/466 native checks across12 distinct actual
  host exits0. Accepted input24, accepted cold13, constructor42/cold19,
  basis26/cold27, Curve binding59, policy36, Circle History33, Circle context
  16, Solid61 and Edit110. Three independent cold hosts restore15 FCStd
  and compare exact metadata plus independently expected native geometry/basis.

## Review and remaining requirements

Fresh read-only review found one P2: encode_owned does not validate finite numbers, while native selection ignores some supplied point/normal fields. Added rejection-before-mutation test: RED5/6 to GREEN6/6. Fixed in Rust before solver/state/log mutation; invalid fields no longer produce non-JSON metadata. No additional actionable checkpoint/sharing/destruction/mouse/context finding. No second review used as a substitute for tests.

The first Circle context host could not import its mandatory helper. It ran no
checks and was excluded; its exact owned process exited -1 after explicit cleanup.
The serial retry copies the source-identical helper beside the unchanged fixture;
its12-host totals above do not include the excluded startup. Logs and terminal
receipt are preserved. No production code or public fixture was changed for it.

- Every original RCORE-01 requirement remains required: GeometryRef/Snapshot/Result, rational/periodic basis, UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- Common produced GeometryResult now enforces bytes16MiB/nodes1million/depth64 before serialization/caller-buffer copy. Source-envelope admission is still necessary only: complete request/output overhead reservation and aggregate allocation/traversal/worker/cancellation bounds remain required.
- Global conversion/deviation bounds and complete parameter/topology maps remain unproven. Valid closed native Solid is not a certificate of these contracts.
- Curve retains owned pre-normalization values at Rust entrypoints: exact text/units/frame/previous/pending options, original mouse points, native reference values and whole FitPoints batches with Circle branch Undo. Full native source identity/revision/attributes/snapshots, GUI-native parameter text such as OnCurve fractions, common ConstructCurve GeometryResult, checked publication, Ellipse foci, Circle History and Rebuild result pairing remain required.
- Offscreen evidence covers native kernel, command, Qt keys, callbacks and scoped FCStd persistence only. Actual mouse, OpenGL rendering and viewport acceptance remain required; historical graphics cohorts are not current reruns.
- The two cold hosts verify five named recipes and one indexed54-reference result, with OM9 imported by bootstrap. Unsignalled native root-frame mutations, actual Solid preview leases and complete original lifetime/cache invariants still require evidence.
- Production deployment/ABI acceptance remains pending. An isolated App19/camera-guard GUI host reuses the single shared SDK; no root FreeCAD build or full runtime copy was made.

The [machine checkpoint](2026-10-11-rhino-core-curve-accepted-input.json) records
source/binary/fixture/archive/launch/log hashes. Plugin SHA256 `421ee9c3e5fd42cf35a18572a2167e9bbdaa1ebad6d1a119235494d12cc9e02c`.
Earlier Runtime44 and full Rust843 before the finite fix are historical evidence,
not pooled into current totals. Offscreen does not certify real mouse/render.
Complete current-source matching requires a future plugin build and relevant
checks after the parallel Gumball edits stabilize; Runtime45 artifacts stay intact.

Next: capture complete admitted native FitPoints/path/tangent sources and original
native parameter input, then ConstructCurve GeometryResult and checked publication.
