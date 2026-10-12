# RCORE-01 — Missing Solid metadata diagnostic review

Scope: `Gui/SolidController.cpp` metadata getter and the six new native
Box/Sphere property-removal witnesses. This is independent source review,
not an additional runtime run or approval of the entire chapter.

The reviewer found that constructing `Ref` directly from a missing attribute
threw a generic Python error before adding the frozen operation/input context.
Actual Runtime32 RED subsequently reproduced exactly six diagnostic failures:
removing OM9Command, OM9FeatureId or SolidParameters on Box and Sphere.
Rollback and strict output-wrapper release already passed.

The correction checks the raw getter result before constructing the owned
reference. Null returns receive StaleGeometry with frozen operation/input and
property context; pythonError clears the pending exception. Successful returns
transfer their owned reference into Ref. Follow-up review found no issue in
this narrow correction. The reviewer ran no tests.

Matching native fixture: RED144/150 → GREEN150/150. See the
[publication checkpoint](../validation/2026-10-11-rhino-core-solid-publication.md)
for actual runtime/module receipts, other cohorts and remaining requirements.
Full RCORE-01 remains in_progress.
