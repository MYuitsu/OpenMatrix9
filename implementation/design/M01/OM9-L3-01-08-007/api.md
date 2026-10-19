# OM9-L3-01-08-007 — Coordinate Numeric Input Primitives (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/coords.rs, crates/om9-scene/src/camera.rs, crates/om9-app/src/session.rs`. Legacy decision: **PORT**.

Oracle — positive: *CPlane là frame có kiểu: local (1,2,3) trên plane dịch ([10,20,30]) thành world (13,21,32); to_local là nghịch đảo.*
Negative: *Local point không có frame là Missing; ngữ pháp x,y[,z] và nhãn lỗi giữ nguyên.*

## 1. Typed API

```rust
coords::{WorldPoint, LocalPoint, LocalFrame, CoordinateInput, resolve, parse_local}
impl LocalFrame for om9_scene::camera::CPlane { to_world = CPlane::point; to_local = dot with x, y, normal }
session.rs parses with coords::parse_local(text, unit) then coords::resolve(Local, Some(&plane))
```

## 2. Errors (exact strings)

- `Missing("coordinate frame") — a local point cannot become world without a frame`
- `Invalid("x,y[,z] coordinate") / Invalid("coordinate number") / Invalid("length conversion")`

## 3. Legacy `rust/` audit

- `rust/src/coordinates.rs`
- `rust/src/coordinate_input.rs`
- `rust/src/cplanes.rs`

Legacy coordinate input is portable logic; OM9 ports the grammar and the error labels unchanged and makes the frame an explicit typed parameter instead of an implicit context.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test coordinates`
- `cargo test -p om9-scene --test coordinate_frame`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-007/OM9-L4-01-08-007-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-007/OM9-L4-01-08-007-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-007/OM9-L4-01-08-007-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-007/positive.json`, `negative.json`
