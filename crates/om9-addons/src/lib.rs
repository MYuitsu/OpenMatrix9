//! OM9 add-on host.
//!
//! Mô hình theo Blender Extensions (manifest TOML + gói zip + repository +
//! bật/tắt + preferences), cải tiến cho OM9:
//! - Settings khai báo trong manifest: host tự vẽ UI, kiểm tra giá trị và di trú khi nâng cấp
//!   mà không cần chạy mã add-on.
//! - Mỗi add-on chỉ được đăng ký đúng các mã lệnh catalog đã khai báo; một mã lệnh chỉ có một nhà cung cấp.
//! - Cài đặt nguyên tử với rollback; safe mode không làm mất danh sách add-on đã bật.
//! - Quyền (files/network/clipboard) phải được người dùng chấp nhận; nâng cấp thêm quyền thì hỏi lại.
//!
//! Lõi không phụ thuộc runtime: Python/WASM/native cắm vào qua trait [`runtime::Runtime`].

pub mod catalog;
pub mod crc32;
pub mod glob;
pub mod host;
pub mod i18n;
pub mod inflate;
pub mod keymap;
pub mod manifest;
#[cfg(feature = "native")]
pub mod native;
pub mod prefs;
pub mod runtime;
pub mod semver;
pub mod settings;
pub mod sha256;
pub mod toml;
pub mod zip;

pub use host::{
    AddonRow, AddonStatus, CommandState, Host, HostConfig, InstallOptions, RepoKind, Startup,
};
pub use manifest::Manifest;
