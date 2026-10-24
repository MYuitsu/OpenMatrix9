# Proposed reusable rule — not applied

User instruction in root AGENTS.md: after verified coding that changes behavior,
present the before/after and proposed skill text, then request consent before
recording that rule in repository/installed skills.

Before: self-intersection repair deletes the smaller crossing face and attempts
hole fill; it can return open output.

After: checked repair splits the surface arrangement, classifies the material
boundary, and returns a closed oriented mesh or an explicit error. Open input is
unsupported by this kernel slice. Unsupported/error cases never invoke deletion
or hole-fill fallback.

Proposed addition to `openmatrix9-feature-port`, subject to user consent:

> Mesh self-overlap repair dùng kernel CSG có sai số và budget rõ ràng: split mặt,
> phân loại trong/ngoài, rebuild shell; chỉ trả thành công sau kiểm tra cạnh kín,
> hướng mặt, vertex fan và giao còn lại. Không xoá face nhỏ hơn hoặc fill hole để
> giả lập thành công. Mesh hở/không hỗ trợ trả lỗi và giữ nguyên document. Dùng
> Mesh.corner_uvs làm nguồn UV; nội suy corner mới và giữ seam. Không suy ra
> watertightness của kết quả giảm lưới từ prepass CSG.

No business rule was written to a repository or installed skill in this batch.
