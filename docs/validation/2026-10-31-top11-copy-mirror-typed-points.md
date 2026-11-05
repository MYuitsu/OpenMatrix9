# Top11 Copy and Mirror: typed points

Copy and Mirror accepted viewport/world-point calls but their pending states were absent from the shared typed-coordinate route. Entering a coordinate in CMD failed with `Missing("active command")`. Both now use the same coordinate parser and point owner as the existing Move/Rotate routes; options are still handled first by the Top11 owner.

The Top11 point router also stored `last_point` before validating the point. A rejected Rotate reference therefore became the base for later relative input. It now updates the base only after the point is accepted. A regression explicitly checks the base after a rejected axial reference.

Both defects were reproduced in controller tests before the fix. Copy and Mirror were tested with toolbar and CMD invocation, typed absolute and relative points, invalid-input retry, a translated and rotated CPlane, Undo/Redo and `.om9` persistence. A separate egui fixture types command names, options and coordinates through the CMD widget. A real 3DM fixture exports and reimports the resulting Copy and Mirror lines. These fixtures do not establish every option combination, all CPlanes, physical keyboard/mouse input, GPU presentation or human M03-UI acceptance.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`: 138 Top11, 84 BRep, 10 option and 3 command-snapshot tests passed; 138 Top11 and 10 option tests passed without default features (with existing unrelated warnings). The CMD-widget and real-native 3DM tests passed. The native adapter was `L:\om9-native-short-preserve-v1\om9-exchange-native.exe`, SHA-256 `130739474bce0b012fb6be6b8fd60ebc73dfe060185baa2ce144b0ec80a13e05`. No openNURBS measurement batch was performed.

Source SHA-256: `session/text_input.rs` `aa922641dc4c9d7d178640c342b6c0ac0399989dd3f59547c4028f9512adcbac`; `session/point_input.rs` `086c102134442bbefd63f0787259d19ac247929d0c7e0789a2ec0f88e295fcc7`; `tests/top11.rs` `63cb1f6002231da5df702d42ae3620f832e7c3124275cc1ee1655f0127c117e6`; `ui/chrome.rs` `c04dd4fd24529bf021b2b98b648ff7ff73cd71a710a47cbb5a0c563741767506`.
