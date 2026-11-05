# Top11 Mirror and Rotate preview

The Top11 preview owner previously supported only Copy and Move, leaving Mirror and Rotate without a displayed transform. Mirror preview now shares the point-defined plane calculation with acceptance and transforms the visible cached scene using the corresponding reflection. Rotate preview uses the same signed angle and rotation matrix as commit; a valid numeric angle draft is also previewed. The preview uses display tessellation and the existing scene point and byte budgets, without modifying the document.

The four-direction fixture first failed because Mirror preview returned no wire; it passes for Mirror and Rotate, toolbar and CMD controller routes, and Copy=Yes/No. It checks independent expected world coordinates as well as preview against the committed point. Other fixtures cover a three-point mirror plane, a tilted CPlane rotation, a numeric rotation draft, and Undo/Redo. A headless egui frame verifies the preview dot enters the UI paint output for both commands; this synthetic fixture does not prove native input, GPU presentation or a desktop manual gate.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`, runtime DLL path `L:\om9-target\debug\deps`: 144 Top11, 84 Top11 BRep and 3 command snapshot integration tests passed with default features; 144 Top11 tests passed without default features. The egui preview-paint test passed with default features. Public source audit passed. No openNURBS measurement batch was performed.

Evidence is limited to these fixtures; additional geometry and display cases, physical mouse/keyboard behavior, GPU presentation, and the complete Top11 desktop acceptance remain open.

SHA-256: `session/top11.rs` `da70f9d8fe00e6b8cf97326109df0b2a4e2e99cacce76c6de3938c5d04cb169e`; `session/transform/advanced.rs` `1813c40ed3ca39b64959018c146714a3e9171e11f7508364fca9d2a87860c931`; `tests/top11.rs` `c9818e16bc23de54ae59d4fb067e8fb6e4d5410a41635ee73439d91e410d5acb`; `ui/chrome.rs` `ec403a7297eeca9370f5f79ac9773511246853d0a0594ac93bfd40f59b6f6624`.
