# RCORE-01 world frames and source attributes — 2026-10-10

Owned geometry descriptors now carry checked support-local to world matrices.
Rust composes `parent_world * native_path * inverse(definition_placement) *
native_support_location`, avoiding double placement because native Part Shape
already contains its feature placement. Direct nested App::Part ancestors and
App::Link paths reuse the native placement/link getters; no solver, Shape bake,
conversion or source mutation is introduced. Every common face has its own
support frame. Pcurves remain face_uv and receive no XYZ world matrix.

Instance representation10 retains a separate owned definition descriptor,
definition identity and definition-object-to-world matrix. The instance's
solid/topology facts remain unknown; classification stays on the definition.
Mirror/nonuniform scale and all four nested LinkTransform combinations follow
the native accumulated path. The unrelated definition ancestor is excluded.
Bounded direct link chains reject cycles/depth>64 and revalidate every link/
definition source plus the root after capture on the GUI thread.

Rust owns document UID, object/type/labels, visibility/selectability, provider
type and inspected native ShapeAppearance material records (four RGBA colors,
shininess/transparency, uuid/image/imagePath). Labels retain UTF-8 and JSON
escapes. Unavailable appearance is null; arbitrary app attributes and Rhino
userdata are explicitly uninspected. This observes the native property; it does
not certify effective inherited link appearance, other metadata or persistence.
Nonempty native material uuid/image/imagePath values lack a host authoring
fixture; initialized FFI string checks prove the Rust side's ownership and
serialization only. Native material colors/scalars are authored on the host.

The final selected plugin passed **78 new +778 regression =856 FreeCAD checks**:
91 derived,76 Bezier,174 analytic,54 NURBS,177 topology,151 descriptor and55
import gate checks. All eight host processes exited0 with the matching selected
module. Full cargo test passed **620 tests**. Plugin SHA256:
`0e5b10751b50fb68fbdeb4c39a919747e0144f4b17db2365f5caefb3ecb2e2e1`. [Machine evidence](2026-10-10-rhino-core-world-attributes.json)
records source/runtime/log/report fingerprints and process outcomes.

Independent rational Bernstein samples agree with native curve/face evaluation
and placement/link APIs. Fixtures cover rotated nested groups, distinct face
locations, six solid-instance faces, live A/canonical definition after B edits,
Undo/Redo, unchanged source BRep/object count/history and serialized snapshots
after source replacement/document close. Missing/member/group/singular paths
reject without mutation. These sampled checks do not establish global parameter
maps, manifoldness or Rhino parity. Existing Line/Polyline/native solvers remain
reused.

Frame ABI is value-only512-byte input/144-byte summary. Attribute ABI is120-byte
input with16-byte text and192-byte native material records. Count/overflow/
alignment/null/UTF8 checks precede borrowing or owned copying; scalar validation
rejects invalid flags/ranges/nonfinite values. Frame validation checks finite
affine/nonsingular matrices and a rigid source inverse; determinant sign uses
normalized columns to preserve mirror orientation without scale overflow.
Short/invalid JSON output remains unchanged. Budgets exclude allocator overhead,
temporary/native staging and global process allocation; release panic/OOM policy
and native exception boundaries remain unchanged.

[Fresh review](../reviews/2026-10-10-world-attributes-review.md) identified a
retained-data undercount. A RED→GREEN aggregate boundary test proves accounting
now includes actual Rust headers (216/material and176/root on this64-bit host),
plus UTF8 content before clone. Another RED→GREEN test distinguishes unavailable
appearance from an empty inspected list. The legacy-shell42-check run with null
exit code is excluded; accepted host runs use pwsh and record exit0.

**Full original RCORE-01 remains in progress.** Group/member/subelement/array
and mesh/cloud instance definitions, arbitrary attributes/userdata, full edge/
pcurve correspondence, native parameter reversals, manifold/solid/cavity,
uniform typed reports and all remaining allocation/worker/mutation/lifecycle
requirements still need evidence in the
[original audit](../reviews/2026-10-10-rcore01-closure-audit.md).
