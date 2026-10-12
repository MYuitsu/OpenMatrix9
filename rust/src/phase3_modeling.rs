//! Portable modeling policy and immutable request ownership. No kernel or host pointers.
use std::collections::HashSet;

pub fn validate_result_count(count: usize) -> Result<(), Reason> {
    if count == 0 {
        Err(Reason::InvalidTopology)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    Unknown = 0,
    Point = 1,
    Curve = 2,
    Face = 3,
    Shell = 4,
    Solid = 5,
    Compound = 6,
    Mesh = 7,
    Cloud = 8,
    Retained = 9,
}
/// Curve Join retains the accepted Phase 2 controller; all other metadata
/// routes to Phase 3 capability validation, which rejects unsupported inputs.
pub fn join_uses_surface_adapter(kind: Kind) -> bool {
    kind != Kind::Curve
}
/// Native surface construction accuracy in millimeters, independently of the
/// user profile-refit tolerance. Imported analytic rails retain CAD precision.
pub const KERNEL_TOLERANCE: f64 = 1e-7;
impl Kind {
    pub fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Unknown,
            1 => Self::Point,
            2 => Self::Curve,
            3 => Self::Face,
            4 => Self::Shell,
            5 => Self::Solid,
            6 => Self::Compound,
            7 => Self::Mesh,
            8 => Self::Cloud,
            9 => Self::Retained,
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Operation {
    Join = 1,
    Explode = 2,
    Trim = 3,
    Difference = 4,
    Intersection = 5,
    Union = 6,
    TwoObjects = 7,
    Sweep1 = 8,
    Sweep2 = 9,
    Loft = 10,
}
impl Operation {
    pub fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            1 => Self::Join,
            2 => Self::Explode,
            3 => Self::Trim,
            4 => Self::Difference,
            5 => Self::Intersection,
            6 => Self::Union,
            7 => Self::TwoObjects,
            8 => Self::Sweep1,
            9 => Self::Sweep2,
            10 => Self::Loft,
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Reason {
    Unsupported = 1,
    InputCount = 2,
    InvalidTopology = 3,
    ClosedSolidRequired = 4,
    MixedInputs = 5,
    Protected = 6,
    Stale = 7,
    Unavailable = 8,
    Cancelled = 9,
    InvalidSnapshot = 10,
    InvalidOptions = 11,
}
impl Reason {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => "Unsupported native CAD input for this operation",
            Self::InputCount => "Select the required independent inputs/components",
            Self::InvalidTopology => "Invalid native topology; inputs are unchanged",
            Self::ClosedSolidRequired => {
                "Boolean requires valid closed solid inputs; open shells are unsupported"
            }
            Self::MixedInputs => "Select curves or surfaces separately",
            Self::Protected => "Preservation-protected input cannot be changed in modeling mode",
            Self::Stale => "A modeling input or its world placement changed; cancel and reselect",
            Self::Unavailable => "Document is unavailable or cannot be altered",
            Self::Cancelled => "Modeling request was cancelled; reselect inputs",
            Self::InvalidSnapshot => "Invalid or oversized modeling request snapshot",
            Self::InvalidOptions => {
                "Unsupported surface settings; use finite positive tolerance and valid point count/style"
            }
        }
    }
}
pub fn surface_options(
    kind: u32,
    style: u32,
    closed: bool,
    section_mode: u32,
    point_count: u32,
    tolerance: f64,
    input_count: usize,
) -> Result<(), Reason> {
    let operation = crate::surface::Kind::from_name(match kind {
        1 => "Sweep1",
        2 => "Sweep2",
        3 => "Loft",
        _ => return Err(Reason::InvalidOptions),
    })
    .ok_or(Reason::InvalidOptions)?;
    if !operation.options_valid(style, closed)
        || section_mode > 2
        || !(2..=256).contains(&point_count)
        || !tolerance.is_finite()
        || tolerance <= 0.
        || tolerance > 1e6
    {
        return Err(Reason::InvalidOptions);
    }
    let rails = operation.rails();
    let minimum = if closed {
        rails + 2 + usize::from(rails == 0)
    } else {
        rails + operation.minimum_sections()
    };
    if input_count < minimum || input_count > 256 {
        return Err(Reason::InputCount);
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputFacts {
    pub kind: Kind,
    pub valid: bool,
    pub closed: bool,
    pub protected: bool,
    pub components: u32,
}
pub fn capability(operation: Operation, inputs: &[InputFacts]) -> Result<(), Reason> {
    validate_inputs(operation, inputs, true)
}
pub fn validate_inputs(
    operation: Operation,
    inputs: &[InputFacts],
    complete: bool,
) -> Result<(), Reason> {
    if inputs.is_empty() || inputs.len() > 256 {
        return Err(Reason::InputCount);
    }
    if inputs.iter().any(|i| i.protected) {
        return Err(Reason::Protected);
    }
    if inputs.iter().any(|i| !i.valid) {
        return Err(Reason::InvalidTopology);
    }
    let min = match operation {
        Operation::Explode => 1,
        Operation::Sweep2 => 3,
        _ => 2,
    };
    if complete && (inputs.len() < min || (operation == Operation::TwoObjects && inputs.len() != 2))
    {
        return Err(Reason::InputCount);
    }
    match operation {
        Operation::Difference
        | Operation::Intersection
        | Operation::Union
        | Operation::TwoObjects => {
            if inputs
                .iter()
                .any(|i| !matches!(i.kind, Kind::Solid | Kind::Compound) || !i.closed)
            {
                return Err(Reason::ClosedSolidRequired);
            }
        }
        Operation::Loft | Operation::Sweep1 | Operation::Sweep2 => {
            if inputs.iter().any(|i| i.kind != Kind::Curve) {
                return Err(Reason::Unsupported);
            }
        }
        Operation::Join => {
            if inputs
                .iter()
                .any(|i| !matches!(i.kind, Kind::Curve | Kind::Face | Kind::Shell))
            {
                return Err(Reason::Unsupported);
            }
            if inputs
                .iter()
                .any(|i| (i.kind == Kind::Curve) != (inputs[0].kind == Kind::Curve))
            {
                return Err(Reason::MixedInputs);
            }
        }
        Operation::Trim => {
            if inputs
                .iter()
                .any(|i| !matches!(i.kind, Kind::Curve | Kind::Face | Kind::Shell))
            {
                return Err(Reason::Unsupported);
            }
        }
        Operation::Explode => {
            if inputs.iter().any(|i| {
                !matches!(
                    i.kind,
                    Kind::Curve | Kind::Shell | Kind::Solid | Kind::Compound
                )
            }) {
                return Err(Reason::Unsupported);
            }
            if inputs.iter().any(|i| i.components < 2) {
                return Err(Reason::InputCount);
            }
        }
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub identity: String,
    pub signature: String,
}
#[derive(Debug)]
pub struct Request {
    document: String,
    inputs: Vec<Snapshot>,
    cancelled: bool,
}
impl Request {
    pub fn new(document: &str, inputs: Vec<Snapshot>) -> Result<Self, Reason> {
        if document.is_empty() || document.len() > 4096 || inputs.is_empty() || inputs.len() > 256 {
            return Err(Reason::InvalidSnapshot);
        }
        let mut identities = HashSet::new();
        for input in &inputs {
            if input.identity.is_empty()
                || input.identity.len() > 4096
                || input.signature.is_empty()
                || input.signature.len() > 4096
                || !identities.insert(input.identity.as_str())
            {
                return Err(Reason::InvalidSnapshot);
            }
        }
        Ok(Self {
            document: document.into(),
            inputs,
            cancelled: false,
        })
    }
    pub fn validate_current(
        &self,
        document: &str,
        inputs: &[Snapshot],
        available: bool,
    ) -> Result<(), Reason> {
        if self.cancelled {
            return Err(Reason::Cancelled);
        }
        if !available {
            return Err(Reason::Unavailable);
        }
        if self.document != document || self.inputs != inputs {
            return Err(Reason::Stale);
        }
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
}
