# Independent review: Hatch current-loop input gate — 2026-10-10

**Conclusion:** No remaining blocker for this bounded native/host integration.
The final source and matching persisted runtime evidence establish that malformed
current-loop NURBS fields reject before the exercised archive-read, native-copy
and reconstruction routes. Existing valid typed-loop editing and export still
pass. This is a partial RCORE-01.T07 increment; the numbered all-family T07
acceptance and wider P0 contracts remain open.

Baseline: `32a72a6`. Reviewed the Hatch-input plan, changed native gateways,
Rust byte ABI as used by those gateways, native/host fixtures, existing typed-loop
adapter and persisted RED/GREEN/regression artifacts. The portable basis policy
has its own independent Rust review; this document concentrates on integration,
ordering, source ownership and evidence scope. This reviewer did not build, run
native/GUI tests, edit production/tests, mutate Git or dispatch agents. Only this
review document was written.

## Resolved blocking findings

**P1 — reading an omitted optional key inserted it.** The first new writer guard
used non-const `overlay["hatch_fields"]` before `validateNativeFields`. Qt's
mutable subscript creates the missing key with a null value. A plain unchanged
overlay consequently appeared to contain Hatch current fields and failed
`Native current fields require transform action`; other field kinds could also
become ambiguous. This was a production regression introduced by the guard,
not a malformed fixture expectation.

The retained `p0-hatch-native-results.json` has 13 checks and fails
`dependency bad basis before archive read` with that action error. Its exit
manifest records build 0/gate 1/CTest 8. The retained CTest log passes only
HatchCurrent; Merge and HatchLoops fail the action error, then HatchLoopEdit
cannot read the dependent fixture. The final guard uses
`overlay.value("hatch_fields")` and `.value("loops")`, preserving omitted keys.
Final gate and all four native regressions pass. The native fixture's plain
unchanged selected row plus malformed dependency/independent/later overlays
exercises the corrected omission behavior.

**Supported native depth was initially blocked by JSON nesting overhead.** The
shared JSON parser's default depth 64 rejected the supported nested curve tree
before the separate native curve-depth limit was reached. The retained portable
depth RED has 16 passes/one failure, exit 101. Hatch now uses bounded parser
depth 256, leaving the curve-tree depth limit 64 and the generic parser's default
64 unchanged. Depth GREEN passes 17 tests; the final suite adds finite tiny-weight
division-overflow coverage and passes all 18 Hatch tests. This correction does
not remove byte, node, curve or numeric limits.

Both findings are resolved in the inspected final sources. Earlier failed or
pre-fix SDK/native runs are retained as diagnostics, not accepted as final proof.

## Ordering and reuse

| Entry | Inspected ordering |
|---|---|
| `applyHatchLoopFields` | Whole current-loop Rust preflight, then native source validation, identity/class/source-index checks, curve construction, closure/topology and final staged assignment |
| `applyHatchFields` | Preflight optional loop fields before source validation and `ON_Hatch staged(*hatch)`; existing scalar/native pattern checks and loop adapter follow |
| Public `hatchBoundary3dm` | Bounded request parsing/basic source envelope, then loop preflight before `inspectArchive`, source geometry lookup/copy, unit/placement transform and derived boundary export |
| Selected archive writer | Preflight all requested selected/dependency rows and member-copy independent overlays before single-source inspection, multi-source merge, new-document preparation or geometry clone |

The writer normalizes `duplicate` to `duplicate_action` in its temporary row
before field checks. The whole top-level scan finishes before archive routes are
chosen; a later invalid row cannot be masked by an earlier valid row's native
reconstruction. Independent member overlays are scanned before their later
`Geometry()->Duplicate()` call. Both Rhino5 preservation and internal geometry
staging use the shared selected writer. All namespaces are scanned before the
multi-source path reads/prunes/copies archives. Source-code ordering also places
the scan before new-document creation; the new native negative fixture directly
executes selected, duplicate, dependency, independent-copy, later-row and
multi-source cases, rather than the new-document branch itself.

The native decoder and editor remain existing code. The guard does not construct
an ON/OCCT object or replace native closure, source ownership, metadata or
topology checks. After preflight, existing source-class/index restrictions still
apply, source userdata is copied only into the reconstructed child, and staged
loops undergo native validity, closure, self-intersection/containment and exact
requested-field checks before assignment. Native PolyCurve append retains
`unique_ptr` ownership until append succeeds. Existing source/archive identity,
digest/unit checks and final native reread precede the atomic output replacement.
The detached value dialog is unchanged; its existing host/native preflight is
called after the dialog stack returns.

The new input path therefore reuses the Hatch editor/constructor/topology/export
and shared Rust NURBS policy. C++ supplies native object calls and serialization;
Python supplies the acceptance/public-boundary invocation. No second geometry
solver or Hatch editor was introduced.

## ABI and numerical scope

`preflightHatchLoopFields` serializes the supplied object to a local compact UTF8
`QByteArray`, then synchronously passes pointer/length to `om9_hatch_loop_input`.
The byte storage stays live and immutable throughout the call. Rust returns a
numeric status, retains no pointer, and exports no allocation requiring a
cross-side free. C++ converts status into `ExchangeError`; the public Python
boundary catches native exceptions and sets a Python error. The final messages
distinguish invalid bytes, malformed/oversized input, schema/unsupported curve,
and invalid control points/weights/knots/domain.

Rust length/null/address-addition/UTF8 guards bound the input slice. They do not
prove arbitrary pointer accessibility; the native caller supplies actual owned
live bytes. Normal malformed-input paths use fallible parsing/validation/status
returns. This review does not certify OOM/panic recovery, arbitrary external
pointer lifetimes, every native exception path or whole-process memory safety.

The 2D current NURBS representation preserves a real boolean rational flag and
uses two nonrational CV values or three homogeneous rational values `(xw,yw,w)`.
Weights must be finite and positive, and dehomogenized coordinates must pass the
shared policy; a tiny finite weight causing division overflow rejects. Native
omitted outer knots are restored for the shared validator. Ghost poles remain
expanded nonperiodic observations, not a claim of compact periodic conversion.
Exact active domain is checked against the native knot interval before native
construction. Native source type and topology checks remain necessary after a
successful numerical preflight.

Limits are explicit: JSON 32 MiB/3 million parser nodes/depth 256; curve tree
16384 nodes/depth 64; loop count 1..16384; degree at most 25, CV count at most
500000, raw knots at most 1000000, shared per-basis storage policy 64 MiB and
coordinate bound `1e150`; the current-loop gate's numeric budget totals NURBS
CV/knot values up to 2 million. The existing native decoder also budgets its
supported non-NURBS values. The writer does not sum the Rust numeric budget
across every selected loop payload into one global process-allocation limit.

Qt JSON parsing/serialization and bounded Rust owned parsing/extraction allocate
before full value validation. This increment postpones native curve/Hatch
reconstruction, not every allocation literally. Arc/Line/Polyline fields,
source-index ownership, complete native schema and topology still use native
validation; a current-fields envelope without a loop object does not receive a
universal before-I/O schema guarantee. Raw archived NURBS/proxy/trim admission,
all-family worker allocations, handles/lifecycle, cancellation and aggregate
process memory remain separate work. Full T07 must not be marked verified.

## Runtime evidence

The original public-boundary RED is a real SDK-module run, not a stub: same macro
SHA `7b5e08e55d83886fbd983eeb76ad2f0dc50bbbae3779865532358ef57b7c5413`, old module
SHA `61708e51c37e9dc150d355b968a5be9ab29886858ae9ce0cea108fc2b87e5e09`, run
`7ab0806026354619863b6b4bd0094a07`. Wrong knot count loses precedence to
`file_size ... absent-source.3dm`; the first asserted ordering check fails and
`ok=false`. This RED is distinct from the introduced optional-key regression
and the parser-depth RED described above.

| Final persisted evidence | Inspected result |
|---|---|
| Native `p0-hatch-native-final-results.json` | 27 checks pass, empty error; invalid basis precedes source validation/archive read for all exercised row routes |
| Native final exit manifest | build 0, gate 0, CTest 0 |
| Final native CTest log | Merge, HatchCurrent, HatchLoops and HatchLoopEdit all pass |
| SDK final exit manifest | configure 0, build 0 |
| Full Rust final log/exit | 541 passed, no failing result; includes all 18 Hatch input tests; exit 0 |
| Host gate run `a74c75a8338a4b3ba157c8639025dbfd` | 13 passed, `ok=true`; all six malformed current NURBS cases reject before missing archive access |
| Existing host typed-loop run `a73d079ddc5b4d9a89d9225c27697adb` | 132 passed, `ok=true` on the same final module |

Final host module SHA is
`8b03c89907672d0579544a1fce934632c231ad5b70e23612fc09bd3c6b790598`;
both host manifests match current macros and the module file on disk. The
13-check fixture verifies wrong knot count, nonnumber CV, zero weight, unordered
knots, inconsistent active domain and rational tuple-width failure. Each case
preserves populated document object/Undo counts and sentinel CAD BRep, keeps
the staged byte sentinel unchanged, and creates no additional staged file.
The valid-basis control reaches the actual missing-source `file_size` error,
so the guard is not an unconditional rejection stub. Native output-byte
sentinels remain unchanged for the exercised selected writer failures.

The 132-check regression supplies the complementary successful path: mm/cm
typed NURBS/Arc/Polyline/PolyCurve loop edits retain exact current fields and
nested metadata, preserve original fixtures, resolve placement once, support
Undo/Redo, reject invalid edits/digest/schema/baseline changes, and retain copied
current-loop files independently. These checks establish the exercised source
reuse/editor behavior, not a universal owner lifecycle or fresh-process cold
reload contract. The malformed-input gate fixture itself intentionally has a
missing source and does not prove successful editing on its own.

Native/SDK/Rust exit claims above are independently backed by persisted exit
JSON. Host `ok=true`/all-check results and manifests were independently inspected;
the harness does not persist per-run process-exit manifests. No independent
second execution was performed by this reviewer. The recorded tool manifest
also has Clippy/Python exit 0; its coverage is supplemental and is not substituted
for the final native/SDK/host proofs.

## Fingerprints

SHA256 values were checked directly against the listed bytes. These freeze the
reviewed snapshot and evidence; mutable runtime executable paths are not a
promise that future builds will retain these hashes.

| File | SHA256 |
|---|---|
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/ThreeDmHatchLoopEdit.cpp` | `d3c038bccabbe2f93974fa553ddb8d5395d5966174473925ea29e1161480976d` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/ThreeDmHatch.cpp` | `6d037bd43f2cef83f82570a9bc688f836c8f85d32637ba387f9559e72251a70c` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/ThreeDmPython.cpp` | `52a81df116e41367fd006504bb8d677dd09c30e3f9e49551d3db52b6f8402a11` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/ThreeDmMerge.cpp` | `775fbc0ead88c3dd23606a640f00fe4dddc585b6bae3f40bf9af12a7d1539ac9` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/ThreeDmHatch.h` | `c2c61d22e984312783257b5ca2e13c4bc4e7a8845a606e599aaccd009c963196` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/RustBridge.h` | `c257949959623fb56b4be879eaa25d1fa83666d8ba3ab3d528f018d1307c2edb` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/tests/native/CMakeLists.txt` | `507104fefe3472b8bfe2c7dc8f82220d0a12a295851649d9e024a5ced731e2a9` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/rust/src/hatch_loop_input.rs` | `41bed3ca4109bd8f032820253b201b469f01f291a27933575f39ae60cd07261d` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/rust/src/builder_history.rs` | `836ebdfd672d8114fa421dcb2ab3173006b94a9618de7586a170b4b210d0b1a8` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/rust/src/lib.rs` | `7921d71840ad56279c252d79a327bc851a0e34431bac036e4e9a16f7798a4e2b` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/rust/tests/hatch_loop_input.rs` | `99361aed9d613b3270ef7c3ab02547bcddf86c9e0c97a0aeb46829a592f09f6c` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/tests/native/rhino_core_hatch_input_gate.cpp` | `54a9d74c14c00fe22a5bdae6e4a23e5131ec6865c65c9a74e073d052ff2788b4` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/tests/rhino_core_hatch_input_gate_smoke.FCMacro` | `7b5e08e55d83886fbd983eeb76ad2f0dc50bbbae3779865532358ef57b7c5413` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/tests/three_dm_hatch_loop_edit_smoke.FCMacro` | `9fb6b07327a160ceae93aa87e0c3e460e73bbd8dbfe91134bfbc3f3f10f842c2` |
| `H:/FreeCAD-src/build/p0-hatch-native-final-results.json` | `58ab92e62065354bce415f722cb2fadbc6217ea5d634840fcae866778165c100` |
| `H:/FreeCAD-src/build/p0-hatch-native-final-exits.json` | `11db90713accd2e22917cf047e0e5408368bbf320fdecf77e30e461480cba241` |
| `H:/FreeCAD-src/build/p0-hatch-native-final-ctest.log` | `b27e5c880039ed3cf5214db7c52a278ffc9b662cfb4706bfddd503c55597f705` |
| `H:/FreeCAD-src/build/p0-hatch-native-results.json` | `5560fc14d5655ff179926eaec982a60f45ba249e885beece42034646c928233b` |
| `H:/FreeCAD-src/build/p0-hatch-native-exits.json` | `3cdb1f83698e3524799b49ef37a930ccd3a7409e736eee3373f128147ad674f3` |
| `H:/FreeCAD-src/build/p0-hatch-native-ctest.log` | `1cbcb2385565052fa0a6391b324c405a4df5f9c2fff5db5b5c2fe9eb04fd1efd` |
| `H:/FreeCAD-src/build/p0-hatch-depth-red-rust.log` | `10a5fcb95ed9f6e23bd6eba1a6ffcdd83cc532990d4b68f56bba681aca3d95e7` |
| `H:/FreeCAD-src/build/p0-hatch-depth-red-exit.json` | `40f5db64ba3684bacc320ae8f5206b1312a76e2d915be838fe480866f80cff66` |
| `H:/FreeCAD-src/build/p0-hatch-depth-green-rust.log` | `4d4b5c2bbedb8b55728b78975fde9453e7b89cc2b488b11a660e4584db6a064e` |
| `H:/FreeCAD-src/build/p0-hatch-depth-green-exit.json` | `8e1c27e07362eb9798b515ea02b0dc3ef4682ff85da284264f0d210e38fd9710` |
| `H:/FreeCAD-src/build/p0-hatch-final-rust.log` | `5b212495df0682b0e3f3b5e62329bffcba8196ace91f7d4f367b9c4d27d52fbc` |
| `H:/FreeCAD-src/build/p0-hatch-final-exit.json` | `8e1c27e07362eb9798b515ea02b0dc3ef4682ff85da284264f0d210e38fd9710` |
| `H:/FreeCAD-src/build/p0-hatch-sdk-final-exits.json` | `b7c326b7db3d5c25dc367c8d81d5683469405523df279c416ff29d962ddfc7d1` |
| `H:/FreeCAD-src/build/p0-hatch-tool-exits.json` | `400c60c08ff2f168b7512205d121fecb35ba59f4ce500daa42783ceeccb16975` |
| `H:/FreeCAD-src/build/rcore-native/ThreeDmHatchInputGateTests.exe` | `4015deeed4e1cbbeb8034ae30a18cf880b4e54c9f18cf6ff934197446357b700` |
| `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | `8b03c89907672d0579544a1fce934632c231ad5b70e23612fc09bd3c6b790598` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7ab0806026354619863b6b4bd0094a07/results.json` | `3695a632de02ff7b7ab892d24525cba30fb2bf05652d84eb995d2742f861c256` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7ab0806026354619863b6b4bd0094a07/merge-native-module.json` | `3510e5d70ee498a69c45df813a852b69593b99a8999f89dbb5c24720fd8d392a` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/a74c75a8338a4b3ba157c8639025dbfd/results.json` | `d5740bc57367c37651292bf25d91a6b41efed3ce039a6abff8313ec4997741f6` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/a74c75a8338a4b3ba157c8639025dbfd/merge-native-module.json` | `dcd38d1f0e9085471e077c6778ff5e69cbf5061c681bf6869a7eec4408e9728b` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/a73d079ddc5b4d9a89d9225c27697adb/results.json` | `cfaeb8dd69a6d68c5409ba01858a4905a86aed0fd5ca5c3fcc92150c6b6a2e20` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/a73d079ddc5b4d9a89d9225c27697adb/merge-native-module.json` | `d3390f10508630612894ef1a7137b85e0655ad329d63f3646227ed7d4aa0f577` |

This report supports the bounded current-loop NURBS admission increment and
reuse of existing successful Hatch routes. It does not advance full T07,
History/reference-owner/worker contracts, every native allocation boundary or
the complete specs00 foundation to finished status.
