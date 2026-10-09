# OM9-SURFACE-001 — Sweep 1

Menu ID: `OM9_SurfaceSweepSweep1Rail`. CMD alias: `Sweep1`.
Verified supported slice: 2026-10-09; full feature remains partially implemented.

## Reference and implementation contract

Matrix 8 Book 1 PDF pp.180–183 (printed pp.170–173) were read through the
Sweep 1 continuation. The package now lives under
`Mod/OpenMatrix9/OpenMatrix9_Codex_Spec_v1`; the requested `ref/matrix9` prefix
is absent. Algorithm choices and bounds below are OM9 decisions.

- Rust owns identity, ordered selection, Chain Edges state and option policy.
  C++/Qt uses native Part/OpenCascade geometry; Python registers the workbench.
- Select one valid edge/connected wire rail, then profiles in order. Profiles
  must all be open or all closed. Explicit `EdgeN`/`WireN` references work.
  Reject duplicates, invalid/zero-length curves and whole faces/solids.
  Host limits: 256 inputs and 1024-byte reference keys.
- `ChainEdges` while selecting a rail collects touching edges into one logical
  rail. Enter validates connectivity; Undo removes its last edge; Cancel clears
  the command. All component subreferences persist in `SourceCurves`.
  Chain mouse selection retains the picked edge of a multi-edge object.
- CorrectedFrenet is the host default; Frenet is an explicit toggle.
  Independent Reverse/Flip and closed-profile numeric seams work.
  Single-edge fractions use the original curve parameter; multi-edge fractions
  use whole-wire arc length, splitting an edge when necessary. Geometry is
  copied independently before seam/kernel operations.
- Automatic compares profiles to the first current profile using centered,
  sampled directions and planar normal alignment. Closed profiles use 128
  candidates; the UI rounds seam fractions to four decimals. Degenerate and
  mismatched closure inputs fail. This is a host alignment heuristic.
  Natural restores original seams while retaining explicit Flip choices.
  Buttons and CMD tokens share these operations.
- Do Not Simplify is the default cross-section mode. Rebuild fits temporary
  copies with Rust, using 2–256 control points (initial host count 16), degree
  `min(3, count−1)`, and `max(513, 4*count+1)` arc-length samples.
  Closed profiles remain periodic. Rebuild is approximate; it provides no
  Refit tolerance or certified maximum error. Sources remain unchanged.
- Closed Sweep requires two or more profiles and closed rails in this host.
  The first section is appended to the native sweep sequence.
  Open-rail closure remains unsupported.
- Dynamic Preview is initially on; turning it off clears displayed preview.
  The Preview button/CMD token explicitly refreshes it. Geometry still validates
  before enabling OK. Preview creates no document object or undo entry.
- OK rebuilds a non-null valid uncapped surface/shell before one transaction.
  Errors disable OK and create no partial output. Esc/Cancel, document close/
  switch and workbench deactivation remove preview. Output retains sources,
  `OM9FeatureId`, `OM9Command`, `SourceCurves` and `SurfaceOptions`.
  Native Undo/Redo and FCStd preserve settings, source links and world placement.
  History OFF creates a snapshot. History ON creates a persistent native
  associative feature; see [advanced options](surface-advanced-options.md).

## Advanced options

Cross-section Refit now accepts a finite millimetre tolerance, builds an adaptive
Rust spline and checks continuous native chord/hull deviation bounds before
acceptance. The History checkbox recomputes after original curves or parent
placements change. CMD `gvSweepHistory` forces the native History variant
(OM9-SURFACE-002). Algorithms, numerical limits and lifecycle are documented in
[advanced options](surface-advanced-options.md).

## Remaining options

Road-like CPlane orientations, Global Shape Blending, trimmed/untrimmed miters,
Simple Sweep, Refit Rail, Point endpoints and dragged seam markers
remain unsupported. Frenet does not claim the Road-like
options. Full Matrix/Rhino compatibility remains unverified.

## Validation

Native tests cover cylinder/torus areas, closed Sweep with two sections, connected
and disconnected rail chains, grouped world placement, multi-edge seams,
Automatic/Natural on reversed periodic sources, open/periodic Rebuild, Preview,
failure recovery, Cancel, Undo/Redo and FCStd source/placement persistence.
See [the validation record](../validation/2026-10-09-surface-options.md).
Advanced continuation: [constraints, Refit, Slash and History validation](../validation/2026-10-09-surface-constraints-history.md).
