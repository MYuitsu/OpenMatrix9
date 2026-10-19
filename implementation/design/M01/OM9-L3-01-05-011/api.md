# Reference shape fixture evaluation — OM9-L3-01-05-011

## Hợp đồng

`box_measures`, `sphere_measures` and `cylinder_measures` return volume and area for the three reference fixtures.

## Bất biến kiểm được

box 400 mm3 / 360 mm2, sphere 523.598775598299 / 314.159265358979, cylinder 50.265482457437 / 75.398223686155

## Test

`crates/om9-kernel/tests/l2_01_05.rs::the_reference_fixtures_match_their_analytic_values` (cùng 18 test khác của crate).

## Ca âm

a zero radius is refused with InvalidInput

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
