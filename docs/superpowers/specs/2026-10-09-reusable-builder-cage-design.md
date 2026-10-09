# Reusable Builder History and cage deformation

The owner explicitly requested implementation now, with reusable services for
later commands, after the Edit/History/special Explode slice was validated.

Rust owns version/capability/schema validation, cage basis/evaluation/inversion,
atomic buffer decisions and reusable parameter data. C++ owns persistent native
FreeCAD links/properties/transactions and typed OCCT/openNURBS adapters. Python
is limited to fixture and module registration glue. These native APIs remain
unsafe boundaries, not a claim that the whole application is memory-safe.

## Durable Builder records

Gem-linked records retain operation ID, schema version, complete settings,
native template geometry and source frame independently of output objects.
Deleting an output does not delete its settings. Explicit restore rebuilds it;
Match Attributes copies only selected compatible records to gems of the same
declared shape. No deletion automatically recreates a deliberately removed model.
The registered affine-template evaluator is an OM9 reusable recipe, not an
invented solver for every unavailable Matrix setting/cutter. Unsupported recipe
versions and foreign links reject before writing. Existing native History policy,
detach, warnings, Undo/Redo and cold FCStd restore apply to new output features.

## Captive deformation

A persistent native CageControl stores rational NURBS degree/count/full-knot/CV
data and exposes control points in native properties. A CageBinding keeps a
frozen source geometry snapshot and control relationship. Each recompute starts
from that snapshot; successive edits cannot compound previous deformation.
Capture always retains a dependency, regardless of Record/Update settings, as
required by CageEdit. Release detaches only selected captives and keeps their
current geometry. Rust handles batch evaluation, finite/domain/bounds validation,
local box falloff and exact affine recognition. Native affine BRep deformation
uses OCCT. Non-affine mesh and supported curve/surface structural deformation
maps stored vertices/control points; unsupported topology is rejected explicitly.

Rhino MorphControl exposes current cage, original frame and explicit captive UUIDs.
Its archive contains current captive geometry, not necessarily original geometry.
Import therefore recovers parameters in the *current* bind cage and caches them
before later edits. Applying the original morph again would deform twice and is
forbidden. Missing/ambiguous references or inverse failures reject atomically.
UUID lookup is restricted to one verified import namespace/archive; geometry
proximity never establishes a captive relationship. Localizer variants that lack
a supported exact mapping are retained and reported rather than discarded.

## Validation and implementation sequence

1. Write failing Rust and native fixtures against missing APIs.
2. Implement Rust cage math and versioned Builder records independently.
3. Add typed archive cage descriptors and explicit relationship fixtures.
4. Integrate persistent native types, commands/APIs and existing History policy.
5. Build one matching SDK module, replay new native fixtures and existing Edit,
   History, Curve and Surface regressions; verify Undo/Redo and cold FCStd restore.
6. Review and synchronize exact Spec/status/README/ledger evidence. Preserve
   concurrent changes and existing source-audit exclusions; do not publish.
