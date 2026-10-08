# OM9-FILE-012 — CurveOnSurface SDK child archive prerequisite

Read-only SDK investigation and isolated diagnostic executables establish a
serialization prerequisite for the next adapter. This does not implement or
complete CurveOnSurface support. SDK source and runtime module remain unchanged.

`build/curve-on-surface-sdk-probe.cpp` constructs a valid closed dimension2
CurveOnSurface from a tagged Arc parameter curve and tagged identity NURBS
surface. A second case includes a separately tagged optional approximation
`m_c3` (dimension2 here, to match the native surface). Source geometry passes
IsValid/IsClosed/Dimension. Both raw payload Write/Read and whole-object
WriteObject/ReadObject are exercised at archive version50.

`rtk proxy cmd /c build/curve-on-surface-sdk-probe.cmd` exits0 after linking against
the existing unchanged static libraries for each SDK. Initial diagnostic links
lacked Windows user/GDI/shell libraries; those toolchain-only failures were
resolved before interpreting the probes. No SDK or production patch was made.

|SDK|Optional child|Write|Payload read|Whole-object read|Exact children|
|---|---|---|---|---|---|
|Pinned modern2425133316|Absent|true|true|1, valid|true|
|Pinned modern2425133316|Present|true|false|0, invalid|false|
|Independent201307115|Absent|true|true|1, valid|true|
|Independent201307115|Present|true|false|0, invalid|false|

Reports: `build/curve-on-surface-modern-probe.json` and
`build/curve-on-surface-legacy-probe.json` in the FreeCAD workspace.
The probe exits0 when it observes these expected diagnostic results; the failed
optional-child reads are not passing compatibility tests.

Both inspected `opennurbs_curveonsurface.cpp` implementations assign the
optional child to `m_c2` rather than `m_c3`, then clear the read-success flag.
The surface read is consequently skipped. The observed decoded record loses
the original parameter identity, optional pointer and surface. In the absent
case, ReadInt resets the flag and all required children are recovered.

The existing explicit CurveOnSurface preflight refusal must stay in place.
A full adapter needs exact parameter/approximation/surface schemas and a proven
archive strategy for this SDK path, preserving native child classes/domains/
userdata. Silently dropping the optional child or replacing the class with
NURBS is not an acceptable recovery. Independent SDK results do not establish
actual Rhino5 application behavior. Broader surfaces, precision, reference/plugin
data and nested model/Hatch archive recovery remain pending under full128/16/6.
