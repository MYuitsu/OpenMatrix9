# Tube base Area — 2026-10-29

The normalized Tube reference lists Radius, Diameter and Area for sizing the base. OM9 now advertises Area after choosing a Center or AroundCurve base center; an area entered in square document units is converted to the first circle radius. Radius and Diameter leave Area mode, and invalid or nonpositive values retain the session without a commit. The second circle and height follow the normal Tube flow.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: twelve Tube controller tests passed, including cm² conversion, switching modes, invalid area, AroundCurve combination, volume and Undo. Seven Tube option-route tests passed through keyboard, pointer and typed input; one simulated viewport Area workflow and two Truncated Cone area regressions passed. The first typed UI test failed because its fixture had not restored command-editor focus; the corrected fixture passed on rerun.

The reference is a two-column extraction; original Matrix prompt behavior, GPU rendering and physical desktop remain unverified. Other Tube and Solid work remains open.
