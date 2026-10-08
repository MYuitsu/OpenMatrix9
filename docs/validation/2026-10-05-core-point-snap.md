# Point Snap and shared point-input validation

Status: partial. Reference: local `specs/01-core/om9-snap-007-point-snap.md`; no PDF reread.

Original `Osnap P` uses Rust bit 8, independently of master bit 1, End bit 2 and Mid bit 4. Menu, CMD and the Point sidebar button share the same state and persisted preference. Sidebar check state is refreshed without re-entering commands. Existing encoded states remain valid.

The native extractor accepts vertex-only Part shapes and compounds; curve endpoints are excluded from Point mode. Native placements, resolved shapes and the existing visibility checks feed the shared projected nearest-candidate picker. Distance, Angle, PictureFrame and Curve point input use that picker before C-plane fallback. A completed Curve click consumes its corresponding release, preventing unintended source selection.

Rust: six core snap tests, release build and Clippy all targets with denied warnings passed. Native CoreSnapGeometry, CoreSnaps, Workbench, CurveController and MatrixSidebar compiled and the module linked.

Native evidence (isolated patched host, each process exited 0):

- Point: 12 checks at `build/point-snap-isolated-1/0923a40b8ed14a6d87b061ace85ec7e8/results.json`. Tests original menu/CMD/sidebar, exact spatial point, hidden-point exclusion despite a coincident curve endpoint, placement, nearest vertex in a compound, master-mode isolation, selection/Undo isolation and workbench reactivation.
- Point restart: five checks at `build/snap-restart-isolated-1/07621435da414026b066059c8032b914/results.json`. Separate process reads saved State=9; fixture does not seed preferences.
- Mid sidebar/persistence: 38 checks at `build/end-snap-isolated-1/a201488b20124f1a9161cfe3d3fef685/results.json`; four restart checks for saved State=5 at `build/snap-restart-isolated-1/62455409ba7b496384b5f2717d0ea0f5/results.json`.
- Shared Angle/Line/Polyline: seven checks at `build/snap-point-tools-isolated-1/f7701fb31e304bfcb8da55c80acbdb0b/results.json`, after a red Line geometry regression. Proves Mid elevation, shared menu/CMD/mouse geometry, release isolation, one Undo/Redo and master toggle during a pending tool.
- Mid PictureFrame: 39 checks at `build/picture-snap-isolated-1/8300a507c9ae4930a6530bb10c11f4b6/results.json`, including exact snapped first/second corners and Coin hover vertices.

Remaining: Points::Feature point clouds, custom point objects without Part Shape, more linked/nested instances and Point integration cases for every point-input command. These tests synthesize Qt input; physical desktop mouse interaction remains unverified. Most of the full 130 core specifications remain outstanding.

Latest post-Point End/Mid regression recheck is failing at the Mid sidebar mouse toggle: `build/end-snap-isolated-1/ebdc9930215341b981335bebda291827/results.json`. Application observation receives the press, but the button's own event filter only receives release; no clicked signal fires. CMD Mid, exact mouse geometry, menu state and sidebar check reflection passed before that failure. Explicit center coordinates did not resolve it. Root cause of the intercepted press is still open; historical 38-check evidence above must not be treated as proof that the current full regression passes. This does not invalidate the separate four-view grid zoom tests.
