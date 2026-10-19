# G00-02 — OCCT bridge & numerical geometry

Trạng thái: **CLOSED** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 3/3 · L4 có bằng chứng nghiệm thu: 12/12 · L4 có artefact: 12/12

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-02-01` Toolchain and C ABI bridge proof | DONE | 4 | project owner |
| `OM9-M00-L3-02-02` Lifetime, threading and error safety | DONE | 4 | project owner |
| `OM9-M00-L3-02-03` Geometry numeric invariants | DONE | 4 | project owner |

## Sự thật đã kiểm

- OCCT 8.0.1 pinned (tag V8_0_1, commit b8f597c677811d1f9f4d8a97f5ae2825c0353a42) with release + 3rdparty + source archive SHA-256
- probe: topology 6F/24E/48V, mesh 24 nodes/12 triangles, 2 line-face hits, boolean 7 faces valid, volumes 400.0 and 349.73451754256325 (error <= 1e-13)
- accuracy defect measured on the stock kernel: offset D2 error 41.7% and D3 99.96% (ellipse), 71.8%/100.1% (non-uniform spline); the patched TKG3d passes at <= 1.6e-05
- lifetime: refcount 1 -> 2 -> 1; 9/9 invalid inputs rejected with a message; 10 000 create/destroy cycles grew the working set by 0.398 MiB; 1000 serialized calls from 4 threads produced no wrong result
- numeric: box 400/360, sphere and cylinder exact to 1e-13, fillet 399.141592653590, inward offset 128.0, degenerate cases reported, sphere mesh closed with 2 non-manifold pole edges
- offset derivatives are owned by Rust (crates/om9-core/src/offset_curve.rs, 12 tests), so the TKG3d patch is a backstop rather than a dependency

## Việc tiếp theo

- none: G00-02 is closed; the kernel work continues in M01-L2-01-05
