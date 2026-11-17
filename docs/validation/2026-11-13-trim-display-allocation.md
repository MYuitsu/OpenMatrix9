# Trim display allocation classification

The trimmed-surface display builder allocates scanline, trapezoid, vertex and
index vectors through one helper. It previously mapped any `try_reserve_exact`
failure to `Budget("trim decomposition")`, which the existing UI memory
recovery classifier does not recognize. The helper now checks vector layout
overflow first and reports an actual allocator refusal as `Budget("bytes")`.
The latter is recognized by the existing consent-based recovery classifier;
an impossible layout remains a distinct overflow and is not treated as a
recoverable memory shortage.

On Windows, a valid but unaddressably large reservation exercised the Rust
allocator refusal path; a separate oversized-layout test exercised the
structural overflow path. The UI classifier recognized the memory error and
rejected the overflow for recovery. The existing headless consent/retry test
also passed, as did the complete `om9-scene` suite. This was not physical RAM
exhaustion, and the allocator refusal was not injected end to end from scene
construction through the desktop UI. The change does not extend recovery to
commands without an existing recovery handler. The current desktop binary was
not replaced.
