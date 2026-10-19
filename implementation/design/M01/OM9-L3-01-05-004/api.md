# NURBS surface evaluation adapter — OM9-L3-01-05-004

## Hợp đồng

`surface_point_and_normal(u, v)` evaluates a radius-5 sphere and returns the point and its unit normal.

## Bất biến kiểm được

|P| = 5, |N| = 1, and N is parallel to P (the radial direction)

## Test

`crates/om9-kernel/tests/l2_01_05.rs::the_sphere_normal_is_the_unit_radial_direction` (cùng 18 test khác của crate).

## Ca âm

a degenerate parameter pair is rejected instead of returning a zero normal

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
