# Top11 Join: reversed curve domains

Join sometimes reverses a segment to connect endpoints. The shared reversal routine previously reset line domains to `[0, length]` and circle domains to `[0,1]`, unlike its NURBS branch. It now retains the input domain while reversing endpoint order or angular direction. An initial regression failed with `[2,5]` changed to `[0,3]`; the corrected code preserves both the line `[2,5]` and circle `[7,12]` domains and checks five reversed fractional points for each.

An application fixture joins two oppositely directed lines with distinct custom domains, using both CMD and toolbar paths. It checks both domains and endpoints, `.om9` save/open and Undo/Redo. Windows x64 with `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`: 107 app Top11, 32 core Top11 and 204 core library tests passed. The focused Join test was rerun after adding the `.om9` assertion and passed.

This does not prove arbitrary reversed NURBS or closed curves, native 3DM transfer of noncontiguous polycurve domains, general affine ellipse fractional parameter fidelity, or physical desktop interaction. It is not an openNURBS measurement batch.
