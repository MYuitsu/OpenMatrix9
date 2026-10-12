# RCORE-01 — Native property journal staged merge, 2026-10-12T06:56:12+07:00

**Status: in_progress. Approved savepoint, Task2/3 and full RCORE-01 remain open.**

FreeCAD's Transaction now exposes a native PropertyMerge stage. Preparation captures
bounded source/destination record metadata and privately copies only missing replay
property values. Existing caller records, original first-write properties, status,
names and object ownership remain unchanged. A replacement object index and property
maps are allocated before transfer. Stale source/destination, wrong target or reused
stage reject without transferring. The final accepted transfer invokes no property
Copy/Paste, destruction, callbacks or allocation. Abandoned stages own only their new
property/record copies, never caller-owned detached objects. Source object creation/
deletion and property moves/renames are refused; existing caller operations remain.

The owned native journal cannot be reimplemented in Rust without violating FreeCAD's
property/transaction ownership. This specific approved native constraint justifies
the C++ API. Rust request, geometry, validation and results remain independent.
Both journals/property values must remain alive and exclusively protected; this API
does not yet provide the Document observer/lifetime barrier or private replay routing.

Old matched App19 native probe is **RED0/1** because this API is absent, actual host0,
no fixture exception. This is an API-absence test, not29 independently observed old
behavior failures. The separate unchanged production atomic432 fixture remains RED.
Native App117 build exits0. Initial native probe25/25 and extended probe **29/29** on
that same App test actual Transaction/Property first-write restoration, abandoned
staging, Copy exception, byte/record limits, unexpected larger copies, source/destination
staleness, wrong/reused stage, detached native New destruction exactly once, external
Del ownership, refused replay object/rename/move operations, no Copy of caller originals
and native dynamic-property addition/removal. The tracked class is a real native test
TransactionalObject; it is not an application-produced Part/source graph or visible
Document Undo/Redo certificate. Those acceptance gates remain in Document integration.

Unchanged common Curve result33/33 and Circle modes109/109 pass on the same new native
App with unchanged Runtime112 plugin: **171/171 final positive checks,three host exits0**.
Earlier25/25 is retained separately. Current atomic baseline **355/432**,48 callbacks/
observations,normal host0/no fixture exception proves History still lacks the new scope.
The historical owner-close timeout witness remains distinct; no fresh close test pass
is claimed. No Rust rerun; prior989/1 belongs to the earlier checkpoint.

Build reuses the single SDK object cache and only native App target dependencies;
DLL/import-library/PDB link output goes to an isolated native directory. Shared SDK
DLL/lib/PDB hashes remain identical. An unchanged small native test launcher is copied
and Gui hard-linked; shared modules/dependencies/runtime are reused. There is no root
configure, full FreeCAD build, complete runtime clone or user binary replacement.
Two failed native compiles115/116 and failed probe compile117 are preserved with their
diagnostics. Final native source candidates all match; they are not an exact compiler
dependency/ABI certificate. New probe119 imports the matching App library and loaded
DLL fingerprint; unchanged plugin does not use the new API. Matching plugin import
selection/production ABI and deployment remain separate future gates.

Remaining original requirements:

- Full original RCORE-01 remains open: stable GeometryRef identity/revision/representation/world, complete owned geometry/topology/attributes/units, adapters1–7, C01–C12, T01–T11, every enabled GeometryResult consumer, and independent read/edit/retain/export of all supported families.
- Initial Tangent now binds actual solved contact fractions to the retained owned selected curves, separately from probes; verifies circle/plane/D1/radial tangency/FromFirst/fixed-point null slots before session and common result publication. Checked analytic output and Deformable native spline basis are bound to the replayed plan. Live line/fixed/FromFirst/spline and genuinely nonplanar NURBS fixtures pass; this does not certify all native solver roots/families/options/backends. Generic CAD Edge/Face snap provenance, every other output consumer and complete old-format acceptance remain required. Raw old owned native_solution packets without contact metadata are rejected; historical saved result bytes remain until current recompute. Pure legacy numeric direct solution paths are retained and are not certified by this checked-contact output gate.
- Raw native offset D3 discrepancy remains enabled RED: independent mathematical derivatives match45 queries, native differs at36. No dependency patch/rebuild. Higher-order parity, extreme asymmetric weights, complete UV/retained trim/nested native chains and periodic parameter continuation beyond captured natural basis still require native acceptance evidence.
- Nineteen captured families/183-query synthetic tangent circles pass owned contact admission; live native nonplanar checks exercise retained contacts on actual spatial NURBS History. This is named fixture coverage, not all native solver roots, curve families/options/backends, global conversion or rigorous deviation bounds. The spatial archive is reopened in the same host; independent cold spatial contact acceptance remains separate.
- Native Transaction staged property merge is implemented and tested, but App::Document::PropertyReplayTransaction, private replay routing, observer/lifetime/history barriers, checked rollback/error cleanup, booking materialization and recipe-v2/legacy Circle integration remain absent. Current new App117 unchanged semantic fixture remains RED355/432,48 callbacks,normal host exit0. Standalone journal tests use real native Transaction/Property with a native tracked test object; they do not prove visible Document Undo/Redo, source/child graph ownership or full consumer/lifecycle semantics. The approved full savepoint contract remains required.
- Task2/3 remain open: generic snap/pick/mouse/render and parameter provenance; snapshot sharing, aggregate allocation/traversal/worker/cancellation bounds; Rebuild and every other enabled result consumer. Per-basis decoder bounds and sampled tests do not establish full aggregate budgets or lifecycle.
- Global conversion/deviation/locus exactness and complete parameter/topology maps remain unproven, with maps null. Full CAD-versus-mesh solid validity/orientation/manifold/cavity coverage, supported attributes and persistence families remain required.
- Fresh reconstructed Runtime93 packets retain genuine resolved fractions and all20 original recipes/points/plans/roles/probe/path parameters. Old Runtime59 fixtures remain immutable. Historical stored results may retain their old bytes; current recompute obtains new complete evidence. Raw legacy producer requests lacking contact metadata are rejected instead of bypassing verification. This does not replace all historical-format acceptance or production deployment requirements.
- Prebuild1503 candidate inputs preserve production candidates at postbuild/postnative checkpoints. Full inventory match is false: one spatial assertion wrapper was corrected after build to inspect initial evidence only while the result is ConstructCurve, avoiding subsequently changed History sources. After postnative, concurrent work changed/added rust/src/command_options.rs, rust/src/lib.rs, rust/tests/command_options.rs. Current entire production source is therefore not certified by Runtime112; selected contact sources/actual fixture bytes still match their launch checkpoint. This is not an exact compiler dependency certificate. Shared SDK/App/Gui/host/solver reused; no native root rebuild, deployment or Git publication.
- Journal admission bounds logical metadata/record traversal and native Property::getMemSize accounting before transfer; unusually larger copies are rejected after copying. Native allocator/container overhead, opaque kernel resources and peak Copy allocation are not fully certified. Both journals/property values must remain alive and exclusively protected; the future Document scope must provide that protection, including Copy/Paste/destruction callbacks. The staged API alone is not a complete atomic replay or worker memory-safety boundary.
- Only native App objects were rebuilt in the single SDK cache, with DLL/import-lib/PDB link outputs redirected. Shared SDK binaries remain byte-identical; one unchanged test launcher was copied and Gui hard-linked for the isolated App test. Current plugin Runtime112 is unchanged and does not call the new API; an explicit matching App SDK import override and complete production ABI/deployment acceptance still remain for Document integration. Rust was not rerun for this native-only component; prior989/1 is historical, not current whole-source certification.

Next: Implement App::Document::PropertyReplayTransaction and before-mutation observer/lifetime/history barriers with private property journal routing before caller booking materializes; use staged merge, checked rollback and protected invalid-output cleanup. Integrate Circle recipe-v2/legacy and matching native App import/host selection, then pass unchanged432/owner-close plus actual Document Undo/Redo/booking and Copy/Paste/cleanup callbacks. Preserve every original RCORE01 requirement; no scope or full goal completion from journal tests.

[Machine evidence](2026-10-12-rhino-core-native-property-merge.json).
