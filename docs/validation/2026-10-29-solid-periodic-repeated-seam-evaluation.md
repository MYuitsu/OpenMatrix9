# Periodic NURBS repeated-seam evaluation — 2026-10-29

The evaluator previously selected an empty knot interval at the right boundary when the periodic seam knot had multiplicity greater than one. It now selects the last nonempty interval to the left. A certified cubic rational repeated-pole fixture checks equal endpoint position, D1 and D2 for a simple seam; D3 is refused. Forward/reversed exact OCCT extrusion produces a closed BRep whose volume agrees with independently sampled polygon area. A second fixture with double seam knot checks equal D0/D1 and rejects unqualified D2.

The second fixture still fails at OCCT snapshot with `BRep face has no tessellation`. The test explicitly retains this failure as an unresolved capability; it does not prove the curve is generally extrudable or that Slab can publish it.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`: `cargo test -j 2 -p om9-core --test curve_ops` passed 9; `cargo test -j 2 -p om9-core --test curve_oracles` passed 9; `cargo test -j 2 -p om9-kernel --features 3dm-brep --test extrusion` passed 3, including the expected snapshot failure; `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 31; and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 15. Broader periodic forms, seam parameter semantics, native tessellation, desktop and public Slab remain open.
