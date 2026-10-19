# Hợp đồng render (đánh giá gate M00-L3-03-02)

## 1. Đường ống đã đo

```
OCCT 8.0.1  BRepMesh_IncrementalMesh  →  mesh text (v/f)  →  gpu-probe  →  vertex + index buffer  →  wgpu
```

- Mesh: sphere r=5, deflection 0,01 → **2 542 vertex / 5 008 tam giác**, 90 600 B.
- Camera: `perspective(fovy 45°, aspect, 0,1, 100)` × `look_at(eye, target, up)`, đưa vào uniform `mat4x4<f32>`.
- Khung hình: 1280×720, RGBA8; không depth buffer (chưa cần cho 1 khối).

## 2. Bất biến đo được

| Phép biến đổi | Kỳ vọng | Đo |
| --- | --- | --- |
| orbit ±30° quanh Y trên cầu | diện tích phủ **không đổi** (đối xứng trục) | 0,36718 / 0,36717 |
| pan +2 theo X | dịch và giảm nhẹ | 0,35894 |
| zoom 2× (vào) | phủ gần kín | 1,00000 |
| zoom 0,5× (ra) | còn ~1/4 diện tích | 0,08421 |
| DPI 1,5× / 2× | **cùng** fraction, nhiều pixel hơn | 0,36710 / 0,36711 |

## 3. Đo hiệu năng

| Bối cảnh | Frame trung bình | p99 |
| --- | --- | --- |
| Vòng lặp tối thiểu (1 tam giác) | 0,242 ms | 0,366 ms |
| Scene có mesh (5 008 tam giác) | 0,755 ms | 1,278 ms |
| Scene @ DPI 2× (2560×1440) | 1,535 ms | — |

GPU memory ước tính cho scene 1280×720: **5,62 MB** (mesh 0,09 MB + target + readback).

## 4. Chưa có

- **Device loss / fallback renderer** (`L4-03-02-04`) — chưa đo.
- Chưa có depth buffer, chưa có nhiều material, chưa có line/edge pass — thuộc M01-L2-01-06.
