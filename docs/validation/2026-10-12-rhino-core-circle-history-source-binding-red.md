# RCORE-01 — Circle History source binding RED, 2026-10-12T03:15:31+07:00

**Status: in_progress. Full original RCORE-01 and Task2/3 remain open.**

No product implementation changed. Native source/Runtime59 plugin/selected App19,
GUI and shared SDK remain unchanged. Rust test cases and authored synthetic
request fixtures are new; no native/plugin build or native suite rerun is claimed.
The full Rust suite was not run. The focused tests deliberately expose failures.

## Real source data and exact observation

Requests are reconstructed from Runtime59 saved results Circle014 (deformable
AroundCurve), Circle015 (bounded Tangent) and Circle016 (fixed first contact and
point source). Native output, validation, input identity and context are retained;
source_snapshots are restored from input_snapshots with the original slot indices.
They are **not intercepted C++→Rust packets**. Complete replacement line/point
snapshots are taken from the same authored native fixture; geometry and identity
are not fabricated. Exact source/data hashes and provenance are recorded.

All three baseline payloads assemble successfully. Six assertions then fail
because the corresponding bad requests are also accepted:

1. Path fraction .5→.25 keeps the old native center(10,0,0), although the actual
   selected line domain0..20 gives center(5,0,0).
2. The sole history_path role is replaced with history_source, with unchanged
   owned path data. AroundCurve no longer has an explicit bound path role.
3. The single history_path reference is duplicated ambiguously.
4. Tangent current point(100,4,0) is off its captured vertical line x=1.
5. The complete first tangent source/ref is replaced by another real captured
   horizontal line y=0, keeping the stale vertical contact(1,4,0) and output.
6. A genuine same-document point(0,10,0) replaces the fixed point, outside the
   stale output circle center(5,4,0), radius4. Native tokens and document UID match.

The existing mixed-document-token gate remains a passing fourth test. Focused
`cargo test --lib history_native_source_ -- --nocapture` is **RED4/10**, six
assertion failures, exit101; compilation completes and no fixture/parser startup
failure is substituted for the tested acceptance. All129 pre-run source/data
inventory files match afterwards. This is not a compiler dependency certificate.

## Corrected initial point oracle

Initial RED4/9 is preserved with its complete RTK output. Its replacement point
(1,45,0) has the same persistent UID but native document token2; the tangent
reference has token1. The early rejection is an identity gate, not proof that
the output circle satisfies the point. The corrected case uses the actual
CircleSide snapshot, token1/epoch3, point(0,10,0). No stamp is normalized or
invented. It is admitted and the intended sixth constraint assertion fails.
The current corrected source was archived before run65; an exact frozen initial
red64 compiler-source hash was not captured and is not claimed.

## Additional provenance observation

Actual saved AroundCurve input_snapshot0 calls its points current resolved
Circle History points but holds(5,0,0), while native_plan/resolved center is
(10,0,0). The stored recipe probe is intentionally overridden by native replay.
Its evidence must be retained and distinguished from the current path anchor;
the current label alone does not establish accurate resolved input provenance.
This observation is separate from the six failing correspondence assertions.

Source ownership, type and stamps are necessary, but these witnesses show they
do not alone bind native_plan to source geometry, roles, normalized parameters
or every fixed/tangent constraint. Native output basis versus its plan is already
checked; that is a different invariant. These tests do not prove independent
native BREP decoding, every source family, conversion/deviation/maps or budgets.

The previously proposed native savepoint review is still pending. The native
interface was not implemented and no approval is inferred from automatic goal
continuation. Independent source validation remains part of the original scope.

Next: Reuse the existing Rust basis/frame/source validators andnative solvers; bind each AroundCurve role andnormalized path fraction to the selected owned curve/domain/frame, andeach Tangent contact/fixed point/radius to its actual owned source and output plane/circle. Preserve every existing supported family andparameter convention;do not substitute a line-only gate or fabricate native evidence. Correct probe/current-point provenance. The native savepoint design review remains pending separately;do not repeat its approval question or implement its new interface before approval. Continue the full RCORE-01 objective.

[Machine evidence](2026-10-12-rhino-core-circle-history-source-binding-red.json).
