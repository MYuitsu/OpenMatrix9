# Sphere retry after admission refusal — 2026-11-06

A Sphere point-completion path cleared its pending state before the managed snapshot reservation could fail. Under simulated pressure the document and Redo survived, but the prompt returned to `Command`, losing the in-progress center or points. Typed FitPoints completion similarly took the points before attempting the commit. The code now retains the pending state until a successful commit, and returns the last point to input on a failed point completion; it does not clone the potentially large point list.

On Windows, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- The controller regression first failed because the Sphere prompt reverted to `Command`; after the fix it passed seven cases: default point and numeric radius, 2Point point and numeric diameter, 3Point, 4Point, and FitPoints Enter. Each simulated reservation refusal preserved the prior document, Redo and prompt; releasing pressure and retrying committed the sphere.
- `cargo test -p om9-app --test solid solid_sphere -j 2`: 54 passed; `--test solid_difference_ui viewport_sphere`: 6 passed; `--test solid_fit_source`: 3 passed. `cargo check -p om9-app --no-default-features -j 2`, public-source audit and `git diff --check` passed, with existing warnings.

The follow-up `cargo test -p om9-app --lib retries_after_snapshot_admission_refusal -j 2` passed both fixture functions: the seven cases above plus four cases using one selected straight curve for Tangent and AroundCurve, each completed by a point or a numeric radius. The four cases preserved source, document, prompt and Redo under simulated pressure, then produced the analytic radius-5 volume and passed Undo/Redo after retry.

These are controller and simulated-viewport fixtures, not physical desktop or native peak evidence. Other Tangent/AroundCurve sources and modes remain unverified under pressure. Document/Undo still retain viewport display, and source/cache ownership remains open. The user approved recording the scoped retry rule; the repository and installed `openmatrix9-feature-port/SKILL.md` were synchronized after the follow-up and both have SHA-256 `119f5a60c8441d1d80f0304626ff628b6a1956333e79c3a4ff0b62f5bb1a6284`.
