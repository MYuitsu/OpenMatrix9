# Shape tessellation interface — OM9-L3-01-05-008

## Hợp đồng

`sphere_mesh(radius, tolerance)` tessellates the sphere and reports the largest node deviation, the triangle count, the boundary edges and the non-manifold edges.

## Bất biến kiểm được

deviation <= deflection at 0.01 and 0.001, zero boundary edges (closed), positive triangle count

## Test

`crates/om9-kernel/tests/l2_01_05.rs::the_tessellation_stays_on_the_exact_sphere` (cùng 18 test khác của crate).

## Ca âm

a non-positive deflection is refused before the mesher runs

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
