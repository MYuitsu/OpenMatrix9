//! Working-geometry exchange policy. Display tessellation never changes kind.
pub fn worker_target(available: u32) -> u32 {
    // Integer arithmetic, round down, no overflow and no fixed four-thread cap.
    (available / 5 * 3 + available % 5 * 3 / 5).max(1)
}

pub fn bounded_workers(available: u32, tasks: u32, ram_slots: u32) -> u32 {
    worker_target(available).min(tasks).min(ram_slots)
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_modeling_worker_target(available: u32) -> u32 {
    worker_target(available)
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_modeling_workers(available: u32, tasks: u32, ram_slots: u32) -> u32 {
    bounded_workers(available, tasks, ram_slots)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GeometryKind {
    CadPoint = 1,
    CadCurve = 2,
    CadBrep = 3,
    Mesh = 4,
    PointCloud = 5,
    SubD = 6,
    Mixed = 7,
    Retained = 8,
    Unknown = 9,
}

impl From<u32> for GeometryKind {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::CadPoint,
            2 => Self::CadCurve,
            3 => Self::CadBrep,
            4 => Self::Mesh,
            5 => Self::PointCloud,
            6 => Self::SubD,
            7 => Self::Mixed,
            8 => Self::Retained,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Capabilities {
    pub select: bool,
    pub snap: bool,
    pub transform: bool,
    pub curve_edit: bool,
    pub surface_edit: bool,
    pub boolean: bool,
    pub export_v5: bool,
}

pub fn capabilities(kind: GeometryKind) -> Capabilities {
    use GeometryKind::*;
    let exchange = matches!(kind, CadPoint | CadCurve | CadBrep | Mesh | PointCloud);
    Capabilities {
        select: exchange,
        snap: matches!(kind, CadPoint | CadCurve | CadBrep),
        transform: exchange,
        // Phase2 owns native curve CV editing; surfaces remain a later phase.
        curve_edit: kind == CadCurve,
        surface_edit: false,
        boolean: kind == CadBrep,
        export_v5: exchange,
    }
}

pub fn preflight(kinds: &[GeometryKind]) -> Result<(), &'static str> {
    if kinds.is_empty() || kinds.iter().any(|kind| !capabilities(*kind).export_v5) {
        return Err("Working exchange requires independently supported geometry");
    }
    Ok(())
}

pub fn snap_allowed(kind: GeometryKind, native_cad: bool, preview: bool, mode: u32) -> bool {
    native_cad
        && !preview
        && match mode {
            2 | 4 => matches!(kind, GeometryKind::CadCurve | GeometryKind::CadBrep),
            8 => matches!(kind, GeometryKind::CadPoint | GeometryKind::CadBrep),
            _ => false,
        }
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_modeling_snap_allowed(
    kind: u32,
    native_cad: bool,
    preview: bool,
    mode: u32,
) -> bool {
    snap_allowed(kind.into(), native_cad, preview, mode)
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_modeling_capabilities(kind: u32) -> u32 {
    let c = capabilities(kind.into());
    u32::from(c.select)
        | (u32::from(c.snap) << 1)
        | (u32::from(c.transform) << 2)
        | (u32::from(c.curve_edit) << 3)
        | (u32::from(c.surface_edit) << 4)
        | (u32::from(c.boolean) << 5)
        | (u32::from(c.export_v5) << 6)
}
