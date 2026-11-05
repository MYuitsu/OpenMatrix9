# Top11 Join: trimmed rational NURBS — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance.

For nonperiodic NURBS with clamped backing knots, Join materializes the exact active interval using homogeneous knot insertion and exact native split parameters, then moves the active boundary pole to the world-space seam midpoint. This preserves the pre-healing active curve without refitting; moving the pole intentionally changes its shape near the seam. Fractional evaluation now maps the two endpoints to their exact domain values, avoiding reversed-domain floating-point underflow.

The core fixture tests six combinations of rational multispan intervals and orientation at 101 fractions, with positional agreement under 1e-10, and rejects an unclamped backing. The app fixture exercises two orientations through CMD and toolbar, checks the midpoint and 101 samples (displacement at most 0.000401 for a 0.0008 gap), .om9 roundtrip and Undo/Redo. This is sampled evidence, not a general proof of numerical stability. A synthetic viewport test covers line Join, not trimmed NURBS.

Windows x64 with CARGO_TARGET_DIR=L:\om9-target and -j 2: core restriction unit tests 2 passed; core curve_ops/top11 integration 48 passed; app Top11/options/BRep/snapshot 164 passed; no-default Top11/options 93 passed (existing unrelated warnings); synthetic viewport 1 passed.

Periodic and unclamped backing NURBS, discontinuous trim boundaries, extreme numerical scales, complex seam continuity, aggregate resource behavior, broader 3DM interoperability and physical M03-UI acceptance remain open. Knot insertion can require extra temporary allocation; this batch does not establish peak memory limits. No native SDK or openNURBS measurement was performed.

Source SHA-256 at validation: evaluation.rs `52b3529e9747e69371e6803e43ad6c89c4fb0ae47942e4daba043ed277da4819`; curve_ops.rs `f510013cee73e59a22f1c039a442c50efffdb85471679a14b13d0f72b3bda250`; top11.rs `e3512b4e6406884655823010ee0b23ae57e97b97b0ce35003ebeb524544c8f32`; app tests/top11.rs `92b48e623c720cbc910f97684cc11ed4d14079cf914ac3e82ceea60567ec0fa8`.

Update: [newer evidence](2026-10-31-top11-join-unclamped-trim.md) supports selected trimmed intervals on unclamped backing; an unclamped support boundary itself remains unsupported.
