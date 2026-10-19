# Hợp đồng viewport/scene (đánh giá gate M00-L3-03-01)

## 1. Bao nhiêu viewport, mỗi view giữ gì

- **4 viewport** theo `dock_layout`: rail trái/phải 300 pt, gap 2 pt → mỗi view `(W−602)/2` (`dock_layout.rs:91`).
- Mỗi view có `ViewState` riêng, gom theo `ViewId` (`ui.rs:326`); camera tách khỏi scene (`camera.rs:41,62`):
  `Viewport { origin, size }`, `Camera { eye, target, up, projection, near, far, revision }`,
  `Projection::{Orthographic, Perspective}`.

## 2. Chia sẻ scene

- **Một** `Scene` cho mọi view: `Scene::capture`, `Scene::revalidate_in_view`, `Scene::pick`, `cache_usage`.
- `SceneObject` giữ pick geometry + display transform; pick nhận `screen`, `camera`, `viewport`,
  `tolerance_pixels`, `PickIntent` và trả `PickHit` (`scene.rs:805`).

## 3. Bất biến

1. Camera **không** được sửa scene; chỉ `revision` thay đổi khi camera đổi.
2. Pick là hàm của (screen, camera, viewport, tolerance, intent) — không phụ thuộc thứ tự vẽ.
3. Mọi viewport dùng chung dữ liệu scene; khác nhau chỉ ở camera và view state.
4. Command event thuần (không egui, không document) — UI chỉ phát sự kiện, xử lý ở tầng session/transaction.
