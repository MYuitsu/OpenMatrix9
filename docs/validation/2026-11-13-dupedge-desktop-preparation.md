# DupEdge desktop preparation

The existing `L:/om9-target/debug/om9.exe` was launched in a separate window after the earlier visible launch failed. A Windows screenshot showed the OpenMatrix9 Untitled interface. A click into its command area was refused by the automation runtime because an Edge window was in front; the target was not activated while the user may be using the machine. The user was asked for a suitable time to take keyboard and mouse control.

The live bridge identified the one new empty document, then created a 10 × 10 × 10 Box in that document. Its save request to `L:/om9-target/dupedge-desktop-test-20261113.om9` failed twice with a reset connection; after each attempt that file did not exist. A state request also once reported a reset and a later retry succeeded, showing the same document with one Box. No DupEdge mouse selection, save/reopen, or desktop Undo/Redo was performed. The test window was left open with this unsaved Box, avoiding interference with other windows. This is preparation, not a completed geometry measurement or desktop acceptance.

Next: when the user is available, bring only this test window forward, save and reopen a private test drawing, then test viewport edge selection and Undo/Redo. Investigate the intermittent live connection separately. Keep the existing build directory; Plan A is open and B1–B3 are deferred.

## Save follow-up

Five authenticated read-only probes of the existing window's direct loopback endpoint responded in 14–26 ms. A direct request using the live host's documented request format then read the same one-Box document and saved it successfully to `L:/om9-target/dupedge-desktop-test-20261113.om9` without moving keyboard or mouse focus. Three stale registration files were observed with no corresponding live process; whether they caused the proxy resets has not been established. No authentication token was recorded. The saved file still needs to be opened in the window and tested with real viewport input when the user is available. No product code or build was changed.

## Discovery latency inspection

The live proxy scans registrations serially and attempts a connection for each before forwarding a request. A read-only probe of the four registrations with a one-second connect timeout found the three registrations whose PIDs were no longer running each timed out after about one second; the active registration connected in 1 ms. This explains at least three seconds of discovery delay in this specific probe, but does not prove why an earlier forwarded request was reset. The registrations were not modified or removed. A production fix would need to preserve discovery of genuinely live windows and the multiple-window safeguard; no such fix is claimed here.

## Desktop Box selection after reopening

After the user said they were ready, the private saved Box document was reopened through the actual Windows Open dialog. DupEdge was invoked through the live controller's menu dispatch, then a real mouse click selected a visible Box edge in Perspective and physical Enter completed the command. The live document contained the original Box and one degree-1 DupEdge curve with poles `[0,10,10]` and `[10,10,10]`, corresponding to the highlighted top edge. Physical Ctrl+Z restored one object and Redo availability; clicking the onscreen Redo icon restored two objects and cleared Redo. The test window remains open with an unsaved result. This checks one Box and one edge in the existing first-target executable; the command itself was dispatched programmatically, not clicked from a menu. It does not test the later small-hole fix, other geometry, or native peak memory.
