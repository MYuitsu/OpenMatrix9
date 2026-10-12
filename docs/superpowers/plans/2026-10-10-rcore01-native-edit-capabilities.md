# RCORE-01 native Edit capability integration plan

> **For agentic workers:** Use superpowers:executing-plans inline. Meaningful
> RED/GREEN, matching host regressions and one final source review are required.

**Goal:** Consume the original common operation/type/options contract on existing
Join/Explode/Trim/Boolean enablement, native input capture and solve boundaries.

**Architecture:** Extend the existing Rust registry with typed NativeEdit options
and native backends. Native controllers retain document/view/permission context;
Rust decides declared types/options and backend before input copy or solve. Reuse
current native input-tree and mesh budgets, then existing private-copy solvers.

**Tech stack:** Safe Rust policy, value-only scalar ABI, existing FreeCAD/OCCT/
Mesh/special Explode adapters and Qt controller/command APIs, shared plugin SDK.

**Spec:** original RCORE-01 adapter items1/3/4/6 and C10/T07; the current
`docs/reviews/2026-10-10-rcore01-enabled-consumer-audit.md` locates the gap.

## Constraints and decisions

- Preserve every existing working Edit branch: curve/surface Join/Trim,
  solid/surface-only compound Booleans, polyhedral mesh Booleans and known
  annotation/block/Hatch Explode adapters. Unsupported concrete inputs reject.
- Keep missing-input commands available so they can prompt. Declaration gating
  enumerates supported types/options; it never authorizes unknown concrete input.
- Flags encode actual existing ExtendLines/ApparentIntersections/DeleteInput;
  the fixed deletion behavior of Join/Explode/Trim and Boolean2Objects modes
  remain explicit. Do not invent a new solver, option or silent mesh fallback.
- Native input-tree/mesh count/value/segment/byte gates precede geometry copies.
  Native closedness/type/homogeneous-set checks still apply before solve.
- Reuse standalone OpenMatrix9Gui/shared SDK/cache; no root FreeCAD rebuild.
- Rust owns policy. C++ resolves native facts/errors and retains GUI ownership;
  no document/Python/kernel pointer escapes into the new scalar ABI.
- Full chapter remains the objective; creation, Surface/Rebuild/cage and other
  original contracts remain open until independently implemented and evidenced.

Ruling: the original approved chapter already specifies the Rust registry and
native adapter boundaries. This implementation realizes that existing design
within the authorized continuation; it does not request a new command design.

## Review focus

Mixed solid/surface compounds; mesh conversion preserving its representation;
known special Explode sources versus unknown retained classes; default flags
and Boolean2Objects modes; source stamp/lock/Undo with rejected native bases.

## Task1 — Shared Rust registry

- [x] Add focused tests for supported native Edit types/backends/options and
  unsupported/mismatching kinds; retain current exchange/inspection behavior.
- [x] Observe meaningful policy RED, implement typed NativeEdit options and
  registry delegation, then focused/full Rust GREEN.
- [x] Add bounded value-only scalar ABI with no retained pointers or panics
  crossing the boundary; a diagnostic API exposes the same real decision.

## Task2 — Existing native consumers

- [x] Write host RED for the shared decision and current malformed basis/segment
  admission through actual GUI commands, without mutating sources or Undo.
- [x] Gate EditController availability with shared declaration; gate concrete
  ordinary/special inputs before copy and current options before every solve.
- [x] Reuse NativeInputTree and extract the existing mesh owner budget into
  CoreDiscreteSnapshot for both prepared import and Edit consumers.
- [x] Matching standalone build and host GREEN; run existing Edit/options/special
  regressions and the prior prepared CAD/mesh/cloud checks on the same plugin.
- [x] Final read-only source review, fix meaningful findings, then synchronize
  original chapter, evidence/manifest, README and ledger without full completion.

## Verification continuation — 2026-10-10

The current standalone plugin is
`ae403df557e6c8ded0b24741a92067d455327979dd3a3a1518d27dad7c6165be`;
the shared FreeCAD host is unchanged. Full Rust verification previously finished
679 tests with actual exit0; Rust production/test sources have not changed since
that run. Same-plugin final4 host runs have actual exit0 for capability20,
special-input10, affine-binding5, edit-commands115 and edit-3dm-special44.

The full final4 runner stopped at edit-options: the host exited1 without a
results report. Preserve that output; its exact exit cause is not established.
The unchanged traced replay `native-edit-edit-options-trace5-20261010` completed
57 passing checks but reached the 300-second observation timeout. It is also
excluded from accepted evidence without an actual exit0. The process was then
confirmed absent by its exact PID before another host was launched.

An independent harness defect was reproduced, not assumed to explain that exit:
`FREECAD_USER_HOME` named a nonexistent directory. FreeCAD `getCustomPaths`
discards such paths. `shared-host-profile-red1-20261010` had 3/4 failing checks
with actual exit0; creating the selected profile before startup gives
`shared-host-profile-green1-20261010`, all4 passing with actual exit0. Native
data/config/cache now stay inside that profile; Python temporary files stay
under the output's temporary directory. No production geometry or shared SDK
change was made for this correction.

The focused options replay `focused6` finished all57 checks with actual exit0.
The resumed Edit/special/Solid/Surface cases also finished successfully. The
Curve/Rebuild fixture initially stopped after11 checks because the command
frame left QMdiArea.activeSubWindow unset. Resolving the document/viewport slot
then exposed a released PySide viewer wrapper in focused7. Both failed fixture
runs remain excluded. Retaining the actual Qt parent/viewer wrappers gives
focused8: all41 original geometry/mouse/Undo/Redo checks and actual exit0.

The remaining26 prepared CAD/mesh/cloud/topology/reference/support cases were
resumed with `-Tag final4 -StartCase discrete-result`. The exact runner session
93841 is terminal with exit0. A fresh provenance check confirms all36 selected
geometry runs,4456 checks, no incomplete cases and the matching plugin above.
`build/record-native-edit-capabilities.py final4` selects focused6 for options,
focused8 for curve-spline and final4 for the other34 cases, verifies every actual
host exit/report/module/shared fingerprint, and retains excluded attempts and
the independent profile RED/GREEN. This evidence closes this integration
checkpoint only; the original chapter scope remains open.

Next construction inventory is recorded in
`2026-10-10-rcore01-native-construction-capabilities.md`: 12 existing families,
their real current options, and uncovered Surface/Rebuild/FitPoints native input
boundaries. It establishes source facts only. Full original RCORE-01, native
Edit output/results/conversion/maps and other consumers remain unfinished.

Accepted native Edit evidence:679 Rust;4456 host checks;36 matching actual
exits0. Fresh review findings fixed with special leaf RED4/10→GREEN10/10
and existing Trim option regression. Full original chapter remains in progress.
