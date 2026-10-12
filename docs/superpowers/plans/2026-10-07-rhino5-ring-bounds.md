# Rhino 5 ring bounds diagnostic plan

> **For agentic workers:** Use superpowers:executing-plans for this authorized continuation. Track verified steps below.

**Goal:** Resolve the remaining ring bounds failure without altering geometry to fit an inaccurate measurement.

**Architecture:** Keep raw Rhino bounding-box comparison and its historical failure. Independently validate native stationary-point witnesses through pristine Rhino File3dm faces. Choose any subsequent correction only from those results.

**Tech Stack:** openNURBS, OCCT, Rhino 5 RhinoCommon/IronPython, Python report analysis.

**Spec:** `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md`.

## Global constraints

- Preserve source 3dm files and the active Rhino document.
- Keep the existing bounds threshold of 0.001 mm.
- Preserve the 49/50 RED acceptance report and script-checksum RED report.
- Diagnostic mesh/search candidates are not a proof of global extrema.
- No production geometry change until the independent diagnosis supports it.

## Review focus

- Rhino bounding boxes may omit contained points: test pristine face PointAt and IsPointOnFace.
- OCCT SameParameter may alter pcurves: do not claim its classification is independent.
- Mirrored geometry: validate both source UUIDs and all coordinate signs.
- IronPython file lifetime: explicitly hold each checksum input open.
- Missing, non-finite, Exterior or mismatched witnesses: reject diagnostic certification.

## Task 1: Independent diagnosis

**Files:** `tests/native/three_dm_bounds_diagnostic.cpp`, generated diagnostic under `H:/FreeCAD-src/build/ring-bounds-diagnostic`, report analyzer under that same folder.

**Interfaces:** Native candidates contain face index, UV and physical point. Actual Rhino report adds pristine-face physical point and IsPointOnFace relation. Analyzer consumes both and emits point deviation, bbox containment residual and input hashes.

- [x] Reproduce and preserve raw bounds discrepancy.
- [x] Produce native candidates, retain review limitation about SameParameter.
- [x] Fix explicit checksum stream lifetime and prepare readonly Rhino witness verification.
- [x] Run actual Rhino diagnostic, verify hashes and Interior witnesses.
- [x] Test analyzer rejection of exterior/non-finite/mismatched/missing witnesses.
- [x] Record the diagnosis and choose the bounded correction supported by evidence.

## Task 2: Correct and verify

- [x] If the geometry is defective, add a reproducing regression and fix the converter. If the raw bbox is defective, implement a general independently verified measurement rather than UUID-specific exceptions.
- [x] Preserve raw measurement evidence; compare corrected measurements at the unchanged threshold and test intentional displacement rejection.
- [x] Review the change, update validation/spec/progress/MANIFEST, and checkpoint source and evidence.

No integration into the public checkout or push is part of this diagnostic continuation.
