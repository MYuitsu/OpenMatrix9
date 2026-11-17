# ExtractIsocurve start — memory recovery

The pending-surface reservation can refuse at the start of ExtractIsocurve,
before a pending session exists. The UI's existing recovery path previously
covered failures when completing a pending isocurve, but not this earlier
failure. Both menu invocation and the typed `ExtractIsocurve` command now offer
the existing memory-recovery dialog on recognized memory refusals.

After explicit user confirmation, the UI retries and, only when still needed,
removes the oldest Undo step per iteration until the command starts or no more
Undo is available. The recovery captures the coordinator authority, epoch,
selection and command draft. If any changes, or another command is running,
it stops before removing Undo. Starting the command does not itself create a
curve; the normal isocurve prompt remains for the next step. A fallible surface
copy allocation is identified as a memory refusal so it can use the same flow.

Five focused headless UI tests cover menu and command invocation, a real egui
confirmation click, no deletion before consent, a retry needing one old Undo
removal, refusal after draft/selection changes, and exhaustion preserving the
drawing and Redo. Old Undo steps already removed on an exhausted or canceled
attempt are not recoverable. The complete shared undo-settings UI suite passed
59 tests; `cargo check` without default features passed. Simulated managed
pressure is not an actual OS allocation failure; desktop interaction and all
source types remain unverified.
