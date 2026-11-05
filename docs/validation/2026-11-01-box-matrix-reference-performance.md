# Box reference and viewport preparation — 2026-11-01

The user supplied a 15.77 s, 982×606, 30 fps Matrix9 recording. It shows a
rectangular base preview, a height-dependent wire box, then a green shaded box;
the sequence is repeated with a second box. Green shaded areas appear around
3.90 s and 13.93 s in the recording. The recording does not provide a reliable
mouse-click timestamp, so these times are not end-to-end latency measurements.
The video remains outside the repository.

Before this change, the live viewport already retained its own line overlays,
but each shared `GpuScene` also allocated three unused line overlay batches and
an always-empty selection-highlight mesh batch. It also allocated an empty ghost
mesh batch when nothing was selected. These unused batches created GPU pipelines
and an empty render pass each time a scene was prepared. They have been removed;
the viewport's actual fixed, moving, ghost and selected-guide overlays remain,
and a ghost mesh batch is still prepared when a selection exists.

Measurements are diagnostic, not a claim of Matrix9 parity:

- Existing release executable, separate headless MCP session and 12 fresh empty
  documents: Box height point to MCP response median 1.78 ms, maximum 3.36 ms.
  This includes protocol and state reporting but excludes viewport and present.
- Debug, 320×240 offscreen GPU fixture, one empty document and one Box: before,
  two separate samples were 23.54 and 21.21 ms from commit completion to GPU
  publication fence. After, seven samples were 13.03, 11.14, 12.20, 10.50,
  11.05, 10.81 and 10.88 ms. These are small uncontrolled samples, not p95 or OS present.
- The GPU publication suite passed seven tests in a temporary lib-test harness
  because a separately running OM9 locked the normal debug executable. This
  covered scene replacement, selected and unselected box/sphere publication,
  mixed object colours and preview visibility. The temporary harness was removed.
- `cargo build -j 2 --release -p om9-app --bin om9` succeeded. The resulting
  `L:/om9-target/release/om9.exe` SHA-256 is
  `f445b79118195cd5a4e69dbbc47132a138551c692f38d84aec1b10b46855f597`.
  The user's separately running debug window was not closed or modified.

An isolated test window was launched without changing the user's document, but
Windows Computer Use returned `GetCursorPos failed: Access is denied` and a
black capture. That test window was closed without editing the fixture. Native
pointer tracking, OS present latency and visual parity with Matrix9 remain open;
do not treat the offscreen samples as evidence that the interactive issue is
fully resolved.
