# OM9-L3-01-05-009 — Geometry Operation Error Contract (typed API)

Owner module `OM9-L2-01-05` (Be Occt Brep Kernel Bridge). Rust-side half of the adapter; the OCCT kernel is not linked in this build, so every kernel call returns `Unsupported("occt kernel")`.
Implementation owner: `crates/om9-core/src/brep_kernel.rs (KernelError)`.

Oracle — positive: *9 KernelError map sang đúng Error và giữ thông điệp người dùng.*
Negative: *Không có biến thể nào map mơ hồ; Unavailable là Unsupported (capability), NativeFault là Invalid (defect).*

## 1. Typed API

```rust
pub enum KernelError { Unavailable, InvalidShape, ClosedSolidRequired, NoSolution, BudgetExceeded, Cancelled, StaleHandle, BelowTolerance, NativeFault }
impl KernelError { message(&self) -> &'static str; into_error(self) -> Error }
impl From<KernelError> for Error
```

## 2. Errors (exact strings)

- `Unavailable -> Unsupported("occt kernel")`
- `Cancelled -> Error::Canceled`
- `BudgetExceeded -> Budget("kernel operation")`

## 3. Note

Thông điệp giữ nguyên văn để command line hiển thị; contract này là nửa Rust của L3-009.

## 4. Test that carries the oracle

- `cargo test -p om9-core --test brep_kernel`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-05-009/OM9-L4-01-05-009-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-05-009/OM9-L4-01-05-009-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-05-009/OM9-L4-01-05-009-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-05-009/positive.json`, `negative.json`
