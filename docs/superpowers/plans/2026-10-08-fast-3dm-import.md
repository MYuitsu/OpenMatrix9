# Faster preservation import — OM9-FILE-012

Preserve all import policies, data, validation, transactional errors and CAD
Wireframe behavior. Work in the development source and a separate runtime;
keep the user's open app/document and the original archive intact.

- [x] Profile supplied oval ring in native Part and OM9, separating preparation,
  binding and first display. Baseline is25–32 seconds plus first display.
- [x] Identify native substages, then eliminate measured repeated work. If
  parallel conversion is warranted, use bounded process-isolated jobs with private archive/model
  state, deterministic result ordering, separate staged file namespaces and join all
  jobs before transaction/cleanup. No GUI/Python properties in worker threads.
- [x] Compare serial and optimized decoded source records/signatures/geometry;
  benchmark unprofiled end-to-end import including first display.
- [x] Run malformed/rollback, preservation/edit/reopen/Undo and native-menu/
  wireframe regressions on final binary; review and record hashes/progress.
- [x] Open final runtime for user and provide measured timings and limits.

This is a behavior-preserving performance refactor; existing skill business
rules remain valid. Do not relax geometry acceptance or silently drop records.
