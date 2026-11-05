# Top11 Mirror plane scale

The Mirror three-point plane used a fixed absolute threshold on the raw cross product of its two edges. Consequently, individually valid perpendicular edges of length `1e-7` could be rejected because their cross product had length `1e-14`. The two-point path similarly rejected small axes. The plane calculation now normalizes each finite, nonzero direction with scale-safe component division before computing the plane normal; the existing angular degeneracy check still rejects collinear and sufficiently near-collinear directions. Preview and acceptance share this calculation.

The three-point controller regression failed before the change. It now checks point geometry and preview against an independent XY-plane reflection for six edge scales from `1e-100` to `1e100`, with all four sign combinations and Undo. The two-point fixture checks small and large axes with toolbar and CMD controller routes; a collinear three-point attempt is rejected without changing the document and can be retried. A real OCCT BRep fixture mirrors a placed box about a plane defined by `1e-9` edges, checking nonempty preview, closedness, volume, bounds and Undo.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`, runtime DLL path `L:\om9-target\debug\deps`: 146 Top11, 85 BRep and three command snapshot integration tests passed with default features; 146 Top11 tests passed without default features. Public source audit passed. No openNURBS measurement batch was performed.

The checks do not establish arbitrary near-collinear plane reliability, all geometry, actual mouse input or GPU presentation. The fixed angular cutoff is retained; this record does not claim an exact robust orientation predicate for extreme angles.

SHA-256: `session/top11.rs` `1e7401e4f55b0fb88bc3c4a26fe0f721ca8cc5721a5efe0692fbd05194839533`; `tests/top11.rs` `2e706ea6cb467323d541e52c83dc61c3819104a2d29cf506e0f2df0704855b54`; `tests/top11_brep.rs` `3a5e3482f8ee17809484d5d97b982e1418230a603f97bccec8138d746ffafd45`.
