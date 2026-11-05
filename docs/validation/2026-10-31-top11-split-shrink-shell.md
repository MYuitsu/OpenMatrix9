# Top11 Split: Shrink on connected BRep boundaries — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. This evidence covers OCCT operations and OM9 archive persistence, not an openNURBS measurement batch.

Previously Shrink was unavailable when a Split target had multiple BRep faces. It is now offered when every supporting surface is nonperiodic. Rust retains selection, transaction and Undo ownership; the OCCT adapter applies the surface modification to each complete resulting open shell, because shared edge and pcurve identities require the native topology API. The modifier preserves original continuity metadata and checks shape validity, area, topology counts and each mapped edge's adjacent faces. Intact solid inputs are not passed to the shrink adapter: Split first produces boundary components.

The joined two-face regression initially failed because Shrink was absent, then passed with the implementation. Fixtures cover coplanar joined faces cut across and along their seam, two perpendicular faces cut by a surface, and a box boundary split by a surface. The tested output retains the expected connected shells, faces, open edges and areas; support domains are bounded. Undo/Redo and a box `.om9` round trip are checked. A simulated viewport test covers selecting cutters with Shrink enabled. These examples do not prove arbitrary topology, continuity quality or full Matrix behavior.

Validation after the final topology guard (external target directory, `-j 2`):

- `cargo test -p om9-app --test top11 --test top11_options --test top11_brep --test command_snapshot`: 136 passed.
- `cargo test -p om9-app --lib split_polysurface_select_cutter_via_viewport`: 1 passed.
- `cargo test -p om9-app --no-default-features --test top11 --test top11_options --test command_snapshot`: 68 passed.
- `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10`: 69 passed.

Periodic supports are still disabled. General complex or non-manifold topology, independent G1 continuity fixtures, 3DM round trips, performance/budgets, SDK behavior, capping parity and physical M03-UI acceptance remain open. This does not establish completion of the other Top11 commands.

Source SHA-256 at validation:

- `crates/om9-app/src/session/top11.rs`: `14800084efe3b5cba0583fce967b22984dc4c510533eed9e6c5507943ecf0a49`
- `crates/om9-app/src/ui/chrome.rs`: `0a22fb9d4ea8dc20973b1dfc1e361e12752474488c10dc0afc8ad533e230319e`
- `crates/om9-app/tests/top11_brep.rs`: `96fdcfdd953fced2b19205469ab51977b4ab28cde8f34b5698e037c616d3049e`
- `crates/om9-kernel/src/measurement.rs`: `4e46501bd4ad8418704e2f9b548436c48da426b917cf02dc4c768235548c07c8`
- `crates/om9-kernel/src/measurement_bridge.cpp`: `cb813bee6d3b3e555c06f8a5de550da698a0160cecdd1a8e6d3ea63236a22cbe`
