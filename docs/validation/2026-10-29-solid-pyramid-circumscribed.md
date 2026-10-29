# Pyramid Circumscribed — 2026-10-29

The normalized Pyramid reference describes a circumscribed base by the center and midpoint of an edge. OM9 now allows switching Inscribed/Circumscribed before fixing the base. Circumscribed treats the entered or picked radius as the apothem, rotates the regular polygon so an edge midpoint lies at the picked direction, and uses the corresponding circumradius for preview and the OCCT pyramid. Undoing the base permits switching styles again.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: five Pyramid controller tests passed, including four and five sides, positive/negative height, preview and committed edge midpoint, volume, mode switching, Undo and resource limits. Two simulated viewport workflows and two keyboard/pointer/typed option-route tests passed; twelve Tube regression tests passed.

The normalized page is a two-column extraction. Original Matrix solver and option persistence, physical desktop and GPU display remain unverified. Star, Edge and direction options remain open.
