# Distance implementation plan

OM9-MEASURE-002 remains part of the full 130-spec 01-core objective. Local spec requires original `Distance`, two picked points and feedback in history/feedback window. `Unit` is an unnormalized cue; no PDF reread.

1. Test Rust Euclidean distance for arbitrary 3D points, coincident points, finite coordinate validation and explicit display-unit conversion. Internal millimeters and mm/cm/m/in/ft are OM9 choices, documented separately from source cues.
2. Add native read-only two-point session sharing menu, mouse and CMD. Current C-plane mouse projection and explicit XYZ CMD coordinates; Esc and document/workbench changes cancel. No model objects, document transaction or selection mutation.
3. Report distance with selected unit in command feedback/history. Validate invalid input without losing first point; Unit changes must not reinterpret existing coordinates.
4. Verify native mouse/menu/CMD, units, cancellation, task permissions and document lifecycle, object/selection/Undo invariance. Completion requires native evidence, not only math tests.
