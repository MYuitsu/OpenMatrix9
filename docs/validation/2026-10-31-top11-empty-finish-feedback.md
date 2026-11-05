# Top11: feedback when ending without edits — 2026-10-31

Status: partial automated evidence; full Top11 and physical M03-UI acceptance remain open.

Previously Enter could end repeatable Duplicate or Trim with a Completed result containing zero objects, and the command-line dispatch discarded the result. Empty finishes now produce a distinct NoChange result; the command dispatch passes Top11 results to the UI status and command history, and the MCP response includes the result message. The session still ends without creating a document transaction. Existing successful completion counts remain visible. Selection completion results now also reach the same feedback path.

A controller regression covers Duplicate and mutual-curve Trim through CMD and toolbar invocation, checking unchanged document, unchanged undo depth and ended session. It also checks a real Trim edit reports one object. OCCT-backed fixtures check empty finish for face/curve and intersecting two-face sessions. One app unit test calls the actual command-draft submission path and checks status and history for Duplicate. These are simulated/controller checks, not physical pointer or display acceptance. MCP was compiled as part of the application test build; its external transport was not exercised.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, cargo -j 2: 190 Top11/options/BRep/command snapshot integration tests, 108 no-default Top11/options tests and one targeted app UI unit test passed. No-default compilation emits existing unrelated warnings. The final source comment was corrected after the default suite compiled; it does not change behavior. No native or openNURBS measurement was performed.

Source SHA-256:

- `crates/om9-app/src/session/top11.rs`: `c449d8b0d4447442cc22d82f0242056b5f654547fe48974baae4aaeb9d3053bb`
- `crates/om9-app/src/session.rs`: `bf3bf0e417307bbcb8dc64c5863d77a08b95e5651c5989244246ed22eae64c8a`
- `crates/om9-app/src/ui.rs`: `688d47865d715e7501231b0c23a0926c11e2d8b77dfc1d192c6c27afc4b8d0f3`
- `crates/om9-app/src/mcp/command.rs`: `a2569c1e217e92aff1f8c81982b9c9b8338170a0129f71e10043f51bb0733f27`
- `crates/om9-app/tests/top11.rs`: `f49fd951a8c0956e29e65a6ffb734fb3a921a0464a5cf9a5b539e4bc16b2a2d6`
- `crates/om9-app/tests/top11_brep.rs`: `610aa931b2a6da632102c090b2c46c7af5f4d9b2d264cada9702318a66330ca5`
