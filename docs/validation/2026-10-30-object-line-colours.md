# Viewport object line colours — 2026-10-30

The user requested green lines for all unselected objects and magenta lines for
selected objects, and explicitly requested that this rule be saved to the skill.
The viewport uses `#008255` and `#FF00FF` for lines, guides and point markers,
independent of stored object and layer colours. Mesh/BRep edges now appear for
unselected objects as well as selected ones. Box/sphere guides retain their
previous scoped recognition and selected on-top behaviour. Face colours,
document geometry, layer attributes, selection and Undo are not rewritten.

Validation, default OCCT configuration with `CARGO_TARGET_DIR=L:/om9-target` and
`-j 2`:

- `cargo test -p om9-app --lib selection_colour_tests`: three passed; checks
  both fixed colour values and renders the magenta tint with a GPU renderer.
- `cargo test -p om9-app --test gpu_navigation_publication`: six passed,
  including a mixed point/curve/mesh document with non-green stored attributes,
  a box and a pair of spheres; selection cycles preserve the document.

These results do not establish an actual desktop screenshot or cover every
possible object representation; GPU depth and box/sphere guide limits are
recorded in their respective display reports. This is a user-directed OM9 rule,
not a claim about all original Matrix modes or saved source colours.
