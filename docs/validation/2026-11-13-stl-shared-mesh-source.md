# Shared authored mesh source in STL worker

2026-11-13. Follow-up to `2026-11-13-stl-brep-authored-mesh-worker.md`: repeated occurrences of one authored-mesh entity now share a single worker-owned snapshot instead of copying it for every instance. Before capture, an admitted temporary index deduplicates object identities so the pending mesh allowance is charged once per source. The worker uses an opaque shared owner within the capture; each occurrence retains its own placement and visibility. The capture reservation includes the mesh allocation and owner until the last occurrence in that capture is dropped.

An OCCT sphere plus two instances of a single authored triangle, one reflected and translated, produced the same positions, triangles and warnings as the retained-document path. The fixture checks that the two occurrences share one allocation and that the pending allowance equals one mesh and its owner. The existing mixed-source fixture also exercises a real `StlExportJob`, cancellation and document/display release after source capture.

With OCCT runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: `cargo test -p om9-app -j 2 --lib session::stl::cancellation_tests` passed 56; `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings. Public source audit: 15,998 source files, 594 bindings, 593 SVGs, no errors; artifact pruning removed 0 and retained 169 older cited runs.

The index still scans occurrences on the calling thread and the worker's lookup scans earlier entries; large instance-count latency has not been measured. At this checkpoint a selected NURBS surface used the document-backed route; a [later follow-up](2026-11-13-stl-surface-source-worker.md) transfers surface and trim metadata together. The document is retained until worker copying completes. This is not completion of source-only Document/Undo, cache eviction, cold restore or v5.

SHA-256 `crates/om9-app/src/session/stl.rs`: `d97c218cb85d35bcdad3cd9dc299bba466765cb4e7f05acc5fa6c01870e45395`.
