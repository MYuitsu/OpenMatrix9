# Top11 transforms in an inactive viewport

Copy, Move, Mirror and Rotate formerly evaluated the pointer ray and transform preview using the currently active viewport's CPlane, then switched the active viewport before accepting the click. A Looking Down-to-Through Finger Move therefore could have a ray parallel to the old plane and no usable point. The egui fixture reproduced this failure before the change.

While one of those four transforms awaits input, the UI now evaluates hover, snap, constraints and transform preview in the pointed viewport's CPlane while preserving the active viewport until the click. The click activates that viewport before acceptance. If the ray cannot intersect the plane, the UI clears the previous hover and snap so a click cannot commit a stale point.

Synthetic egui fixtures exercise Copy and Move through Looking Down to Through Finger in four signed X/Z directions, compare preview paint to the pointer and inspect committed geometry. Mirror and Rotate fixtures compare independent expected coordinates to preview paint and commit in the same view switch. Another fixture snaps to an off-plane scene point, checks Project on and off from a third view, and compares with the commit. A parallel-ray fixture verifies that no stale point is committed. These fixtures do not prove physical input, GPU presentation, every CPlane or all option combinations.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`, runtime DLL path `L:\om9-target\debug\deps`: 12 Top11-related library tests passed with default features and 12 passed without default features. Separately, 144 Top11 and two UI replay integration tests passed with default features. Public source audit passed. No openNURBS measurement batch was performed.

SHA-256: `ui.rs` `ed57ee7a268961bcd03b65bd5f39527e7ab047819ee6adfc8b6ce22201203035`; `ui/chrome.rs` `2df023911e141288fe324c3147bab82c57b6599872f86c056d058c7e000288cc`; `session/top11.rs` `91711c1d2efda31d6dee9efe9447ab2309b07efa4f5e22a50489d0399dd43957`.
