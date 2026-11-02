# Extract Surface: bounded registration mapping

The recovered `modMillC_Defaults.bas` procedure `Proc_198_35_1A2F840`
registers `SolidExtractSurface` near `01A358C6`, the external command
`_ExtractSrf` near `01A358AA`, and the alternate `_ExtractSrf _Copy=Yes`
near `01A35895`. The corresponding Ghidra procedure is `FUN_01a2f840`.
These associations identify the primary and alternate command bindings; they
do not reveal the external solver or prove its persisted defaults.

The normalized M03 Extract Surface reference describes separating component
surfaces from a polysurface, a right-click Copy=Yes alternate and OutputLayer
Current/Input. The initial OM9 correction uses a numeric face index and
current output layer. A later checkpoint adds interactive layer choice and a
right-click Copy binding and a multi-face viewport selection path. General
topology and UI acceptance remain open. No recovered
source or artwork is reproduced here.
