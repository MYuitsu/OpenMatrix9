# OpenMatrix9 0.3.4-202610120539

The three approved core Gumball specifications (OM9-INFO-020/021/022) are implemented: parametric object Scale, Object Alignment and Relocate Gumball. Primitive dimensions and supported expressions remain editable. Associative Scale preserves admitted source definitions and dependencies; native Compound-derived helpers keep their appearance through Undo/Redo and fresh-process FCStd reopening. Mesh alignment uses actual vertex, edge and planar-facet input. All four viewports share the Gumball frame.

Local validation after the stock-header correction: **30 native processes / 955 checks**, one plugin SHA256 `77fa3d086a1ed9a388d81e3dd3fa1fe6ff813bad758a5954e4e9d1880c28fda3` and one host SHA256 `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`. [Detailed checks and supported limits](docs/validation/2026-10-12-gumball-core-completion.md).

Version accounting starts from 0.0.3: `aa` stays 0, `bb` increases by three completed specifications, and `cc` increases by one completed task. The suffix records Asia/Saigon time after the passing local cohort. The patch counter is retained when the specification counter increases.

GitHub Actions builds only OM9 against official FreeCAD 1.1.4 Windows x64 with Python 3.11. The candidate branch is `codex/gumball-release-0.3.4`. Its workflow repeats the 30 native cases, the full Rust suite, eight independent analytic BRep fixtures, worker integration and launcher regressions. Packaging rejects changed binaries, version or release metadata after verification. The exact candidate stock CI result is supplied by its actual Action run; this local record does not certify that result.

Download **om9-release-files** from the successful candidate [GitHub Actions run](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml?query=branch%3Acodex%2Fgumball-release-0.3.4). It contains the plugin ZIP, SetupLauncher ZIP and shared SHA256SUMS. Extract the artifact ZIP and then the SetupLauncher ZIP; run **Start-OM9.cmd**. [Hướng dẫn tiếng Việt](INSTALL.vi.md) · [English installation](INSTALL.en.md).

These are tested branch artifacts; a tag or GitHub Release is not created by a branch build. Tagged publication remains gated on metadata and main ancestry. Save and close FreeCAD before updating its loaded plugin. The launcher preserves previous plugin backups and user models.
