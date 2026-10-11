# Gumball bốn viewport — native implementation 2026-10-11

Phạm vi: ba spec đã duyệt `OM9-INFO-020/021/022`. Một Rust session dùng chung
origin/basis/extents/operation; native Qt/Coin giữ scene và source viewport.
Không mở rộng Smart Targets hoặc Clayoo/SubD. Không đổi nền, selection color
hoặc thứ tự view qua Gumball.

## Kết quả triển khai

- Command On/Off đồng thời bốn ô; inactive first click bắt handle cùng event.
- Move/Scale/Rotate, Alt Copy, Shift Scale3D, Ctrl-start widget-only và late
  Ctrl planar Extrude ở từng ô; preview chung, một batch/Undo.
- Bounds Scale, projection culling, plane hai màu/cross, menu vòng trắng/đuôi
  chấm; logical pixels và stroke theo DPI100/150/200%, zoom/maximize/restore.
- Relocate origin/X/optional Y preview/accept/cancel cùng frame; native
  End/Mid/Point pointer snaps và cancel ở mỗi ô. Idle focus giữ relocated frame.
- Native menu Snappy/Smooth, Strength, settings/zero hide, Restore Defaults,
  orbit theo shared pivot, không thêm geometry Undo.
- Box dimension Scale, uniform Sphere Radius Scale và representable Link
  ScaleVector; native Undo/Redo, Alt Copy và delete/Undo/Redo giữ ownership.
- Native planar Mesh Segment Object frame; whole-owner/component pivot cache
  độc lập, Scale/Undo/Redo giữ Segment metadata. Native Part::Cut phụ thuộc Box
  được recompute đúng sau Scale và phục hồi đúng sau Undo.
- Native callback thay parametric topology bị reject và rollback toàn batch;
  source/passive view close, navigation và document cleanup dọn đúng owner.

## Runtime và kiểm tra

Chỉ build standalone `OpenMatrix9Gui` bằng `build/build-om9-plugin.ps1`.
Shared host/SDK `build/relWithDebInfo` được dùng lại; không rebuild FreeCAD root
hoặc copy một host khác. Runtime `build/gumball-four-view-runtime-20261011`.

Plugin SHA256:
`FF0427D720EC8CFD5D24EEDFB32FBA7E538CFB65F98E06398404DAF42284D69F`.
Shared host SHA256:
`98EEF899354A8AA84CFA2934E6E1A3C8558C7E6FFF3B89C347C06731F06018A8`.

Latest full Rust run: **874 checks / 125 suites**, exit0. Focused Gumball:
**45 / 5 suites**, gồm additive FFI record sizes, bounded/null admission,
invalid-output atomicity, rigid owner frame và signed scale reference.
Các kiểm tra toàn Rust còn bao gồm công việc Curve/RCORE song song; không
coi số đó là chứng nhận full RCORE hoặc một clean release source closure.

Native All: **29 processes / 781 checks**, exit0, cùng SHA ở trên.
Suite `H:/FreeCAD-src/build/shared-host-tests/gumball-final-clean-20261011-1d1e1e14/suite.json`.
[Machine-readable evidence](2026-10-11-gumball-four-view-completion.json).

Tái chạy trên shared host:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File build/build-om9-plugin.ps1 -RuntimeDirectory build/gumball-four-view-runtime-20261011
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File Mod/OpenMatrix9/tests/run_gumball_smoke.ps1 -PluginRuntime build/gumball-four-view-runtime-20261011 -OutputDirectory build/shared-host-tests/gumball-repeat -Scope All
rtk cargo test --locked --release --manifest-path Mod/OpenMatrix9/rust/Cargo.toml
```

Mỗi lượt native dùng OutputDirectory mới để profile không giữ Enabled/settings
từ lượt trước. Đóng app dùng runtime trước khi rebuild DLL.

All gồm core9processes, view/settings/owner/Mesh13, warm/cold2 và regression5.
Four-view65 và lifecycle19 chạy riêng ở DPI1/1.5/2. PointerRelocate25 nằm
trong cùng cohort, không cộng duplicate standalone runs. Mesh Segment frame
và native Segment retention qua affine Scale/Undo/Redo/cache có13checks.

## Lỗi đã tái hiện trước khi sửa

| Lỗi native/Rust | RED | Kiểm tra sau sửa |
|---|---|---|
| Chỉ một overlay | Accepted plugin cũ:1/4 | Four-view native scene search, first click, single Undo |
| Box setter tính Shape trước Length callback | Native parametric Scale rollback | Owner native dimensions/Radius/Link |
| Callback thay topology trong native recompute | Owner callback commits wrong Shape | Final solid intersection, rollback2checks |
| Native Link scalar journal ghi đè ScaleVector | Undo/Redo anisotropy mất | Owner ordinary Scale + Alt Copy |
| Delete làm mất Link replay registration | Scale→delete→Undo delete→UndoScale→RedoScale sai vector | Value-only owner registration giữ tới document close |
| Nét mảnh ở DPR1.5 | Actual Coin draw-style width không đạt logical profile | Native scene stroke widths ở3DPI |
| Orbit giữ center trước Relocate | Native center khác accepted pivot | Center refresh khi shared frame đổi |
| Mesh transform xóa native Segment lists | Mesh Segment frame reader sees zero segments | Native transform adapter preserves metadata; Scale/Undo/Redo retention |
| Component kế thừa/ghi đè whole-owner pivot cache | Relocated whole Mesh làm Segment nhận sai origin | Native tests kiểm tra cache isolation theo cả hai chiều |
| Tiny interior Scale bị kéo khỏi biên thật | Rust anchor-1.2 thay vì-0.1 | Inside anchor tại bounds; outside/zero fallback riêng |

Fresh final reviewer `/root/gumball_completion_review` nêu3P2(DPR/orbit/Link
restoration) và1P1(native Mesh transform mất Segment metadata). Các lỗi đã sửa;
reviewer recheck cache isolation và independent expected geometry của Part::Cut,
không thấy concrete residual. Bộ tổng hợp native29processes đã đạt; không suy
pass từ nhận xét reviewer.

Read-only documentation check xác nhận3metadata rows,6manifest hashes và63local
links, đồng thời giữ các acceptance gate chưa đạt. Public source audit đạt
2640source files,511icon bindings,510SVG và zero errors tại thời điểm kiểm tra.

## Ranh giới còn mở

Generic/expression-driven parametric Scale, Link shear/external-document,
Mesh edge/vertex/light/camera Object frames chưa có exact owner adapters; reject
có lý do thay vì bake Shape hoặc chọn World ngầm. Whole Mesh/Points, native planar Mesh Segment và native
curve/face frames có adapter. Nonplanar/degenerate/inconsistent Mesh Segment
hoặc multiple components giữ alignment trước và báo UnsupportedFrame. Vì vậy status tổng020/021/022 giữ
`partially_implemented` / `validated_supported_slice`, chưa claim mọi
representation hoặc full original-host parity.

Cache pivot process-local256entries, không claim cold FCStd widget persistence.
Không thêm associative History graph. Geometry/report/layer/Link/retained3DM
FCStd proof nằm trong warm/cold/regression suite. Failed posttransaction
callback rollback trên shared host làm mất Redo cũ; preflight rejection giữ Redo.

Safe Rust sở hữu portable layout/settings/ray/scale admission; C++ giữ OCCT,
Qt/Coin, native property notifications và FreeCAD transaction. Link replay
adapter bounded131072value owner IDs; transient User3 phục hồi trong synchronous
notification, không lưu flag hoặc raw object pointers trong Rust/cache.

Không có release/push/version claim trong bản native evidence này. Checkout
chứa dependency RCORE/Curve đang sửa song song chưa commit; staged Gumball
changes không tạo một clean build/release closure riêng. Version bb chỉ tăng
khi đủ acceptance toàn spec; aa chỉ theo quyết định người dùng.
