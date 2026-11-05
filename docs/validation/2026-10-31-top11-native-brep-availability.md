# Top11 retained native BRep availability

Retained openNURBS `NativeBrep` and editable OCCT `Brep` share the same broad geometry kind. The toolbar therefore enabled some topology operations for retained native payloads even though their handlers require editable OCCT BReps. The controller fixture reproduced this for Explode before the change.

Explode, Join, Split and Trim are now disabled with an explicit reason when the current selection includes a retained native BRep. Copy, Move, Rotate and Mirror remain enabled; Mirror uses the existing placement operation and preserves the opaque payload. This is an honest availability guard, not an implementation of native BRep topology editing. With a single preselected object, invoking Trim through its command route can still open object selection so the selection can be changed.

Fixtures cover one native BRep, two native BReps and a native BRep mixed with a curve; they verify disabled states and reasons, rejection without mutation for applicable direct routes, and a Mirror placement with Undo. An egui widget fixture confirms Explode and Split occupy visible slots without click handling while Mirror remains clickable. The BRep fixture has a synthetic nonempty payload and display triangle; it does not validate a real 3DM BRep or native topology editing. Actual desktop input and GPU presentation are not established.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`, runtime DLL path `L:\om9-target\debug\deps`: 151 Top11, 85 BRep, three command snapshot and two UI replay integration tests passed with default features; 151 Top11 tests passed without default features. The focused egui widget fixture passed. Public source audit passed. No openNURBS measurement batch was performed.

SHA-256: `session/top11.rs` `60b9fe15810cd359cb4855040f2ddbc29bc72fb22deca3711e296da705f03690`; `tests/top11.rs` `1cef6a5f5aaab7c8188affc2bad31b4be7920c6ee3409f4a66f2134e61c72f0c`; `ui/chrome.rs` `6cf79d221bb74a3c996c93c14c71e52d89d5e3b9d1ccdf097a389ea78030dd0c`.
