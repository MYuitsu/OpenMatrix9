# Angle implementation plan

OM9-MEASURE-001 within the entire 130-spec core scope. Local specs are authoritative; no PDF reread.

1. Rust angle between two directed 3D lines defined by four points, including separate line origins. Test right/acute/parallel/opposite/near-parallel vectors, scale invariance, invalid input and zero-length lines. Display degrees in 0..180 is an OM9 choice; preserve reference cues separately.
2. Native original Angle uses prompts Start of first line, End of first line, Start of second line, End of second line; menu, mouse and CMD share session and feedback/history. No model or Undo output.
3. Integrate shared O-Snaps, explicitly required by source workflow. Read and implement applicable 01-core snap specs; picking only C-plane coordinates cannot prove geometry snapping. TwoObjects cue remains tracked until its supported object/curve semantics are normalized and tested.
4. Native tests must prove four-point workflow, feedback, invalid line retry, Esc/switch/task/document lifecycle, snapping to geometry and unchanged model/selection/Undo.
