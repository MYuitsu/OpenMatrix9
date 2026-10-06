//! Source archive identity and preservation policy. No host or parser dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveMode {
    GeometryOnly = 0,
    Preserve = 1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveCapability {
    Editable = 1,
    DisplayRetained = 2,
    Retained = 3,
    Incompatible = 4,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchiveIdentity {
    pub import_namespace: String,
    pub source_uuid: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityError;
fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(b)
            }
        })
        && bytes.iter().any(|b| *b != b'0' && *b != b'-')
}
impl ArchiveIdentity {
    pub fn new(import_namespace: &str, source_uuid: &str) -> Result<Self, IdentityError> {
        if !canonical_uuid(import_namespace) || !canonical_uuid(source_uuid) {
            return Err(IdentityError);
        }
        Ok(Self {
            import_namespace: import_namespace.into(),
            source_uuid: source_uuid.into(),
        })
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_mode_valid(mode: u32) -> bool {
    mode <= 1
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_capability_valid(capability: u32) -> bool {
    (1..=4).contains(&capability)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_legacy_export_allowed(mode: u32) -> bool {
    mode == ArchiveMode::GeometryOnly as u32
}
