// SPDX-License-Identifier: LGPL-2.1-or-later
//! Modelling capability policy (M03 surface/solid operations, wave W8).
//!
//! Ported from the legacy bridge (`rust/src/phase3_modeling.rs`, itself portable: "no kernel
//! or host pointers"). It decides whether a set of inputs may run an operation, and why not,
//! before any kernel is asked. The typed `Reason` values and their user-facing messages are
//! kept verbatim, because they are the wording the command line shows.
use crate::Result;

/// Native surface construction accuracy in millimetres, independent of the user's profile
/// refit tolerance.
pub const KERNEL_TOLERANCE: f64 = 1e-7;

/// The largest input or option count any modelling operation accepts.
pub const MAX_MODELING_INPUTS: usize = 256;

/// What an input object is, at the level of modelling capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Unknown,
    Point,
    Curve,
    Face,
    Shell,
    Solid,
    Compound,
    Mesh,
    Cloud,
    Retained,
}

impl Kind {
    pub fn from_wire(value: u32) -> Option<Self> {
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
    pub fn wire(self) -> u32 {
        match self {
            Self::Unknown => 0,
            Self::Point => 1,
            Self::Curve => 2,
            Self::Face => 3,
            Self::Shell => 4,
            Self::Solid => 5,
            Self::Compound => 6,
            Self::Mesh => 7,
            Self::Cloud => 8,
            Self::Retained => 9,
        }
    }
}

/// The modelling operations the capability policy covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Join,
    Explode,
    Trim,
    Difference,
    Intersection,
    Union,
    TwoObjects,
    Sweep1,
    Sweep2,
    Loft,
}

impl Operation {
    pub fn wire(self) -> u32 {
        match self {
            Self::Join => 1,
            Self::Explode => 2,
            Self::Trim => 3,
            Self::Difference => 4,
            Self::Intersection => 5,
            Self::Union => 6,
            Self::TwoObjects => 7,
            Self::Sweep1 => 8,
            Self::Sweep2 => 9,
            Self::Loft => 10,
        }
    }
    pub fn from_wire(value: u32) -> Option<Self> {
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

/// Why an operation cannot run. The messages are the command-line wording, kept verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Unsupported,
    InputCount,
    InvalidTopology,
    ClosedSolidRequired,
    MixedInputs,
    Protected,
    Stale,
    Unavailable,
    Cancelled,
    InvalidSnapshot,
    InvalidOptions,
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

/// What the caller knows about one selected input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputFacts {
    pub kind: Kind,
    pub valid: bool,
    pub closed: bool,
    pub protected: bool,
    pub components: u32,
}

impl InputFacts {
    pub fn new(kind: Kind, valid: bool, closed: bool) -> Self {
        Self {
            kind,
            valid,
            closed,
            protected: false,
            components: 1,
        }
    }
}

/// A modelling result must contain at least one object.
pub fn validate_result_count(count: usize) -> std::result::Result<(), Reason> {
    if count == 0 {
        Err(Reason::InvalidTopology)
    } else {
        Ok(())
    }
}

/// Curve Join keeps the curve controller; every other kind routes to the surface/solid
/// capability path.
pub fn join_uses_surface_adapter(kind: Kind) -> bool {
    kind != Kind::Curve
}

/// The surface operation a style/tolerance request belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Sweep1,
    Sweep2,
    Loft,
}

impl SurfaceKind {
    /// How many rails the operation consumes.
    pub fn rails(self) -> usize {
        match self {
            Self::Sweep1 => 1,
            Self::Sweep2 => 2,
            Self::Loft => 0,
        }
    }
    /// How many sections the operation needs on top of its rails.
    pub fn minimum_sections(self) -> usize {
        if self == Self::Loft { 2 } else { 1 }
    }
    /// Styles: Normal, Straight Sections, Loose, Tight, Uniform, Developable. Only Loft
    /// offers the styles, and Developable cannot close a loft.
    pub fn options_valid(self, style: u32, closed_loft: bool) -> bool {
        match self {
            Self::Loft => style <= 5 && !(style == 5 && closed_loft),
            _ => style == 0,
        }
    }
}

/// Validate a surface operation request. `kind`/`style`/`section_mode` are the wire values the
/// command line sends; the tolerance is the profile refit tolerance, not the kernel accuracy.
#[allow(clippy::too_many_arguments)]
pub fn surface_options(
    kind: u32,
    style: u32,
    closed: bool,
    section_mode: u32,
    point_count: u32,
    tolerance: f64,
    input_count: usize,
) -> std::result::Result<(), Reason> {
    let operation = match kind {
        1 => SurfaceKind::Sweep1,
        2 => SurfaceKind::Sweep2,
        3 => SurfaceKind::Loft,
        _ => return Err(Reason::InvalidOptions),
    };
    if !operation.options_valid(style, closed)
        || section_mode > 2
        || !(2..=MAX_MODELING_INPUTS as u32).contains(&point_count)
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
    if input_count < minimum || input_count > MAX_MODELING_INPUTS {
        return Err(Reason::InputCount);
    }
    Ok(())
}

/// Admit an operation on the selected inputs (`capability` in the legacy module).
pub fn capability(operation: Operation, inputs: &[InputFacts]) -> std::result::Result<(), Reason> {
    validate_inputs(operation, inputs, true)
}

/// The full admission rule. `complete` is false while the user is still selecting: count
/// minimums are not enforced yet, but a bad or protected input is already refused.
pub fn validate_inputs(
    operation: Operation,
    inputs: &[InputFacts],
    complete: bool,
) -> std::result::Result<(), Reason> {
    if inputs.is_empty() || inputs.len() > MAX_MODELING_INPUTS {
        return Err(Reason::InputCount);
    }
    if inputs.iter().any(|input| input.protected) {
        return Err(Reason::Protected);
    }
    if inputs.iter().any(|input| !input.valid) {
        return Err(Reason::InvalidTopology);
    }
    let minimum = match operation {
        Operation::Explode => 1,
        Operation::Sweep2 => 3,
        _ => 2,
    };
    if complete
        && (inputs.len() < minimum || (operation == Operation::TwoObjects && inputs.len() != 2))
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
                .any(|input| !matches!(input.kind, Kind::Solid | Kind::Compound) || !input.closed)
            {
                return Err(Reason::ClosedSolidRequired);
            }
        }
        Operation::Loft | Operation::Sweep1 | Operation::Sweep2 => {
            if inputs.iter().any(|input| input.kind != Kind::Curve) {
                return Err(Reason::Unsupported);
            }
        }
        Operation::Join => {
            if inputs
                .iter()
                .any(|input| !matches!(input.kind, Kind::Curve | Kind::Face | Kind::Shell))
            {
                return Err(Reason::Unsupported);
            }
            // Joining curves and surfaces at once is ambiguous: the user must pick one class.
            if inputs
                .iter()
                .any(|input| (input.kind == Kind::Curve) != (inputs[0].kind == Kind::Curve))
            {
                return Err(Reason::MixedInputs);
            }
        }
        Operation::Trim => {
            if inputs
                .iter()
                .any(|input| !matches!(input.kind, Kind::Curve | Kind::Face | Kind::Shell))
            {
                return Err(Reason::Unsupported);
            }
        }
        Operation::Explode => {
            if inputs.iter().any(|input| {
                !matches!(
                    input.kind,
                    Kind::Curve | Kind::Shell | Kind::Solid | Kind::Compound
                )
            }) {
                return Err(Reason::Unsupported);
            }
            if inputs.iter().any(|input| input.components < 2) {
                return Err(Reason::InputCount);
            }
        }
    }
    Ok(())
}

/// A command-facing error for a refusal: the label keeps the reason code so the command line
/// can show `Reason::message` and the log keeps the code.
pub fn refusal(reason: Reason) -> crate::Error {
    crate::Error::Invalid(reason.message())
}

/// Convenience for callers that want `crate::Result`.
pub fn require(operation: Operation, inputs: &[InputFacts]) -> Result<()> {
    capability(operation, inputs).map_err(refusal)
}
