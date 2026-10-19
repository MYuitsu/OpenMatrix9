> Continuation (2026-10-19): the user moved the active foundation to M01. Native BRep source/topology is now implemented and checked; [current evidence](2026-10-19-brep-selection.md) supersedes the missing-policy/native-BRep-pending statements below. Earlier checkpoint facts remain historical. External UI review is still pending.

# M02 / L2-01-07 — nested occurrence subobjects

Date: 2026-10-19. This continues the [video implementation checkpoint](2026-10-19-selection-video-review.md)
and closes its remaining **occurrence-path subselection** limitation. Native BRep remains open.

Subselection now stores `SelectedSubobject { root, path, subset, source, world_epoch }` rather than an
object/subelement pair. The complete ordered member path participates in identity: two nested members
that share the same mesh and face number remain distinct. Two mirrored root occurrences also remain
distinct. Root-only API calls cannot guess a member of an instance.

The controller checks the captured publication, root source stamp, exact scene member path, effective
visibility/selectability and original mesh topology key before adding or toggling a reference. It never
rematches by location. Publication invalidates retained references; Undo cannot make an old reference
valid again. Retained path allocations are included in the controller's budget.

Ctrl+Shift picking passes the complete hit reference to the controller. Polygon outlines resolve the
same member path and use its world transform in all four viewports. Toggling one path preserves another
path to the same shared leaf. Escape clears the session selection without adding history.

## Evidence

- Five new tests in `crates/om9-app/tests/subobject_occurrences.rs` exercise mirrored roots, two paths
  inside one root, duplicate suppression/toggle, forged paths/sources, stale publications, hidden members,
  actual Ctrl+Shift widgets and exact polygon coordinates in four viewports.
- **37 targeted Rust tests passed**, including those five and the existing root-subobject, selection,
  popup, Gumball and cold archive regressions. Clippy with warnings denied and Python compilation pass.
  Commands and log hashes: [occurrence checks](2026-10-19-occurrence-checks.json).
- Three native occurrence replays pass: GPU 100%, GPU 150% and the default painter 100%. Each has four
  state/history checks and five screenshot probes: bright green outline in all four views, and absence
  of that outline on an unselected sibling. The fixture has two root occurrences, two nested branches
  per root, and one shared cube mesh leaf. A fourth replay repeats the twelve-check/three-pixel-probe
  standard selection/Gumball scenario on the new executable. All four use the same source fingerprint.
  Native measured values, source/executable/image hashes: [occurrence runtime](2026-10-19-occurrence-native.json).

Native inputs, including focus, are synthetic events through actual application widgets. This is not a
physical desktop-input or human-review signoff. The public-source audit still exits 1 with 11,735 existing
manual/reference/raster reports; it is not a release pass. No reference image was copied into evidence.

## Remaining blocker

The canonical `OpenMatrix9_Codex_Spec_v2/` directory is still absent. Bounded checks also found zero v2
paths in `git ls-files`, no v2 root in `HEAD`, and no v2 entry in the local reference-location registry.
The current model explicitly refuses BRep geometry, while the selected kernel API offers operation/test
results rather than a document-owned persistent BRep representation and topology binding.

Consequently the native BRep part of L3-001/003 and its downstream subentity display/handle acceptance
cannot be closed from mesh evidence. The user's AGENTS instruction requires the v2 architecture/backend
policy before choosing that owner/FFI boundary. The request for the relocated v2 path remains pending.
After that native scope is implemented and verified, the separately planned M02-UI external review remains.
The full L2 and the full M02 milestone are not marked complete.
