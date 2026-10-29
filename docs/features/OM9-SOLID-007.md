# OM9-SOLID-007 — Fillet Edge, partial standalone implementation

The normalized M03 reference describes choosing edges and NextRadius, Enter,
then editing fillet handles and Enter. Menu and CMD now support an initial
staged path for one root BRep, including a tested joined open shell: set positive NextRadius,
select one or more edges (changing NextRadius between selections if needed),
Enter to calculate a bounded BRep preview, and Enter
to replace the source in one transaction. Undo returns from preview or removes
the last edge; Cancel publishes nothing. Preview is a green line overlay on
the original scene, using source placement. For an edge with two distinct
topological endpoints, each endpoint has a handle; `CurrentRadius` changes
the selected endpoint and the preview interpolates between radii. A closed
edge without distinct endpoints retains one sampled constant-radius handle.
A failed recompute retains the old preview.
The older numeric two-argument
path still commits immediately.

The adapter rejects varying radii on automatically propagated multi-edge
tangent contours; that branch needs a dedicated fixture. No original default
radius is invented. It does not yet provide handle dragging, additional
interior handles, chain/previous-edge options,
general open-shell topology, instance paths or ambiguity selection; ambiguous overlapping
edge picks are refused. F6, SDK/History, native intermediate resource bounds,
actual desktop/human UI and preview GPU acceptance remain open. See
`docs/validation/2026-10-28-solid-fillet-source-map.md` and
`docs/validation/2026-10-28-solid-audit.md`.
# Endpoint classification follow-up (2026-10-28)

Closed edges with a shared topological vertex use the existing single sampled
handle; endpoint lookup failures are reported instead of being treated as
closed edges. Cylinder classification, constant fillet, app commit and Undo
passed automated checks. Moving the closed handle and general closed curves
remain unverified; see `docs/validation/2026-10-28-solid-audit.md`.

ShowRadius now controls the preview handle text in selection and preview
stages. OM9 starts with Yes to preserve its previous display; the original
default is unknown. Automated option routes and preview state were tested;
actual desktop rendering remains unverified.

`SetAll=` updates all currently implemented preview handle radii atomically;
failed rebuilds retain the previous values and preview. Two independent box
edges, commit and Undo were tested. Additional handles and desktop routes are
still outstanding.

`LinkHandles=Yes` applies CurrentRadius to all implemented handles; No edits
the chosen handle. OM9 starts at No; the original default remains unknown.
Simulated keyboard, token and typed routes, two-edge geometry and rollback
passed; actual desktop and additional handle types remain unverified.

`SelectEdges` returns from preview to edge picking while retaining current
edge and radius edits; Enter rebuilds preview. Two-edge geometry, option routes,
viewport picking and Undo were checked in simulation; desktop remains open.

The OCCT adapter now accepts a nonuniform radius law on a fully selected open
tangent contour when the endpoint values agree at shared vertices. A split
straight box edge and app outer-handle edit were tested. At that checkpoint
coincident shared handles could not be edited together individually; see the
subsequent update below. Curved/closed chains and desktop UI remain unverified.

Shared endpoints on the same tangent contour are now synchronized by OCCT
topological vertex identity during CurrentRadius edits; failed previews restore
all affected radii. Two straight segments and a non-tangent shared corner were
tested. Coincident handles still render separately; desktop and performance on
large BReps remain unverified.

Coincident handles at a shared tangent vertex can now be selected as one hit
in simulated viewports; nearby unrelated handles remain ambiguous. Actual
desktop picking and general topology remain unverified.

The OCCT adapter now accepts interior radius controls on fully selected open
contours, with fractions measured along the oriented edge. Straight and split
straight-edge fixtures verify local BRep effects; at that checkpoint the application
did not expose AddHandle or project viewport picks to such controls. General curved
and closed contours remain unverified.

The subsequent AddHandle checkpoint connects repeated viewport picks on
selected open edges to preview; CurrentRadius, SetAll and LinkHandles include
the added controls. A translated straight-edge box was tested in simulation.
At that checkpoint RemoveHandle was not implemented; see the update below.
Closed/general curved edges, dragging and desktop remain open; CopyHandle
was added at the later checkpoint below.

The subsequent RemoveHandle checkpoint adds a selection mode for added
controls, refuses default handles and rebuilds preview after removal.
Two-removal, option-route, volume and Undo checks passed in simulation;
native rebuild-failure rollback and desktop interaction remain unverified.

CopyHandle now selects a source radius and supports repeated new-edge or
existing-handle destinations in preview. Simulated straight-edge geometry,
option routes and Undo passed. General curves, dragging and desktop remain
open.

Shared-vertex classification now batches candidates into one OCCT topology
query. Behavior and error paths were regression-tested; large-model latency
and native peak memory still require measurement.

