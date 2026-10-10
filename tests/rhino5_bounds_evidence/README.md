# Rhino 5 bounds evidence checks

Run `python -m unittest discover -s tests/rhino5_bounds_evidence -p "test_*.py"`.

These validate the numerical diagnostic evidence, not production tessellation.
The 0.001 mm comparison remains unchanged. A fallback requires a demonstrably
incomplete raw BRep box, converged 1e-5/1e-6 diagnostic meshes, trimmed-BRep
projection, cross-distance checks and independent Interior/native-matching
point witnesses. Missing data, changed geometry, inadequate convergence,
an omitted known point or actual 0.002 mm displacement rejects.

The screening allowance is empirical. Mesh tolerance controls edge midpoint
deviation; this is not a certified global extrema algorithm. Raw49/50 remains
separate from the supplementary numerical outcome. See
`docs/validation/2026-10-07-rhino5-ring-bounds.md` for actual Rhino evidence.
