# G-MVP — checklist để nghiệm thu bởi người có thẩm quyền

**Trạng thái khi giao tài liệu:** NOT_TESTED · Không có chữ ký · Chưa phát hành.

## Chặn cứng
- [ ] G00 và E01..E12 có signed PASS, source/build/fixtures khớp.
- [ ] Pháp lý/SBOM/asset provenance được phê duyệt.
- [ ] Không còn blocker dữ liệu, BRep, interop, Undo, license, corrupted files hoặc regression.

## Danh sách L4 phải có evidence thực
- [ ] `OM9-M09-L4-GMVP-001` — Đối chiếu tiền điều kiện G00 đến E12
- [ ] `OM9-M09-L4-GMVP-002` — Tạo ma trận truy vết MVP
- [ ] `OM9-M09-L4-GMVP-003` — Audit giấy phép, nguồn và SBOM
- [ ] `OM9-M09-L4-GMVP-004` — Build sạch và nhất quán trên môi trường đích
- [ ] `OM9-M09-L4-GMVP-005` — Workbench 4 viewport, command và điều khiển
- [ ] `OM9-M09-L4-GMVP-006` — Kernel OCCT: BRep, NURBS, trim, boolean
- [ ] `OM9-M09-L4-GMVP-007` — Undo, Redo, Cancel và selection state
- [ ] `OM9-M09-L4-GMVP-008` — Lưu/mở, migration và bảo vệ tệp
- [ ] `OM9-M09-L4-GMVP-009` — Đối chiếu phạm vi nhập 3DM
- [ ] `OM9-M09-L4-GMVP-010` — Luồng thiết kế nhẫn từ rail tới setting
- [ ] `OM9-M09-L4-GMVP-011` — Recompute nhẫn và đá, conflict minh bạch
- [ ] `OM9-M09-L4-GMVP-012` — Cutter và an toàn hình học sản xuất
- [ ] `OM9-M09-L4-GMVP-013` — STL: đơn vị, winding và external viewer
- [ ] `OM9-M09-L4-GMVP-014` — Kiểm đầu vào lỗi, recovery và hiệu năng
- [ ] `OM9-M09-L4-GMVP-015` — Kiểm thử thao tác người dùng theo OS/GPU
- [ ] `OM9-M09-L4-GMVP-016` — Đóng gói, cài đặt, rollback và kênh phân phối
- [ ] `OM9-M09-L4-GMVP-017` — Triage lỗi và kiểm toán bằng chứng
- [ ] `OM9-M09-L4-GMVP-018` — Quyết định ký G-MVP GO/NO-GO

## Ký / quyết định
- [ ] Human reviewer đối soát raw logs / SHA / fixtures và ký GO hoặc NO_GO.
- [ ] Known gaps, supported hardware, license và rollback được công bố.

**Result:** NOT_TESTED (template).  **Source SHA:** unknown.  **Reviewer:** not assigned.
