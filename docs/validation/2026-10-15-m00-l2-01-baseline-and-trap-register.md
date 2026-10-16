# M00 L2-01 — đóng băng baseline, re-pin và sổ đăng ký bẫy

**Ngày:** 2026-10-15 (UTC) · **Phạm vi:** `OM9-M00-L2-01` (gate G00-01)
**Mục đích:** đo nguyên nhân gốc thay vì suy đoán, **chốt một baseline duy nhất**, và liệt kê từng bẫy
còn lại kèm cách chặn.

Quyết định đã được người dùng chốt: **phương án (b)** — chuyển pin sang revision khớp bản đang dùng.
Không có status nào được promote thành PASS. Không sửa nội dung định nghĩa canonical ngoài con SHA.

> **Trạng thái phạm vi pin — HOÃN (2026-10-15, sau khi người dùng yêu cầu bỏ qua hash):**
> Người dùng còn cập nhật cho M03–M07 nên phạm vi pin **chưa chốt toàn cục**. Tại thời điểm ghi bản
> ghi này, cây đang **không đồng nhất**:
>
> | Nhóm | Pin |
> | --- | --- |
> | M00, M01, M02 | `746a17f1…` (pin mới) |
> | M03, M04, M06 | `c116f8f2…` (pin cũ) |
> | M05, M07 | **lẫn cả hai** trong cùng milestone |
> | `build/*-incoming` (nguồn merge) | `c116f8f2…` (pin cũ) |
>
> Nguyên nhân: một thay đổi song song ngoài phiên làm việc này — `tools/sync_qymcad_revision.py`
> (không do phiên này tạo, chạy lúc ~6:41:55) — đã đổi M03–M07 ngược về pin cũ. Vì thế câu "0 SHA cũ
> còn lại" ở mục 2 dưới đây chỉ đúng cho **M00–M02** tại thời điểm re-pin, không còn đúng toàn cây.
> Hệ quả đã biết: M03–M07 ghi `revision = c116f8f2` nhưng `blob_sha` vẫn là digest của `746a17f1`.
> **M00 L2-01 không bị ảnh hưởng** — evidence của gate G00-01 gắn với `746a17f1` và vẫn validate 12/12.
> Việc chốt phạm vi pin toàn cục (và re-pin `build/*-incoming`) để lại cho người dùng.

---

## 1. Sự thật baseline (đã đo, không phỏng đoán)

Đo bằng cách so **blob SHA-1 toàn cây** giữa bản `QymCAD/` trên đĩa và từng revision upstream:

| Revision | Ngày | Vai trò sau re-pin | File |
| --- | --- | --- | --- |
| `09a7f09ebeb8` | 10-12 | revision **đã tham chiếu** bởi công việc 2026-10-13 (giữ làm lịch sử) | 1412 |
| ~~`c116f8f224eb`~~ | 10-14 | pin cũ, **đã nghỉ** — từng là PR head `refs/pull/238/head` | 1689 |
| **`746a17f12789`** | 10-15 | **PIN HIỆN HÀNH** — trên `main`, khớp tuyệt đối bản trên đĩa | 1698 |
| `a171295aff93` | 10-15 | `main` tip lúc audit | 1705 |

`QymCAD/` khớp `746a17f1` **1698/1698 — 0 lệch, 0 thừa, 0 thiếu**. Pin cũ `c116f8f2` lệch 33 file nội
dung và thiếu 9 file so với bản đang dùng. Đó chính là "commit trôi nổi" mà pass criteria của
`L4-01-01-01` cấm — giờ đã hết.

## 2. Đã điều chỉnh

| Việc | Trước | Sau |
| --- | --- | --- |
| Checkout ghim | không tồn tại | `vendor-research/QymCAD`, `HEAD = 746a17f1`, `status --porcelain` sạch, 1698 file |
| Provenance | chỉ link GitHub trong `l4.json` | `source-provenance.json`: timestamp, toolchain, 4 revision, license, quyết định baseline |
| Checkout log | không có | `source-checkout.log` — lệnh thật + exit code |
| Inventory | không có | `source-file-inventory.csv`: 1698 dòng, 0 path trùng, blob SHA-1 + SHA-256 + size |
| **Re-pin package** | `OM9_Milestones/` ghim PR head cũ | **M00–M02**: 3015 file, 19 642 tham chiếu SHA, 1096 `blob_sha`, 4 `blob_sha` sai được sửa — sau đó M03–M07 bị đổi ngược bởi thay đổi song song, xem banner đầu trang |
| `standalone/provenance.json` | chỉ có `commit` 09a7f09 | thêm `m00_baseline_pin` + ghi rõ `commit_role` là lịch sử, không phải baseline |
| Trạng thái | tất cả `NOT_STARTED` | 2 `BLOCKED_APPROVAL`, 10 `BLOCKED_DEPENDENCY`, **0 PASS**; 2 task đã thực thi thật |

Kiểm chứng sau re-pin (đo trên M00–M02): **3272/3272 file JSON trong `OM9_Milestones` parse được**,
`blob_sha` lệch pin = **0**, `promoted_status = 0`.

> Lưu ý về `blob_sha`: bản sửa chỉ diễn ra khi hash ghi trong bản ghi khác hash tại pin, và phạm vi thay
> thế được giới hạn trong đúng cặp `path` ↔ `blob_sha` của một object JSON. Bốn giá trị sai
> (`README.md`, `LICENSE`) không tồn tại trong cây nào — chúng là lỗi ghi từ trước, không phải do đổi pin.

## 3. Sổ đăng ký bẫy

### B1 — Baseline không xác định — **ĐÃ CHẶN**
Đo được bản copy = `746a17f1`; đã re-pin và đóng băng. Evidence từ nay đo trên `vendor-research/QymCAD`.

### B2 — Pin từng là PR head — **ĐÃ CHẶN**
Pin hiện hành nằm trên `main`. Nhưng upstream **không có tag/release**, nên lần re-pin sau vẫn phải đi
qua đúng quy trình: clone → checkout → chứng minh blob-identity → cập nhật record. Đừng ghim PR head.

### B3 — Hai bản QymCAD — **ĐÃ VÔ HẠI**
Sau re-pin, `vendor-research/QymCAD` và `QymCAD/` **cùng một revision** (`746a17f1`), nên câu hỏi "đọc
bản nào" không còn rủi ro. Quy tắc vẫn giữ: `vendor-research/QymCAD` là nguồn evidence (có `git
rev-parse HEAD`), `QymCAD/` là bản đọc hằng ngày.

### B4 — Package đặc tả không được git track — **CÒN, GIỜ RỦI RO CAO HƠN**
`git ls-files OM9_Milestones` = **0 file**; cả cây là `?? OM9_Milestones/`. Vừa rồi **3015 file bị sửa
mà không có git để diff hay hoàn tác**. Đây là bẫy nghiêm trọng nhất còn lại. Chưa commit vì AGENTS cấm
tự ý commit — cần bạn quyết định.

### B5 — Thiếu tool protocol M00 — **CÒN**
`tools/validate_m00_v013.py`, `tools/next_m00_task.py`, `tools/validate_m00_execution.py` vẫn không tồn
tại. Tôi chỉ thay được một phần rất hẹp: `tools/validate_m00_l2_01_evidence.py` (gate G00-01, từ chối
mọi bản ghi PASS).

### B6 — Lệch số lượng canonical — **CÒN**
Package khẳng định "95 L2 / 772 L3". Đếm thật (bỏ thư mục `reference`): **103 L2 tổng, 96 L2 nếu trừ
M00, 792 L3**. `L4-07-03-03` có tiêu chí "no count drift" nên G00-07 sẽ fail nếu đối chiếu với chính
con số trong runbook.

### B7 — Evidence có chiều thời gian — **ĐÃ CHẶN**
Baseline đóng băng thành commit + `timestamp_utc`; checkout là chỉ-đọc, không commit upstream.

### B8 — License chưa ai ký nhưng code đã dùng — **CÒN**
`standalone/provenance.json` khai adapter openNURBS (`eb92af3b…`) trong khi M00 `L1.json` ghi license
openNURBS là `CUSTOM_REVIEW_REQUIRED`, và QymCAD là AGPL-3.0-or-later. Thuộc G00-04/G00-06. Kết luận
license khác đi ⇒ phần native phải làm lại. Rẻ nhất: xếp lịch review trước khi viết thêm native code.

### B9 — M01 tiêu thụ quyết định của M00 — **CÒN**
2287 L4 ngoài M00 không task nào khai dependency trỏ vào M00; chỉ `M01.L1.json` khai
`depends_on_milestones: [M00]`. M01 có thể tự chốt những gì G00-01…G00-05 lẽ ra chốt trước. Cách chặn:
mỗi lần M01 chạm một quyết định thuộc G00-0x thì ghi ngay vào evidence của gate đó.

## 4. Việc chỉ con người làm được

1. **Commit `OM9_Milestones/` vào git** (bẫy B4) — nay cấp thiết hơn, vì 3015 file vừa đổi không có
   bản sao lịch sử nào.
2. **Ký review** cho `L4-01-01-01/02/04`: dữ liệu đã đủ điều kiện PASS, chỉ thiếu `review.decision=
   APPROVED` + tên reviewer theo `human_approval_policy`.
3. **Chốt con số canonical** 95/772 (bẫy B6).
4. **Xếp lịch review license** (bẫy B8) trước khi thêm native code.
5. Cân nhắc thêm `vendor-research/` vào `.gitignore` (29 MB mã AGPL bên thứ ba). `.gitignore` đang có
   sửa đổi của bạn nên tôi không tự sửa.

## 5. Tái lập và hoàn tác

```powershell
# kiểm chứng baseline hiện hành
git -C vendor-research/QymCAD rev-parse HEAD        # -> 746a17f1...
git -C vendor-research/QymCAD status --porcelain    # -> rỗng
python tools/repin_qymcad_baseline.py --verify      # -> 0 blob_sha lệch

# sinh lại evidence
python tools/record_m00_l2_01_baseline.py
python tools/record_m00_l2_01_status.py
python tools/validate_m00_l2_01_evidence.py

# hoàn tác re-pin (tool đối xứng)
python tools/repin_qymcad_baseline.py --old 746a17f127897e921a57118b6caad08968602bf5 `
                                      --new c116f8f224ebffe0ada049c91c0d15df3634f00d --apply
```

## 6. Không được suy ra từ bản ghi này

- Chưa build/test QymCAD, chưa chạy OCCT/GPU/3DM.
- Không xác nhận API Rust nào của QymCAD tồn tại ở revision ghim.
- Không nâng bất kỳ L4/L3/L2/M00 hay gate G00-01 nào thành PASS.
- Không phê duyệt license.
