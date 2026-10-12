# Script Rhino 5 kiểm chứng bản tối ưu Phase1/2

Chạy một lệnh trong Rhino5 đang mở:

```text
_-RunPythonScript "H:\FreeCAD-src\build\om9-perf-dev\tests\rhino5_verify_phase12.py"
```

Entry IronPython2.7 chỉ bootstrap API Rhino/.NET và in tiến độ; không chứa product logic. Nó gọi helper PowerShell để tái sử dụng các gate ứng dụng hiện có. Product validation/state/curve/snap-cache vẫn thuộc Rust; FreeCAD/Qt/OCCT/openNURBS là native bridge/kernel. Không có thay đổi product hoặc rebuild runtime trong đợt viết script này.

Script dùng runtime `H:/FreeCAD-src/build/om9-perf-sdk`, manifest SHA256 `5d06e5d1fd9070a14afb69800aa53f99c6001ddc1dc4bc159959f3e7476eac6d`. Source/runtime được verify trước/sau; không tự lấy report mới nhất hoặc chuyển sang bản cũ khi lỗi. Runtime đổi sau này phải cập nhật verifier theo một đợt nghiệm thu mới.

Luồng kiểm chứng tuần tự:

1. Rhino5 thực: ba fixture rational/periodic/placed, chỉnh/CV/Join/Rebuild/current geometry rồi SaveAs; OM9 đọc lại chính các file của lần chạy đó.
2. Rhino5 thực: 11fixture Copy/Paste hai chiều, native CAD/mesh/cloud, units/placement/block/current geometry/selection và clipboard sống sau source exit; OM9 đọc lại tất cả SaveAs.
3. 6gate Copy/Paste/rollback/cache/snap/worker60%/curve workflow;13gate topology/editor/UI/Rust ownership/exchange;1native JSON và actual Copy/Paste A/B;2warm snap workloads CAD10k và mesh/cloud triệu điểm.
4. Tổng hợp38report host; kiểm từng assertion, ownedPID/exit0, manifest/module/macro/report hash, fixture set và script immutability.

Hai report UI `core_end_snap_smoke`/`core_point_snap_smoke` hiện không báo đường dẫn/hash module đã nạp. Verifier giữ giới hạn bằng chứng này: kiểm executable/source/runtime hash và đánh dấu `loaded_module_reported=false`;36report geometry/ownership còn lại bắt buộc có hash module đúng. Không nâng report UI thành chứng minh độc lập loaded-module.

Chạy mất vài phút; thông báo stage/elapsed được in tối đa30giây một lần. Wrapper dùng timedThread.Join để bơm STA messages, guard tránh chạy lặp trong cùng session và ghi stdout/stderr riêng. Bản vẽ caller không nhận Open/New/SaveAs; tất cả hình học test thuộc tiến trình/profile riêng. Script dùng clipboard hệ thống. Timeout/error báo FAIL và giữ log; không giả PASS theo process exit hoặc số assertion đơn thuần.

Report mỗi lần chạy:

```text
H:\FreeCAD-src\build\rhino5-phase12-user\<run-id>\phase12-verification.json
```

PASS chỉ chứng minh phạm vi Phase1/2 hiện hữu và các workload đã chuẩn bị. Các ngưỡng warm snap không chứng minh tốc độ cold index hoặc toàn ứng dụng; không chứng nhận full openNURBS/Phase3–5.

## Bằng chứng triển khai

Actual owned-caller harness RED khi entry mới chưa tồn tại. PowerShell parser đã phát hiện và sửa delimiter biến trước runtime. Reader12case: valid report, failed/missing checks, wrong manifest/module/executable/PID/exit, missing geometry module observation, changed macro và2UI scoped bindings đều đạt. Existing tools60/60 đạt.

Actual default entrypoint trong Rhino5 đã đạt:14fixture621Rhino checks,38host reports/769checks,saved reread175. Caller harness 10/10 đạt: giữ geometry/attributes/selection/units/tolerance/path/document identity/dirty state, guard được giải phóng, không cảnh báo STA. [Machine evidence](rhino5-phase12-user-script.json).0prepared batches pending;7packages open;total future batches unknown.

Clipboard observer retries exactbusy only5×20ms; product Copy/Paste untouched. Sourceexit producer sequence/format and failed-observer owner/process/formats are recorded. One raw attempt stopped on busy clipboard; another on missing Rhino format while later observed clipboard belonged to Edge; those failures were retained, not converted to PASS. A two-fixture current-point/native-cm diagnostic85checks passed. The reader then caught nullable ownedPID in legacy clipboard reports; the new test-only macro now records its actual processPID. Unicode error logging uses .NET WriteAllText. Final fresh caller run passed with all38ownedPIDs. No product/source/runtime hash changed.


## Clipboard fixture correction — 2026-10-09

User run `a80470d163354641a37c7605ec499d58` is retained as FAIL. Its actual Rhino14 fixtures/621checks and saved reread175 passed; viewport Ctrl+V in the first host gate failed. The old report omitted native command stderr, so the original cause remains an inference. The same rapid-publication symptom was reproduced with the native Paste exception `Windows clipboard is busy; try Paste again` captured inside the handler. No evidence identifies a particular foreign application.

New test-only `modeling_clipboard_user_smoke.FCMacro` preserves the original23 checks and adds a focus-ready assertion. Fixture restoration uses Qt/OLE and waits for the exact archive to remain readable with an unchanged sequence for100ms, deadline2seconds. It pumps GUI messages and rejects changed/missing foreign data without republishing. Raw observer reads may wait briefly for busy locks; menu/Command/shortcut Copy/Paste handlers are untouched and never retried. Native stderr and before/after working-object counts are reported. The main verifier now records the exact failed host report, hash and error. Product Rust/native source and runtime hashes remain unchanged.

RED [captured native Paste failure](H:/FreeCAD-src/build/om9-perf-dev/build/modeling_clipboard_publication_diagnostic-1/8d40758af0d446548c32740c4438c1c1/results.json); GREEN [80 rapid-publication shortcuts](H:/FreeCAD-src/build/om9-perf-dev/build/modeling_clipboard_publication_stable-1/5e54e37f209247319c87b0c1b0505c2d/results.json): each invokes Paste once and creates two working objects;10 fixture busy reads absorbed before dispatch; foreign clipboard sentinel remains unchanged. Separate stable-clipboard control80/80 passed. Six targeted host reports/97checks passed. Tools60/60 and report-reader12/12 passed. One overlapping diagnostic attempt is excluded; other failed observer/publication attempts remain historical diagnostics.

Fresh actual Rhino caller [results.json](H:/FreeCAD-src/build/rhino5-phase12-session-test/adb190eff50c4a30bc30d71feb6ab165/results.json) passed 10/10, document snapshot unchanged, warnings empty. Full default user entry passed14 fixtures/621Rhino checks,38host reports/770checks, saved reread175. [Exact full run](H:/FreeCAD-src/build/rhino5-phase12-user/0a73cdc66d924e3a819fb1f8caeab4ff/phase12-verification.json). This replaces the current verifier status; earlier success and the user's failed run remain in machine evidence. Two UI-only loaded-module limitations remain.

Named batch: Phase1/2 clipboard-fixture regression and complete manual-entry replay. No prepared batches pending for this correction;7full-openNURBS packages remain open; total future batches unknown. Next action: user may run the same Rhino command again; continue remaining implementation packages separately. No skill business-rule change or product rebuild.


## User run 0820a4cc — Explorer replaces clipboard

Latest user execution remains FAIL; prior full PASS is historical, not relabeled as this run. Phase2 three fixtures/125Rhino checks,35host checks and55SaveAs reread checks passed. Clipboard eight of11 fixtures completed;391of392 attempted assertions passed. Ninth fixture `current-extrusion`: OM9 producer13/13 passed and published Rhino format at sequence11368, still present after closing owned documents. The Rhino consumer observed sequence11383, owner `explorer` PID13672, CF_HDROP(15)/Shell IDList Array/FileNameW/file-descriptor formats and no Rhino5 format. This is external clipboard replacement, distinct from the earlier transient busy-lock fixture correction. Evidence does not establish who initiated the Explorer operation.

[User summary](H:/FreeCAD-src/build/rhino5-phase12-user/0820a4cc36784674b95da2e84f16b114/phase12-verification.json), [Rhino report](H:/FreeCAD-src/build/rhino5-phase12-user-clipboard/eb5c369c9cce4281a67961d69a02fdd6/rhino5-results.json), [producer report](H:/FreeCAD-src/build/rhino5-phase12-user-clipboard/eb5c369c9cce4281a67961d69a02fdd6/current-extrusion/results.json). No product, runtime or verifier script changed. No application replay was needed to identify the already-recorded owner/sequence/format mismatch; no automatic republish or retry is used to disguise the lost durability proof.

Use the same Rhino entrypoint. During this clipboard test, do not Copy/Cut files, text or geometry in another application; those operations replace the shared Windows clipboard. Wait for PASS/FAIL before another Copy/Cut. One existing14fixture manual-entry replay is prepared/pending;7full-openNURBS packages remain open; total future batch count unknown. This failed execution is a retry of the existing batch, not a new fixture or implementation-completion count.


## User replay ca077635 — full PASS

[Exact user execution](H:/FreeCAD-src/build/rhino5-phase12-user/ca0776352874444ba4efb42381175320/phase12-verification.json) passed14fixtures/621Rhino checks,38owned OM9 reports/770checks and175SaveAs reread checks. All38report hashes, assertion flags, owned PIDs, exit0, macro/executable hashes and36reported loaded-module hashes validated; both Rhino report hashes/case counts and the verifier script hashes matched. Prior failures remain historical. The external-clipboard14fixture replay is complete; no replay pending for that correction. This verifies the scoped Phase1/2 behavior, not every openNURBS type or whole-application performance. The user reports some Copy cases still slow; investigate separately.7full-openNURBS packages remain open; total future batch count unknown.


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
