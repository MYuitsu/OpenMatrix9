# OM9-L3-01-05-001 — OCCT FFI Safe Handle Lifecycle (typed API)

Owner module `OM9-L2-01-05` (Be Occt Brep Kernel Bridge). Rust-side half of the adapter; the OCCT kernel is not linked in this build, so every kernel call returns `Unsupported("occt kernel")`.
Implementation owner: `crates/om9-core/src/brep_kernel.rs (HandleStore)`.

Oracle — positive: *HandleStore cấp phát slot + generation; get/remove O(1); slot được tái dùng với generation mới.*
Negative: *Handle cũ sau khi xoá là stale (không alias object mới); xoá hai lần trả StaleHandle; store không rò slot.*

## 1. Typed API

```rust
pub struct KernelHandle { pub slot: u32, pub generation: u32 }
pub struct HandleStore<T> { .. }  // Vec slot + generation + free list + live count
impl<T> HandleStore<T> { new/with_capacity/insert/get/get_mut/remove/is_alive/len }
pub trait KernelBackend { build/destroy/transform/tessellate }
```

## 2. Errors (exact strings)

- `KernelError::StaleHandle -> Error::Stale("kernel handle")`
- `KernelError::Unavailable -> Error::Unsupported("occt kernel")`

## 3. Note

Không đụng tới UNSAFE: handle là chỉ số thế hệ trong Rust; lớp native sẽ chỉ nhận slot khi OCCT được duyệt.

## 4. Test that carries the oracle

- `cargo test -p om9-core --test brep_kernel`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-05-001/OM9-L4-01-05-001-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-05-001/OM9-L4-01-05-001-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-05-001/OM9-L4-01-05-001-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-05-001/positive.json`, `negative.json`
