# DupBorder: single surface and managed memory

The standalone, untrimmed surface path cloned its surface and allocated its output list before the existing managed scratch admission. It now uses the same tracked, admitted and fallibly allocated path as a selection of several surfaces. Trimmed surfaces and B-rep handling remain on their existing paths.

For a translated planar surface, simulated managed pressure was tested through both CMD and menu. Refusal displayed the recovery prompt and preserved the document and Redo; clicking consent after pressure was released retried and produced four boundaries with placement and Undo/Redo, without removing an Undo entry. A separate held-pressure test called the recovery helper for one and two surfaces: one oldest Undo entry was removed, producing four and eight boundaries respectively, and the resulting transaction could be undone. These are separate tests, not an end-to-end physical memory exhaustion test.

Validation: six focused DupBorder UI/memory tests and 26 DupBorder integration tests passed; nine no-default-feature DupBorder tests passed before the final return-expression cleanup, followed by a successful no-default-feature cargo check. The debug application build and public-source audit passed. Existing unrelated compiler warnings remain. The artifact pruning tool removed no runs because older runs remain cited by records.

Real allocation failure across every path, native peak memory, other surface shapes under pressure and physical desktop interaction remain unverified. The admission is an estimate, not a hard native memory limit. Plan A remains open and B1–B3 remain deferred.
