# Edit History and special object adapters

User request: implement remaining History, block/text/dimension/cage behavior
for both native FreeCAD objects and retained Rhino 5 3DM records.

## Source and scope

Matrix8 Book1 PDF44 and48–53 were read through the History chapter;
Explode PDF84–85 through Join. Normal Join/Trim/Boolean can break History.
Join History (`OM9-TOOLS-017`, `gvJoinHistory`) creates a child of independent
parent curves. History Record/Update initially On; suspension retains links;
Clear detaches selected incoming links; editing an intermediate child breaks
incoming links while its descendants follow it. Undo restores that change.
Explode menu preserves blocks; explicit CMD Explode expands instances.
Text becomes curves; dimensions become curves plus text; cages yield control
components. Explode keeps objects in containing groups.

## Implementation

Rust owns History graph/policy, cycle detection, scheduling and session decisions.
Native C++ persistent FreeCAD feature/property links implement recompute and
transactional detach. Join History retains parents; ordinary destructive Edit
warns/detaches affected links inside the same transaction. Lock and warnings
are explicit document settings. Source edits propagate through generations;
the original baseline suspended recompute when either Record or Update was
disabled, preserving the last snapshot and links. The later RCORE-09 policy
revision separates them: Record controls new links; only Update suspends/resumes
existing recorded descendants. With updates enabled, disconnected/null/error sources clear active
child and descendant geometry while retaining records for recovery; direct
parent deletion detaches History and retains the last snapshot. Undo restores
the prior geometry and links.

Native Explode adapters resolve App::Link definition members in instance/world
coordinates, convert font contours to exact curves and separate dimension
curves from editable label text. Cages use explicit control/component metadata,
not arbitrary geometry tagged by name. Retained 3DM records decode their UUID
from a hash-verified embedded source archive using typed openNURBS geometry;
units, transforms, source provenance and conversion decisions persist.
Recursive block expansion is a documented OM9 choice. Missing fonts, corrupt
source payloads, cyclic definitions or unknown control types reject before commit.

Previews remain transient. A commit is atomic, keeps unrelated definitions and
referenced geometry, and supports Cancel/Undo/Redo/FCStd save/reload.
Python remains registration/test glue; geometry and behavior are native/Rust.

## Validation

Failing fixtures precede implementation. Rust tests cover multi-generation
graphs, cycle/detach/suspend/lock and restore. Native fixtures cover parent
update, child detach preserving descendants, Undo/Redo/reload and document
isolation; block button versus CMD, nested/rotated instances, glyph geometry,
dimension label preservation, native cage components and retained 3DM UUID
conversion. Existing Edit/Curve/Surface suites remain required regressions.

Broader gem Builder reconstruction and every Rhino annotation style are separate
capabilities; this change must report exact supported types rather than imply
complete Matrix/Rhino compatibility.
