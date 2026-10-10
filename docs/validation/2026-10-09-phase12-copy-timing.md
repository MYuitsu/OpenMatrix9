# Phase1/2 Copy/Paste timing — 2026-10-09

User requested timing logs because the automatic run made some copies feel slow. Add elapsed wall-time logs to the test driver only; product Rust/native code and runtime hashes are unchanged. The previous user full run ca077 remains PASS14fixtures/621Rhino/770OM9/175saved checks with its original script hashes.

The same Rhino command now prints live `[timing] fixture | direction | operation/stage: seconds` lines, once per completed stage. Per-process JSONL files are in `timings/`; `timing-summary.json` ranks actual Copy/Paste totals and separates full host-session durations. Each event records UTC, monotonic elapsed time, PID and success/failure. Host reports retain events and existing `copy_seconds`/`paste_seconds`. Wrappers restore original native/host callables even after failure, invoke each command once and retain the original exception. Logging file errors are reported without masking command failure. Stage durations are inclusive and must not be summed. Publication includes a nested capture; atomic write includes native encoding/validation. Native internal conversion/writer/reread substeps still require further native profiling if optimization is resumed.

Run inside Rhino5:

```text
_-RunPythonScript "H:\FreeCAD-src\build\om9-perf-dev\tests\rhino5_verify_phase12.py"
```

## Scoped actual application probe

Three fixtures — ring, edited/new curves and mesh — PASS204Rhino checks,3owned OM9 reports/104checks.54events/12actual Copy/Paste totals. IronPython clock/import and native-thread timing wrappers executed in actual Rhino5/FreeCAD; all report/macro/runtime/fixture hashes and owned PID/exit0 bindings validated.66tool tests (6timing-specific) and12report-reader cases PASS. The actual main PowerShell final report writer produced54events and12ranked totals from these real application logs. This is not a replay of the complete updated user entry.

| Fixture | Rhino Copy (s) | OM9 Paste (s) | OM9 Copy (s) | Rhino Paste (s) |
|---|---:|---:|---:|---:|
| current-ring | 0.116 | 6.165 | 17.091 | 0.075 |
| edited-and-new | 0.029 | 0.611 | 0.881 | 0.010 |
| current-mesh | 2.479 | 0.831 | 1.990 | 0.030 |

Ring OM9 Copy17.091s: native encode/validate15.759s, GUI snapshot0.459s, clipboard publication0.796s. Its full owned OM9 session111.219s also includes expensive correctness oracles and persistence; it must not be presented as Copy time. These figures locate the dominant boundary; they do not demonstrate a performance improvement or isolate the native writer's internal root cause.

[Rhino report](H:/FreeCAD-src/build/rhino5-phase12-user-clipboard/fda5c6bc4f6c40268a7b08afe5db2ac2/rhino5-results.json), [timing summary](H:/FreeCAD-src/build/om9-perf-dev/build/phase12-timing-summary/29f5ce48fbb544079205a597f01a757f/timing-summary.json), [machine evidence](phase12-copy-timing.json).

Prior failed probes retained: sandbox temp/clipboard access; raw archive-count assertion incorrectly included definition records. No failed probe is relabeled PASS. Unit-suite temporary-directory sandbox errors were rerun with an owned workspace temp directory:66/66PASS. No runtime rebuild or skill business-rule change.

One full14fixture updated user-entry timing replay is prepared/pending; this existing verifier replay is separate from implementation packages.7full-openNURBS packages remain open; total future batch count unknown. Next action: user runs the same entrypoint to collect live timings on the full automatic matrix; optimize the dominant measured stage only after deeper native attribution.


## Rhino5 timing reader compatibility correction — 2026-10-09

User9974 run remains FAIL. Its caller hit `TypeError: expected str, got bytes` while reading JSONL. IronPython5 `open(...,'rb').read()` returns `str`, unlike CPython3 bytes. Prior application probes tested writing logs and the PowerShell summary; they did not execute the caller's live reader in IronPython, which was a verification gap. Native Copy/Paste code/runtime unchanged.

The child continued and reported14fixtures/621Rhino checks,38OM9reports/770checks,175saved reread checks. Each owned host report/hash/PID/exit0/assertion set verified, but the final immutable-test-script guard detected a changed `rhino5_verify_phase12.py`, so full acceptance is explicitly rejected. The caller file was observed restored to its older92line body before this repair edited it. Evidence does not establish the writer. Repair waited until the child completed; no monitored helper/caller file was changed by this repair while that process ran. [User child report](H:/FreeCAD-src/build/rhino5-phase12-user/9974f4fd1aa94ffe8df81bcab884f987/phase12-verification.json), [caller traceback](H:/FreeCAD-src/build/rhino5-phase12-user/9974f4fd1aa94ffe8df81bcab884f987/manual-entry-error.txt). Do not collapse the positive assertion counts into a full PASS.

Reader now normalizes ASCII stream output into text, uses a text newline and parses text JSON; Unicode is escaped in JSONL, so character length equals byte count, including CRLF. Partial lines remain pending, complete lines emit exactly once. Formatting uses Unicode. Entry reloads the helper from disk to discard an earlier cached reader. Reader I/O, JSON and TypeError faults warn visibly while authoritative child results still control PASS/FAIL. Original checks/geometry commands are not retried or relaxed. The legacy launcher was observed repeatedly rewritten to its old body even after the first reader probe. Its writer remains unidentified. The corrected body is now a fresh file `rhino5_verify_phase12_timing.py`; its exact raw file hash equals the actually compiled caller snapshot saved by the successful Rhino probe. Legacy filename is preserved, and script immutability validation remains intact.

Actual Rhino5 RED [reader TypeError](H:/FreeCAD-src/build/rhino5-timing-reader-regression/5bb910894a9846bb9192b483319c334b/results.json); actual Rhino5 GREEN [12/12reader/caller checks](H:/FreeCAD-src/build/rhino5-timing-reader-regression/7c38ad16b42846b0b1ea6aeefe0d60c2/results.json): CRLF/partial line/Unicode, live native Rhino output, no duplicate lines, missing directory, cached-module reload, and nonfatal reader warnings. Both exact executed helper/caller sources saved beside GREEN.7timing unit tests and total67tool tests PASS;12report-reader cases PASS. One log fixture is used in this regression, not12geometry fixtures. Diagnostic fixture/bootstrap failures retained separately; only the final verified runtime probe is accepted.

One existing14fixture full user-entry replay prepared/pending.7full-openNURBS packages remain open; total future batch count unknown. Next: run `_-RunPythonScript "H:\FreeCAD-src\build\om9-perf-dev\tests\rhino5_verify_phase12_timing.py"`; keep test script files unchanged until it returns PASS/FAIL. This correction is test bootstrap glue required by Rhino/FreeCAD APIs, with no Rust/native product migration or skill business-rule update.


## Full timing-entry replay 4f741966 — PASS

[User application report](H:/FreeCAD-src/build/rhino5-phase12-user/4f741966ee8541e096b4a4f80a90dc4f/phase12-verification.json): 14 fixtures, 621 Rhino assertions, 38 owned OM9 reports / 770 assertions, and 175 saved-file reread assertions PASS. All 38 report hashes/owned PIDs/exit codes/macro and executable hashes, 36 loaded-module hashes, both Rhino reports and all 203 test-script hashes verified against current files. The earlier 9974 failed attempt remains historical. The corrected full timing entry is now verified; zero prepared replays remain for this correction.

[Timing summary](H:/FreeCAD-src/build/rhino5-phase12-user/4f741966ee8541e096b4a4f80a90dc4f/timing-summary.json): 198 successful events and 44 actual Copy/Paste totals, bound to the reports from this run, with no logging errors. Ring OM9 Copy 16.146s, native encode/validate 14.925s (92.4%), GUI snapshot 0.375s and clipboard publication 0.795s. Ring OM9 Paste 5.556s. Mesh OM9 Copy 1.863s and Rhino Copy 2.357s. Other OM9 Copy totals 0.739–0.854s; small-object clipboard publication takes 0.471–0.607s. Stages are inclusive and must not be summed. The ring owned OM9 session of 101.163s includes startup, oracles, save/reopen and exit; it is not Copy duration.

Product runtime unchanged; logging proves attribution at the current coarse boundaries, not a performance improvement or that Python is the main bottleneck. Next: Profile native encode/validation sub-stages for ring Copy and clipboard publication overhead for small objects; preserve current-geometry and rollback checks. Internal native conversion/write/reread costs still require finer profiling. Seven full-openNURBS packages remain open; total future batch count unknown.
