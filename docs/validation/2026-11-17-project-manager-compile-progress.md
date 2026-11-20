# Project Manager compilation progress

The implementation is still in progress; this is not the P07 acceptance batch.

On Windows, `rtk cargo check -p om9-app --lib --target-dir L:/om9-target -j 2` succeeded with 0 errors and 11 warnings after fixing two return-type errors in the exchange projection helper. The final check includes the catalog, snapshot, UI and native dialog code as of this record. It did not execute tests, native dialogs, GPU rendering or the application, and it does not establish behavior or acceptance. The plan defers those checks until P00–P06 are integrated.

Git blob hashes of key source files at this check, in order:

| Source | Git blob SHA-1 |
| --- | --- |
| `crates/om9-app/src/project_manager.rs` | `ebaafd254f1c79b870a80fc0f2ad75604254ae52` |
| `crates/om9-app/src/project_catalog.rs` | `b37bb8a747e2c860515716d7abcd691c48839709` |
| `crates/om9-app/src/ui/projects.rs` | `6af28958bbb10d3e4812b4168b70ffb92d0c2df2` |
| `crates/om9-app/src/session/project_state.rs` | `0b17a90e7bc930f854d7d45ae72da2a3fd3d3c0f` |
| `crates/om9-native-owner/src/file_dialog.rs` | `a29140cf941ab7e00db469d8f9934f7d5a5e9891` |

Further edits invalidate this as evidence of their compilation. The P07 scope and open limitations remain in the [plan](../plans/2026-11-17-project-manager-gap-plan.md) and live ledger.

## Follow-up compile

After the Mngr Files/range filters, retained-source refusal, worker-backed selection encoding and shared snapshot asset ownership changes, the same library check again returned 0 errors and 11 warnings. This remains a compile-only observation; no tests or runtime probes ran. Key source hashes at the follow-up check:

| Source | Git blob SHA-1 |
| --- | --- |
| `crates/om9-app/src/project_manager.rs` | `e34fb450b812dbcb14253217627f77d9a6b3f16e` |
| `crates/om9-app/src/project_catalog.rs` | `fff0c84645fa5e143d15bab4ca0693db9f485c1c` |
| `crates/om9-app/src/ui/projects.rs` | `eec94b3e877b9e8b02aed247368101ace0971c55` |
| `crates/om9-app/src/session/clipboard.rs` | `8d59fffb05c1673fba58f810fbdb4c9fef04b327` |
| `crates/om9-app/src/headless.rs` | `2f108755d074ce3f16a713013cf0eaac38763cda` |

## Later compile

The same library check again returned 0 errors and 11 warnings after adding worker-backed full-document snapshot encoding, shared selected-entity ownership, and customer/designer/status directory records. Tests and runtime remain unexecuted. Git blob hashes at this check were `4df7e919e3e22f4a5f456ab92c962bf8c94dd7c4` for `project_manager.rs`, `678ef953eea94ec8014a20b18ba48d40dcc76410` for `ui/projects.rs`, and `e0457cab0eb930736007c5804e7183f7ade92cf0` for `ui.rs`.

## Snapshot lifecycle and compact manifests

The same library check subsequently returned 0 errors and 11 warnings after deferring manager staging until Save is chosen in the named-snapshot Open prompt and omitting redundant selected geometry from newly archived bags. This is compilation only; Cancel, Save, source deletion and restoration assertions were added but not run. At that check the respective Git blob hashes were `20a04d7ac80c4532db70232aa44cd770141c9412` (`project_manager.rs`), `0f19f742cae33c6706cb76cc6a22b6770e2a09d8` (`ui/projects.rs`), `a150ccb792177e7b315df3656413f4d26ad359a8` (`ui/files.rs`) and `0211cf3f2ae58b6fc0428d45a5c0cd985fb2c8bd` (`session/files.rs`).

## Catalog worker and close guard

After moving catalog writes to a worker and adding a retry/discard close guard, the same library check again returned 0 errors and 11 warnings. No worker, failure, close or native UI behavior was exercised. Git blob hashes: `49fe17f93a1f4079de121934451584080e032f94` (`project_catalog.rs`), `2f9ffb0675779b183e10250c9339b4ccb0e456a4` (`project_manager.rs`), `8a222d5889fc3f42dcbc168beb4b63d2e63281aa` (`ui/projects.rs`), `01c923d50743c571dade5fa573401023c92fe251` (`ui/files.rs`) and `0211cf3f2ae58b6fc0428d45a5c0cd985fb2c8bd` (`session/files.rs`).

## Named-version overwrite and capture shutdown

The same library check later returned 0 errors and 11 warnings with named-version overwrite, source-root validation and background-worker close handling. This is compilation only. Git blob hashes were `956fc5af601fc0bb9a2b81f147aff4fdefbe9050` (`project_manager.rs`), `a6541d856ecde6a144ff48618e97bda603b67849` (`session/clipboard.rs`), `b19ea7a5bbbfc41a5799d17e64edb303c2b15e13` (`session/jobbag.rs`), `e76e26c141c14f0a6e3350b5221613b520dd0134` (`ui/projects.rs`), `15b5b44f036b908cf2903de4ac57b6d06d149387` (`ui.rs`) and `dc4a4beaf4f6011de8c33098ed20b9ce936cf291` (`ui/files.rs`).

## Versioned schema and import worker

The same library check later returned 0 errors and 11 warnings after adding schema migration, typed text filters and worker-backed project imports. The worker's cancellation, stale-state checks, resource bounds and read-only file handling remain untested. Git blob hashes were `46a1833c3008fba6f423c123d6e20a466163b37a` (`project_manager.rs`), `dd6cbc6791f36e1f8c8a5ed47efa478fca49f91d` (`project_catalog.rs`), `72998c416ce3a5ec540e6d1efa968b9a44c2bd4f` (`ui/projects.rs`), `7628d926a936cd0a6f649823718b710198ea1370` (`ui.rs`) and `b101268dbb5d0d98331aec96a6479ad4a34f6ce3` (`ui/files.rs`).

## Embedded metadata replacement guard

The library check returned 0 errors and 11 warnings after adding pre-publication checks for embedded manager schema and referenced resource metadata. At that check, Git blob hashes were b6d09267b6cfedf4f1ec7585e355d22ba0010223 (session/files.rs) and 79618228073bc7b8aad903e960669f0947b37a02 (session/project_state.rs). A regression assertion was added afterward and has not been compiled or run. The guard does not decode nested bag archives; integrity, memory accounting, and replacement behavior remain to be validated in P07.

## Selection root admission

The library check returned 0 errors and 11 warnings after replacing the fixed 64-root cap with configured object and estimated allocation limits plus a bounded manifest, and moving UI entity clones after capture admission. Git blob hashes at the check were `4ea2e8f0ab0e4cde2970d1bfb5e9b86f1ad9a409` (`project_manager.rs`), `216af3450efcd5f071fa9e8a771f47a6f4369c69` (`session/jobbag.rs`), and `2afed3dccf50eec716db7a1d57002d090ee0474b` (`ui/projects.rs`). A 65-object and constrained-budget regression assertion was added but not run; the library check does not compile test modules. Physical peak accounting is not established by this check.

## Embedded archive admission

The library check returned 0 errors and 11 warnings after extending the replacement guard to decode referenced archives with incremental codec admission and check root identities and named-document summaries. At that check the Git blob hashes were `76ae46572f393f0a72caeaf5bac24e54a523005b` (`om9-archive/src/lib.rs`), `e0f2b8c9dd68a56406ee24627ea1872eb9b97efa` (`project_manager.rs`), `93444761cf14b3f4eb6539c30cdb58128c6f8b2d` (`session/project_state.rs`), and `6f46e70ae50aabf65588a323957b7ec4c0439277` (`session/files.rs`). A corrupt-archive failure-atomicity assertion was added, but no tests or runtime checks were run. Legacy inline bags are outside this archive check; decoder reservations remain estimates and physical peak and responsiveness require P07 evidence.

## Catalog refresh archive validation

The same library check returned 0 errors and 11 warnings after routing catalog startup and refresh through the admitted archive and summary checks before replacing the current catalog. Git blob hashes were `97dab7f45c9f6a64c54cd7d491fe6bcf16749179` (`session/project_state.rs`) and `ff18fc835cdcce9e2658a93d01a4b6e69180527b` (`ui/projects.rs`). A corrupt-catalog refresh regression is present but has not been compiled or run. Catalog loading and archive validation still run synchronously and the catalog asset allocations themselves do not yet have complete shared admission accounting.

## Catalog loading allowance

The library check returned 0 errors and 11 warnings after adding progressive catalog manifest/asset admission and accounting for manager collections and metadata in worker estimates. Git blob hashes were `ea1af10da0b0a1c8dc18867122a1de3a5ffe4e5e` (`project_catalog.rs`), `046aa75365d1836dfa8e132935e987ce674fc9cc` (`project_manager.rs`), and `b60832b0d7e47cb2786f17084491b1ff71452069` (`ui/projects.rs`). An admission-refusal assertion was added but remains unrun. The reservation covers loading and validation only: retained catalog assets after publication, actual physical peak, and moving the load off the UI thread remain open.

## Background catalog load

The library check returned 0 errors and 11 warnings after moving catalog loading and archive validation to a cancellable worker with a shared admission peer, stale path/generation/revision checks, and a shutdown wait. Git blob hashes were `c48a09156249009d3529993e0d1de06e10c1de1d` (`ui/projects.rs`), `d81ac5e776f122661bd2411eef91837c37468b6a` (`ui.rs`), `84cd474052c3bd1e613f3b7d2bb9ec59564b04a8` (`ui/files.rs`), and `ad9ab8ed9e46dcd7dd19b39ae5c8ea410fe29a47` (`session/project_state.rs`). Updated corrupt-catalog and added stale-edit regressions remain uncompiled and unrun. Cancellation is checked between reads and codec admission phases; a single filesystem read is not interrupted. Loaded assets still need lifetime accounting after publication.

## Capture provenance and thumbnail retry

The library check returned 0 errors and 11 warnings after checking bag provenance against the decoded archive and rejecting stale capture input, and after allowing thumbnail scheduling to retry an admission or worker-start failure. Git blob hashes were `df3031a3143880d5474f1eb6db58c64e48e9a32f` (`project_manager.rs`) and `c5fbc121eb5d06e5225e4b41968943afe86d0f02` (`ui/projects.rs`). Updated source-identity and stale-selection assertions are unrun; thumbnail retry behavior and archive migration compatibility also await P07.

## Semantic history references

The library check returned 0 errors and 11 warnings after projecting clean semantic references with selected history and adding an atomic transaction mutation to remap and insert them. Git blob hashes were `bd5e4f7083cd36e589b08f43fc31b0750a49c5ac` (`om9-core/src/transaction.rs`), `461ca828d72417c3b17d4155dc486b6d0ed7c240` (`om9-exchange/src/lib.rs`), and `c7d0929df339fb897a776f47976fa23cfcf51930` (`session/clipboard.rs`). The extended clipboard and Job Bag Undo/Redo test at blob `7663e31e1ace895ae8c810c5cdb4a36f622cc328` was not compiled or run. This is not evidence for retained native-source or texture dependency closure.

## Duplicate manifest keys

The library check returned 0 errors and 11 warnings after adding bounded JSON preflight before Project Manager deserialization, so duplicate object keys cannot be silently overwritten in maps. Git blob hash: `6c6c68f1469fb9b136d829fefe9c024a57d9c4ff` (`project_manager.rs`). The duplicate named-version-key assertion is unrun, as are legacy manifest compatibility and peak-accounting checks. Retained-source projection remains open: retained-only records are output by the existing Preserve export mode, so indiscriminately retaining all unselected records would reintroduce content.

## Retained relation closure foundation

The library check returned 0 errors and 11 warnings after adding a bounded, cycle-safe resolver for transitive native-record relations from selected host bindings and using it in the current Job Bag refusal. Git blob hashes were `e5e8ea684e56dd6e1bde4873c619bd73ed3143f1` (`om9-core/src/retained.rs`) and `395bd49840f11bcb9811d1f4506de5b71b55ec6b` (`session/clipboard.rs`). A closure/cycle/missing-target assertion is unrun. This is groundwork only: selected retained sources are still refused until safe partial projection, identity/baseline remapping and atomic paste are implemented.

## Catalog read cancellation

The library check returned 0 errors and 11 warnings after catalog manifests and assets were read and hashed in bounded chunks with cancellation checks. Git blob hash: `498d3ddab734619f8b4c4b0d3cc5160ee76caa89` (`project_catalog.rs`). The extended cancellation assertion is unrun; an individual blocking OS read, manager deserialization and later asset hydration remain non-interruptible. Retained-source partial projection and durable asset lifetime accounting remain open.

## Retained inventory exclusion foundation

The library check returned 0 errors and 11 warnings after introducing an explicit excluded-record disposition and a partial retained inventory projection. Preserve-mode overlay maps excluded records to Omit; required records with an uncaptured live host still refuse projection. At the check, Git blob hashes were `2bf0211ae70a334e9b72967166aed4be8387ebaa` (`om9-core/src/retained.rs`), `caffb9dc186e799f436551324b56fb63acb64133` (`om9-core/src/provenance.rs`), `0c46bb9c49c33ca17c25abf8cc517d9cc046a3a5` (`om9-exchange/src/current_overlay.rs`), `93934a7cce0e94989a3b8c7a23110a73e6d48918` (`om9-plugin-host/src/staged.rs`) and `59db201cc37dcedc8e3c2e66c1c4a74ab9dcf134` (`om9-sdk/src/lib.rs`). The overlay assertion at blob `2f9218f534986a6c311166412c7bf6900e1f3304` was added after compilation; all tests remain unrun. This extends serialized retained metadata and needs compatibility and native runtime verification. Job Bag still refuses retained-source capture pending dependency-host inclusion, remapping and atomic paste.

## Bound native object closure

The library check returned 0 errors and 11 warnings after extending Job Bag selection closure to include live top-level objects reached through native relations, iterating again for their history and block dependencies. Git blob hash: `0c833010aabb1870eccf7378c67bf3cf85a4a9a3` (`session/clipboard.rs`). This is untested groundwork: missing required definitions/layers are not yet projected, and the UI still refuses retained-source capture until full remapping and atomic paste exist.

## Retained identity remapping foundation

The library check returned 0 errors and 11 warnings after adding a retained-selection remapper for resource, source, namespace and host identities. It rejects occupied or missing hosts, preserves native baselines and marks rebound hosts edited instead of claiming unchanged native content. Git blob hash: `4e6f7f440449378c6b5eef4240d9fa74a7e08f7d` (`om9-core/src/retained.rs`). The added remap/baseline assertion is unrun. The API is not connected to Job Bag capture or paste; legacy edited records without field baselines and native export behavior still need explicit verification, and full atomic transaction admission remains open.

## Retained transaction insertion foundation

The library check returned 0 errors and 11 warnings after adding preassigned retained identity remapping and a transaction mutation that checks resource/source/namespace identities, validates destination hosts after prior insertions, and stages retained state with the candidate document. Git blob hashes were `3bf1a826e4cda89fc4013981ae7102e04bbf5e93` (`om9-core/src/transaction.rs`) and `45a2304a743f1d79f9b06b1767f5f0a04f47b775` (`om9-core/src/retained.rs`). The extended transaction Undo/Redo assertion is uncompiled and unrun. This is not connected to Job Bag capture or paste; full admission, stale/failure behavior, native roundtrip and compatibility still need P07 evidence.

## Retained capture and paste integration

The library check returned 0 errors and 11 warnings after connecting partial retained-source projection to Job Bag capture and remapped retained insertion to paste. Bound layers get new destination identities to avoid binding an existing destination layer twice. Git blob hashes at the check were `93ed2584578b9e1c93b0c17f186ca0accd1a276c` (`om9-core/src/retained.rs`), `eec0e9257b20cf597575ced4a382c9c894c83348` (`om9-exchange/src/lib.rs`) and `35358a960e9b15cbfa33f5fc1c462c3968759d30` (`session/clipboard.rs`). The authored point/relation/resource Undo/Redo fixture at blob `64514c7de71df3aab572bc628005247b08ed9afa` was added afterward and is uncompiled and unrun. Required bound definitions or layers outside the projected hosts still refuse; native roundtrip, edited-baseline behavior, compatibility, memory admission and texture resources remain open.

## Native layer dependencies

The library check returned 0 errors and 11 warnings after including native-relation layer hosts, their ancestors and their exchange records in Job Bag projection and paste. Git blob hashes were `e25c7aff4e38cd959def8add1c2e13e67d5a6da4` (`session/clipboard.rs`) and `5e4332b5faad777140b6eccbc103eeafd23da138` (`tests/clipboard_om9.rs`). The extended authored relation/layer/resource/Undo/Redo fixture remains uncompiled and unrun. Required unreferenced definitions, native roundtrip, edited baselines, budget accounting and texture resources remain open.

## Native definition dependencies

The library check returned 0 errors and 11 warnings after expanding required native-bound definition/member hosts, nested definitions and member layers, and adding those definitions to the selection projection. Git blob hashes were `5f1852e32a99d330435ec83bd5138c7205f5d251` (`session/clipboard.rs`), `29e7c821067b7737ffe3aa2fa5f24667a527b0bf` (`om9-exchange/src/lib.rs`) and `0ec477497d787feafe0611538a904daafa026a67` (`tests/clipboard_om9.rs`). The extended authored definition/layer/resource/Undo/Redo fixture remains uncompiled and unrun; native roundtrip, edited baselines, memory accounting, texture resources and more complex definition relationships remain open.

## Explicit embedded texture resources

The library check returned 0 errors and 11 warnings after adding an explicit `om9-resource:<UUID>` texture reference, document-level resource validation, and transactional texture-resource remapping on copy/paste. Git blob hashes at the check were `b3efae91b46fd9bfb9473d45237e4c846eef0507` (`om9-core/src/model.rs`), `7906e5da26d20ab2e1fbaa25817071fd2b668ec2` (`om9-core/src/rhino_semantics.rs`) and `00e5fcbeca1e6223b1fe6ef5c1593cf5c0df9de2` (`session/clipboard.rs`). The fixture at blob `b0436aae75ba88757a511df37afb959136a8dc6e` was updated after compilation and remains unrun. Legacy external texture strings are not inferred to be embedded resources; renderer behavior, native export, memory accounting and compatibility remain unverified.

## Bag decode admission

The library check returned 0 errors and 11 warnings after adding incremental decoder reservations to bag insertion and named-snapshot restoration/export. The reservation includes input bytes when catalog assets have no retained owner accounting, and remains live through the UI consumer. Git blob hashes were `f483df1e9c029e6db595ca3ab3b4eef20a9566f9` (`session/project_state.rs`), `c264fc49259a4681f0d59f6947f7284b0a38cbfa` (`session/clipboard.rs`), `e2c016d0c9733e81cc059abd7e0254a79f30c5fe` (`ui.rs`) and `57f04afebe5073eb650a086f396917f0ee8aafe5` (`ui/projects.rs`). The admission-refusal assertion at test blob `c69408362c4fa246f09afcd9975a3e1792af563a` was added after the check and remains unrun. The public decode-only method transfers the returned document to its caller without a retained reservation; lifecycle integration, aggregate physical/GPU peak and cancellation remain to be verified in P07.

## Retained source layer context

The library check returned 0 errors and 11 warnings after selected native closure began including all original layer records for each contributing namespace, reflecting the native writer's existing refusal to remove a primary-source layer without closed settings and opaque-reference remapping. Git blob hashes were `2b628a834ca7a6f0730d593ebd7a8635e77e1757` (`om9-core/src/retained.rs`) and `3bc35daad26ef38d247c187eef60aa55e2ef1e1b` (`tests/clipboard_om9.rs`). The extended authored fixture remains uncompiled and unrun. This does not prove native publication: geometry-only mode, deleted/unknown layers, secondary native tables and serializer admission can still refuse.

## Bounded manager serialization and project import cancellation

The library check returned 0 errors and 11 warnings after manager JSON serialization was bounded during writing, project archive reads and hashes gained chunked cancellation, and nested import decoding gained cancellation checks before publication. Git blob hashes at the check were `3c3d3502b98cee1f6bcc441e3e660308719800cd` (`project_manager.rs`) and `db305986b4532b30cf1a5b39cfa35c3bebd3566e` (`ui/projects.rs`). The current `project_catalog.rs` blob is `2e79a86da9d1b7288e7b29517f1f666a489a2771`, changed after the check only by an unrun test assertion. Oversize/cancellation assertions remain uncompiled and unrun; individual OS reads, JSON deserialization, asset hydration and serialization cannot be interrupted mid-call. Worker peak estimates still require P07 validation.

## Manager schema error classification

The library check returned 0 errors and 11 warnings after typed Project Manager decode failures were classified as invalid schema instead of I/O failures. Git blob hash: `80895b401fa9a4bda4e30229fe1d46e32ed8d0b4` (`project_manager.rs`). The malformed-type assertion remains unrun. Persistent reservations for shared catalog assets still need lifetime-aware ownership; a frame-level estimate alone would not cover worker publication or later sharing.

## Shared project asset leases

The library check returned 0 errors and 11 warnings after UI capture, named capture, catalog load, project import and thumbnail publication began transferring worker allowances into weakly associated immutable-asset leases. Leases are reclaimed when the last strong asset owner is gone. Git blob hashes were `e94f1503deb0949a202f13937814be3e1b652a0f` (`ui.rs`) and `1dd13b7c9c37c9d8f442ad9c660e449cd4282c3c` (`ui/projects.rs`). The capture/shared-owner lifetime assertion is uncompiled and unrun. Embedded-manager startup and direct public manager mutations are not covered by these UI worker leases; metadata, GPU allocations, replacement accounting, duplicate charging and failure/cancellation behavior remain unverified.

## Embedded asset hydration ownership

The library check returned 0 errors and 11 warnings after embedded-manager hydration gained an admission reservation and transferred hydrated asset allowances into UI leases. Named-snapshot restore/export recognize an existing asset lease to avoid charging its encoded bytes twice. Git blob hashes were `4991c32efba3d86331e58ac8a45fc108626bd643` (`session/project_state.rs`), `70049fa14264e0c225b5602620cb1034c3df11ca` (`ui.rs`) and `11cf7782161abf587d14cd72d6f409fc980d92be` (`ui/projects.rs`). The extended startup/shared-owner assertion remains uncompiled and unrun. Public direct manager mutations and public decode-only ownership, metadata/GPU allocation, full replacement accounting and unfiltered/filtered insert double charging remain open.

## Leased bag insertion accounting

The library check returned 0 errors and 11 warnings after numbered-bag insertion began recognizing encoded input already covered by a lease in the current memory account; direct public insert calls retain the conservative full-input charge. Git blob hashes at the check were `5ffb933cfbc07eaf4601795f4618bac8813a10bb` (`session/clipboard.rs`), `6281fd2eeafa183568c725d08743227aa96a6eef` (`session/jobbag.rs`) and `26dea8ceea146490be3006c1b228dffb04fba630` (`session/project_state.rs`). The current `ui/projects.rs` blob is `406b10682194ac08a1e294a49707b2344b2d7c51`, changed after the check only by an unrun assertion. Lease-account matching, filtered insertion peak and cross-document replacement remain untested.

## Thumbnail worker shutdown

After resolving a borrow-checker error, the library check returned 0 errors and 11 warnings with thumbnail cancellation checkpoints and background result draining while the panel is hidden or the window is closing. Git blob hashes at the check were `94d545bf30bbd1bb22d34469c28eac8ba1627baf` (`headless.rs`), `c8105a15c13ff58cd3ecb85cd2a8d25caf39d936` (`ui/files.rs`) and `90efb182c139714d0bf3e302243e9a2e06700200` (`ui.rs`). The current `ui/projects.rs` blob is `3cf26c3c5ba517e1a5058c709ed1c3c31ff86fbb`, changed after the check by an unrun close/drain assertion. A single GPU initialization/render or image encode call is not interruptible; native window shutdown and GPU peak remain unverified.
