# Geom intersection query adapter — OM9-L3-01-05-007

## Hợp đồng

`line_face_hits()` casts a line along +Z through a 10x10x4 box and reports the hit count and the first hit.

## Bất biến kiểm được

two hits, the first at (5, 5, 0)

## Test

`crates/om9-kernel/tests/l2_01_05.rs::a_line_through_the_box_hits_two_faces` (cùng 18 test khác của crate).

## Ca âm

a line that misses the box returns zero hits, not an error

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
