# G00-03 — UI/Viewport/GPU technology

Trạng thái: **CLOSED** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 3/3 · L4 có bằng chứng nghiệm thu: 10/10 · L4 có artefact: 10/10

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-03-01` Source UI architecture and constraints | DONE | 3 | project owner |
| `OM9-M00-L3-03-02` GPU renderer stability and performance | DONE | 4 | project owner |
| `OM9-M00-L3-03-03` Picking + transactional selection | DONE | 3 | project owner |

## Sự thật đã kiểm

- eframe 0.36.2 with Renderer::Wgpu; four viewports as (W-602)/2; one shared Scene with per-view camera; command events are egui-free
- no custom wgpu pipeline/WGSL exists yet and glow is not compiled in: the 3D GPU path is the gap recorded for M01-L2-01-06
- the OCCT sphere mesh (2542 vertices, 5008 triangles) is uploaded and rendered at 0.755 ms mean / 1.278 ms p99 for 1280x720; camera orbit/pan/zoom and DPI behave as geometry predicts
- device loss: the lost callback fires once on destroy, the destroyed device refuses to draw, and a fresh device recovers in 0.337 s; the machine offers a software adapter (Microsoft Basic Render Driver, Dx12, Cpu) as the fallback
- picking: 7/7 checks on the OCCT mesh (object, face t=9.147 vs analytic 9.0, vertex distance 0, edge 1e-6, tolerance 0.0207 from 2 px); the existing gumball (15) and affine (5) tests prove one-undo-step move and cancel that keeps undo

## Còn chặn

- the three L3 need a review record before the gate closes

## Việc tiếp theo

- the owner signs L3-03-01..03, then G00-03 closes and the work moves to G00-04
