# Transform matrix service — OM9-L3-01-05-006

## Hợp đồng

`transform_box(dx,dy,dz,rotate_deg,scale)` applies scale*rotation*translation and returns bounds, volume and validity.

## Bất biến kiểm được

translation moves z to 5..9 without changing the 400 volume; a 90 degree turn keeps the symmetric bounds and the volume; a 2x scale cubes the volume to 3200

## Test

`crates/om9-kernel/tests/l2_01_05.rs::a_transform_moves_the_box_and_keeps_its_volume` (cùng 18 test khác của crate).

## Ca âm

a non-finite scale is refused by the bridge guard

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
