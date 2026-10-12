---
name: openmatrix9-native-mapping
description: Use when interpreting Matrix90 VB6 exports or Ghidra pseudocode, mapping a form event to a native address, or resolving original behavior needed for an OpenMatrix9 Rust feature.
---

# Matrix90 native evidence

Rust là ưu tiên số 1 khi triển khai hoặc chuyển code OM9. Đọc [quy tắc Rust và ranh giới native](../openmatrix9-workflow/references/rust-first.md) trước khi chọn ngôn ngữ/FFI/worker. Safe Rust sở hữu logic, validation và dữ liệu độc lập; C++ chỉ là bridge cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Ghi rõ lý do và owner của mọi ngoại lệ; không coi native FFI là memory-safe hoặc skill update là migration hoàn tất.

Use the recovered files to explain the selected behavior for Rust. Full VB6 reconstruction is a separate task requiring a user request.

1. Resolve the checkout and run openmatrix9-workflow `route native-mapping`. Read the source-of-truth, address, mapping and confidence guides returned there.
2. Locate one container/procedure in `ref/matrix9/vb6-lite/Matrix90`. Record `Attribute VB_Name`, signature, control identity and eight-digit native VA.
3. Match that VA to the **definition** of `FUN_<same VA>` in `ref/matrix9/ghidra/Matrix90.exe.c`. Read only its bounded body and necessary callers/callees. A call-site hit is not a function definition. Never load the full export into context.
4. Compare native disassembly with pseudocode for major branches, runtime calls and error paths. Keep inferred COM members/types explicit; address equality proves identity, not complete understanding.
5. Record the evidence using [mapping-playbook.md](references/mapping-playbook.md). Write derived notes under `analysis/matrix90/`; link the selected OM9 feature and update the live progress ledger. Preserve raw exports.

For parsing difficulties, read Guide `03_PARSING_VB_OUTPUT.md` or `04_PARSING_GHIDRA_C.md`. For timeout/missing body/thunk, read `08_FAILURE_CASES.md` and retain the canonical procedure address. Guide `06_RECONSTRUCTION_RULES.md`/`13_TARGET_LAYOUT.md` apply only to an explicitly requested VB6 listing.

Return a behavioral contract for the Rust feature: verified inputs/state changes/calls/error paths, uncertainties, and the smallest next evidence read. Do not label approximate C pseudocode as original C++ source or compilable recovered VB6.

## Cập nhật nghiệp vụ sau khi code

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](../openmatrix9-workflow/references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
