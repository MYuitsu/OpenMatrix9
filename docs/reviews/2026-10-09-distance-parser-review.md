# Distance / Angle shared-parser review

2026-10-09. Read-only review of `Gui/CoreDistance.cpp` and `.h`, with the shared adapter, command router, snap state, Rust parser/report, and relevant fixture sources inspected for boundary evidence. No Distance source or test edits, native build, or GUI execution by this reviewer.

Reviewed SHA256: CoreDistance.cpp `6cc3348342338c3970eddaccc50554a603c07f58cf95f11018b27401d57ea724`; CoreDistance.h `904212f176cac66caaeb7d56565a3956eb96cff60a9bfeb8c784dc134880c187`.

## Findings

**No blocking defect found in the two reviewed consumer files.** One low-severity source-backed provenance issue is at the upstream router boundary:

- **P3 — original typed text loses surrounding whitespace before this consumer receives it.** `CurveController.cpp:267` builds `input=text.trimmed()` and line 289 passes `input` to `CoreDistance::submit`; PictureFrame has the same boundary at line 303. `CoreDistance.cpp:119` correctly passes its own untrimmed `text` to the shared parser, but that value has already lost the original spaces. `CommandConsole.cpp:212` returns the actual input unchanged, `rust/src/coordinates.rs:136` permits surrounding whitespace, and `coordinate_input.rs:161` / `:234` deliberately retain the supplied raw text. Thus an accepted input such as `  w1,2,3  ` gets serialized as `w1,2,3`, contrary to exact original-text provenance. Geometry and units remain correct. Suggested repair: retain the trimmed local string for command/option dispatch but pass the original `text` to these active shared-parser consumers; add padded point and padded scalar input assertions in the cross-consumer fixture. This is outside this subtask's source ownership and was not changed. No native reproduction was run here.

## Directly checked invariants

| Area | Direct source evidence | Review conclusion |
|---|---|---|
| Document lifetime | delete and active-document signals cancel at CoreDistance.cpp:33-34; context capture at :64; owner/UID/unit verification at :49-54 through CoreCoordinates.cpp:103 | No retained pointer is used after normal document deletion/switch; same-name reopen is covered by the new existing Distance fixture. |
| Unit/input frame | command-start owned context; parse uses the captured context; verification occurs before acceptance and final publication; shared world-source adapter bypasses transformation | Input model scale remains independent of display `Unit=...`; later CPlane navigation does not silently change the latched construction frame. |
| Accepted previous point | previousPoint reads `accepted.back()` at :56-59; first/intermediate points append only after validation at :91-105 | A rejected parser input or zero-length Angle endpoint does not advance the relative base. Four-point Angle ordering is preserved. |
| Final report | finish builds a candidate vector and full report before host publication at :74-85; checkContext immediately before property write | A parser/serializer/context failure before publication leaves the prior completed report intact. No document object/metadata/Undo transaction is opened. |
| Native points | F4, snap, and plane pick use worldCoordinateInput with distinct source tags at :129/:137/:143 | No second frame or unit transform is applied to native world input. |
| Hover and Once | only button presses reach native point path; accepted intermediate points call acceptedPoint; success cancel clears transient state | Invalid endpoints retain Once; completion clears transient state. Omitting a separate final acceptedPoint call has no persistent Once leak because successful cancel clears transient state. |
| Mouse selection | releaseTarget is a QPointer; paired left release is consumed even after final completion (:125/:134) | Prevents the accepted final click release from becoming an unintended host selection; no raw widget-lifetime pointer found. |
| Error handling | standard parse/geometry errors retain state; unknown errors cancel; checkContext catches failures and clears state | No data mutation or stale report publication found. Base exceptions reach the generic cancellation branch, losing detailed wording but remaining safe. |

## Nonblocking limits / follow-ups

- Distance's timer checks only host availability, so a unit edit cancels/reports on the next accepted-input attempt rather than immediately while idle. Unlike the revised PictureFrame timer, it does not proactively show the stale-unit error. All acceptance/publication paths still verify context, so this is a UX consistency follow-up, not a mixed-unit geometry defect.
- For malformed unit settings, the current `checkContext()` displays the underlying error as-is. Such messages (for example `Invalid or unsupported document unit context`) may lack an explicit restart instruction; the session still cancels. PictureFrame now adds one. A dedicated diagnostic assertion would establish consistent behavior if required for Distance too.
- `Unit=Model_Units` / model-unit default remains the explicit previously accepted compatibility gap: `unit=0` retains mm output; other unit choices change display only. This review does not promote the entire measurement feature or RCORE-03/04.
- Completed `om9MeasurementReport` intentionally remains the last successful result during a new/cancelled/failed session. This matches the fixture assertion preserving the prior report; clearing it at start would change that contract.
- Source tests cover mixed frame/world/relative input, mixed suffix and model scale, invalid phase retry, degenerate Angle retries, exact accepted records, document switch/reopen, read-only task interaction, Undo/Redo/selection preservation, and world coordinate transcript. Parent reported warm cross-consumer native run `88fadbc1c69b4c7991303690f27e59f5` passing 31 checks; this reviewer did not independently rerun or inspect runtime artifacts, and source coverage is not represented as fresh runtime proof.


## Parent verification after repair

All three source-reviewed repairs now have final native GREEN: shared parser33 (`d22e08b99eb04154bdf399a0d9678d9c`), PictureFrame47 including transaction-lock rejection and retry (`1240cd70aff24826b7669eaae99d614c`), and cold restore4 (`66b6730ef65d4236bbf82a0aaf1fdb3e`). All seven affected regression processes exited0. The final evidence record links every result and verifies macro/module hashes. Earlier pending statements above describe the review snapshot; these findings are resolved in the validated increment.
