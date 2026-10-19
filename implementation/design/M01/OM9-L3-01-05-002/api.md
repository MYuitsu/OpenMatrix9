# OM9-L3-01-05-002 — B-Rep Shape Type And Validation (typed API)

Owner module `OM9-L2-01-05` (Be Occt Brep Kernel Bridge). Rust-side half of the adapter; the OCCT kernel is not linked in this build, so every kernel call returns `Unsupported("occt kernel")`.
Implementation owner: `crates/om9-core/src/brep_kernel.rs (BrepBody)`.

Oracle — positive: *Tam giác hợp lệ qua validate; shape kind đọc được; tolerance biên đúng.*
Negative: *Index thiếu, wire không nối, cạnh trùng, wire hở, face/shell sai, cạnh dài 0, cặp đỉnh gần hơn dung sai, vượt budget đều bị từ chối.*

## 1. Typed API

```rust
pub struct BrepBody { vertices, edges, wires, faces, shells, kind, tolerance }
pub const MIN_TOLERANCE / MAX_TOLERANCE
impl BrepBody { validate(&self, &Limits) -> Result<()>; shape_kind(&self) -> ShapeKind }
```

## 2. Errors (exact strings)

- `Invalid("brep tolerance") / Invalid("brep vertex tolerance") / Invalid("brep edge tolerance") / Invalid("brep face tolerance")`
- `Missing("brep vertex"|"brep edge"|"brep wire"|"brep face")`
- `Invalid("brep edge endpoints"|"brep edge closure"|"brep wire chain"|"brep wire duplicate edge"|"brep wire closure"|"brep face wires"|"brep shell faces"|"brep shell duplicate face")`
- `Invalid("geometry below kernel tolerance") — hình học ≤ dung sai kernel`
- `Budget("brep body")`

## 3. Note

So sánh dung sai là quy tắc exact (|a-b| <= tol); một cặp đỉnh cách nhau <= dung sai thân bị từ chối vì kernel sẽ collapse.

## 4. Test that carries the oracle

- `cargo test -p om9-core --test brep_kernel`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-05-002/OM9-L4-01-05-002-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-05-002/OM9-L4-01-05-002-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-05-002/OM9-L4-01-05-002-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-05-002/positive.json`, `negative.json`
