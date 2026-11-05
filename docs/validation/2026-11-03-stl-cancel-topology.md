# STL topology and weld cancellation

P3b partial: STL export checks its cancellation token periodically while welding vertices and traversing topology. The existing non-cancellable topology API remains available. The app computes topology once for its receipt and no longer repeats the same topology traversal in the writer; the writer still checks indices, numeric validity, degeneracy and quantization. Open and non-manifold results remain explicitly reported.

Windows with default OCCT/openNURBS, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: nine exchange STL unit tests passed, including a deterministic cancellation during topology traversal. Three app cancellation tests passed, including a pre-cancelled weld and cancellation after temporary file creation. All 23 STL integration tests passed. The weld test proves cancellation before allocation, not a timed mid-weld interruption; no latency or performance comparison was measured.

The cancellation API is not yet connected to a UI worker; OCCT calls remain individually non-interruptible and checks in other loops or OS I/O may be delayed. No peak-memory, desktop responsiveness or global budget claim follows from avoiding the duplicate topology pass. The 256 MiB aggregate limit, ownership accounting, background export and remaining plan gates remain open. The active OM9 session was not touched.
