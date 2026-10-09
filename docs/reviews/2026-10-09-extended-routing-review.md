# Extended coordinate grammar and phase routing review

Independent read-only review on 2026-10-09, against baseline `349203d` and the uncommitted increment in `H:/FreeCAD-src/Mod/OpenMatrix9`. Scope: `rust/src/{coordinates.rs,coordinate_input.rs,curve.rs,solid.rs}` and `rust/tests/{extended_coordinates.rs,coordinate_input.rs,extended_coordinate_sessions.rs}`. Read `AGENTS.md`, `C:/Users/nguye/.codex/RTK.md`, and `docs/superpowers/plans/2026-10-09-rhino-core-extended-input.md`. Reviewed dependency code only where needed to understand the scoped callers. This reviewer changed no production source, test source, Git state, or configuration; built no binaries and launched no UI. This report is the only written deliverable.

## Findings and repair disposition

### P2 — exact XY-elevation singularities after many turns: repaired

Initial `coordinates.rs:333` rejected a tangent singularity only when the rounded radian cosine was within `64 * f64::EPSILON` of zero. Exact odd quarter-turns at large, supported angles can exceed that allowance after degree conversion and trig argument reduction. Independent mathematical example: `w0.00001,0.00001<99990` has elevation `99990 % 180 = 90`, so the height is undefined. With the implementation's `99990 * (PI / 180)` conversion, the independent double-precision oracle produced `cos = -2.00652482910263e-14`, exceeding `64 * EPSILON = 1.4210854715202004e-14`, and height approximately `704807407.245276` mm. That finite height also passed the `1e9` mm final-coordinate bound. The input could therefore be accepted as an arbitrary distant point and serialized as valid geometry.

Correction requested: reject exact odd quarter-turns using validated degree-domain information before tangent evaluation; preserve the near-singularity and final-world checks. The parent reproduced RED, then added `units::angle_degrees` and `elevation_degrees.rem_euclid(180.) == 90.` before the existing cosine guard. This reviewer reread the repair and independently ran the existing rebuilt `extended_coordinates-450b7828ebb426e8.exe`: **8 tests passed**, including positive/negative multi-turn, DMS and fraction forms and a nearby nonsingular acceptance case. No further defect established in that repair. Native rejection/output-atomicity evidence remains the parent's gate.

### P2 — Sphere previous-point routing: inherited defects repaired

Initial `solid.rs:393` used `(phase != 1).then_some(data.last)` as the relative base. `data.last` is construction state and does not establish an accepted point. These defects also existed for Cartesian input at baseline; newly supported relative polar/spherical input exposed them.

1. `Sphere; FitPoints; rw5<90` incorrectly succeeded using `[0,0,0]` despite no accepted point. `Sphere; Tangent; Point; rw5<0` had the same invented base. A rejected first relative point must preserve phase/history; undoing the only accepted fit point must also remove its base.
2. `Sphere; 4Point; 5<180; 5<0; 5<90; wr5<90,5` retained the second point as base after accepting the third. Correct third point `(0,5,0)` plus offset `(0,5,5)` gives fourth point `(0,10,5)`, sphere center `(0,0,10)` and radius `sqrt(125)`. Stale second point `(5,0,0)` produces `(5,5,5)`, center `(0,0,5)` and radius `sqrt(50)`. The radius-constrained chord/orientation path has the same omission.

The parent added `Data::previous_accepted: Option<Point>`, initialized to `None`; successful typed points, native points/references and nonempty fit batches update it. Scalar construction continues using `data.last`; accepted-base state belongs to the same snapshots as Undo. This reviewer reread those success/error paths and independently ran the existing rebuilt `extended_coordinate_sessions-79fe6bd9cc55cb59.exe`: **13 tests passed**, including first-relative rejection after mode selection/Undo and the third-point FourPoint base. No further defect established in the reviewed repair. Parent-reported other Solid regressions and future native fixtures were not independently executed by this reviewer.

### P2 — Circle accepted Radius probe is absent from the next relative base: Rust repair verified

`curve.rs:675` resolves typed relative points from `self.points.last()`. A successful Circle Radius probe in `circle::Circle::pick` returns `Waiting` after `set_radius` without appending that point. The next relative orientation therefore uses the second circumference point instead of the last accepted Radius probe. This also existed for Cartesian input at baseline.

Independent analytic reproduction:

```rust
let mut s = CurveSession::default();
s.start("Circle").unwrap();
for text in ["3Point", "5<180", "5<0", "Radius", "5<90"] {
    s.input(text).unwrap();
}
// Accepted Radius probe is (0,5,0), radius = sqrt(50).
s.input("wr5<0<90").unwrap();
// Relative orientation should be (0,5,5).
// Correct center: (0,sqrt(12.5),sqrt(12.5)).
// Current stale base gives orientation (5,0,5), center (0,0,5).
```

Keep an accepted-point base distinct from construction points, and restore it consistently when Undo removes Radius/orientation input. Centre-mode Orientation also accepts a direction point without appending it; a later relative size point can resolve from the wrong base even where projection hides the radius difference. The reproduction was sent to the parent; custom Rust RED was not run by this reviewer because no test source/build was authorized.

Repair followup: the Circle implementation now owns `circle_previous` and bounded accepted-input snapshots, updating the base on successful typed/native/mouse points and fit batches without appending Radius/orientation probes to solver construction points. This reviewer reread the repair and independently executed **15/15 initial Circle context**, **11/11 Circle advanced**, and **16/16 Circle History** tests; the rebuilt context suite subsequently passed **16/16** after opaque native-context keys were added. The exact analytic probe-center regression now passes. See `2026-10-09-circle-input-context-review.md` for ownership/checkpoint analysis, exact repaired source hashes and native reference-restoration findings. Their repairs were independently source-reviewed. The reviewer directly read the parent's matching native **16/16 GREEN** replay, which covers selected path/Direction/Radius restoration, association, stale guards, cancel/restart, and the separately reproduced inherited F4 stale-source bypass. The relative-base defect is repaired in the Rust paths; this bounded Circle evidence does not close all P0 contracts.

## Contracts checked without another defect established

- `NumericSyntax` distinguishes leading `<` events from point syntax containing commas/`<`. Circle, Rectangle, Ellipse and Solid resolve recognized options first. Unsupported leading angle constraints return recoverable errors before accepting point/scalar input; pending Curve option/length state is retained.
- Explicit Cartesian, polar, spherical, XY-elevation and surveyor notation remains typed. `components_mm` contains evaluated Cartesian length components; radius/height/horizontal lengths and `angles_rad` are separate in `notation`. The additive report uses `grammar: rhino5_point_v2`, retains original text/context/result fields and does not represent angle values as millimeters.
- Prefix parsing preserves local `r`/`R`/`@`, absolute world `w` and general relative world `wr`. The notation-specific `rw` and `@w` branches reject unsupported permutations/Cartesian use. Rotated/translated CPlane tests demonstrate world bypass and local relative vectors without adding the plane origin twice.
- Finite/domain checks guard parsed components and resolved world points; negative radii, malformed numeric pieces, missing previous point, ambiguous extra separators and wrong angle dimensions reject. XY height and transformed world bounds are checked before accepted state/output.
- The shared resolver owns UTF-8 original text and snapshots. JSON controls are escaped; typed notation is additive; world sources carry `notation: null`. ABI size probes/short buffers and later invalid combined-report requests preserve numeric/report output atomicity. No host pointer is retained in `Resolved`.
- Circle/ellipse selected-center and tangent-constraint wrappers append actual points to construction vectors; Circle fit batches append their final accepted point. Their normal last-vector routing was inspected. This is not evidence that every native reference identity/phase is complete.
- The existing Solid length callback policy remains distinct: `mm`, `cm`, `in` and bare millimeters only. The new tests explicitly reject unsupported Solid feet/fractions/meters; this review does not claim that the shared parser migration closes the open Solid unit-policy contract.

## Executed evidence and limits

Existing test executables were run without rebuilding:

- Initial snapshot: `extended_coordinates` **7/7**, `extended_coordinate_sessions` **11/11**, `coordinate_input` **15/15**.
- After singularity/Solid repairs: `extended_coordinates` **8/8**, `extended_coordinate_sessions` **13/13**.
- Scoped `git diff 349203d --check` passed before repairs and at the final repaired source freeze; full-suite/build/final native checks remain the parent's responsibility.

### Final native evidence and Sphere fixture oracle review

The reviewer directly read the parent's final saved results and matching manifests; the reviewer did not launch these native runs. All use the final rebuilt native module SHA-256 `71a762ab77445d52ef113d3a578c65a7c79c177d2e31cdcd50901b2739078e45`:

| Replay | Independently read result | Fixture SHA-256 |
|---|---|---|
| `build/merge_replay-1/c7bb56dc826d47b7883713ae8af1730c/results.json` | Extended native input: **58/58 pass**, `ok: true` | `1d28073eb8ca9f3f9798a53e3cd64b928311334f6c3da78167e0342212e48873` |
| `build/merge_replay-1/a1f2a4e052724596b7b9233d7bad0945/results.json` | Cold restore: **4/4 pass**, exact polar report/dimensions/placement/embedded image retained | `1d28073eb8ca9f3f9798a53e3cd64b928311334f6c3da78167e0342212e48873` |
| `build/merge_replay-1/97245d1aa9a546c69c6a212d761b9b06/results.json` | Circle context: **16/16 pass**, final replay after rebuilt test-only FFI module move | `0858f9ceeae318b03b4285432f67148255779236f64e33643609e2f1823cd71d` |

The extended fixture covers typed notation/report fields and world geometry, rotated/translated CPlane, supported relative/world prefixes, explicit units, recoverable malformed/unsupported/singular input, Curve and Solid point routing, first-relative FitPoints rejection, native image Undo/Redo, and saved provenance. Parent reports exit 0 for these final runs.

The reviewer independently inspected the final `tests/core_extended_input_smoke.FCMacro:129` oracle correction. The prior diagnostic `build/merge_replay-1/7102c9b80ca746f2af59e50619b5a741/results.json` rejected Sphere FourPoint although it created exactly one object and transaction, with volume `4.188790204786391`, center of mass near zero, and analytic surface `Sphere (Radius : 1, Center : (0, 0, 0), Direction : (0, 0, 1))`. Its display-derived bounding center had X `-0.0009483356314779479`. That bounding-box center was therefore not an accurate oracle for the analytic sphere center.

The corrected fixture requires the native spherical surface center to be within `1e-8` of zero, its radius within `1e-8` of one, and the independent volume within `1e-8` of `4*pi/3`. This preserves the intended regression: `w1,0,0; w0,1,0; w-1,0,0; wr1,0,1` must resolve its fourth point from the third accepted point to `(0,0,1)`, giving center zero and radius one. The old second-point base yields `(1,1,1)`, center `(0,0,1)` and radius `sqrt(2)`, which still fails all three corrected analytic/volume assertions. No production repair or weakened accepted-base assertion is warranted by the bounding-box failure; the fixture-only oracle correction is sound. The final fixture hash was independently read and matches both native/cold manifests above.

The parent reports RED/GREEN for the repairs; this reviewer independently executed the GREEN Rust binaries listed above, reread the repaired source, and inspected the final native/cold evidence. Live constraints, locale, complete Solid units, common session reduction, snap identity and remaining P0 contracts stay open. This incremental review is not approval of all P0 completion or exhaustive Rhino runtime parity.
