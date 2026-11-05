//! Điểm cắm runtime. Lõi host không biết Python/WASM/native chạy thế nào; nó chỉ cần
//! nạp add-on, nhận lại danh sách mã lệnh add-on thực sự đăng ký, chạy lệnh và gỡ khi tắt.

use crate::manifest::Manifest;
use crate::settings::SettingValue;
use std::collections::BTreeMap;
use std::path::Path;

pub struct LoadRequest<'a> {
    pub id: &'a str,
    pub dir: &'a Path,
    pub manifest: &'a Manifest,
    pub settings: &'a BTreeMap<String, SettingValue>,
}

/// Đường cong đã lấy mẫu (toạ độ thế giới, mm).
#[derive(Debug, Clone, PartialEq)]
pub struct CurveInput {
    pub points: Vec<[f64; 3]>,
    pub closed: bool,
}

/// Dữ liệu một lần chạy lệnh. Settings do host điền từ giá trị đã kiểm tra.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandInput {
    pub command: String,
    pub curves: Vec<CurveInput>,
    pub plane_normal: [f64; 3],
    pub settings: BTreeMap<String, SettingValue>,
}

impl Default for CommandInput {
    /// No curves, world-Z plane normal.
    fn default() -> Self {
        CommandInput {
            command: String::new(),
            curves: Vec::new(),
            plane_normal: [0.0, 0.0, 1.0],
            settings: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshOutput {
    pub name: String,
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
    /// JSON object (vd. `{"cut":"round","diameter_mm":1.5}`) – host ghi vào extension `om9.gem`.
    pub meta_json: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandOutput {
    pub meshes: Vec<MeshOutput>,
    pub messages: Vec<String>,
}

/// Giới hạn kết quả một lệnh, để add-on lỗi không làm treo tài liệu.
pub const MAX_OUTPUT_MESHES: usize = 20_000;
pub const MAX_OUTPUT_VERTICES: usize = 5_000_000;
pub const MAX_MESSAGES: usize = 200;

impl CommandOutput {
    /// Kiểm tra kết quả trước khi host ghi vào tài liệu.
    pub fn validate(&self) -> Result<(), String> {
        if self.meshes.len() > MAX_OUTPUT_MESHES {
            return Err(format!(
                "add-on trả về quá nhiều mesh ({} > {MAX_OUTPUT_MESHES})",
                self.meshes.len()
            ));
        }
        let mut total = 0usize;
        for m in &self.meshes {
            total += m.vertices.len();
            if total > MAX_OUTPUT_VERTICES {
                return Err(format!(
                    "add-on trả về quá nhiều đỉnh (> {MAX_OUTPUT_VERTICES})"
                ));
            }
            if m.vertices.len() < 3 || m.triangles.is_empty() {
                return Err(format!("mesh {:?} rỗng", m.name));
            }
            if m.vertices.iter().flatten().any(|v| !v.is_finite()) {
                return Err(format!("mesh {:?} có toạ độ không hữu hạn", m.name));
            }
            let n = m.vertices.len() as u32;
            if m.triangles.iter().flatten().any(|&i| i >= n) {
                return Err(format!("mesh {:?} có chỉ số đỉnh vượt phạm vi", m.name));
            }
        }
        Ok(())
    }
}

pub trait Runtime: Send {
    /// Nạp và gọi `register`. Trả về các mã lệnh đã đăng ký.
    /// Lỗi → host tự gọi `unload` để dọn phần đã đăng ký dở.
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String>;
    /// Gọi `unregister` và giải phóng. Không được panic.
    fn unload(&mut self, id: &str);
    fn settings_changed(&mut self, _id: &str, _settings: &BTreeMap<String, SettingValue>) {}
    /// Chạy một lệnh đã đăng ký.
    fn invoke(&mut self, _id: &str, _input: &CommandInput) -> Result<CommandOutput, String> {
        Err("runtime này không chạy lệnh".into())
    }
    /// One step of an interactive command: a JSON event in, a JSON reply out
    /// (see `om9_addon_abi::session`). The host never interprets the bytes here.
    fn session(&mut self, _id: &str, _event: &[u8]) -> Result<Vec<u8>, String> {
        Err("add-on này không có lệnh tương tác".into())
    }
}

/// Largest JSON step accepted in either direction.
pub const MAX_SESSION_JSON: usize = 64 << 20;

/// Runtime cho add-on `library`: không chạy mã, host tự đăng ký assets.
pub struct DataRuntime;

impl Runtime for DataRuntime {
    fn load(&mut self, _req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn unload(&mut self, _id: &str) {}
}

/// Runtime giả dùng cho CLI quản lý/kiểm thử đóng gói: kiểm tra entry tồn tại rồi
/// "đăng ký" đúng các lệnh đã khai báo. Không thực thi mã add-on.
pub struct DeclareOnlyRuntime;

impl Runtime for DeclareOnlyRuntime {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        if let Some(e) = &req.manifest.entry
            && !req.dir.join(e).is_file()
        {
            return Err(format!("không tìm thấy entry {e}"));
        }
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mesh(v: usize, t: [u32; 3]) -> MeshOutput {
        MeshOutput {
            name: "g".into(),
            vertices: vec![[0.0; 3]; v],
            triangles: vec![t],
            meta_json: "{}".into(),
        }
    }
    #[test]
    fn output_validation() {
        assert!(
            CommandOutput {
                meshes: vec![mesh(3, [0, 1, 2])],
                messages: vec![]
            }
            .validate()
            .is_ok()
        );
        assert!(
            CommandOutput {
                meshes: vec![mesh(3, [0, 1, 3])],
                messages: vec![]
            }
            .validate()
            .unwrap_err()
            .contains("vượt")
        );
        assert!(
            CommandOutput {
                meshes: vec![mesh(2, [0, 1, 1])],
                messages: vec![]
            }
            .validate()
            .is_err()
        );
        let mut m = mesh(3, [0, 1, 2]);
        m.vertices[1][2] = f64::NAN;
        assert!(
            CommandOutput {
                meshes: vec![m],
                messages: vec![]
            }
            .validate()
            .unwrap_err()
            .contains("hữu hạn")
        );
    }
}
