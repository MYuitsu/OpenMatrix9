# 2026-10-22 — `pick_all` candidate-depth fix

## Symptom

`om9-app/tests/pick_disambiguation.rs::escaping_the_choice_keeps_the_previous_selection`
failed deterministically in the checked-out tree: the click on two overlapping points produced
**0** ambiguity candidates where the oracle requires **2**. The failing run is
`implementation/evidence/M02/2026-10-22-pick-disambiguation-retry.log`.

## Cause

`crates/om9-scene/src/picking.rs::pick_candidates` used one depth early-out for both query modes:

```rust
if sphere_depth - radius > best_depth {
    continue;
}
```

`best_depth` is the nearest depth already accepted. For a nearest-only pick this is a valid
optimisation. For `Scene::pick_all` (`nearest_only == false`) it drops a farther object that is
still a distinct object under the pointer, so the disambiguation popup depends on the order in
which the spatial index visits candidates.

## Fix

```rust
// The depth early-out is only sound when one nearest hit is wanted. A disambiguation
// query must keep every distinct object under the pointer, including one hidden behind
// the nearest object, so it must not prune by the depth of an earlier candidate.
if nearest_only && sphere_depth - radius > best_depth {
    continue;
}
```

Also removed the now-obsolete `let _ = nearest_only;`.

## Regression

`crates/om9-scene/tests/acceptance_l2_01_07.rs`:
`two_objects_on_one_ray_are_both_candidates_in_every_publication`

It creates two points on the same camera ray (z = 0 and z = -4) 32 times with fresh identities,
asserts `pick_all` returns both roots nearest-first every time, and asserts `pick` still returns
only the nearest root.

## Verification

| Run | Result |
|---|---|
| `cargo test -p om9-scene --test acceptance_l2_01_07` | 9 passed |
| `cargo test -p om9-scene` | 82 passed |
| `cargo test -p om9-app --test pick_disambiguation` × 10 | 10/10 runs, 5 passed each |
| `cargo test -p om9-app` selection/Gumball targets | 63 passed |
| `cargo clippy -p om9-scene --all-targets -- -D warnings` | exit 0 |

No document format, kernel ABI, selection-set contract or Gumball transaction changed. The fix
only affects candidate enumeration for the ambiguity popup; `pick` with `nearest_only = true`
keeps its previous early-out.
