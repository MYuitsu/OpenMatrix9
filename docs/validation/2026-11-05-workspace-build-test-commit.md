# Build/test toàn workspace và checkpoint — 2026-11-05

Theo yêu cầu: build lại, chạy một lượt test rồi commit toàn bộ file hợp lệ trên `hieu-nang`. **Release build đạt; regression suite chưa xanh: 4.497 passed, 21 failed, 183 ignored.** Không chạy lại suite hoặc thay assertion để đổi kết quả. Các lỗi cần được đối chiếu contract trước khi sửa; không mặc định tất cả là lỗi oracle hoặc tất cả là lỗi production.

## Cấu hình và kết quả

- Base `d20a03029`; staged tree trước validation `d14402dc6da4764b3d29f363703e1045fda9a67b`, 651 file trong diff. [Kết quả có cấu trúc và failure excerpts](2026-11-05-workspace-build-test-commit.json).
- Windows x64, workspace defaults gồm `occt-brep`, SDK OCCT theo pin 8.0.1/openNURBS đã cấu hình. `CARGO_TARGET_DIR=E:\om9-surface-snap-validation`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`, Cargo `-j 2`.
- `cargo build --release --workspace --locked -j 2`: exit 0, 425,96 giây. [Log](../../build/standalone-validation/ui-artifacts/workspace-final-20261105-081437/release-build.log).
- `cargo test --workspace --locked --no-fail-fast -j 2 -- --test-threads=1`: **một invocation**, exit 101, 2050,08 giây gồm 7 phút 27 giây chuẩn bị test target. **4.497 passed / 21 failed / 183 ignored**, 427 terminal target summaries gồm cả target không có test. Không coi 427 là số batch hoặc fixture. [Log](../../build/standalone-validation/ui-artifacts/workspace-final-20261105-081437/workspace-tests.log).
- Release `om9.exe` chạy headless create/inspect fixture: cả hai exit 0, đọc lại 4 object, restoration ready. Đã đồng bộ runtime DLL vào thư mục release. Đây là bằng chứng loader/CLI, không là desktop/GPU presentation hoặc kiểm chứng hình học toàn miền. [Inspect](../../build/standalone-validation/ui-artifacts/workspace-final-20261105-081437/release-inspect.stdout.log).
- Adapter 3DM build bằng `tools/build-native-exchange.ps1 -OutDir E:\om9-surface-snap-validation\release`: exit 0, 14,08 giây. [Log](../../build/standalone-validation/ui-artifacts/workspace-final-20261105-081437/native-build.log). Binary SHA-256 `41509fd16acaef8f45fd32a8ee925c51c408ed6cdd59d0ae6d4ec686a4768d93`.
- Python syntax: 16 staged files; PowerShell parser: 4 staged scripts, đạt. Tool unittest được chạy một lượt: 12 tests, 7 error records do thiếu PDF Rhino 5 được fixture yêu cầu; suite này không đạt. Audit nguồn trên workspace hiện có là kiểm tra riêng. [Tool log](../../build/standalone-validation/ui-artifacts/workspace-final-20261105-081437/tool-tests.log).

## Các target Rust thất bại

- `-p om9-app --lib`
- `-p om9-app --test escape_commands`
- `-p om9-app --test file_widgets`
- `-p om9-app --test global_status_surface`
- `-p om9-app --test layer_dialogs`
- `-p om9-app --test mesh_fill`
- `-p om9-app --test shell_docking`
- `-p om9-app --test solid_difference_ui`
- `-p om9-app --test transform_corrections`
- `-p om9-app --test ui_events`
- `-p om9-app --test viewport_layout`
- `-p om9-plugin-host --lib`

Các failure không được bỏ hoặc retry trong lượt này. Icon audit quan sát 31 slot trong khi oracle kỳ vọng 20; nguyên nhân/contract cần xác minh. File widgets, Escape, tight-budget dialogs, mesh fill, docking, Solid/Transform/UI axes và plugin staging có failure riêng; test controller đã đạt không thay thế bằng chứng cho các đường này.

### 21 test thất bại

- `ui::icon_size_audit::icon_size_audit_all_categories_at_common_resolutions_and_scales`
- `ui::icon_size_audit::icons_resize_in_the_same_window_and_restore_without_changing_slots`
- `escape_discards_builder_and_setting_previews_without_document_changes`
- `close_save_failure_at_active_path_keeps_prompt_draft_and_path`
- `failed_picker_path_shows_retry_without_loop_and_cancel_preserves_redo`
- `held_opening_enter_cannot_accept_save_on_the_new_prompt`
- `overwrite_replace_button_accepts_archive_once_and_replayed_reply_cannot_replace`
- `owned_overwrite_cancel_and_stale_picker_response_keep_a_newer_owner`
- `real_file_menu_cancel_and_toolbar_picker_reply_preserve_or_replace_exactly_once`
- `shortcuts_preserve_numeric_draft_and_save_as_requests_a_new_path_after_save`
- `x1_escape_still_cancels_a_running_command_under_a_failure`
- `empty_show_confirmation_needs_no_mutation_buffers_in_a_tight_budget`
- `a_mesh_object_fills_its_faces_after_the_grid`
- `builder_area_stays_reserved_and_non_interactive`
- `invalid_command_stays_visible_in_the_status_surface_and_the_transcript`
- `left_column_renders_the_documented_sections_with_builders_below_projects`
- `viewport_pipe_mixed_chain_accepts_endpoint_and_station_points`
- `blockers_refuse_instead_of_reusing_unrelated_grips_or_inventing_facing`
- `construction_grid_colors_only_positive_axes_like_the_reference`
- `jewelry_grid_axes_follow_each_view_cplane`
- `staged::tests::attribute_and_label_edits_are_staged_and_readable_before_commit`

Chi tiết assertion nằm trong JSON kèm theo và log gốc. Các nhóm đạt như FileState core, Top11, GPU publication hoặc STL backend chỉ giữ phạm vi của chính chúng; không đóng các gate UI/accuracy còn lỗi hoặc chưa chạy.

## Native inventory: một attempt thất bại, không là batch đạt

Named attempt: `workspace-final-20261105 / native-source-inventory`, dùng matrix hiện có của `tools/validate_native_source_inventory.py`. Build adapter mới đạt nhưng validator dừng ở **1 fixture packet `canonical.json`, 1 native call `--write`, 0 file 3DM được kiểm chứng, 0 source inventory views hoàn tất**. Native owner trả `NativeExit(1)`, stderr `OM9 native exchange (packet preflight): missing/unknown record fields`; active child processes sau kết thúc bằng 0. [Log](../../build/standalone-validation/ui-artifacts/workspace-final-20261105-081437/native-inventory.log).

Chưa xác định bên fixture hay native contract cần thay; không gọi đây là expected rejection của negative corpus. Các fixture và circle oracles phía sau chưa chạy; không báo chúng passed. Roadmap giữ prepared queue hiện có: 1 batch legacy chờ Rhino 5 (4 fixtures/7 objects), standalone prepared list đang rỗng. Packages 2–8 vẫn mở, full future batch count **unknown**; attempt này không tạo thêm planned batch hoặc đóng package. Tiếp theo khi được yêu cầu: đối chiếu packet/schema trước khi retry đúng batch; không chạy lại toàn workspace vì việc đó.

## Phạm vi commit và vệ sinh

Toàn bộ thay đổi code/test/assets/tài liệu không bị ignore được đưa vào checkpoint theo yêu cầu; không stage build artifacts, DLL, raw private references hoặc models. Phiên OM9 đang mở và bản vẽ giữ nguyên. Không push/publish.

Sau validation chỉ chuẩn hóa line endings/EOF ở 24 file và whitespace TOML. Helper xác nhận parsed TOML/JSONL, nội dung Rust trước token cuối và văn bản quy tắc skill không đổi; proof hashes nằm trong JSON. Đây không là sửa logic hoặc sửa 21 failure. Không build/test lại sau thay đổi thuần whitespace/tài liệu. Record này không tuyên bố toàn bộ plan viewport/export đã hoàn tất hoặc bản hiện tại là regression baseline xanh.

Kiểm tra cuối: public source audit exit 0 (15.197 files, 594 icon bindings, 593 SVG, errors rỗng); staged diff whitespace đạt. Bảy local links, bốn JSON records và đủ 21 failure details được kiểm. Prune đã xác minh resolved targets nằm dưới workspace, xóa 1 run không còn được dẫn; giữ 10 mới nhất cùng các run được hồ sơ bảo vệ, không xóa target ngoài checkout. Không có build hoặc test suite retry.
