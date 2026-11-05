# Undo RAM ceiling, inventory cadence and viewport/export plan

Date: 2026-11-02. Supplement to [Undo memory retention](2026-11-02-undo-memory-policy.md). Implements the user's clarified RAM settings and records the separately requested [viewport/export plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md). The earlier executable fingerprint belongs to the earlier patch; the new fingerprint is below.

## Implemented behavior

- Default Undo budget remains 1024 MB and minimum remains one step. MB means 1,048,576 bytes. Configurable maximum is `floor(total physical RAM / 2 / MiB)`; a machine below 2 GiB clamps the default to its ceiling. Core policy validation rejects an excessive setting before changing the active policy. Invalid persisted settings leave the active policy intact.
- Windows physical RAM capacity is obtained from `GlobalMemoryStatusEx`, not available RAM, page-file capacity or GPU memory. Safe Rust owns the 50% rule, accounting and retention. The platform adapter is one documented unsafe Windows API call in `om9-native-owner/src/system_memory.rs`, reusing the pinned `windows-sys` dependency and adding its SystemInformation feature. No new C++ code or portable unsafe policy was added. [Windows API contract](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-globalmemorystatusex).
- After each accepted recorded edit, charge a conservative bound for the newly retained baseline and journal storage. Reconcile the full journal inventory at 100 MB of accounted growth, earlier if the estimated budget is exceeded, on explicit policy changes and on Undo/Redo ownership changes. Opening Options reads cached statistics rather than traversing all retained documents every UI frame.
- The complete inventory deduplicates tracked document owners and embedded resource payloads. Between inventories the estimate may overcount shared data, intentionally causing an earlier reconciliation. The 100 MB test uses repeated edits over an 8 MB shared resource to establish this conservative counter behavior; it does not prove 100 MB of newly allocated physical memory.
- Full managed-storage inventory is not exact process RSS: allocator overhead, native heaps, independent external snapshots and GPU allocations are outside this estimate. The existing aggregate hard admission limit remains 256 MB by default and can refuse work before the 1024 MB soft budget. Minimum retained steps may exceed the soft budget. None of these settings reserve system memory.
- New/Open/Master replacement keeps the application preference. Geometry, source identity and valid Redo are unchanged by a settings update. The active desktop process and document were left untouched.

If a platform RAM query is unavailable, the adapter returns unknown, the UI reports that the 50% ceiling is unverified, and its editable range falls back to 1024 MB. Non-Windows discovery is not implemented. This fallback does not certify a hardware-enforced ceiling.

## Verification

Windows, default features, external `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`:

| Batch | Passed | Evidence established |
|---|---:|---|
| `om9-core --test undo_memory` | 7 | Half-RAM bounds/clamp, conservative 100 MB reconciliation, shared payload accounting, oldest eviction, minimum retention, contiguous Undo/Redo, failed/no-op safety, no 256-step product cap |
| `om9-core --test atomic_transactions` | 17 | Atomic publication and prior Redo protections |
| `om9-core --test transaction_scopes` | 9 | Scope/savepoint/cancellation and nonrecording behavior |
| `om9-core --test resource_transactions` | 12 | Shared resource ownership and admission atomicity |
| `om9-app --lib undo_settings::tests` | 4 | Headless egui icon/OK clicks, preferences persistence/corruption, excessive persisted budget rejection, New/Master retention |
| `om9-native-owner --lib system_memory::tests` | 1 | Actual Windows capacity query returns a plausible nonzero result |

Total: **50 tests passed**. A simulated 16 GiB capacity allows at most 8192 MB, rejects 8193 MB without changing the policy and clamps to 512 MB when capacity becomes 1 GiB. This tests arithmetic, not real low-memory pressure.

Separate executable built with `cargo rustc -p om9-app --bin om9 -j 2 -- --emit=link=L:/om9-target/debug/om9-undo-ram.exe`. File size: 92,574,208 bytes. SHA-256: `a7fba082893d4a84dc5b29e6f1c05c1adb3d396e78ec55fd66103643b1a5901a`. Its isolated `--mcp` initialize/exit check passed. It did not attach to the live OM9 window. The ordinary executable remained in use and was not replaced.

`tools/public_source_audit.py` passed: 15,037 source files, 594 icon bindings, 593 SVG files, zero errors. `tools/prune_build_artifacts.py --apply` removed zero stale runs and retained the newest ten plus 117 older runs still cited by records. No release benchmark, physical mouse, presented-frame timing, actual near-cap allocation or desktop layout acceptance was performed in this batch.

## Plan and remaining implementation

The plan records bounded Rhino 5 native procedure evidence (names, RVAs, source lines, source fingerprint and decompiler limits) and version-specific Blender 5 source links. Neither commercial decompiled implementation nor Blender GPL code was copied into OM9.

P0 first separates existing hard memory admission from soft Undo retention, then P1–P3 separate canonical geometry, viewport meshes/GPU caching and independent STL quality. GPU cache is planned at no more than 50% of the selected adapter's VRAM, also constrained by the OS budget. P4 uses the user's confirmed **512 MB** threshold for coalesced geometry checkpoint/revalidation from canonical inputs. P5 requires matched release and desktop evidence. GPU budgeting, export separation and checkpoints are **planned, not implemented** here.

Canonical geometry must already meet document tolerance at commit. Memory accounting, Undo eviction and checkpoints do not authorize degrading committed geometry or reconstructing a final model from a maximum of 256 Undo commands. Display triangles remain derived data; export evaluates its immutable source capture with its own tolerance.

Next exact document: [PLAN.md, P0](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md#p0--memory-admission-and-baseline-measurements). Next action: inventory existing admission receipts and specify the smallest safe separation, followed by a P1/P3 export-quality slice and its independent geometric oracle. Business-rule skills remain unchanged pending consent for their exact proposed text.

## Proposed reusable skill text, not applied

Before this supplement, Undo settings had a 1024 MB default without a hardware-derived configurable ceiling, and full inventory was performed at every trimming check. After this supplement, Windows settings enforce a 50% physical-RAM configurable ceiling and full inventory is periodic/pressure-driven, with cached conservative statistics between inventories.

Proposed text for the applicable OM9 workflow/Undo test references: "Undo defaults to 1024 MiB and minimum one step; configurable maximum is 50% of total physical RAM, clamping the default on smaller machines. Reconcile full tracked history storage every 100 MiB of conservatively accounted retained growth and earlier on pressure, policy or ownership changes. Preserve valid Undo/Redo and committed geometry. Tracked inventory is not RSS, and independent hard admission remains in force. GPU budgeting and 512 MiB geometry checkpoints require separate implementation evidence."

The user's AGENTS.md business-rule policy requires consent before recording this text into a reusable skill. This document records the proposal and technical evidence only.
