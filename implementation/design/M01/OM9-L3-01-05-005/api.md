# Face/wire/edge topology access — OM9-L3-01-05-005

## Hợp đồng

`topology(kind)` reports the **unique** vertex/edge/face/wire counts and `BRepCheck` validity for box, sphere and cylinder.

## Bất biến kiểm được

box 8/12/6/6; sphere one face and one wire; cylinder three faces and three wires; all valid

## Test

`crates/om9-kernel/tests/l2_01_05.rs::face_wire_edge_topology_is_exact` (cùng 18 test khác của crate).

## Ca âm

an unknown primitive kind returns InvalidInput

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
