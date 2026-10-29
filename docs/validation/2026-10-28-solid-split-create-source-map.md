# Boolean Split and Create Solid: bounded registration mapping

The recovered VB6 registration procedure `Proc_198_35_1A2F840` in the
registered `modMillC_Defaults.bas` associates `SolidBooleanSplit` with the
external `BooleanSplit` command at addresses near `01A35D3A` and
`01A35D52`; it associates `SolidCreateSolid` with `CreateSolid` near
`01A35DFA` and `01A35E12`. The same-address Ghidra definition is
`FUN_01a2f840`; the bounded registration establishes command identity, not
the external implementation, solver, stages or defaults.

The M03 Solid parameter records list both as missing and do not resolve
type/default/range/prompt. Existing OCCT `sew` and `sewed_solid` APIs
solve narrower sewing problems and are not proof of either complete command.
The current palette therefore shows both as unsupported rather than dispatching
them to a different solver. No recovered source or artwork is copied here.

## Further source comparison — Rhino 5 command help

The original target is Rhino-era Matrix; the public [Rhino 5 BooleanSplit
help](https://docs.mcneel.com/rhino/5/help/en-us/commands/booleansplit.htm)
describes two selection stages, shared/unshared pieces, and permits an object
to occur in both sets. It specifies `DeleteInput` Yes/No but does not state
a default on that page. The [Rhino 5 CreateSolid help](https://docs.mcneel.com/rhino/5/help/en-us/commands/createsolid.htm)
describes creating one or more closed regions from intersecting surfaces or
polysurfaces and `DeleteInput` Yes/No. Neither the recovered registration
nor these pages prove the Matrix wrapper's full runtime or defaults. Exact
topology/error policy, multi-output mapping and implementation gates still
need testing; current M03 records themselves remain incomplete.

Later on 2026-10-28, the closed single-solid BRep slice of Boolean Split was
enabled; Create Solid remains disabled. See the Solid audit for evidence and
remaining input, lifecycle and UI limitations.

The subsequent Create Solid checkpoint enables a partial intersecting-BRep
regions slice; its UI and topology limits are recorded in the Solid audit.
