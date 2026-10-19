# OM9-L3-01-05-010 — Kernel Thread And Transaction Policy (typed API)

Owner module `OM9-L2-01-05` (Be Occt Brep Kernel Bridge). Rust-side half of the adapter; the OCCT kernel is not linked in this build, so every kernel call returns `Unsupported("occt kernel")`.
Implementation owner: `crates/om9-core/src/brep_kernel.rs (KernelPolicy)`.

Oracle — positive: *Policy rảnh nhận call và transaction; policy thread-locked chỉ nhận đúng thread.*
Negative: *Call chồng -> Busy; transaction chồng -> Busy; sai thread -> Permission.*

## 1. Typed API

```rust
pub struct KernelPolicy { in_flight, open_transaction, thread_locked }
impl KernelPolicy { admit_call(&self, same_thread: bool) -> Result<()>; admit_transaction(&self) -> Result<()> }
```

## 2. Errors (exact strings)

- `Error::Busy (call/transaction đang mở)`
- `Error::Permission("kernel thread affinity")`

## 3. Note

Mutex thật nằm ở session; policy là bản ghi kiểm tra được, không cấp phát.

## 4. Test that carries the oracle

- `cargo test -p om9-core --test brep_kernel`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-05-010/OM9-L4-01-05-010-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-05-010/OM9-L4-01-05-010-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-05-010/OM9-L4-01-05-010-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-05-010/positive.json`, `negative.json`
