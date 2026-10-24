# M02 `OM9-L2-03-06` — Febe Viewport Pictureframe And Calibration

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-06` (M02)

## Audit result

Picture Frame was **not implemented** before this audit. Added the reference-image service and
project persistence in the existing Rust app:

- PNG/JPEG import with bounded decode, immutable bytes, SHA-256 and aspect-locked quad on the active CPlane.
- Move/scale/bounds with typed rejection of zero/negative sizes.
- Opacity, lock and visibility policy; locked/hidden references are visible but not selectable.
- Known-distance calibration in mm/pixel.
- Embedded `.om9` manifest with base64 image bytes, missing-external warning and hash-mismatch block.
- Viewport texture painting: SHA-keyed texture cache, projected quad and opacity tint.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Image import | `RasterAsset::from_png`, `PictureFrame::import_png` | unit 5, persistence 2 |
| `-002` | Position/scale | `translate`, `set_width`, `bounds` | unit 5 |
| `-003` | Transparency/lock | `set_opacity`, `set_locked`, `selectable` | unit 5 |
| `-004` | Known-distance calibration | `calibrate`, `mm_per_pixel` | unit 5 |
| `-005` | Reference persistence | `PictureFrameStore`, `om9.pictureframe.v1` | unit 5, persistence 2 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `om9-app pictureframe unit` | **6 passed** | `implementation/evidence/M02/2026-10-22-pictureframe-unit.log` |
| `om9-app pictureframe_persistence` | **3 passed** | `implementation/evidence/M02/2026-10-22-pictureframe-persistence.log` |
| `om9-app --features gpu-sphere --lib` | **161 passed** | `implementation/evidence/M02/2026-10-22-pictureframe-app-lib.log` |
| App/workspace clippy | exit 0 | `implementation/evidence/M02/2026-10-22-app-clippy-final.log` |
| `cargo fmt --all -- --check` | exit 0 | `implementation/evidence/M02/2026-10-22-app-fmt.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-pictureframe-public-source-audit.log` |

## Manual gate and limits

- **M02-UI human review is still pending.** The service and persistence are tested headlessly;
  the actual viewport image draw and file-picker flow are not visually signed off.
- macOS runtime is unverified.
- PNG and JPEG are supported; other raster formats remain unsupported by this service.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-06-pictureframe-calibration-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-06-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-06-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-06-00*/`
