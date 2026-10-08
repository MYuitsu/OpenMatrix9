# Curve execution ledger — plan: docs/superpowers/plans/2026-10-04-curve-commands.md

- User resumed implementation after plan presentation; execute inline in existing `codex/matrix9-menu` checkout, preserving local edits and source refs.
- Ruling: use existing local specs without PDF review, per explicit user instruction in AGENTS.md.
- Ruling: typed native Python C API calls to FreeCAD Part bindings supply the geometry adapter. Rust retains parsing, session and output. This uses the host's native Part/OpenCascade module without a new Part C++ SDK dependency; entered text is never evaluated as code.
- Ruling: initial construction plane is world XY. Full per-viewport Matrix C-plane management remains a distinct slice; document this limitation and do not claim full coordinate-system parity.
- Semantic RED: curve_session integration tests failed because the Curve module was missing. GREEN: 8 new semantic tests pass; full Rust suite 17 tests passes.
- Native RED: curve_smoke.FCMacro fails at missing OM9CommandInput before controller implementation; artifact build/curve_smoke-1/f36d9697f1504a37b70309d316909333/results.json.
- Native build: first invocation lacked MSVC include environment (type_traits/list missing); rerun through vcvars64, not a source-code defect.
- Features remain PARTIAL until unsupported options and native verification are resolved. No whole-Curve completion claim.
