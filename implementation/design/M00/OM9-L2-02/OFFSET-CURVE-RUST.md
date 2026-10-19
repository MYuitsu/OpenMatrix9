# Offset curve do Rust sở hữu — `crates/om9-core/src/offset_curve.rs`

Ngày: 2026-10-17 · Trạng thái: **12 test xanh, clippy sạch, workspace check xanh**

## 1. Vì sao module này tồn tại

OCCT 8.0.1 tính sai `D2`/`D3` của offset curve (xem `OCCT-UPGRADE-GATE.md` §6): `D2` sai dấu số hạng
theo pháp tuyến, `D3` có số hạng `6·Dr²/R⁴` thừa, hệ số `6` thay vì `9`, và `D3r` thiếu `/R²`. Bản vá
`TKG3d` sửa được, nhưng nó là **backstop**: mỗi lần OCCT đổi version lại phải vá, và mọi đường nội bộ
khác của OCCT (offset surface, fillet) vẫn dùng toán của họ.

Nguyên tắc của dự án: **hợp đồng đạo hàm nằm ở Rust**, kernel chỉ là nơi sản xuất hình học solid.

## 2. Công thức đã cài (đã kiểm bằng oracle giải tích + sai phân)

```text
A = p′ × V,  R = |A|,  Dr = A·A′,  D2r = A·A″ + A′·A′,  D3r = A·A‴ + 3·A′·A″

P   = p  + d·(A/R)
P′  = p′ + d·(A′/R − A·Dr/R³)
P″  = p″ + d·(A″/R − 2·A′·Dr/R³ − A·D2r/R³ + 3·A·Dr²/R⁵)
P‴  = p‴ + d·(A‴/R − 3·A″·Dr/R³ − 3·A′·D2r/R³ + 9·A′·Dr²/R⁵
                − A·D3r/R³ + 9·A·Dr·D2r/R⁵ − 15·A·Dr³/R⁷)
```

`P‴` cần `A‴ = p⁗ × V`, nên `CurveBasis` phải cung cấp đạo hàm tới bậc 4.

## 3. API

| Thành phần | Vai trò |
| --- | --- |
| `trait CurveBasis` | `domain()` + `derivatives(u) -> Derivatives` (p, d1..d4) |
| `LineBasis`, `CircleBasis` | basis giải tích (đạo hàm đóng, chính xác máy) |
| `NurbsBasis<'a>` | rational B-spline: basis derivative bằng hệ số α (`p!/(p−k)!·Σ α·N_{i+j,p−k}`), rồi quy tắc thương cho đạo hàm Euclid |
| `OffsetCurve<B>` | `derivatives` (D0..D3), `point`, `curvature`, `deviation_against` |
| `OffsetCurve: CurveBasis` | cho phép lồng offset / so sánh bằng cùng trait |

Lỗi: hướng offset song song tiếp tuyến → `Error::Unsupported("offset direction parallel to the tangent")`;
khoảng offset không hữu hạn → `Error::Invalid("offset distance")`; tham số ngoài miền → `Error::Invalid`.

## 4. Test (12) và oracle

| Test | Oracle |
| --- | --- |
| `a_line_offset_is_a_parallel_line` | giải tích: `D2 = D3 = 0` |
| `a_circle_offset_is_the_concentric_circle` | giải tích: bán kính `r + d`, `D2 = −R·rad`, curvature `1/(r+d)` |
| `a_negative_distance_moves_the_circle_inward` | giải tích |
| `a_non_uniform_parameterisation_keeps_the_offset_exact` | **warped circle** `θ(u)=u+0.6·sin u` — offset vẫn là đường tròn chính xác, nhưng `R = r·θ′` biến thiên: đúng chế độ làm OCCT sai |
| `the_finite_difference_oracle_agrees_with_the_closed_form` | sai phân 5 điểm + Richardson của **điểm** |
| `the_occt_8_0_1_expansion_fails_the_same_oracle` | **negative control**: công thức sai của 8.0.1 (chép nguyên) phải lệch > 1e-2 — khoá regression |
| `a_rational_quarter_circle_offsets_to_the_larger_circle` | giải tích: `|P| = r + d` |
| `rational_nurbs_derivatives_match_finite_differences` | sai phân của điểm |
| `a_non_uniform_cubic_spline_matches_finite_differences` | knot không đều `0,0.15,0.55,0.85,1` + offset |
| `a_direction_parallel_to_the_tangent_is_unsupported` | hợp đồng lỗi |
| `invalid_inputs_are_rejected` | hợp đồng lỗi |
| `deviation_between_identical_offsets_is_zero` | dùng cho refit |

Kết quả: `12 passed, 0 failed`; toàn bộ `om9-core`: **374 test / 44 suite / 0 fail**; `cargo clippy
--all-targets` sạch; `cargo check --workspace` xanh.

## 5. Hai lỗi bắt được trong lúc làm (ghi lại để không lặp)

1. **Oracle `d4` của test sai** — tôi viết nhầm hệ số (`3t2² + t1t4 + t1⁴` thay vì `t1⁴ − 4t1t3 − 3t2²`).
   Phát hiện bằng sai phân của `d3`; nếu tin oracle mù thì test sẽ "đỏ giả" và dễ sửa code đúng thành sai.
2. **`NurbsBasis` mất toàn bộ basis ở mút phải domain** (mọi `N_i` bằng 0 tại `u = 1` → `w = 0`).
   Sửa bằng quy ước "mút phải thuộc span cuối" trong `basis_value` bậc 0, và dùng hệ số α thay vì bảng
   `ndu` biên (bảng đó chia cho `ndu[pk+1][r−k]` với chỉ số âm ở trường hợp `r < k`).

## 6. Quy tắc kèm theo (adapter contract)

- Ứng dụng **không** gọi `Geom_OffsetCurve::D2/D3`, `Geom_OffsetCurveUtils::EvaluateD2/D3`, hay
  `GeomAdaptor_Curve` trên offset. Mọi nhu cầu đạo hàm offset đi qua `om9_core::offset_curve`.
- Khi cần curve cụ thể để trim/boolean: **refit** từ `OffsetCurve` (sample + `deviation_against` kiểm
  tolerance) rồi mới đưa sang kernel; không dùng object offset của kernel làm nguồn sự thật.
- Vì vậy với các đường offset của OM9, patch `TKG3d` trở thành **không bắt buộc**; patch vẫn giữ cho
  các thao tác OCCT tự dùng offset nội bộ (offset surface, fillet) cho tới khi upstream sửa D3.
