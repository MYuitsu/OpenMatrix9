# Solid Intersection: bounded native registration evidence

Scope: menu registration only, not reconstruction of the Boolean solver.

The recovered VB6 listing identified by the local reference registry as `code/vb6-lite` associates `SolidIntersection` with the Rhino Boolean Intersection invocation in `modMillC_Defaults.bas`, procedure `Proc_198_35_1A2F840`. Its native address is `01A2F840`; the invocation and entry strings occur near `01A35BBA` and `01A35BD2`. In the registered Ghidra export, the definition `FUN_01a2f840` starts at line 462372; address equality identifies the registration procedure, not its complete behavior. The inspected definition header and bounded registration do not establish the external command's solver, selection stages, default options, or error semantics.

OM9 uses the local feature contract, OCCT operations and its Rust session/transaction owner. The historical OM9 `DeleteInput=No` default is a host decision, not evidence of Matrix's default. Runtime evidence and remaining limitations are recorded in `docs/validation/2026-10-28-solid-audit.md`. No recovered source is copied into the product.
