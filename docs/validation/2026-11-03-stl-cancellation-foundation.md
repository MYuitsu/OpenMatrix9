# STL cancellation foundation

Follow-up: weld and topology now check cancellation during their loops; see [follow-up evidence](2026-11-03-stl-cancel-topology.md). The limitations below describe the earlier delivery.

P3b partial: an STL capture may be exported with a shared `StlCancellation` token. Cancellation is checked before work, between sources, around topology and weld processing, and while streaming the temporary file. The final rename and cancellation decision are serialized: `cancel()` returning true means this export cannot subsequently replace the destination; after publication it returns false. An error or accepted cancellation removes the temporary file and keeps the prior destination.

Windows, default OCCT/openNURBS, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: two unit tests passed, including a separate thread cancelling after temporary-file creation and checking old bytes and cleanup, and a post-publication cancellation reporting too late. All 23 STL integration tests passed, including cancellation before an export starts. The concurrent test confirms temporary creation before cancellation but does not measure how many bytes reached disk.

This API is not connected to a background job or UI cancel button. It cannot interrupt a single synchronous OCCT operation, welding or topology traversal; cancellation is observed at their boundaries. Publication locking does not establish disk-full, process-crash or power-loss behavior. RAM admission, background worker lifecycle, New/Open/close cancellation during an active job, desktop responsiveness and the remaining plan gates are open. The live OM9 session was not changed.
