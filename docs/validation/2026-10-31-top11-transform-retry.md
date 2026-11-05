# Top11 Move, Mirror and Rotate: retry and CPlane reference — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. Before this change, a degenerate Mirror plane or Rotate destination could discard the pending prompt. Rotate accepted a reference at its center, preventing meaningful completion; off-plane reference points also produced an incorrect angle because their axial components were included in the dot product.

The existing pre-publication retry checkpoint now covers Move, Mirror and Rotate as well as Duplicate, and does not restore a prompt if the document revision changed. Rotate validates the first reference before advancing and projects both reference vectors perpendicular to the active CPlane normal before computing their signed angle. An axial or zero projection is rejected. This projection is an OM9 interpretation of rotation about the CPlane normal, not proof of unobserved Matrix internals.

CMD and toolbar fixtures cover repeated Mirror points, collinear 3Point Mirror, zero/axial Rotate references and retry, plus an overflowing Move destination followed by retry. Geometry, unchanged document on failure and Undo are checked. Other fixtures check off-plane points on world XY and a tilted CPlane, including clockwise rotation. A synthetic egui viewport fixture clicks an invalid Mirror point and retries successfully. Before the fix, the recovery and off-plane angle regressions failed; the latter returned approximately 0.57 degrees instead of 90.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: app Top11/options/BRep/snapshot 156 passed; no-default app 88 passed; the default-feature viewport test passed. The results do not establish every error path, post-publication behavior, desktop input, GPU presentation, 3DM or human M03-UI acceptance. Other Top11 commands remain open. No openNURBS measurement batch was performed.

Source SHA-256: `crates/om9-app/src/session/top11.rs` 65fef4106a398a372f34527ecdd05fe1036b78a5ff9febb838ab6e9ba579ff83; `crates/om9-app/src/ui/chrome.rs` 343bf9117de476d0da56ea4da003efe6851bf26d959730bcd403c2e39c71ac00; `crates/om9-app/tests/top11.rs` 905313ea746ad4a7696e893936c1da32b685d5b397ebcd53eb758c65a51d0e22.
