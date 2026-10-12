# Shared O-Snaps implementation plan

Part of the full 01-core objective. Local End Snap source requires curve ends, polyline/joined-curve interior vertices, closed seams and surface/polysurface corners, with both End and master O-Snap enabled. Preserve original `Osnap E`; no PDF reread.

1. Rust owns master/End state and selection of nearest projected candidate within a cursor aperture. Master toggle preserves individual mode settings. Aperture is in logical display pixels; native layer converts/provides projected coordinates. Defaults and aperture range are explicit OM9 choices.
2. Native extracts world-space candidates from visible geometry, respecting placements, document ownership, hidden objects and supported source topology. Closed seams and surface corners must be covered; only open-edge endpoints are insufficient.
3. Native menu, mouse and CMD share mode state; expose/check states, persist user preferences. Point picker returns exact candidate world coordinate; C-plane fallback applies only when no enabled candidate qualifies. Distance, Angle and then existing Curve/PictureFrame point tools share it without broad refactor.
4. Verify pixel aperture at multiple DPI, nearest/ties, off state preserving modes, hidden/foreign/transformed geometry, joined/seamed curves and surfaces, no selection/model/Undo mutation, native measurement against real endpoints. Extend other 01-core snap modes separately from state support; do not claim their geometry implemented from toggles.
