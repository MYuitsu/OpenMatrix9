# Hatch affine geometry and isolated pattern transformation

Continue the approved full openNURBS specification and hatch plan inline.
The previous goal turn made progress: native hatch dependencies, unit spacing,
833 GUI checks/native14 and verified checkpoint. Full completion remains open.

1. Write native RED tests for determinant-one in-plane scale, world reflection,
   shear and anisotropic transformations. Compare sampled native loop points and
   pattern line origins, repeated offsets and dash endpoints in world coordinates
   against an independent affine oracle; include tilted planes and mm/cm.
2. Replace the SDK determinant shortcut with explicit plane/UV rebasing on a
   staged hatch. Keep native loop representations, types and userdata. Conformal
   maps retain pattern identity; general maps use an isolated transformed pattern
   so sibling hatches sharing the source pattern remain unchanged.
3. Pass native model context through all preservation transform/copy routes.
   Register generated pattern identities and refresh hatch dependency edges before
   closure. Require exact reread facts and payloads, source immutability and
   atomic errors. Handle named built-ins explicitly rather than assuming a table.
4. Build isolated runtime, test host retained copy/member/block paths and run
   native/GUI regressions. Document scoped evidence and remaining display/current
   editing/legacy/gradient/resources/component/document/history/version work.
5. Update current README/spec/coverage/ledger and SHA-verified checkpoint.

Pattern coordinate evidence: pinned opennurbs_hatch.h describes base/offset in the
line's rotated frame, and signed dash lengths along its direction. Official
reference: https://developer.rhino3d.com/api/cpp/class_o_n___hatch_line.html.
Native mathematical and Rhino5 serialized validation does not replace actual
Rhino application display acceptance, which remains required.


## Hatch native affine progress — 2026-10-07

- [x] Explicit native plane/UV loop rebasing for named determinant-one scale,
  reflection, shear/anisotropic and tilted-plane cases in mm/cm; isolated complete
  transformed line patterns and preserved siblings, stable generated UUIDs.
- [x] Native canonical/baseline copied-member closure and FreeCAD shared selected,
  promoted/copied block proxies, current placement/scale, Undo/Redo, FCStd and
  atomic singular errors. Published line-frame mathematical/serialized oracle.
- [ ] Actual Rhino5 pattern rendering/roundtrip oracle, class-wide built-in/solid,
  gradients, editable/display Hatch fields, child-loop userdata/refs, incomplete
  document native-reader repair and remaining full class/category requirements.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-affine.md` — isolated
FreeCAD857/857 across41 suites; native15/15,36.07s. Earlier dated affine guards are
superseded only in this tested scope. Full openNURBS remains in_progress.
