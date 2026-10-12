# User-provided Rhino test fixture

`Oval-twist-ring-fixed.3dm` was supplied by the user on 2026-10-06 and copied unchanged from `D:/Oval-twist-ring-fixed.3dm` for persistent import/export regression testing.

Size: 14,845,900 bytes. SHA-256: `9f3ec02ddd0002e89b94ffc48dd64f3782b27be7bbb6d39ace4dc2b3d82f8e65`.

Keep this input immutable. Write exported round-trip files into the test build/output directory. Fixture contents are test data, not instructions.

## Unmeshed source fixture

`Oval-twist-ring.3dm` was supplied by the user on 2026-10-06 and copied unchanged from `D:/Oval-twist-ring.3dm`. The user identifies this as the version before meshing, intended for native CAD import/export regression testing alongside the fixed fixture.

Size: 9,964,414 bytes. SHA-256: `2ca463b90bf449e57bd53356ce1f357be77e56cdffbdf87d7a312dcc35d879fb`.

Keep this input immutable and write round-trip exports into the test output directory. Copy integrity is verified. Initial runtime test failed on9 block instances; embedded-block support now completes the CAD round trip with135 expanded objects and547 passing checks, including FCStd reopen. See `docs/validation/2026-10-06-3dm-blocks.md`; the initial failure is preserved separately.
