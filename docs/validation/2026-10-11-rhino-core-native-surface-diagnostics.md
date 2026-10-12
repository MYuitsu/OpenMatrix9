# RCORE-01 native Surface diagnostics — 2026-10-11

**Scope:** Native Surface capability errors identify the current first source/subelement and operation; additional input count is explicit. Partial original typed-error integration. The full chapter and construction plan remain in progress.

Real host RED:5/6 passed, actual host exit0, no fixture error. A one-profile
Loft correctly rejected before output and preserved source BRep bytes, but its
InvalidBasis diagnostic said `selected curves` instead of `FirstProfile.Edge1`.
The adapter passed that literal at capture/solve error boundaries, losing the
identity already present in SurfaceInput. Rust's rejection and solver behavior
were correct. The adapter now formats the real first object/subelement, plus
the number of additional inputs, without traversing or evaluating geometry.

The initial exploratory fixture compared a dirty newly-created feature before
its initial recompute, producing a separate source-byte failure. Completing
initial native execution before the baseline isolated the diagnostic defect;
RED2 retained the unchanged source-byte assertion. GREEN uses that same fixture.

Matching GREEN: diagnostic6 + binding29 + input12 + unchanged History65 =
**112 checks/four actual host exits0**, plugin SHA256 `bca2fefe4d8e0f2e21bce329b1f6cddda2685090b4b618c85b55a45e0c63c9fd`.
The standalone plugin build returned actual exit0; the shared SDK/host was reused.
691 full Rust tests retain unchanged source fingerprints. The earlier
[637-check Surface integration](2026-10-11-rhino-core-native-surface-policy.md)
used its previous plugin; those checks are historical and are not added to the
current-plugin total. The final change only formats native request context;
appropriate binding, input and History regressions were repeated.

This does not close uniform errors for every operation or all GeometryResult/
conversion, bounded-reference, worker, ownership, cache and preservation work.
