# Full openNURBS jewellery exchange implementation

User-approved plan: 2026-10-07, comprehensive pinned openNURBS exchange with Rhino5 output; preserve secondary data, no animation or universal editor. Specification: docs/superpowers/specs/2026-10-06-full-opennurbs-design.md; current gaps: docs/validation/2026-10-07-full-opennurbs-gap-audit.md.

- [x] 1. Normalize coverage and compare actual linked runtime registrations.
- [ ] 2. Complete CurveOnSurface/PolyEdge mapping, reference ownership/remap/current export.
- [ ] 3. Multi-shell/cavity BRep, signed orientation and safe native retention.
- [ ] 4. Remaining geometry and native-only preservation/version compatibility.
- [ ] 5. Annotation/style/content/layout preservation.
- [ ] 6. Resources and linked block dependency closure/FCStd portability.
- [ ] 7. Document merge/settings/units/UUID policy and safe userdata/history.
- [ ] 8. Complete evidence matrix, review, primary integration and installed acceptance.

Global constraints: pinned SDK eb92af3ba1806b0a34a99aba0d3bda83e3d46083; public output V5/50; no implicit CAD meshing/data loss; immutable user fixtures; existing 0.001mm ring bounds tolerance and area/volume thresholds; one transaction, atomic output, selected closure; never count assertions as percentage coverage. Execute build/link/install/application tests serially. Preserve primary/user changes and do not push.

Verification: native CTest in build/3dm-preserve-native, actual isolated FreeCAD in build/3dm-preservation-sdk, actual Rhino5 scripts where applicable. Each package requires scoped tests and evidence; standalone/native success never implies Rhino application or full feature completion.
