---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 154-154
section_title: Extract Isocurve
section_id: m8book1-161-extract-isocurve
---
# Extract Isocurve

Source: matrix8_book_1.pdf, pages 154-154

<!-- section m8book1-161-extract-isocurve | Extract Isocurve | m8book1 pages 154-154 -->

<!-- page 154 -->

Where is this Command:
Access this command from Curve>Offset
Curve on Surface
Right Click Command: Offset Curve on Surface
ThroughPoint
Steps:
Flat: Straight line segments are added between
the curves and joined. The object goes on the
same layer as the original.
• Select Offset Curve on Surface from the Curve
menu.
• Select a curve on surface.
• Select the base surface.
• Type the offset distance.
Round: Tangent arcs are added between the
curves and joined. The offset curve goes on the
same layer as the original.
Yellow curve is an offset off the rings surface edge,
### Command Line Options:
ThroughPoint: Offsets through a picked point
instead of at a set distance. The ThroughPoint
option lets you keep picking offset locations for
the next curve on the surface at any distance.
Flip: Reverses the direction of the offset.
## Offset Curve on Surface
Number: Specifies the number of curves to be
offset. Each curve is offset the offset distance
from the previous curve.
## Extract Isocurve
‘OffsetCrvOnSrf’
Copies a curve or surface edge on a surface
so that all locations on the copied curve are a
specified distance from the original curve and lie
on the surface. The new curve on the surface may
be larger (offset outside) or smaller (offset inside)
the original curve.
‘ExtractIsocurve’
Creates curves along a surface by duplicating a
surfaces isocurves.
Where is this Command:
Access this command from Curve>Extract
Isocurve
