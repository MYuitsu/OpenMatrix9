# Boolean Two Objects: bounded native registration evidence

The locally registered VB6 listing associates `SolidBooleanTwoObjects` with
the Rhino `Boolean2Objects` command in `modMillC_Defaults.bas`, procedure
`Proc_198_35_1A2F840`. The native procedure address is `01A2F840`; command
and entry references occur near `01A35C7A` and `01A35C92`. The matching
Ghidra definition is `FUN_01a2f840`. This establishes registration, not the
external Rhino command's preview, solver, input, defaults or lifecycle.

The normalized local edit contract describes five result states and preview
cycling. Those are separate evidence from the bounded registration and remain
incompletely implemented in standalone OM9. See the Solid audit for the
specific tested slice and outstanding gaps.
