//! Working-geometry exchange policy. Display tessellation never changes kind.
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
        // Direct curve/surface editing belongs to subsequent phases.
        curve_edit: false,
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
