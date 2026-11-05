# Top11 Mirror: block instances and point clouds — 2026-10-31

Status: partial automated evidence; full Top11 and physical M03-UI remain open.

Mirror previously disabled block-only and cloud-only selections, and a block in a mixed selection failed in geometry baking. The core admission now recognizes instance and cloud kinds. For an instance, reflection is composed with its placement in world coordinates; Copy=Yes creates a new instance sharing the same definition, and Copy=No changes the existing placement. The controller commits placement and geometry replacements in one transaction; cloud reflection reuses the existing portable geometry operation. No new native code was required.

Controller fixtures cover nested shared definitions, CMD and toolbar, Copy=Yes/No, two picked plane points and CPlane Y axis, scene transforms, .om9 roundtrip and Undo/Redo. A mixed block/cloud Copy=No test checks the cloud's point, normal and color and one transaction. The core fixture checks three points under nonuniform scale/shear and an arbitrary world reflection plane. An egui test sends simulated viewport pointer events to Mirror a block and checks the reflected member location and Undo; it is not physical desktop acceptance or proof of GPU presentation.

Windows x64, CARGO_TARGET_DIR=L:\om9-target and -j 2: 61 selected core tests, complete scene suite 112 passed, app Top11/options/BRep/snapshot 197 passed, no-default Top11/options 112 passed, plus the focused viewport unit test. The core disabled-reason text was updated after the batch and the focused mirror core and UI tests passed on final source. No-default reports existing unrelated warnings. No native or openNURBS measurement was performed.

General rendering of reflected nested meshes/BReps, 3DM export, unsupported entity kinds and the physical M03-UI gate remain open; this evidence does not claim complete Top11 acceptance.

Source SHA-256:

- `crates/om9-core/src/top11.rs`: `b6b189f43fa1ff753bb3853fa01641b0232d543db096bad18a3c7d3988c25381`
- `crates/om9-core/tests/top11.rs`: `d2697e4ac340730cb0901206d32d416d00fafc5284542fc512a466f8738ac53b`
- `crates/om9-app/src/session/top11.rs`: `df15d2d0a1f25e90181cb410c22aca4d4f1eb4da65fae85bc8a43e70ae7639c3`
- `crates/om9-app/src/ui/chrome.rs`: `7821a5c4f0ead7de41c9c1a437a996046b3cf2dabda573c2e9f2fe615c709bd4`
- `crates/om9-app/tests/top11.rs`: `f51e935f7af8d60765f1e90e8b493a093ebfd56696484a5e2f0cdcb9fff2686a`
