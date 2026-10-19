# NURBS curve evaluation adapter — OM9-L3-01-05-003

## Hợp đồng

`curve_point_and_derivative(u)` evaluates a rational quadratic NURBS quarter circle of radius 5 built from poles/weights/knots and returns the point plus its first derivative.

## Bất biến kiểm được

at u=0.5 the point is (5/sqrt2, 5/sqrt2, 0) with |P| = 5, and the derivative magnitude is positive

## Test

`crates/om9-kernel/tests/l2_01_05.rs::the_rational_nurbs_quarter_circle_evaluates_exactly` (cùng 18 test khác của crate).

## Ca âm

a parameter outside the knot domain is not accepted by the kernel

## Nguồn

Crate sản phẩm `crates/om9-kernel`, promote từ proof harness `native/occt-rust-proof` sau khi gate
`G00-02` đóng. OCCT pin `8.0.1` (`native/occt/pin.json`).
