# Top11 Copy and Rotate direction scale

Copy treated a nonzero displacement of length at most `1e-12` as zero; its repeated-direction/distance options also used a normalization that rejected small vectors and squared coordinates that could underflow or overflow. Rotate used the same absolute direction-length threshold for reference points. Both failures were reproduced before this change.

Copy now distinguishes an exactly zero displacement from a representable nonzero one and normalizes reused directions with scale-safe component division. Distance uses `hypot` instead of summing squared components. Rotate normalizes reference vectors before projecting them into the rotation plane; the existing angular degeneracy guard remains in effect. Non-finite displacement is rejected before publishing a copy. Mirror's existing direction normalization reuses the same helper; its angular guard is unchanged.

Controller fixtures verify preview and actual Copy/Rotate results for signed vectors of `1e-200`, `1e-13` and `1e200`, both toolbar and CMD command invocation, and Copy's UseLastDirection/UseLastDistance options. A separate fixture submits tiny coordinates through CMD. Existing zero, axial-reference and retry regressions also pass. These tests do not prove physical keyboard/mouse interaction, GPU presentation or arbitrary near-axis numerical conditioning.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`, runtime DLL path `L:\om9-target\debug\deps`: 150 Top11, 85 BRep, three command snapshot and two UI replay integration tests passed with default features; 150 Top11 tests passed without default features. Public source audit passed. No openNURBS measurement batch was performed.

SHA-256: `session/top11.rs` `d68fc39e8dd90787c05d633a853507b31cc262b96577207e184b084589886431`; `tests/top11.rs` `81f241ae30b35b7b77ffbf4c324f78daaf3435220a322487369544eb1d5da710`.
