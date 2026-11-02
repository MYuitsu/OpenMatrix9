# Authorized model-space annotation owner (2026-10-27)

User explicitly authorized implementing all thirteen remaining Measure catalog commands.
New owner: OM9-L2-03-21, with L3-001..013 mapping in order to catalog suffixes
004,006,007,008,009,010,011,012,013,014,015,016,017. These are model-space
annotations; paper/layout drafting is outside this owner. Original catalog ids/icons remain.

Safe Rust owns versioned `om9.annotation` records, witness coordinates, dimension values,
text placement, style and derived geometry. Existing transactions own atomic creation/edit,
Undo and native OM9 serialization. There is no native annotation API dependency.

Horizontal/vertical/aligned/rotated dimensions take two witnesses and a text position in
the captured CPlane. Rotated also exposes Angle. Angular takes vertex, two ray points,
then text/radius position and reports the smaller angle. Radial/diameter take one selected
circle/arc and a text position. Leader takes tip, elbow, text position and text; Text Block
takes a position and text. All creation previews are read-only, with the same layout as commit.

Edit Dimension changes witnesses/text position/override and style on one selected dimension.
Edit Text edits one selected Text Block/Leader. Recenter moves dimension text to its measured
line/arc midpoint. Dimension Options edits the selected annotation style, or session defaults
when nothing is selected. Each committed object edit is one Undo; session defaults are not
document edits. Every annotation stores its own style and survives native OM9 serialization.

Defaults (OM9 decisions, not claimed recovered Matrix defaults): text 2.5mm, arrow 1mm,
precision 2, scale 1, rotated angle 0 degrees. Display labels are derived from witnesses;
text override does not change the measured value. Witnesses are snapshots, without automatic
association to later edits of source geometry. Native Rhino dimension round-trip and layout
styles are not claimed by this initial owner.

`Text` remains the already approved solid Text alias. `TextBlock`/`DimText` name the new
annotation command; both its original catalog icon and its feature id dispatch here.
`Dim` names Horizontal; `DimVertical` names Vertical because the source catalog repeats Dim.

Evidence must distinguish actual runtime tests and UI replay from this contract.
