// SPDX-License-Identifier: LGPL-2.1-or-later
//! Add-on OM9 "Gem On Curve" — triển khai độc lập, dựa trên hành vi mô tả trong đặc tả M05
//! (Start/End Position, Spacing mm, Z Offset; chia theo độ dài cung; không trùng đá ở mối nối
//! đường cong kín; không chồng đá).
//!
//! Lệnh:
//! - `BuilderGemOnCrv` (gõ `gvGemOnCrv`): đặt đá tròn lên mỗi đường cong đang chọn.
//! - `BuilderGemCountOnCrv` (gõ `gvGemCountOnCrv`): chỉ báo số viên và khe thực.

use om9_addon_abi::sdk::{Addon, CurveData, Output, Request};

pub type Vec3 = [f64; 3];

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn len(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: Vec3) -> Option<Vec3> {
    let l = len(a);
    (l > 1e-12 && l.is_finite()).then(|| scale(a, 1.0 / l))
}
fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    add(a, scale(sub(b, a), t))
}

/// Polyline đã chuẩn bị: bỏ điểm trùng, với đường kín thì đóng vòng; có bảng độ dài tích luỹ
/// và tiếp tuyến tại đỉnh (trung bình hai đoạn kề → sai số góc bậc hai theo bước lấy mẫu).
pub struct Path {
    pts: Vec<Vec3>,
    cum: Vec<f64>,
    tans: Vec<Vec3>,
    pub closed: bool,
}

impl Path {
    pub fn new(points: &[Vec3], closed: bool) -> Result<Path, String> {
        let mut pts: Vec<Vec3> = Vec::with_capacity(points.len() + 1);
        for &p in points {
            if pts.last().is_none_or(|&q| len(sub(p, q)) > 1e-9) {
                pts.push(p);
            }
        }
        // đường kín có thể đã lặp điểm đầu ở cuối
        let mut closed = closed;
        if pts.len() > 2 && len(sub(pts[0], *pts.last().unwrap())) <= 1e-9 {
            pts.pop();
            closed = true;
        }
        if pts.len() < 2 {
            return Err("đường cong suy biến (ít hơn 2 điểm phân biệt)".into());
        }
        if closed {
            pts.push(pts[0]);
        }
        let mut cum = vec![0.0];
        for w in pts.windows(2) {
            cum.push(cum.last().unwrap() + len(sub(w[1], w[0])));
        }
        let m = pts.len();
        let dirs: Vec<Vec3> = pts
            .windows(2)
            .map(|w| unit(sub(w[1], w[0])).unwrap_or([1.0, 0.0, 0.0]))
            .collect();
        let tans = (0..m)
            .map(|i| {
                let (a, b) = if closed {
                    (
                        if i == 0 {
                            dirs[dirs.len() - 1]
                        } else {
                            dirs[i - 1]
                        },
                        if i == m - 1 { dirs[0] } else { dirs[i] },
                    )
                } else {
                    (
                        dirs[i.saturating_sub(1).min(dirs.len() - 1)],
                        dirs[i.min(dirs.len() - 1)],
                    )
                };
                unit(add(a, b)).unwrap_or(a)
            })
            .collect();
        Ok(Path {
            pts,
            cum,
            tans,
            closed,
        })
    }

    pub fn length(&self) -> f64 {
        *self.cum.last().unwrap()
    }

    /// Điểm và tiếp tuyến đơn vị tại độ dài cung `s` (đường kín: lấy theo modulo).
    pub fn at(&self, s: f64) -> (Vec3, Vec3) {
        let total = self.length();
        let s = if self.closed {
            s.rem_euclid(total)
        } else {
            s.clamp(0.0, total)
        };
        let i = match self.cum.binary_search_by(|c| c.partial_cmp(&s).unwrap()) {
            Ok(i) => i.min(self.pts.len() - 2),
            Err(i) => i.saturating_sub(1).min(self.pts.len() - 2),
        };
        let seg = self.cum[i + 1] - self.cum[i];
        let t = if seg > 0.0 {
            ((s - self.cum[i]) / seg).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let p = lerp(self.pts[i], self.pts[i + 1], t);
        let tan = unit(lerp(self.tans[i], self.tans[i + 1], t)).unwrap_or(self.tans[i]);
        (p, tan)
    }

    pub fn centroid(&self) -> Vec3 {
        let n = if self.closed {
            self.pts.len() - 1
        } else {
            self.pts.len()
        };
        scale(
            self.pts[..n].iter().fold([0.0; 3], |a, &b| add(a, b)),
            1.0 / n as f64,
        )
    }
}

/// Số viên tối đa và khe thực.
/// - Kín:  n = ⌊L/(d+g)⌋,       g' = L/n − d
/// - Hở:   n = ⌊(L+g)/(d+g)⌋,   g' = (L − n·d)/(n − 1)   (hai viên đầu/cuối chạm hai đầu đoạn)
pub fn gem_count(
    length: f64,
    diameter: f64,
    gap: f64,
    closed: bool,
) -> Result<(usize, f64), String> {
    if !(diameter > 0.0 && gap >= 0.0 && length > 0.0) || !length.is_finite() {
        return Err("cần đường kính > 0, khe ≥ 0 và đường cong có độ dài".into());
    }
    let eps = 1e-12;
    if closed {
        let n = (length / (diameter + gap) + eps).floor() as usize;
        if n == 0 {
            return Ok((0, 0.0));
        }
        return Ok((n, length / n as f64 - diameter));
    }
    let n = ((length + gap) / (diameter + gap) + eps).floor() as usize;
    if n == 0 {
        return Ok((0, 0.0));
    }
    let g = if n > 1 {
        (length - n as f64 * diameter) / (n - 1) as f64
    } else {
        0.0
    };
    Ok((n, g))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    /// Chia đều cho khít đoạn đặt đá (khe thực ≥ khe tối thiểu).
    Fit,
    /// Giữ đúng khe đã nhập, phần dư ở cuối.
    FixedGap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    /// Mặt bàn đá hướng theo pháp tuyến CPlane (halo, viền mặt nhẫn).
    Plane,
    /// Mặt bàn đá hướng ra ngoài tâm đường cong, trong mặt phẳng CPlane (eternity trên ring rail).
    Outward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub diameter: f64,
    pub spacing: f64,
    pub start_pct: f64,
    pub end_pct: f64,
    pub z_offset: f64,
    pub layout: Layout,
    pub orientation: Orientation,
}

impl Params {
    pub fn from_request(r: &Request) -> Result<Params, String> {
        let p = Params {
            diameter: r.number("diameter", 1.5),
            spacing: r.number("spacing", 0.2),
            start_pct: r.number("start", 0.0),
            end_pct: r.number("end", 100.0),
            z_offset: r.number("z_offset", 0.0),
            layout: match r.text("layout", "fit") {
                "fit" => Layout::Fit,
                "fixed_gap" => Layout::FixedGap,
                o => return Err(format!("layout {o:?} không hợp lệ")),
            },
            orientation: match r.text("orientation", "plane") {
                "plane" => Orientation::Plane,
                "outward" => Orientation::Outward,
                o => return Err(format!("orientation {o:?} không hợp lệ")),
            },
        };
        if !(0.0..=100.0).contains(&p.start_pct) || !(0.0..=100.0).contains(&p.end_pct) {
            return Err("Start/End Position phải trong 0–100 %".into());
        }
        Ok(p)
    }
}

/// Một viên đá: tâm gờ (girdle), trục u (theo đường cong), v, w (pháp tuyến mặt bàn).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub center: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

pub struct Layouted {
    pub gems: Vec<Placement>,
    pub real_gap: f64,
    pub segment_length: f64,
    pub closed_loop: bool,
}

fn frame(tan: Vec3, normal: Vec3, outward_hint: Option<Vec3>) -> (Vec3, Vec3, Vec3) {
    let u = tan;
    let mut w = match outward_hint {
        Some(o) => unit(sub(o, scale(u, dot(o, u)))),
        None => unit(sub(normal, scale(u, dot(normal, u)))),
    };
    if w.is_none() {
        // tiếp tuyến song song pháp tuyến: chọn trục vuông góc bất kỳ, ổn định
        let alt = if u[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        w = unit(cross(u, alt));
    }
    let w = w.unwrap();
    let v = cross(w, u);
    (u, v, w)
}

/// Bố trí đá trên một đường cong theo tham số Matrix.
pub fn layout(path: &Path, p: &Params, plane_normal: Vec3) -> Result<Layouted, String> {
    let total = path.length();
    let normal = unit(plane_normal).unwrap_or([0.0, 0.0, 1.0]);
    let (s0, mut s1) = (total * p.start_pct / 100.0, total * p.end_pct / 100.0);
    let full_loop = path.closed && p.start_pct == 0.0 && p.end_pct == 100.0;
    if !path.closed && s1 <= s0 {
        return Err("End Position phải lớn hơn Start Position trên đường cong hở".into());
    }
    if path.closed && !full_loop && s1 <= s0 {
        s1 += total; // đường kín: cho phép đoạn vắt qua mối nối
    }
    let seg = if full_loop { total } else { s1 - s0 };
    let (n, fit_gap) = gem_count(seg, p.diameter, p.spacing, full_loop)?;
    let gap = match p.layout {
        Layout::Fit => fit_gap,
        Layout::FixedGap => p.spacing,
    };
    let pitch = p.diameter + gap;
    let first = if full_loop { s0 } else { s0 + p.diameter / 2.0 };
    let centroid = path.centroid();
    let mut gems = Vec::with_capacity(n);
    for k in 0..n {
        let (c, t) = path.at(first + k as f64 * pitch);
        let hint = match p.orientation {
            Orientation::Plane => None,
            Orientation::Outward => {
                // hướng ra ngoài trong mặt phẳng: vuông góc tiếp tuyến và pháp tuyến, chọn dấu xa tâm
                let o = cross(t, normal);
                Some(if dot(o, sub(c, centroid)) < 0.0 {
                    scale(o, -1.0)
                } else {
                    o
                })
            }
        };
        let (u, v, w) = frame(t, normal, hint);
        gems.push(Placement {
            center: add(c, scale(w, p.z_offset)),
            u,
            v,
            w,
        });
    }
    Ok(Layouted {
        gems,
        real_gap: gap,
        segment_length: seg,
        closed_loop: full_loop,
    })
}

/// Mesh đá tròn kiểu brilliant đơn giản hoá (tỉ lệ theo đường kính d):
/// mặt bàn 56 % d, chiều cao crown 15 % d, gờ 3 % d, độ sâu pavilion 43 % d.
pub fn round_gem_mesh(d: f64, segments: usize) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let n = segments.max(8);
    let r = d / 2.0;
    let g = 0.03 * d;
    let table_r = 0.28 * d;
    let crown_top = g / 2.0 + 0.15 * d;
    let culet = -g / 2.0 - 0.43 * d;
    let mut v = vec![[0.0, 0.0, crown_top]]; // 0: tâm mặt bàn
    for (radius, z) in [(table_r, crown_top), (r, g / 2.0), (r, -g / 2.0)] {
        for i in 0..n {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            v.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    let culet_i = v.len() as u32;
    v.push([0.0, 0.0, culet]);
    let ring = |k: u32, i: usize| 1 + k * n as u32 + (i % n) as u32;
    let mut t = Vec::with_capacity(6 * n);
    for i in 0..n {
        t.push([0, ring(0, i), ring(0, i + 1)]); // mặt bàn
        for k in 0..2 {
            // crown (0→1) và gờ (1→2), pháp tuyến hướng ra ngoài
            let (a, b, c, e) = (
                ring(k, i),
                ring(k + 1, i),
                ring(k + 1, i + 1),
                ring(k, i + 1),
            );
            t.push([a, b, c]);
            t.push([a, c, e]);
        }
        t.push([ring(2, i), culet_i, ring(2, i + 1)]); // pavilion
    }
    (v, t)
}

fn place_mesh(local: &[Vec3], pl: &Placement) -> Vec<Vec3> {
    local
        .iter()
        .map(|&q| {
            add(
                pl.center,
                add(add(scale(pl.u, q[0]), scale(pl.v, q[1])), scale(pl.w, q[2])),
            )
        })
        .collect()
}

pub const GEM_SEGMENTS: usize = 24;

pub struct GemOnCurve;

impl GemOnCurve {
    fn curves(r: &Request) -> Result<&[CurveData], String> {
        if r.curves.is_empty() {
            return Err("Chọn ít nhất một đường cong trước khi chạy lệnh".into());
        }
        Ok(&r.curves)
    }
}

impl Addon for GemOnCurve {
    const COMMANDS: &'static [&'static str] = &["BuilderGemOnCrv", "BuilderGemCountOnCrv"];

    fn invoke(r: &Request, out: &mut dyn Output) -> Result<(), String> {
        let p = Params::from_request(r)?;
        let place = r.command == "BuilderGemOnCrv";
        if !place && r.command != "BuilderGemCountOnCrv" {
            return Err(format!("lệnh {} không thuộc add-on này", r.command));
        }
        let (local_v, local_t) = round_gem_mesh(p.diameter, GEM_SEGMENTS);
        for (ci, c) in Self::curves(r)?.iter().enumerate() {
            let path = Path::new(&c.points, c.closed)?;
            let lay = layout(&path, &p, r.plane_normal)?;
            let kind = if lay.closed_loop { "kín" } else { "đoạn" };
            if lay.gems.is_empty() {
                out.message(&format!(
                    "Đường cong {}: dài {:.3} mm, không đủ chỗ cho đá Ø{:.2} mm",
                    ci + 1,
                    lay.segment_length,
                    p.diameter
                ));
                continue;
            }
            out.message(&format!(
                "Đường cong {} ({kind}, L = {:.3} mm): {} viên Ø{:.2} mm, khe thực {:.3} mm",
                ci + 1,
                lay.segment_length,
                lay.gems.len(),
                p.diameter,
                lay.real_gap
            ));
            if place {
                for (k, pl) in lay.gems.iter().enumerate() {
                    // The frame (girdle center, table axis, x) lets cutters, prongs and
                    // settings find the stone; OM9 keeps it in step with Move/Rotate.
                    let v3 = |a: Vec3| format!("[{},{},{}]", a[0], a[1], a[2]);
                    let meta = format!(
                        "{{\"cut\":\"round\",\"diameter_mm\":{},\"spacing_mm\":{},\"index\":{},\"curve\":{},\"source\":\"gem_on_curve\",\"center\":{},\"axis\":{},\"x\":{}}}",
                        p.diameter,
                        lay.real_gap,
                        k,
                        ci,
                        v3(pl.center),
                        v3(pl.w),
                        v3(pl.u)
                    );
                    out.add_mesh(
                        &format!("Gem {:.2} mm", p.diameter),
                        &place_mesh(&local_v, pl),
                        &local_t,
                        &meta,
                    )?;
                }
            }
        }
        // The host reports what it actually inserted ("đã đặt N viên"); the add-on only
        // describes the layout, so a builder preview never claims that anything was placed.
        Ok(())
    }
}

#[cfg(feature = "export")]
om9_addon_abi::export_addon!(GemOnCurve);

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(d: f64, n: usize) -> Vec<Vec3> {
        let r = d / 2.0;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64;
                [r * a.cos(), r * a.sin(), 0.0]
            })
            .collect()
    }
    fn params() -> Params {
        Params {
            diameter: 1.5,
            spacing: 0.2,
            start_pct: 0.0,
            end_pct: 100.0,
            z_offset: 0.0,
            layout: Layout::Fit,
            orientation: Orientation::Plane,
        }
    }

    #[test]
    fn counts_match_formula() {
        let l = std::f64::consts::PI * 17.35;
        for (d, exp) in [(1.3, 36), (1.5, 32), (2.0, 24)] {
            let (n, g) = gem_count(l, d, 0.2, true).unwrap();
            assert_eq!(n, exp);
            assert!((n as f64 * (d + g) - l).abs() < 1e-9);
            assert!(g >= 0.2);
        }
        let (n, g) = gem_count(10.0, 1.5, 0.2, false).unwrap();
        assert_eq!(n, 6);
        assert!((6.0 * 1.5 + 5.0 * g - 10.0).abs() < 1e-12);
        assert_eq!(
            gem_count(17.0, 1.5, 0.2, true).unwrap().0,
            10,
            "khít tuyệt đối không bị làm tròn mất"
        );
        assert_eq!(gem_count(1.0, 1.5, 0.2, true).unwrap().0, 0);
        assert!(gem_count(10.0, 0.0, 0.2, true).is_err());
    }

    #[test]
    fn ring_rail_32_stones_no_overlap_axes_correct() {
        let path = Path::new(&circle(17.35, 4000), true).unwrap();
        assert!((path.length() - std::f64::consts::PI * 17.35).abs() < 1e-3);
        let lay = layout(&path, &params(), [0.0, 0.0, 1.0]).unwrap();
        assert_eq!(lay.gems.len(), 32);
        assert!((lay.real_gap - 0.2033).abs() < 1e-3);
        let r = 17.35 / 2.0;
        for (i, g) in lay.gems.iter().enumerate() {
            assert!((g.center[0].hypot(g.center[1]) - r).abs() < 1e-3);
            assert!(
                dot(g.u, scale(g.center, 1.0 / r)).abs() < 1e-5,
                "trục u ⟂ bán kính"
            );
            assert!(
                (dot(g.w, [0.0, 0.0, 1.0]) - 1.0).abs() < 1e-9,
                "Plane: mặt bàn hướng +Z"
            );
            let next = lay.gems[(i + 1) % 32].center;
            assert!(len(sub(next, g.center)) > 1.5 - 1e-6, "không chồng đá");
        }
    }

    #[test]
    fn outward_orientation_points_away_from_center() {
        let path = Path::new(&circle(17.35, 720), true).unwrap();
        let p = Params {
            orientation: Orientation::Outward,
            ..params()
        };
        for g in layout(&path, &p, [0.0, 0.0, 1.0]).unwrap().gems {
            let radial = unit([g.center[0], g.center[1], 0.0]).unwrap();
            assert!((dot(g.w, radial) - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn coarse_ellipse_tangent_error_small() {
        let (a, b, n) = (10.0, 6.0, 50);
        let pts: Vec<Vec3> = (0..n)
            .map(|i| {
                let t = std::f64::consts::TAU * i as f64 / n as f64;
                [a * t.cos(), b * t.sin(), 0.0]
            })
            .collect();
        let path = Path::new(&pts, true).unwrap();
        let mut worst: f64 = 0.0;
        for g in layout(&path, &params(), [0.0, 0.0, 1.0]).unwrap().gems {
            let th = (g.center[1] / b).atan2(g.center[0] / a);
            let exact = unit([-a * th.sin(), b * th.cos(), 0.0]).unwrap();
            worst = worst.max(dot(g.u, exact).abs().min(1.0).acos().to_degrees());
        }
        assert!(worst < 0.3, "lệch {worst}°");
    }

    #[test]
    fn start_end_and_fixed_gap_and_z_offset() {
        let line = [[0.0, 0.0, 0.0], [10.5, 0.0, 0.0]];
        let path = Path::new(&line, false).unwrap();
        let fit = layout(&path, &params(), [0.0, 0.0, 1.0]).unwrap();
        assert_eq!(fit.gems.len(), 6);
        assert!(
            (fit.gems[5].center[0] - 9.75).abs() < 1e-9,
            "Fit: viên cuối chạm cuối đoạn"
        );
        let fixed = layout(
            &path,
            &Params {
                layout: Layout::FixedGap,
                ..params()
            },
            [0.0, 0.0, 1.0],
        )
        .unwrap();
        assert!(
            (fixed.gems[5].center[0] - 9.25).abs() < 1e-9,
            "FixedGap: dư 0.5 mm ở cuối"
        );
        let half = layout(
            &path,
            &Params {
                start_pct: 50.0,
                ..params()
            },
            [0.0, 0.0, 1.0],
        )
        .unwrap();
        assert!(half.gems.iter().all(|g| g.center[0] >= 5.25 - 1e-9));
        let z = layout(
            &path,
            &Params {
                z_offset: 0.3,
                ..params()
            },
            [0.0, 0.0, 1.0],
        )
        .unwrap();
        assert!(z.gems.iter().all(|g| (g.center[2] - 0.3).abs() < 1e-12));
        assert!(
            layout(
                &path,
                &Params {
                    start_pct: 60.0,
                    end_pct: 40.0,
                    ..params()
                },
                [0.0, 0.0, 1.0]
            )
            .is_err()
        );
        // đường kín: đoạn vắt qua mối nối được chấp nhận
        let ring = Path::new(&circle(17.35, 2000), true).unwrap();
        let wrap = layout(
            &ring,
            &Params {
                start_pct: 90.0,
                end_pct: 10.0,
                ..params()
            },
            [0.0, 0.0, 1.0],
        )
        .unwrap();
        assert!(
            !wrap.closed_loop && wrap.gems.len() == 6,
            "20 % chu vi ≈ 10.9 mm → 6 viên"
        );
    }

    #[test]
    fn closed_input_with_repeated_first_point_is_not_duplicated() {
        let mut pts = circle(17.35, 400);
        pts.push(pts[0]);
        let path = Path::new(&pts, false).unwrap();
        assert!(path.closed);
        assert_eq!(
            layout(&path, &params(), [0.0, 0.0, 1.0])
                .unwrap()
                .gems
                .len(),
            32
        );
    }

    #[test]
    fn gem_mesh_is_closed_and_proportioned() {
        let (v, t) = round_gem_mesh(1.5, 24);
        assert_eq!(v.len(), 1 + 3 * 24 + 1);
        assert_eq!(t.len(), 6 * 24);
        // mỗi cạnh dùng đúng 2 lần → mesh kín (manifold)
        let mut edges = std::collections::BTreeMap::new();
        for tri in &t {
            for k in 0..3 {
                let (a, b) = (tri[k], tri[(k + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        assert!(edges.values().all(|&c| c == 2));
        let zmax = v.iter().map(|p| p[2]).fold(f64::MIN, f64::max);
        let zmin = v.iter().map(|p| p[2]).fold(f64::MAX, f64::min);
        assert!(
            ((zmax - zmin) / 1.5 - 0.61).abs() < 1e-9,
            "tổng độ sâu 61 % d"
        );
        // thể tích dương ⇒ pháp tuyến hướng ra ngoài
        let vol: f64 = t
            .iter()
            .map(|f| dot(v[f[0] as usize], cross(v[f[1] as usize], v[f[2] as usize])))
            .sum::<f64>()
            / 6.0;
        assert!(vol > 0.0);
    }

    #[test]
    fn degenerate_inputs() {
        assert!(Path::new(&[[0.0; 3], [0.0; 3]], false).is_err());
        assert!(Path::new(&[[0.0; 3]], true).is_err());
        // tiếp tuyến song song pháp tuyến: vẫn ra khung trực chuẩn
        let path = Path::new(&[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]], false).unwrap();
        for g in layout(&path, &params(), [0.0, 0.0, 1.0]).unwrap().gems {
            assert!((len(g.w) - 1.0).abs() < 1e-9 && dot(g.w, g.u).abs() < 1e-9);
        }
    }

    #[test]
    fn gem_records_carry_a_json_frame() {
        struct Sink(Vec<String>);
        impl Output for Sink {
            fn add_mesh(
                &mut self,
                _: &str,
                _: &[[f64; 3]],
                _: &[[u32; 3]],
                meta: &str,
            ) -> Result<(), String> {
                self.0.push(meta.to_owned());
                Ok(())
            }
            fn message(&mut self, _: &str) {}
        }
        let r = Request {
            command: "BuilderGemOnCrv".into(),
            curves: vec![CurveData {
                points: circle(17.35, 512),
                closed: true,
            }],
            settings: Default::default(),
            plane_normal: [0.0, 0.0, 1.0],
        };
        let mut sink = Sink(Vec::new());
        GemOnCurve::invoke(&r, &mut sink).unwrap();
        assert_eq!(sink.0.len(), 32);
        let m: serde_json::Value = serde_json::from_str(&sink.0[0]).unwrap();
        let axis: Vec<f64> = m["axis"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert!(
            (axis[2] - 1.0).abs() < 1e-9,
            "table faces the CPlane normal: {axis:?}"
        );
        let c: Vec<f64> = m["center"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert!(((c[0] * c[0] + c[1] * c[1]).sqrt() - 17.35 / 2.0).abs() < 1e-6);
        assert_eq!(m["diameter_mm"], 1.5);
    }
}
