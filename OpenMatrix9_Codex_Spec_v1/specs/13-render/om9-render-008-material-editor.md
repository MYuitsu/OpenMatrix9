---
id: OM9-RENDER-008
name: Material Editor
command: null
domain: 13-render
module: Render Menu
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-008 — Material Editor

Alias tương thích: `Chưa có alias command`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Material Editor có Metals, Gem, Pearl, Colored Ground Plane, Image Ground Plane, Architectural, Material(Cloth), Glass, Plastic, Emitter. Chọn category/material rồi objects và green Apply; GV/User lọc thư viện. Save/Save As material gắn Style hiện hành, Export/Import .gvVisMat, Reset về gốc. Reflection Normal cho phản xạ overlaps, Complex bỏ overlapping surfaces nhưng cho phản chiếu nhau, Forced bỏ phản chiếu lân cận; Alpha bật transparency, Default Ground Plane override style rồi Style Save. Metal Quality ảnh hưởng report SG; chỉ User cho thêm alloy/SG,22 KY có hai giá trị 17.88/17.89 chưa thống nhất nên không lấy làm hằng. Color Grid đổi undertone/highlight-shadow, Color Picker Hue, tint/shade, RGB, X reset white, preview Update trước Apply. Reflection Amount 0 flat 1 metallic, Polish 0 matte 1 glossy, Reflection Quality 1 low 6 high; Anisotropy chỉ khi không glossy. Texture presets Bead Blast, Brushed Matte Large Fast/Slow, Brushed Shiny, Florentine, Glass Blast, Hammer Matte/Shine, Ice, Pyramid Knurl Large/Small, Rough Matte Large/Small, Sandblast Fine/Heavy, Satin, Stone Wheel; Bump Height, Map Rotation, Map Tiling U/V phụ thuộc surface UV và Rendered/Presentation mode. Per-object Scene override master environment với mapping Spherical/Mirrorball/Cubic/Angular, brightness/rotation/color; Toon On, color, Line Width, Edge Detection từ outline tới mọi góc; Light Emissive, color, Intensity. Gems Transparent/Opaque Color/Image; Transparent Image dùng Refraction Image/Multiplier và Color Image/Multiplier, chọn crop/Hue/Brightness/Saturation; Opaque Image crop/RGB, map rotation/tiling; Properties Reflection Amount/Gloss, Refraction Index Glass→Diamond, Refraction Gloss cloudiness, Dispersion, Refraction Depth số bounce, SG; Texture/Scene/Toon/Light tương tự metal. Pearl Body Color/Overtone dùng hai swatches radial gradient từ tâm tới 90° và slider transition, Overtone Intensity, Luster, Reflection Gloss, SG, Blemish Intensity predefined bump. Colored Ground Plane Single Color/Gradient; Image Ground Plane Color map không height, Texture map thêm relief; Architectural tương tự image, Cloth/Glass color theo metal/properties gem, Plastic theo metal, Emitter theo Light. Scene Editor áp không cần selection, Style Save/SaveAs, Export/Import .visstyle, Delete custom, Reset default. Environment Image mặc định hoặc Color: maps Planar/Cubic/Spherical/Mirrorball, crop/reset/Hue/Brightness/Saturation/save, brightness/Map Rotation U/V. Lighting Image/Color hoặc Physical Lighting với Style Lights, Caustics Max Photon/Multiplier/Search Distance. Camera Show Camera chỉ planar views, drag source/target đổi Perspective, Render Curves pipes curves, Vignetting, Set Camera Target, Depth of Field, F-Number thấp blur hơn, Bokeh Blade Count, Film Grain, Bloom post process Weight/Size (50 trộn nửa), Glare/ Diffraction post process.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Objects` | `object` | 0 | Không đặt trong mẫu |
| `Document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Category` | `choice` | Metals, Gem, Pearl, Colored Ground Plane, Image Ground Plane, Architectural, Material(Cloth), Glass, Plastic hoặc Emitter; mỗi nhóm có bộ điều khiển riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Library` | `choice` | GV hoặc User là scope thư viện; chỉ User cho thêm alloy/Specific Gravity. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Material` | `reference` | Identity material đang sửa/áp; tương thích .gvVisMat cần codec adapter. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Reflection Mode` | `choice` | Normal cho phản xạ giữa mặt overlap; Complex bỏ overlap surfaces nhưng giữ phản chiếu nhau; Forced giảm phản chiếu lân cận. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Alpha` | `boolean` | Bật kênh alpha cho material theo renderer capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Default Ground Plane` | `boolean` | Đặt ground-plane material override trong Style hiện hành. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Metal Quality` | `choice` | Alloy/quality có Specific Gravity trong thư viện; không lấy ví dụ densities làm constants và giữ xung đột 22KY chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Specific Gravity` | `number` | Tỷ trọng material để báo cáo trọng lượng; đơn vị tương đối, không dùng các densities minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color Grid` | `text` | Màu undertone/highlight-shadow được chọn; schema màu cần adapter chuẩn hóa. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color Picker/Hue` | `number` | Điều chỉnh hue trong picker; thang số chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color Picker/RGB` | `text` | Ba channels màu; encoding/range phải được khai báo bởi adapter màu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color Picker/Tint Shade` | `number` | Điều chỉnh sáng/tối của màu; thang số chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Reflection Amount` | `number` | 0 flat, 1 metallic cho metal; ảnh hưởng phản xạ của nhóm material tương ứng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Polish Level` | `number` | Metal: 0 matte, 1 glossy. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Reflection Quality` | `number` | Metal: 1 low, 6 high; chất lượng/độ rõ reflection, không số bounces. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Anisotropy` | `number` | Độ lệch phản xạ theo trục cho brushed effect; chỉ có khi Polish Level không glossy, thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Texture` | `reference` | Preset/asset texture surface; giữ mapping UV, không lấy tên library làm bằng chứng asset đã có. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bump Height` | `number` | Biên độ bump texture; đơn vị material adapter cần xác minh, không sửa BRep. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Map Rotation` | `number` | Góc xoay texture trên surface UV, độ theo chính sách UI OM9. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Map Tiling U` | `number` | Số/tỷ lệ lặp texture theo U; không đơn vị. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Map Tiling V` | `number` | Số/tỷ lệ lặp texture theo V; không đơn vị. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scene/Override` | `boolean` | Override environment theo object; master environment vẫn giữ nguyên nếu không override. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scene/Mapping` | `choice` | Spherical, Mirrorball, Cubic hoặc Angular cho ảnh environment của object. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scene/Image` | `reference` | Map phản xạ scene theo object. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scene/Brightness` | `number` | Độ sáng của map environment theo object; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scene/Map Rotation` | `number` | Góc xoay map phản xạ, độ theo chính sách UI OM9. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scene/Color` | `text` | Màu environment của object; schema màu cần adapter. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Toon` | `boolean` | Bật đường viền toon cho material. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Toon/Color` | `text` | Màu đường viền toon. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Toon/Line Width` | `number` | Độ dày đường toon; đơn vị renderer chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Toon/Edge Detection` | `number` | Mức phát hiện từ silhouette đến các góc bên trong; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Light/Emissive` | `boolean` | Bật emission của material, không tạo thêm light geometry. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Light/Color` | `text` | Màu emission. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Light/Intensity` | `number` | Cường độ emission; đơn vị renderer chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Mode` | `choice` | Transparent Color, Transparent Image, Opaque Color hoặc Opaque Image; mỗi mode có texture/color riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Refraction Image` | `reference` | Ảnh map khúc xạ riêng trong Transparent Image. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Refraction Multiplier` | `number` | Mức tác động ảnh khúc xạ; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Color Image` | `reference` | Ảnh màu cho Transparent Image hoặc Opaque Image. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Color Multiplier` | `number` | Mức tác động ảnh màu trong mode có multiplier. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Crop` | `reference` | Miền crop ảnh input; schema vị trí/kích thước phải khai báo trước xử lý. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Hue` | `number` | Hiệu chỉnh hue ảnh trong mode hỗ trợ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Brightness` | `number` | Hiệu chỉnh brightness ảnh trong mode hỗ trợ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Saturation` | `number` | Hiệu chỉnh saturation ảnh trong mode hỗ trợ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/RGB` | `text` | Hiệu chỉnh channels ảnh Opaque Image; colorspace/range cần adapter. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Reflection Gloss` | `number` | Độ gloss reflection của gem; không đồng nhất Refraction Gloss. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Refraction Index` | `number` | Chỉ số khúc xạ từ loại glass đến diamond; không tự đặt numeric IOR theo tên stone chưa có dữ liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Refraction Gloss` | `number` | Độ gloss khúc xạ; giảm để có cloudiness theo contract. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Dispersion Enabled` | `boolean` | Bật slider dispersion; flag kích hoạt và giá trị dispersion là hai fields riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Dispersion` | `number` | Mức tán sắc gem; cao làm ánh sáng tập trung hơn, thấp tán rộng hơn theo control được mô tả; thang renderer cần xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Gem/Refraction Depth` | `number` | Số lần bounce khúc xạ; sample policy kiểm tra count nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Body Color` | `text` | Hai swatches của radial gradient màu thân pearl; lưu định dạng gradient có version. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Overtone` | `text` | Hai swatches của radial gradient overtone từ tâm tới khoảng 90 độ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Gradient Transition` | `number` | Vị trí chuyển tiếp gradient theo slider; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Overtone Intensity` | `number` | Mức hiện màu overtone trên body pearl. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Luster` | `number` | Độ sáng/phản xạ sâu của pearl; tăng làm pearl reflective hơn, giảm làm dull. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Reflection Gloss` | `number` | Mức matte tới gloss của pearl; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pearl/Blemish Intensity` | `number` | Cường độ bump blemish predefined; không thay topology model. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Ground Plane/Color Mode` | `choice` | Single Color hoặc Gradient cho Colored Ground Plane. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Ground Plane/Color Map` | `reference` | Ảnh màu background; không tự tạo relief. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Ground Plane/Texture Map` | `reference` | Ảnh texture có hiệu ứng relief theo renderer adapter. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Mode` | `choice` | Image hoặc Color; scene editor hoạt động không cần object selection. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Image` | `reference` | Map environment của Style/master scene. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Color` | `text` | Màu environment khi dùng mode Color. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Mapping` | `choice` | Planar, Cubic, Spherical hoặc Mirrorball. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Crop` | `reference` | Miền crop của ảnh environment; giữ nguyên asset và lưu crop riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Hue` | `number` | Hiệu chỉnh hue map environment. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Brightness` | `number` | Độ sáng map environment. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Saturation` | `number` | Độ saturation map environment. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Map Rotation U` | `number` | Góc xoay map theo U, độ theo policy UI OM9. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Environment/Map Rotation V` | `number` | Góc xoay map theo V, độ theo policy UI OM9. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lighting/Mode` | `choice` | Image, Color hoặc Physical Lighting. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lighting/Image` | `reference` | Map chiếu sáng; không đồng nhất map environment chỉ phản xạ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lighting/Color` | `text` | Màu map chiếu sáng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lighting/Style Lights` | `reference` | Danh sách light objects trong scene Style; từng light phải resolve. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Caustics` | `boolean` | Bật caustics cho lights có renderer capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Max Photon` | `number` | Số photons của caustics; semantics/count policy cần renderer xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Multiplier` | `number` | Hệ số caustics; thang và normalization do renderer xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Search Distance` | `number` | Khoảng tìm caustics; units renderer chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Show Camera` | `boolean` | Hiện camera trong planar views; không tự đổi Perspective geometry. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Render Curves` | `boolean` | Renderer biểu diễn curves dạng pipes; radius chưa được mặc định hóa. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Vignetting` | `boolean` | Bật hiệu ứng tối cạnh camera. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Camera Target` | `reference` | Điểm/đối tượng focus camera; thay target để đặt vùng rõ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Depth of Field` | `boolean` | Bật độ sâu trường ảnh quanh camera target. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `F-Number` | `number` | F-number camera; thấp blur mạnh hơn quanh target, cao giữ vùng rõ lớn hơn. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bokeh Effect` | `boolean` | Bật bokeh để điều khiển polygon của điểm sáng ngoài focus. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Blade Count` | `number` | Số cạnh polygon bokeh; chỉ dùng khi Bokeh Effect bật. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Film Grain` | `boolean` | Bật hiệu ứng grain trong kết quả render; không mô phỏng kernel chưa có. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bloom Effect` | `boolean` | Bật bloom sau render. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bloom/Weight` | `number` | Mức trộn bloom map: 50 trộn 50% render và 50% bloom; chưa xác lập default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bloom/Size` | `number` | Kích thước blur/glow bloom map; đơn vị renderer cần xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Glare Effect` | `boolean` | Bật hiệu ứng glare sau render nếu adapter hỗ trợ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Glare/Weight` | `number` | Mức trộn glare map tương tự Bloom Weight; không lấy số minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Glare/Size` | `number` | Kích thước glare map tương tự Bloom Size; unit renderer cần xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Diffraction` | `boolean` | Bật hiệu ứng diffraction sau render; các fields chưa mô tả cần mở rộng sau xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-008` và các vai trò Objects, Document; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tách model material metadata/density khỏi shader renderer và scene per-object; typed parameter groups theo category/tab, clone User preset trước sửa, preserve SG conflict; render-preview asynchronous rồi explicit Apply.
2. C++/Qt: đăng ký và điều phối native command `OM9-RENDER-008`; chuyển kế hoạch Material Editor sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-008`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 0, max: None },
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Category", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Library", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Material", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Reflection Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Alpha", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Default Ground Plane", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Metal Quality", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Specific Gravity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color Grid", kind: ParameterKind::Text, required: false },
        Parameter { name: "Color Picker/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color Picker/RGB", kind: ParameterKind::Text, required: false },
        Parameter { name: "Color Picker/Tint Shade", kind: ParameterKind::Number, required: false },
        Parameter { name: "Reflection Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Polish Level", kind: ParameterKind::Number, required: false },
        Parameter { name: "Reflection Quality", kind: ParameterKind::Number, required: false },
        Parameter { name: "Anisotropy", kind: ParameterKind::Number, required: false },
        Parameter { name: "Texture", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bump Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Map Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Map Tiling U", kind: ParameterKind::Number, required: false },
        Parameter { name: "Map Tiling V", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scene/Override", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scene/Mapping", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scene/Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Scene/Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scene/Map Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scene/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Toon", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Toon/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Toon/Line Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Toon/Edge Detection", kind: ParameterKind::Number, required: false },
        Parameter { name: "Light/Emissive", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Light/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Light/Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem/Refraction Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem/Refraction Multiplier", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Color Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem/Color Multiplier", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Crop", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Saturation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/RGB", kind: ParameterKind::Text, required: false },
        Parameter { name: "Gem/Reflection Gloss", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Refraction Index", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Refraction Gloss", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Dispersion Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem/Dispersion", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem/Refraction Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Body Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Pearl/Overtone", kind: ParameterKind::Text, required: false },
        Parameter { name: "Pearl/Gradient Transition", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Overtone Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Luster", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Reflection Gloss", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pearl/Blemish Intensity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Ground Plane/Color Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Ground Plane/Color Map", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Ground Plane/Texture Map", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Environment/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Environment/Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Environment/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Environment/Mapping", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Environment/Crop", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Environment/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Saturation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Map Rotation U", kind: ParameterKind::Number, required: false },
        Parameter { name: "Environment/Map Rotation V", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lighting/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Lighting/Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Lighting/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Lighting/Style Lights", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Caustics", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Max Photon", kind: ParameterKind::Number, required: false },
        Parameter { name: "Multiplier", kind: ParameterKind::Number, required: false },
        Parameter { name: "Search Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Show Camera", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Render Curves", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vignetting", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Camera Target", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Depth of Field", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "F-Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bokeh Effect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Blade Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Film Grain", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bloom Effect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bloom/Weight", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bloom/Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Glare Effect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Glare/Weight", kind: ParameterKind::Number, required: false },
        Parameter { name: "Glare/Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diffraction", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- SG đổi report nhưng không geometry; material scene override master đúng object, gem image crop/alpha và pearl gr a di en ts đúng; Anisotropy unavailable khi glossy;22 KY chưa chọn hằng; DOF target/F-number và Bloom weight 0/100 có fixture.
- OM9-RENDER-008: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

> This section is intentionally separate theo hợp đồng feature-derived material. Fill it when implementing this feature. Các hành vi chưa xác định phải được ghi rõ.

### Inputs

- TODO: normalize supported object types and required selection state theo hợp đồng feature.

### Parameters and defaults

- TODO: verify exact semantics, units, ranges, and defaults theo hợp đồng feature.

### Output

- TODO: define resulting OpenMatrix9 object(s), geometry type, and ownership/history relationships.

### Preview / commit / cancel

- TODO: define interactive lifecycle only where supported by source.

### History / dependency model

- TODO: verify whether this feature records/uses History and define the OpenMatrix9 dependency graph.

### Error / invalid-input behavior

- TODO: define explicit errors and no-op/cancel cases.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-RENDER-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-008` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
