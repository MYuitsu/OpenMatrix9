# RCORE-01.T10 host parameter dependency — final independent review

Status: **bounded approval for the authored full analytic Circle/Line T10 fixture; no remaining review blockers in that scope**. Review date: 2026-10-10. This accepts the frozen implementation and the four normal-exit native runs below. It does not close all P0 chapters or certify general curve conversion.

Scope: `docs/superpowers/plans/2026-10-10-rhino-core-host-parameter-dependency.md`, copied Rust dependency policy/ABI, original native analytic extraction, owned target/query lineage, actual host sample binding, document/unit/world guards, creation/error/History/lifetime and FCStd boundaries. Base and observed HEAD: `ebdf5c1cfa3205f9ba1311cfec5f453a4cec0264`; implementation is in the working tree. Accepted module SHA-256: `e9f49c9e5b78814aa39da86110243cca05e11574f75c784c226a72de0f9a6d4f`.

The reviewer inspected frozen sources and saved execution outputs, independently rehashed the files and module, and verified JSON/check counts, process exits, distinct PIDs, module/macro bindings and geometry evidence. No production/test/build file was edited and no build, GUI or test execution was launched by the reviewer. The only reviewer write is this document. Passing statements below describe the inspected persisted executions, not a separate reviewer rerun.

## Resolved findings

| Finding | Final source and evidence | Disposition |
| --- | --- | --- |
| P2: two independently bounded error legs could accept a point beyond the requested host distance | `dependency_point` directly computes and bounds host-to-selected mapped/raw `host_point_residual`, distinct from host-to-ON `host_source_residual`; status 116 and a matching 64-byte ABI summary expose failure/result. The additive-error regression fails on the former policy and passes in the final suite. | Resolved for each inspected query. |
| P1: deleting Source left successful stale output/report, or Lock blocked FreeCAD link clearing | `onChanged` handles actual null Source/UnitSource property changes, clears both Shapes and report, retains Recorded/dirty/error semantics; `onBeforeChange` permits host Source link clearing while its target is removing. The fix does not depend on `onLostLinkToObject`, which this host does not invoke. Locked deletion, deletion Undo/Redo and unit deletion/recovery pass. | Resolved. |
| P2: transactional source/ancestor/group changes could skip the Undo/Redo refresh callback | Relevant transactional callbacks and `onChanged` set `PendingTransactionUpdate`; `onUndoRedoFinished` refreshes frame links, dirties/touches recorded features and invalidates missing inputs. Native source, query, parent placement, reparent and deletion Undo/Redo checks pass. Update Off keeps an explicit dirty flag. | Resolved within exercised native transactions. |
| P2: world-authored point was transformed twice in a placed output parent | Source and output ancestors are tracked. `setWorldPoint` resets local Placement and bakes the inverse actual output-parent frame into a copied vertex before assigning Shape. Part recompute placement synchronization therefore retains the intended world point. Independent source/output translation, rotation, grouping and Undo oracles pass. | Resolved within native rigid placement hierarchies. |
| P2: a feature removed by creation Undo retained active document callbacks | Both changed and recomputed callbacks require `containsObject(this)`. Connections disconnect during document reset, normal unsetup and derived destruction. Native creation Undo followed by group/placement/recompute work leaves the Undo-held record's Shapes/report/dirty state unchanged; Redo and close pass. | Resolved; no crash or use-after-free had been claimed. |
| Actual restore RED: `touch()` from `onDocumentRestored` was subsequently purged by host restore | Restored Recorded features are explicitly dirty; `mustExecute` honors `HistoryDirty` as well as touched state. The former 195-check restore failure is retained; final warm and fresh-process restore/recompute/edit/Undo checks pass, including Update Off followed by Update On. | Resolved. |

The former additive-error example used length tolerance `1e-7`, ON source `[0,0,0]`, mapped output `[7.5e-8,0,0]` and actual host `[-7.5e-8,0,0]`: both old legs passed while the returned point was `1.5e-7` from the host. The final direct bound rejects it. This is an enforced result bound rather than reliance on the sum of intermediate bounds.

## Source assessment

`evaluate` selects the original OCCT Edge support/domain through `BRep_Tool::Curve` before any NURBS conversion. Full Circle axes/radius/domain produce the analytic ON circle; Line endpoints/support/domain produce the analytic ON line. Trimmed circles and other supports explicitly reject. Edge location includes Part's Placement; the additional global-placement-times-local-inverse transform supplies its ancestors once. The independent native placement oracle supports this interpretation.

The existing `importCurveObserved` prepares one owned ON NURBS buffer, constructs the OCCT target from that buffer and evaluates the same actual target for forward/inverse/mapped/raw observations. The returned BRep uses that target and its observed domain. The actual original world OCCT sample is passed to the copied Rust policy; the selected result becomes the owned vertex. No fitted replacement curve, retained ON owner or borrowed native query row crosses the dependency ABI.

Rust validates form/domain/finite values/positive declared bounds/frame/method, actual forward and inverse observations, intermediate/native residuals and direct host bindings. Form2 rejects raw reuse even at a coincident seam; form1 grants raw reuse only for its validated individual query. FFI bool/reserved checks and copied values are present. The new consumer does not expand the existing sampled report into a continuous map or global deviation certificate.

Native capture/verification uses document UID, object name/subelement/native ID, process-local session, generation/revision/epoch, ancestor stamps, world placement and explicit document unit context. Unit context parsing rejects corrupt provenance; reports state millimetres and `FreeCAD_world_mm`. Verification follows conversion and precedes transaction creation; initialization evaluates again inside the single creation transaction. Invalid input rejects before creation, and initialization/recompute failures abort creation. Successful reports retain source identity, unit snapshot, original/mapped/inverse parameters, distinct mapped/raw/host residuals, source orientation, target basis and copied requested result.

Persistent native Source/UnitSource/ancestor links and recipe own the dependency. A numeric runtime session token alone is process-local diagnostic data and is not globally unique identity or fresh-process proof. Recorded recompute captures a fresh guard. Failure clears Shape, ConvertedShape and QueryReport and leaves dirty/error state. Update Off deliberately preserves owned frozen results with dirty state; Update On refreshes them. History Record/Lock/direct-edit detach integrate with existing native policy. The document-wide recomputed callback catches an errored source with nonnull Shape because this FreeCAD skips successful-object signals/descendants in that error branch. Source-error recovery passes.

Host source inspection establishes the relevant deletion, transaction, retained Undo object, Part Placement and native Shape restore/serialization paths. Normal lifecycle disconnection plus membership checks close the reviewed callback paths; this does not establish all allocation, concurrency or sanitizer properties.

## Accepted runtime evidence

All paths in the next table are under `build/merge_replay-1/`. Every accepted result has `ok=true`, every listed check succeeds, every launcher record has `wait_completed=true` and `exit_code=0`, and every native manifest matches the actual current module and final fixture hashes. All use the shared `H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe`. Manifests bind module and macro bytes; the source tables below separately record the frozen reviewed source bytes.

| Run folder | Fixture | Checks | Actual launcher PID |
| --- | --- | ---: | ---: |
| `ae32ae1dad1d4ffb92b9b1454f003e6d` | Authored warm host dependency | 205 | 76512 |
| `78c8485007a0439aa8a8b0029b511a28` | Fresh-process cold restore from the warm FCStd | 33 | 72132 |
| `7163602460754b4382cc230d464cea84` | Existing History controls | 83 | 2124 |
| `7eefb67170ab408085f8851ea9581249` | Existing native object lifecycle controls | 31 | 26740 |

This establishes **238 authored assertions + 114 existing controls = 352 native-host assertions**. Warm/cold start times and process artifacts establish separate executions; session integers alone do not. The full Rust log independently totals **556 passed**, exit 0, including 8 new dependency and 22 existing conversion/ABI tests. Clippy all-targets exit is **0 with existing repository warnings**, not zero warnings. The saved actual native converter control has **159 successful checks**, with build/run exit 0. Tools 50 run/49 passed/1 skipped is implementer-reported tool output only and is excluded from the 352 count and independent persisted acceptance.

For the radius-10 circle at original angle `0.31`, the saved actual map reports target parameter `0.3260076920042823` and inverse `0.3099999999999995`. The independent analytic host point is `[9.523335698857133, 3.050586364434435, 0]`; the retained native target at the mapped parameter gives `[9.523335698857137, 3.0505863644344298, 0]`. Their distance is `6.4047456679787536e-15 mm`. The same target at raw parameter `0.31` is displaced by `0.1591726790759251 mm`, and raw reuse is rejected. The Line form1 sample returns `[4.5,3,4]` through proven raw use. Maximum reported host-to-selected residual across valid warm/cold observations is `1.393761091115239e-14 mm`.

The authored fixture covers exact creation inventory/one Undo, 17 rejected operations with identical before/after native inventory/source BRep hashes/UndoRedo counts, finite/domain/tolerance failures, unsupported extraction, source/query edits and Undo/Redo, source/output ancestors and reparenting, explicit units, History Update/Lock/Record, direct-edit detach/Undo, null and nonnull errored sources, deletion under Lock, unit removal/recovery and Undo-held creation callbacks. Raw observed BRep files are retained and their stated fingerprints match the actual bytes.

Cold restore retains the warm saved report/query/target basis unchanged before recomputation, plus native source link/recipe/identity. Actual source and point BReps are byte-exact. Actual converted degree, knots, multiplicities, weights, domain and periodic state are exact. The actual converted BRep is **not byte-exact**: native ASCII persistence changes a few near-zero poles. Independent pole comparison yields maximum `2.8493378257508002e-18 mm`, below the declared `1e-12 mm` serialization bound. The raw saved/cold target hashes are `11049a4936a11ea848564041dfcee9e57bc63de1d3a0ad9ddff1001e0c184832` and `2346289b631c08d8ec9c458cfb8510e2534cefcb43e38bcb25197851d08d0cd2`; the unnormalized raw diff is retained. Sixty-five independent radial/plane target samples have maximum error `3.552713678800501e-15 mm`. This supports bounded numerical target persistence, not byte identity or a whole-curve certificate. Fresh recompute/edit/Undo and frozen Update Off/On restore pass.

## RED and excluded attempts

- `dba53aa53c1b48ab86772ca0d2ac422a`: actual missing-API RED on the prior module `e97cad9efd76181853d7ac70af874833f5dadfc47b7063fab809f99e0aecc680`, initial macro `31a875e6924730052f178e643f08fe4a4bc670a742f5a47e693106d4fa1bbf1b`. Normal process exit does not make its failed result acceptance.
- Rust policy stub: 1 passed/6 failed; the later direct host-bound regression also failed before its fix. Both behavioral RED logs are fingerprinted below. Initial SDK failures were missing complete OCCT type/explicit ON constructors, not behavioral RED or runtime acceptance.
- `d1c4ea290d054cc2b3e5d7f223bb10b0`: earlier basic four-check pass; superseded module/fixture, excluded.
- `d70d87fa2abe4553a624b1a08358da96`: fixture incorrectly treated a Box Edge Line as unsupported. `975dfbf058d847be9bb11661903b84e2`: fixture mishandled empty Python link representation. Both fixture errors were corrected and excluded.
- `56bb97536c634e3ba90bf064be05860e`: genuine restore-dirty defect after 195 checks, fixed by dirty-aware `mustExecute`; excluded from pass counts.
- `70c8437f056b433b8eba9359a86821fb`: 205 passing assertions but cleanup missed a restored document name, reached a Save prompt and the exact smoke-owned process PID 38592 was stopped; process exit -1. Excluded.
- `d9cbcfdb259b4aed8735e8a46b870b32` and `f23e45dbe7d749a1a2f0e51d22b9bf98`: earlier 205-check passes superseded by final macro newline/basis serialization corrections; excluded.
- `8c4502552c964dbcbcafcc269bceb302` and `d7047f7530e7468489b267461ab950bd`: cold byte-identity expectations failed on actual target ASCII rounding. The final fixture preserves raw differences and checks exact structure plus an explicit numerical bound; it does not normalize away those failures. Excluded.

## Final byte fingerprints

SHA-256 was independently read from actual files. Repo-relative paths are relative to `H:/FreeCAD-src/Mod/OpenMatrix9`. Absolute host paths identify the FreeCAD source used to inspect lifecycle, placement and persistence. Mutable plan checkboxes and validation/evidence documentation are outside this frozen source table.

| File | SHA-256 |
| --- | --- |
| `Gui/CurveParameterPoint.cpp` | `c6f491a380ace978f9b3824c6c98a75de9824dfb7861eeae634fcd7f2f7875ed` |
| `Gui/CurveParameterPoint.h` | `99ec12a3b18e50e3eb299cf9ad88cf00e0b5576cb33a01d180fadf84df95e966` |
| `Gui/CurveConversionContract.h` | `d6604efaf0b30cf61d201f984633c2c844162290c74c576df698f27cfd9c7a74` |
| `Gui/CurveConversionContract.cpp` | `5c950c807e3b7aa2537b9919d4589c6fb0b7521fb462a22ff2d76a6a2dbe0e03` |
| `Gui/ThreeDmCurves.cpp` | `4c65cb8baf9e194c5888125da7ce63841f132ada867767237cd136c6ea457669` |
| `Gui/AppOpenMatrix9Gui.cpp` | `88b52edab835accbb22db0c44930f7cbd56d0e8a380f2be551c780bbcc4900da` |
| `Gui/CMakeLists.txt` | `73167d03ec4ded4b14ad80cd41e7730eebbe558d0d883dba7bda964694a10d45` |
| `Gui/HistoryFeature.cpp` | `821914fc271c76c855c6cb9c2485c1836c7a388f9afb8a17b331c51581e31de5` |
| `Gui/CoreObjects.cpp` | `fedbdfba5593f89d5b4bf2f6148aabd58112e4f2c7bb5cc43e105d3c8acaa98e` |
| `Gui/CoreObjects.h` | `3800b6922ad57be6bd4bb2a93a7ba8c034f8ee16a610637bf955a80c496feb03` |
| `Gui/CoreUnits.cpp` | `43aa730498e4e509c773e0e2eec33a59754e173e66a75a13707aad2f2185a101` |
| `rust/src/curve_conversion.rs` | `db3c0a42d4ca8fa67e7cb6d3265d7236c50b596cc29b429d00eddb71b45b4e5e` |
| `rust/src/curve_conversion_ffi.rs` | `ced17dac9bb3fed9377dbba4163065ca59039bcccaa7b1c654916db86d88e741` |
| `rust/src/core_objects.rs` | `6af0c1de8b6cef78b009d88c1ca3012e868179b28f10cd8a17587de6b289024f` |
| `rust/tests/curve_parameter_dependency.rs` | `08b3018c70dc948ee597efa306ed91860f661299128d46844573d7776c5c54e4` |
| `rust/tests/curve_conversion.rs` | `48646ec56f8754352930debc0401492e7d92d50a7829221bc12630486366a590` |
| `rust/tests/curve_conversion_ffi.rs` | `e4a6d70923215455ad2654781964f7d0c8c19e82ffa927f8a8344cebfed7a665` |
| `tests/rhino_core_host_parameter_dependency_smoke.FCMacro` | `aff75cd72e655c68b65004a5390bdf7d9bcc0b4021d7b5982046e40565455f95` |
| `tests/history_smoke.FCMacro` | `fbe0de2b3fd041563c892b8408ea5bc46e9d4bdadf78e42b3d13f898a6b65c6f` |
| `tests/core_object_lifecycle_smoke.FCMacro` | `3ea583a03bbd049452ddde14e92da1c699d607e4b4063f785f08c4314004e721` |
| `H:/FreeCAD-src/src/App/Document.cpp` | `eab191b4e5fea89e804670bb7ccac17d7f5393a5642289b428234a9d26613f94` |
| `H:/FreeCAD-src/src/App/Document.h` | `e80203c5fd612e2ffea460e3edee48bad0666258af6e666b70e9fad42a8a6e4a` |
| `H:/FreeCAD-src/src/App/DocumentObject.cpp` | `c5952938af681e47b40241a9792d3deb2a747aae5f93014afcc73815da62b217` |
| `H:/FreeCAD-src/src/App/DocumentObject.h` | `12b7b5260b1a40e60817e7c02546a56e9132cdb94a1196080c39aaf1e5282b33` |
| `H:/FreeCAD-src/src/App/Transactions.cpp` | `086a785f965ae366b8c6c9f03b98b59e8982ab01866bbcd08daf47fbc10bba75` |
| `H:/FreeCAD-src/src/App/PropertyLinks.cpp` | `2ce66fed0ad5209913c4eae47eff1253303c8e3fd7c4450617ef97b0f5ad7638` |
| `H:/FreeCAD-src/src/App/PropertyLinks.h` | `0576661d1645c13054052f4da0d230a8873d56e3be163dce0c529bafbe6709fc` |
| `H:/FreeCAD-src/src/App/GeoFeature.cpp` | `afbdc2ec56bfcf63cc0f860bb25f390b202c64cb5b8c3af8715ca52e06ec1ccc` |
| `H:/FreeCAD-src/src/App/GeoFeatureGroupExtension.cpp` | `deaedb1fcab832ff91786c34f62e918eb1c9cbd38bbb60b30467ef906dd7dd8c` |
| `H:/FreeCAD-src/src/Mod/Part/App/PartFeature.cpp` | `f75b38817bac5a46860c956ea78e76d3561815ad6390bb9f521eb86e2e6056fd` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShape.cpp` | `2f7de58f556e3ce2cb0877ebdb1a83536d1e73be2d8a2b9243056083f0619e5e` |
| `H:/FreeCAD-src/src/Mod/Part/App/PropertyTopoShape.cpp` | `da3e5682155e33872c742adeaf6798c376d328239065abe9d16663a191c52ee7` |

Accepted binary, launcher, result, manifest, persistence and numerical-control records:

| File | SHA-256 |
| --- | --- |
| `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | `e9f49c9e5b78814aa39da86110243cca05e11574f75c784c226a72de0f9a6d4f` |
| `build/merge_replay-1/ae32ae1dad1d4ffb92b9b1454f003e6d/process.json` | `90c847cfce380bb9acf6a440cf310edf8e218d9d39d4b3c7e6da011950ff3750` |
| `build/merge_replay-1/ae32ae1dad1d4ffb92b9b1454f003e6d/results.json` | `484495f5217fd4186e058e8187c55ceba1e4f0c1874bce53af4064ee9e954422` |
| `build/merge_replay-1/ae32ae1dad1d4ffb92b9b1454f003e6d/merge-native-module.json` | `148d9efcde4c1566df44adaea75b639f8bdcd526a647d3beddc989fde33d764a` |
| `build/merge_replay-1/78c8485007a0439aa8a8b0029b511a28/process.json` | `43ad15aba9e8c65084d86a013580b4e25996c2c33783f118e2478677c825dcab` |
| `build/merge_replay-1/78c8485007a0439aa8a8b0029b511a28/results.json` | `e1875c3f5d731481f4a39893c83bc1d81547595b9f8fe3cc09c46f8d3754658b` |
| `build/merge_replay-1/78c8485007a0439aa8a8b0029b511a28/merge-native-module.json` | `148d9efcde4c1566df44adaea75b639f8bdcd526a647d3beddc989fde33d764a` |
| `build/merge_replay-1/7163602460754b4382cc230d464cea84/process.json` | `466d6202ffc204b11332ec267bd80922b59d7a52f0687cac99b247e9ad1ebd16` |
| `build/merge_replay-1/7163602460754b4382cc230d464cea84/results.json` | `819b78a0632e02ad40de31a3e7dd03527ea86e911b6a44aeb95319d16d47e7be` |
| `build/merge_replay-1/7163602460754b4382cc230d464cea84/merge-native-module.json` | `849e0959ecb74f1aa83536eeece7a17fdd664bbc6358707b64a413e0b906157d` |
| `build/merge_replay-1/7eefb67170ab408085f8851ea9581249/process.json` | `49a3151886119683cf1c5035279eedbc7a71e80c6b7f0678aaeac9ce93767fda` |
| `build/merge_replay-1/7eefb67170ab408085f8851ea9581249/results.json` | `222b47b49cf22f2252d7d0ed3b6b48b51c37d63372a5e825fc13eae17ab2b8b6` |
| `build/merge_replay-1/7eefb67170ab408085f8851ea9581249/merge-native-module.json` | `1f1b20836f0041516ba7ed4ae9d1091db351150d0155c99aa92eff9552739a57` |
| `build/merge_replay-1/ae32ae1dad1d4ffb92b9b1454f003e6d/host-parameter.FCStd` | `ce934273f48dba2318fbf8edffa4d9454be5488abbbbecc7e8c4be9edff5c730` |
| `build/merge_replay-1/ae32ae1dad1d4ffb92b9b1454f003e6d/host-parameter-frozen.FCStd` | `671fd619b523c07504ef7ad063889a66cddc228a86a00f21b9728c63ad849229` |
| `build/merge_replay-1/ae32ae1dad1d4ffb92b9b1454f003e6d/host-parameter-persistence.json` | `eff679db5079472ef632dabc8efb1016182d936f05b080095751e058087d660a` |
| `H:/FreeCAD-src/build/p0-host-parameter-rust-results.json` | `cb2e03855a388503299aacd0732b2112c3faa0bdc8227c7b0ecca139cfb2a23b` |
| `H:/FreeCAD-src/build/p0-host-parameter-rust-tests.log` | `45739f992bea59452887c74cc58cabff2517b3c50ba5e8e3eec78bba9ec535b2` |
| `H:/FreeCAD-src/build/p0-host-parameter-clippy.log` | `f5c79be93a22974dced490d21450561a4b4c23ab9392f8d6774c6d0e9b01696f` |
| `H:/FreeCAD-src/build/p0-host-parameter-red.log` | `eb91730121b6c81ec4311e2263e57b8ba021fd021be880b7a16a0a1a321fbdaf` |
| `H:/FreeCAD-src/build/p0-host-parameter-host-bound-red.log` | `63c3d0bb51eb53e1d1db28dd428fda71ce71ee7094bea3f49672427a2a7f8b54` |
| `H:/FreeCAD-src/build/p0-curve-parameters-native-host-dependency-results.json` | `e26fe3b7a29800f15144411bfe360da197549aa937b048b5ddb86688f191d471` |
| `H:/FreeCAD-src/build/p0-curve-parameters-native-host-dependency-exit.json` | `bcd761c4a95ca0a7e4603f7b70ede69a9af33a1b9796c72d4b697f03a7c1c129` |

## Acceptance boundaries

Approval is limited to the authored full analytic Circle and Line host extraction, requested point policy and exercised native History/lifecycle/warm/cold persistence paths on this frozen module. Trimmed circles, arbitrary original NURBS/other analytic families, composed reversal/subdomain parameter maps and general extraction remain unsupported. Fixture length/parameter bounds are `1e-7 mm` and `1e-10`; numerical persistence allows the separately declared `1e-12 mm` pole round-trip bound. Those bounds and discrete observations are not global guarantees.

This is no continuous parameter map, certified global locus/deviation, universal byte-exact target restore, all-worker/thread/sanitizer/all-allocation proof or native SDK/FFI memory-safety certification. Cold target evidence concerns the authored saved Circle case; warm ancestor checks do not make every possible saved hierarchy certified. Rhino 5 runtime parity remains unavailable from the recorded executable crash. General T07 input/ownership/allocation boundaries and other P0 chapters remain required. **All-P0 remains in progress.**
