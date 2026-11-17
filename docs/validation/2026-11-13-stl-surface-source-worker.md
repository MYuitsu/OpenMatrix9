# NURBS surface source handoff for STL

2026-11-13. The STL worker now admits a source snapshot for a NURBS surface together with its optional trim extension. The worker copies the source under the capture reservation, shares the copy for repeated occurrences of the same entity, and releases the captured document after all authored sources have been copied. Mesh-only and surface-only selections can use the same path when OCCT is enabled; synchronous export continues to use the retained document. Document-backed and detached export use the same surface conversion and trim validation routine.

The mixed fixture combines an OCCT sphere with a planar NURBS face with an inner hole. Its worker-side source capture retains the trim extension and releases the old document; soup and warnings match document-backed export. An independent planar-area check gives 84 square units and excludes triangle centroids from the hole. The production worker wrote an STL with the expected triangle count. A separate untrimmed surface-only fixture compares detached and document-backed soup. These checks do not establish arbitrary trim, surface types, or native peak memory.

With OCCT runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: `cargo test -p om9-app -j 2 --lib session::stl::cancellation_tests` passed 58; `cargo test -p om9-app -j 2 --test stl_export` passed 68; `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings. Public source audit: 15,999 source files, 594 bindings, 593 SVGs, no errors. Artifact pruning removed 0 and retained 169 older cited runs.

The source document remains retained until the worker finishes copying, capture scans occurrences on the caller thread, and lookup can scan earlier entries. Native allocations, large-instance latency, desktop/GPU and broad trim cases remain unverified. Source-only Document/Undo, general representation store, cold restore and v5 remain incomplete.

SHA-256 `crates/om9-app/src/session/stl.rs`: `968536f2230cfc4afa282f6bf19afb8fa770b6faf771a86ef87e5f4842f477dd`.
