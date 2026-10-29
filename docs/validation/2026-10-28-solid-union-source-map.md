# Solid Union: bounded native registration evidence

Scope: menu registration only, not reconstruction of the Boolean solver.

The recovered VB6 listing identified by the local reference registry as `code/vb6-lite` places the `SolidUnion` entry and the Rhino Boolean Union invocation in `modMillC_Defaults.bas`, procedure `Proc_198_35_1A2F840`. Its native procedure address is `01A2F840`; the entry and command strings occur in the bounded assembly near `01A35A3A` and `01A35A52`. The listing did not expose an `Attribute VB_Name` header in this file. In the registered Ghidra export, the definition `FUN_01a2f840` starts at line 462372; a separate hit at line 454609 is a call site. Address equality identifies the procedure, not its complete behavior. The inspected decompiler definition header and bounded assembly registration do not reveal the external Rhino command's geometry algorithm, all input stages, options, or error semantics. Those remain separate questions.

The OM9 implementation uses the local milestone's OCCT union capability and the existing Rust session/transaction owner. It does not copy recovered source into the product. The multi-selection behavior and actual target evidence are recorded in `docs/validation/2026-10-28-solid-audit.md`; the correspondence above alone is not runtime or UI acceptance evidence.
