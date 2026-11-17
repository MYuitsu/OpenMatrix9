# Trim metadata and surface source consistency

An entity can carry a surface and `om9.trim.surface` metadata containing another
copy of its underlying surface. Previously `Document::validate` validated the
trim payload in isolation, so changing the geometry alone could leave the
document valid while its trim referred to different geometry. A focused test
first reproduced acceptance of the inconsistent document.

Document validation now verifies that the trim belongs to a surface and that
its underlying surface equals that entity's geometry. In the tested transaction,
an inconsistent geometry edit is refused without changing the document or
Redo; changing the geometry and trim together succeeds and supports Undo/Redo.
A mismatched document is also refused when saving, preserving a pre-existing
destination. The rule applies to document validation, including definition
members, but the transaction and save fixtures exercise root entities only.

After the change, 34 document-graph and 14 semantic tests passed; five archive
persistence tests passed; 25 isocurve and 76 wireframe tests passed (four
wireframe diagnostics ignored). Seven existing real 3DM fixtures passed with
the compatible P1 Release adapter. Desktop, malformed legacy-file recovery and
all geometry-edit commands have not been individually tested.
