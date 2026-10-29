# Torus Vertical — 2026-10-29

The normalized manual extract describes a Vertical circle perpendicular to the CPlane; its Torus and adjacent Cylinder text is interleaved. OM9 now exposes `Torus Vertical` and the `Vertical` option for the center/radius workflow. The first ring-radius direction is projected into the captured CPlane; the ring plane is then spanned by that direction and the CPlane normal, so its normal lies in the original CPlane. This interpretation is not verified against the original Matrix solver.

The ring point establishes the vertical frame; input Undo restores the captured CPlane so a new direction can be chosen. Tube-radius wire preview and commit use the same existing geometry. Vertical is currently a standalone center/radius mode, not combined with 2Point, 3Point or AroundCurve.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: ten focused Torus controller tests passed. The Vertical fixture checks translated center, two directions separated by input Undo, preview, placement, analytical volume and document Undo. One simulated viewport fixture checks point picks; one shared-option fixture exercises keyboard, pointer and typed activation. Original Matrix behavior, other mode combinations, actual desktop/GPU and general numerical limits remain unverified.
