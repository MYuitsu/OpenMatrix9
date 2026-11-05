# Top11 Copy and Move preview

The viewport preview owner previously read only the separate Transform command state, so Top11 Copy and Move had no preview. Both now use the same translation candidate for preview and point acceptance, including Copy repeat options. Preview transforms the visible cached scene geometry for the selected roots, including nested instances, without changing the document. It uses display tessellation and is subject to the existing point and byte budgets.

The four-direction point fixture failed before the change and now passes for both CMD and toolbar controller routes. Additional fixtures compare each preview with committed Copy points for FromLastPoint, UseLastDirection, UseLastDistance, their combination and Vertical, and compare a placed nested instance with its committed scene position. They also check that preview does not change the document; the direction fixture covers Undo/Redo.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`: all 141 Top11 integration tests passed with default features and all 141 passed without default features. Public source audit passed. The initial executable launch without the runtime DLL path failed; adding `L:\om9-target\debug\deps` to PATH resolved it. No openNURBS measurement batch was performed.

These controller/scene fixtures do not prove physical mouse handling, visible GPU presentation, every geometry type or every CPlane and option combination. Mirror and Rotate Top11 previews and overall Top11 desktop acceptance remain open.
