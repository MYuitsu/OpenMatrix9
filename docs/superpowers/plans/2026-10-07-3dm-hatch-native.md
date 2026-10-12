# Hatch native preservation, pattern references and placement

Continue the approved full openNURBS design inline. The preceding cloud/legacy5 package passed817 GUI checks/native13. This is the next required geometry/style package, not a redefinition of completion.

1. Native RED fixtures: solid/line HatchPatterns, selected hatches using a nonzero pattern table index, rational outer and inner loops, plane/basepoint/rotation/scale/user text, mm/cm and block placements. Assert the required pattern UUID appears in inventory dependencies and survives selected closure with exact pattern fields. Missing patterns and unsupported gradients must fail before output replacement.
2. Inventory complete named hatch facts and native loop curve hashes; HatchPattern description/fill/line angle/base/offset/dashes. Add indexed dependency to HatchPattern and reject missing custom indexed references. Validate native payloads before enabling writer handling.
3. Preserve native hatch geometry, pattern reference and userdata/attributes through selected and block routes. Unit normalization must scale pattern spacing as well as boundaries. Rigid and uniform placement must be verified; arbitrary affine hatch pattern handling must be implemented or explicitly remain pending with a precise guard, never silently accepted from the pinned ON_Hatch::Transform determinant shortcut.
4. Focused host import/retained placement, current source/member/copy/proxy, parent, Undo/Redo/FCStd and atomic errors; build isolated runtime and run required regressions. Derive hatch display/current-field editing and full affine pattern support in the subsequent part of the same full objective.
5. Refresh README/spec/coverage/ledger/evidence and checkpoint. Do not claim complete hatch/openNURBS coverage, actual Rhino5 application acceptance or public integration from a narrow native test.

Pinned evidence: ON_Hatch::Transform does not scale PatternScale and skips loop rebasing when abs(det)==1, including shears/reflections; ScalePattern separately scales along hatch-plane x. Full appearance under general affine transforms needs explicit pattern handling beyond calling native Transform. Native block matrices can remain preserved without flattening hatch appearance.


## Hatch native references progress — 2026-10-07

- [x] Custom line HatchPattern UUID dependency and full named pattern fields; rational native outer/inner loops, plane/basepoint/rotation/scale/bounds, selected closure and exact native payload validation.
- [x] Native mm/cm pattern-spacing normalization, retained placement/parent, Undo/Redo, independent copy and FCStd after external source deletion; NaN/Infinity dash rejection.
- [ ] Hatch display/current-field/loop editing, general affine/reflected patterns, broader block/member/proxy, built-in/solid/gradient, corrupt-source/reference and child-loop userdata/legacy cases.
- [ ] Remaining full geometry/annotation/style/resource/component/document/history/version audit, actual Rhino5 acceptance, comprehensive review and public integration.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-native.md` — isolated FreeCAD833/833 across40 suites, native14/14. Rust unchanged at prior75/75/fmt evidence. Native retention is not editable/rendering support; full openNURBS remains in_progress.


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
