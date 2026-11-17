# Solid Text height follows document units — 2026-11-12

In a centimetre document, `Text H 1` previously constructed the glyph at the same scale as `Text H 1` in a millimetre document. The reproduced result had a 0.2 mm extrusion depth and volume 0.0240234375 mm³; the correct equivalent for the existing font-size behavior is `Text H 10` in a millimetre document, with 2 mm extrusion depth. The centimetre regression failed before the fix.

The shared Solid Text argument dispatcher now converts the input height to millimetres once before building the outline and extruded body. The staged menu input checks the converted value against the font's finite positive `f32` range, rather than checking the unconverted document number. On error it retains the entered text and height prompt; the document and Redo remain intact. This preserves the existing font and extrusion behavior, including the distinction between font size and actual glyph bounding height.

`text_height_follows_document_units_after_save_and_reopen` covers four routes: centimetre and inch documents, each through direct CMD and staged menu dispatch. It begins with a saved/reopened document, compares glyph bounds and volume with the equivalent 10 mm invocation, independently checks the 2 mm extrusion depth, and verifies full Undo/Redo and saved/reopened result. A too-large converted height is rejected while preserving document and Redo; the staged route retains the height prompt. This is a same-font metamorphic oracle plus an independent depth check, not a certification of glyph shapes, all text strings, physical desktop entry, explicit unit suffixes or native peak memory.

Validation: the Solid workflow suite passed 118 tests with one ignored; `solid_text_builds_exact_outline_solids` passed; and the no-default-features fallback test passed. The fallback case does not itself test Solid Text.

The debug executable built successfully in the existing `L:/om9-target` directory with `-j 2`; the public source audit reported no errors. This change was not checked with physical desktop input.
