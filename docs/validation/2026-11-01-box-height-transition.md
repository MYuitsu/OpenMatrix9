# Box height transition — 2026-11-01

The user reported that immediately after picking the second Box base corner,
the preview jumped down. In a perspective camera, the height ray previously
intersected a vertical plane through the first base point. At the screen position
of the second corner that plane can yield a nonzero signed height. A regression
using a real perspective projection demonstrated the problem before the fix:
the stationary second corner at z=0 produced z=8.57 mm.

Corners, Diagonal and Center now anchor the height plane at the second corner,
projected onto the captured base plane. Center's numeric-dimension path falls
back to its first point when no second point exists. This maintains zero height
at the selected corner and allows signed positive or negative height by moving
the pointer. At zero height the base rectangle remains visible rather than
disappearing. ThreePoint, Vertical and Cube retain their prior height semantics.

Validation on the default OCCT target with `CARGO_TARGET_DIR=L:/om9-target`,
`-j 2`:

- `cargo test -p om9-app --lib box_`: 10 passed, including perspective rays in
  four signed base quadrants, positive and negative height, a rotated CPlane,
  and a simulated egui second click followed by stationary, up and down motion.
- 36 existing Box integration tests passed via a temporary lib-test harness;
  the separately running debug executable prevented Cargo relinking its normal
  binary. The temporary harness was removed.
- The pre-fix perspective ray test failed as above; the new one passes.
- `cargo build -j 2 --release -p om9-app --bin om9` succeeded; the release
  executable SHA-256 is
  `c532d5542cf2b6ca5717ce45ce356af29ab6b48a8cf82725b5eeaeafaafd591f`.
  The separate debug window stayed open.

The egui test supplies synthetic events; physical desktop pointer behaviour and
image parity are not yet verified. This is the user's requested OM9 fix and is
not a claim about the original Matrix implementation.
