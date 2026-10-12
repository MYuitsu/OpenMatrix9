//! OM9-SOLID-012 / OM9-SOLID-014: native solid creation sessions.
//! Host decisions: mm, right-handed CPlane, standalone BRep, 1e-7 mm minimum.
use crate::coordinates::{Frame, NumericSyntax, PointInput, classify_numeric};
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};
type Point = [f64; 3];
pub struct WirePreview {
    pub segments: Vec<[Point;2]>,
    pub markers: [Point;3],
}
/// Value-only publication observation. The GUI owns all native objects and
/// supplies both object-local and world matrices as separate calls.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolidPublicationFrame {
    pub expected: [f64; 16],
    pub actual: [f64; 16],
}

/// 0: same frame, 1: non-finite, 2: unsupported frame, 3: stale frame.
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_publication_frame(input: SolidPublicationFrame) -> u32 {
    use crate::geometry_frame::{Frame as GeometryFrame, FrameError, IDENTITY};
    for matrix in [input.expected, input.actual] {
        if let Err(error) = GeometryFrame::compose(IDENTITY, IDENTITY, IDENTITY, matrix) {
            return if error == FrameError::NonFinite { 1 } else { 2 };
        }
    }
    let epsilon = 64. * f64::EPSILON;
    if input.expected.iter().zip(input.actual).all(|(expected, actual)| {
        (*expected - actual).abs() <= epsilon * expected.abs().max(actual.abs()).max(1.)
    }) { 0 } else { 3 }
}
mod geometry;
const MIN: f64 = 1e-7;
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Point, s: f64) -> Point {
    a.map(|x| x * s)
}
fn dot(a: Point, b: Point) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn finite(p: Point) -> bool {
    p.iter().all(|x| x.is_finite() && x.abs() <= 1e9)
}
fn number(text: &str) -> Result<f64, String> {
    let text = text.trim().to_ascii_lowercase();
    let (text, k) = if let Some(s) = text.strip_suffix("mm") {
        (s, 1.)
    } else if let Some(s) = text.strip_suffix("cm") {
        (s, 10.)
    } else if let Some(s) = text.strip_suffix("in") {
        (s, 25.4)
    } else {
        (text.as_str(), 1.)
    };
    let n = text
        .trim()
        .parse::<f64>()
        .map_err(|_| "Enter a number, for example 10 or 10mm".to_owned())?
        * k;
    if !n.is_finite() { return Err("Enter a valid number".into()); }
    if n.abs() > 1e9 { return Err("This value is too large. Enter a smaller value".into()); }
    Ok(n)
}
fn dimension(n: f64) -> Result<f64, String> {
    if !n.is_finite() { return Err("Enter a valid number".into()); }
    if n == 0. { return Err("Enter a value other than zero".into()); }
    if n.abs() < MIN { return Err("This value is too small. Enter a larger value".into()); }
    if n.abs() > 1e9 { return Err("This value is too large. Enter a smaller value".into()); }
    Ok(n)
}
pub(crate) fn dimension_valid(n:f64)->bool{dimension(n).is_ok()}
pub(crate) fn validate_frame(origin:Point,axes:[Point;3])->Result<(),String>{
    if !finite(origin)||axes.iter().flatten().any(|x|!x.is_finite()){return Err("Invalid CPlane".into());}
    for i in 0..3{for j in 0..3{if (dot(axes[i],axes[j])-if i==j{1.}else{0.}).abs()>1e-6{return Err("CPlane must be orthonormal".into());}}}
    if dot(cross(axes[0],axes[1]),axes[2])<0.999999{return Err("CPlane must be right handed".into());}
    Ok(())
}
pub(crate) fn native_output_valid(kind:Kind,b:&[f64;15])->bool{
    if b.iter().any(|x|!x.is_finite()){return false;}
    let origin=[b[0],b[1],b[2]];let axes=[[b[3],b[4],b[5]],[b[6],b[7],b[8]],[b[9],b[10],b[11]]];
    if validate_frame(origin,axes).is_err(){return false;}
    if kind==Kind::Sphere{return dimension_valid(b[12])&&b[12]>0.&&b[13]==0.&&b[14]==0.&&origin.iter().all(|x|x.abs()+b[12]<=1e9);}
    if b[12..].iter().any(|&n|!dimension_valid(n)||n<=0.){return false;}
    for bits in 0..8{let mut p=origin;for i in 0..3{if bits&(1<<i)!=0{p=add(p,scale(axes[i],b[12+i]));}}if !finite(p){return false;}}
    true
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    Box = 1,
    Sphere = 2,
}
impl Kind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name
            .trim()
            .trim_start_matches('_')
            .to_ascii_lowercase()
            .as_str()
        {
            "box" | "solidboxcornertocornerheight" | "om9_solidboxcornertocornerheight" => {
                Some(Self::Box)
            }
            "sphere" | "solidspherecenterradius" | "om9_solidspherecenterradius" => {
                Some(Self::Sphere)
            }
            _ => None,
        }
    }
    pub fn caption(self) -> &'static str {
        if self == Self::Box { "Box" } else { "Sphere" }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Waiting,
    Commit,
    Cancelled,
}
#[derive(Clone)]
struct Data {
    phase: u32,
    first: Point,
    last: Point,
    previous_accepted: Option<Point>,
    axes: [Point; 3],
    length: f64,
    width: f64,
    mode: Mode,
    mode_selected: bool,
    points: Vec<Point>,
    accepted_points: Vec<Point>,
    circle: Option<(Point, f64, Point)>,
    radius: f64,
    point_constraint: bool,
    constraint_types: Vec<bool>,
    output: Option<[f64; 15]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Corners,
    Diagonal,
    ThreePoint,
    Vertical,
    Center,
    Cube,
    TwoPoint,
    FourPoint,
    FitPoints,
    AroundCurve,
    Tangent,
}
#[derive(Clone)]
pub struct Session {
    kind: Kind,
    data: Data,
    frame: (Point, [Point; 3]),
    history: Vec<Data>,
}
impl Session {
    pub fn capability_request(&self,stage:crate::native_solid_capability::Stage)
        ->Result<crate::native_solid_capability::Request,crate::geometry_contract::CapabilityError>{
        use crate::geometry_contract::{CapabilityError,OperationId};
        if self.phase()==0{return Err(CapabilityError::UnsupportedOperation(OperationId::ConstructSolid));}
        if self.kind==Kind::Box && !self.data.mode_selected && stage!=crate::native_solid_capability::Stage::Declare {
            return Err(CapabilityError::UnsupportedOptions);
        }
        if self.data.constraint_types.len()>3{return Err(CapabilityError::UnsupportedOptions);}
        Ok(crate::native_solid_capability::Request{kind:self.kind,mode:self.data.mode,stage,phase:self.phase(),
            point_count:self.data.points.len(),length:self.data.length,width:self.data.width,radius:self.data.radius,
            point_constraint:self.data.point_constraint,constraint_types:self.data.constraint_types.clone(),
            first:self.data.first,axes:self.data.axes,frame_origin:self.frame.0,frame_axes:self.frame.1,
            circle:self.data.circle,output:self.data.output})
    }
    pub fn construction_capability(&self,input:crate::geometry_contract::RepresentationKind,stage:crate::native_solid_capability::Stage)
        ->Result<crate::geometry_contract::OperationCapability,crate::geometry_contract::CapabilityError>{
        use crate::geometry_contract::*;
        operation_capability(OperationId::ConstructSolid,input,OperationOptions::NativeSolid(self.capability_request(stage)?))
    }
    pub fn new(kind: Kind) -> Self {
        let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        Self {
            kind,
            data: Data {
                phase: 1,
                first: [0.; 3],
                last: [0.; 3],
                previous_accepted: None,
                axes,
                length: 0.,
                width: 0.,
                mode: if kind == Kind::Box {
                    Mode::Corners
                } else {
                    Mode::Center
                },
                mode_selected: true,
                points: Vec::new(),
                accepted_points: Vec::new(),
                circle: None,
                radius: 0.,
                point_constraint: false,
                constraint_types: Vec::new(),
                output: None,
            },
            frame: ([0.; 3], axes),
            history: Vec::new(),
        }
    }
    pub fn phase(&self) -> u32 {
        self.data.phase
    }
    /// Declarative phase presentation for the common option reducer. Geometry
    /// input/defaults remain owned by this native construction session.
    pub fn command_options(&self, generation:u64, scale:f64) -> Result<crate::command_options::OptionSession,String> {
        use crate::command_options::{OptionContext,OptionSession,OptionSpec};
        if self.kind==Kind::Sphere { return self.sphere_command_options(generation,scale); }
        if self.kind != Kind::Box || self.phase()==0 || self.phase()==6 {
            return Err("No editable Box options".into());
        }
        let first=self.phase()==1;
        let diagonal=self.phase()==2 && self.data.mode==Mode::Diagonal;
        let mut schema=Vec::new();
        for (name,alias) in [("Diagonal","D"),("3Point","3P"),("Vertical","V"),("Center","C")] {
            let mut spec=OptionSpec::action(name).alias(alias);
            if first&&name=="3Point" {spec=spec.alias("3");}
            spec.shown=first;
            if !first { spec=spec.disabled("Select the construction mode before the first point"); }
            schema.push(spec);
        }
        let mut cube=OptionSpec::action("Cube");cube.shown=diagonal;
        if !first && !diagonal { cube=cube.disabled("Cube is available in the Diagonal branch"); }
        if diagonal { cube=cube.alias("C"); }
        schema.push(cube);
        let mut undo=OptionSpec::action("Undo").alias("U").hidden();
        if self.history.is_empty(){undo=undo.disabled("No input to undo");}
        schema.push(undo);
        schema.push(OptionSpec::action("Cancel").alias("Esc").hidden());
        for key in ["Mode","Length","Width","Height"] { schema.push(OptionSpec::native_input(key).hidden()); }
        let question=match self.phase(){
            1 if !self.data.mode_selected=>"Box. Choose a method".into(),
            1 if self.data.mode==Mode::Center=>"Center of base".into(),
            1 if matches!(self.data.mode,Mode::ThreePoint|Mode::Vertical)=>"Start of edge".into(),
            1=>"First corner of base".into(),
            2 if matches!(self.data.mode,Mode::Diagonal|Mode::Cube)=>"Other corner of box".into(),
            2 if matches!(self.data.mode,Mode::ThreePoint|Mode::Vertical)=>"End of edge".into(),
            2 if self.data.mode==Mode::Center=>"Other corner of base or length".into(),
            2=>"Other corner of base".into(),
            3=>format!("Width. Press Enter to use length <{} mm>",self.data.length),
            4=>format!("Height. Press Enter to use width <{} mm>",self.data.width.abs()),
            _=>return Err("Unsupported Box input phase".into()),
        };
        let mut model=OptionSession::new(schema,std::collections::BTreeMap::new(),OptionContext::new(generation,scale).feature("OM9-SOLID-012"))?;
        model.set_question(&question)?;
        Ok(model)
    }
    fn sphere_command_options(&self,generation:u64,scale:f64)->Result<crate::command_options::OptionSession,String>{
        use crate::command_options::{OptionContext,OptionSession,OptionSpec};
        if matches!(self.phase(),0|6){return Err("No editable Sphere options".into());}
        let first=self.phase()==1;
        let mut schema=Vec::new();
        for (name,alias) in [("Center","C"),("2Point","2"),("3Point","3"),("4Point","4"),("Vertical","V"),("AroundCurve","A"),("FitPoints","F"),("Tangent","T")]{
            let mut spec=OptionSpec::native_input(name);spec.shown=first;
            if first{spec=spec.alias(alias);}
            schema.push(spec);
        }
        for (name,alias,shown) in [("Radius","R",self.phase()==9||(self.phase()==20&&self.data.points.len()==2)),("Diameter","D",false),("Point","P",self.phase()==20),("Undo","U",!self.history.is_empty())]{
            let mut spec=OptionSpec::native_input(name);if shown{spec=spec.alias(alias);}spec.shown=shown;schema.push(spec);
        }
        for name in ["Mode","Path","OnCurve","Curve","Cancel"]{schema.push(OptionSpec::native_input(name).hidden());}
        let question=match self.phase(){
            1=>"Center of sphere",
            5=>"Radius. Pick a point or enter a positive value",
            7=>"Other diameter endpoint or diameter",
            8=>"Second point",
            9=>"Third circle point or radius",
            10=>"Fourth point outside circle plane",
            11=>"Direction off chord to orient radius circle",
            12=>"Radius for the picked chord",
            15=>"Add points. Press Enter to fit sphere",
            16=>"Select a curve. Use Path=Object.EdgeN",
            17=>"Pick center on curve. Use OnCurve=0..1",
            20 if self.data.point_constraint=>"Pick a non-tangent point",
            20=>"Select a tangent curve. Use Curve=Object.EdgeN",
            21=>"Resolve tangent sphere",
            22=>"Radius for two tangent or point constraints",
            _=>return Err("Unsupported Sphere input phase".into()),
        };
        let mut model=OptionSession::new(schema,std::collections::BTreeMap::new(),OptionContext::new(generation,scale).feature("OM9-SOLID-014"))?;
        model.set_question(question)?;Ok(model)
    }
    pub fn output(&self) -> Option<[f64; 15]> {
        self.data.output
    }
    /// Independent accepted numerical input evidence. Native reference owners,
    /// units and conversion facts are bound separately by the host adapter.
    /// This is not a GeometryResult or a certificate of live source validity.
    pub fn construction_evidence(&self,generation:u64)->Result<String,u32>{
        use crate::{builder_history::Json,prepared_result::{map,string,encode_owned},
            geometry_contract::RepresentationKind,native_solid_capability::Stage};
        if generation==0{return Err(5);}
        self.construction_capability(RepresentationKind::Point,Stage::Construct).map_err(|_|2u32)?;
        let output=self.output().ok_or(2u32)?;
        if !finite(self.data.last)||self.data.previous_accepted.is_some_and(|p|!finite(p))
            ||self.data.points.iter().any(|p|!finite(*p)){return Err(4);}
        let vector=|values:&[f64]|Json::Array(values.iter().map(|&n|Json::Number(n)).collect());
        let points=|values:&[Point]|Json::Array(values.iter().map(|p|vector(p)).collect());
        let mode=match self.data.mode{
            Mode::Corners=>"Corners",Mode::Diagonal=>"Diagonal",Mode::ThreePoint=>"3Point",
            Mode::Vertical=>"Vertical",Mode::Center=>"Center",Mode::Cube=>"Cube",Mode::TwoPoint=>"2Point",
            Mode::FourPoint=>"4Point",Mode::FitPoints=>"FitPoints",Mode::AroundCurve=>"AroundCurve",Mode::Tangent=>"Tangent",
        };
        let circle=self.data.circle.map_or(Json::Other,|(center,radius,normal)|map([
            ("center",vector(&center)),("radius",Json::Number(radius)),("normal",vector(&normal))]));
        let evidence=map([
            ("schema_version",Json::Number(1.)),("operation",string("ConstructSolid")),
            ("feature_id",string(if self.kind==Kind::Box{"OM9-SOLID-012"}else{"OM9-SOLID-014"})),
            ("kind",string(self.kind.caption())),("mode",string(mode)),("generation",string(&generation.to_string())),
            ("coordinates",string("document_world_mm")),("phase",Json::Number(self.phase() as f64)),
            ("frame_origin",vector(&self.frame.0)),("frame_axes",points(&self.frame.1)),
            ("first",vector(&self.data.first)),("last",vector(&self.data.last)),
            ("previous_accepted",self.data.previous_accepted.map_or(Json::Other,|p|vector(&p))),
            ("axes",points(&self.data.axes)),("points",points(&self.data.points)),
            ("accepted_points",points(&self.data.accepted_points)),
            ("frame_role",string("session_input_frame_at_capture")),
            ("length",Json::Number(self.data.length)),("width",Json::Number(self.data.width)),
            ("radius",Json::Number(self.data.radius)),("circle",circle),
            ("point_constraint",Json::Boolean(self.data.point_constraint)),
            ("constraint_types",Json::Array(self.data.constraint_types.iter().map(|&v|Json::Boolean(v)).collect())),
            ("output",vector(&output)),
            ("lifetime",string("immutable_construction_input_evidence_not_current_validity")),
            ("source_provenance",string("numeric_inputs_only_native_owners_bound_separately")),
        ]);
        encode_owned(&evidence)
    }
    pub fn set_frame(&mut self, origin: Point, axes: [Point; 3]) -> Result<(), String> {
        validate_frame(origin,axes)?;
        self.frame = (origin, axes);
        Ok(())
    }
    pub fn prompt(&self) -> String {
        if self.kind == Kind::Box && let Ok(model) = self.command_options(1,1.) { return model.snapshot().text; }
        let action = match self.phase() {
            1 => {
                if self.kind == Kind::Box {
                    if self.data.mode == Mode::Center {
                        "Base center"
                    } else {
                        "First corner (Diagonal / 3Point / Vertical / Center / Cube / Esc)"
                    }
                } else {
                    "Center (2Point / 3Point / 4Point / FitPoints / Vertical / AroundCurve / Tangent / Esc)"
                }
            }
            2 => {
                if matches!(self.data.mode, Mode::Diagonal | Mode::Cube) {
                    "Opposite diagonal corner"
                } else if matches!(self.data.mode, Mode::ThreePoint | Mode::Vertical) {
                    "End of first edge or Length"
                } else {
                    "Other base corner or Length"
                }
            }
            3 => "Width (Enter = Length)",
            4 => "Height (Enter = Width)",
            5 => "Radius (pick a point or enter a positive value)",
            6 => "Ready to create solid",
            7 => "Other diameter endpoint or Diameter",
            8 => "Second point",
            9 => "Third circle point or Radius",
            10 => "Fourth point outside circle plane",
            11 => "Direction off chord to orient radius circle",
            15 => "FitPoints: add points, Enter to fit, Undo / Esc",
            12 => "Radius for the picked chord",
            16 => "Select a native curve (Path=Object.EdgeN)",
            17 => "Pick center on curve (OnCurve=0..1)",
            20 => {
                if self.data.point_constraint {
                    "Pick a non-tangent Point"
                } else {
                    "Select tangent curve (Curve=Object.EdgeN; Point / Radius after two constraints)"
                }
            }
            21 => "Resolve tangent construction or Undo / Esc",
            22 => "Radius for two tangent/Point constraints",
            _ => "Command:",
        };
        format!("{}: {}", self.kind.caption(), action)
    }
    pub fn input(&mut self, text: &str) -> Result<Effect, String> {
        if self.phase() == 0 || self.phase() == 6 {
            return Err("No editable solid command".into());
        }
        let before = self.data.clone();
        match self.input_inner(text.trim()) {
            Ok(effect) => {
                if effect == Effect::Waiting
                    && self.data.phase != 0
                    && !text.trim().eq_ignore_ascii_case("undo")
                {
                    self.history.push(before);
                }
                Ok(effect)
            }
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    fn input_inner(&mut self, text: &str) -> Result<Effect, String> {
        if text.eq_ignore_ascii_case("cancel") || text.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Ok(Effect::Cancelled);
        }
        if text.eq_ignore_ascii_case("undo") {
            self.data = self.history.pop().ok_or("No input to undo")?;
            return Ok(Effect::Waiting);
        }
        if self.kind == Kind::Sphere && text.eq_ignore_ascii_case("point") && self.phase() == 20 {
            self.data.point_constraint = true;
            return Ok(Effect::Waiting);
        }
        if self.kind == Kind::Sphere && text.eq_ignore_ascii_case("radius") && self.phase() == 9 {
            self.data.phase = 12;
            return Ok(Effect::Waiting);
        }
        if self.kind == Kind::Sphere
            && text.eq_ignore_ascii_case("radius")
            && self.phase() == 20
            && self.data.points.len() == 2
        {
            self.data.phase = 22;
            return Ok(Effect::Waiting);
        }
        let mode_text = text
            .strip_prefix("Mode=")
            .or_else(|| text.strip_prefix("mode="))
            .unwrap_or(text)
            .to_ascii_lowercase();
        if self.kind == Kind::Box && mode_text == "corners" {
            return Err("Corners is unavailable. Choose Diagonal, 3Point, Vertical or Center".into());
        }
        let mode = match (self.kind, mode_text.as_str()) {
            (Kind::Box, "diagonal") => Some(Mode::Diagonal),
            (Kind::Box, "vertical") => Some(Mode::Vertical),
            (Kind::Box, "center") => Some(Mode::Center),
            (Kind::Box, "cube") => Some(Mode::Cube),
            (_, "3point" | "3p") => Some(Mode::ThreePoint),
            (Kind::Sphere, "center") => Some(Mode::Center),
            (Kind::Sphere, "vertical") => Some(Mode::Vertical),
            (Kind::Sphere, "2point" | "2p") => Some(Mode::TwoPoint),
            (Kind::Sphere, "4point" | "4p") => Some(Mode::FourPoint),
            (Kind::Sphere, "fitpoints") => Some(Mode::FitPoints),
            (Kind::Sphere, "aroundcurve") => Some(Mode::AroundCurve),
            (Kind::Sphere, "tangent") => Some(Mode::Tangent),
            _ => None,
        };
        if let Some(mode) = mode {
            if self.phase() != 1
                && !(self.kind == Kind::Box
                    && mode == Mode::Cube
                    && self.phase() == 2
                    && self.data.mode == Mode::Diagonal)
            {
                return Err("Select the construction mode before the first point".into());
            }
            self.data.mode = mode;
            self.data.mode_selected = true;
            match mode {
                Mode::FitPoints => self.data.phase = 15,
                Mode::AroundCurve => self.data.phase = 16,
                Mode::Tangent => {
                    self.data.phase = 20;
                    self.data.first = self.frame.0;
                    self.data.axes = self.frame.1;
                }
                _ => {}
            }
            return Ok(Effect::Waiting);
        }
        if self.kind == Kind::Box && !self.data.mode_selected {
            return Err("Choose Diagonal, 3Point, Vertical or Center before entering a point".into());
        }
        if text.is_empty() {
            return match self.phase() {
                3 => self.numeric(self.data.length),
                4 => self.numeric(self.data.width.abs()),
                15 => {
                    let (c, r) = geometry::fit_points(&self.data.points)?;
                    self.data.first = c;
                    self.finish_sphere(r)
                }
                _ => Err("Enter a point or dimension before confirming".into()),
            };
        }
        if let Some((option, value)) = text.split_once('=') {
            if self.kind == Kind::Sphere
                && option.eq_ignore_ascii_case("Radius")
                && matches!(self.phase(), 20 | 22)
                && self.data.points.len() == 2
            {
                let r = dimension(number(value)?)?;
                if r <= 0. {
                    return Err("Radius must be positive".into());
                }
                self.data.radius = r;
                self.data.phase = 21;
                return Ok(Effect::Waiting);
            }
            if self.kind == Kind::Sphere
                && option.eq_ignore_ascii_case("Diameter")
                && self.phase() == 5
            {
                return self.numeric(number(value)? * 0.5);
            }
            if self.kind == Kind::Sphere
                && option.eq_ignore_ascii_case("Radius")
                && self.phase() == 9
            {
                let r = dimension(number(value)?)?;
                if r <= 0.
                    || r < dot(
                        sub(self.data.points[1], self.data.points[0]),
                        sub(self.data.points[1], self.data.points[0]),
                    )
                    .sqrt()
                        * 0.5
                {
                    return Err("Radius must be at least half the chord".into());
                }
                self.data.radius = r;
                self.data.phase = 11;
                return Ok(Effect::Waiting);
            }
            let allowed = match self.phase() {
                2 if self.kind == Kind::Box && self.data.mode == Mode::Center => "Length",
                3 => "Width",
                4 => "Height",
                5 => "Radius",
                7 => "Diameter",
                12 => "Radius",
                22 => "Radius",
                _ => "",
            };
            if allowed.is_empty() || !option.eq_ignore_ascii_case(allowed) {
                return Err("Option is unavailable at this step".into());
            }
            return self.numeric(number(value)?);
        }
        let point_only = self.phase() == 1 || (self.kind == Kind::Box && self.phase() == 2
            && self.data.mode != Mode::Center);
        let syntax = if point_only && text == "0" { NumericSyntax::Point } else { classify_numeric(text) };
        match syntax {
            NumericSyntax::Point => {}
            NumericSyntax::Scalar if point_only => return Err("Pick the required point or enter coordinates X,Y,Z; 0 means the construction-plane origin".into()),
            NumericSyntax::Scalar => return self.numeric(number(text)?),
            NumericSyntax::AngleConstraint => {
                return Err("One-shot angle constraints are not supported in this command phase".into());
            }
        }
        let frame = Frame::new(self.frame.0, self.frame.1)?;
        let point = PointInput::parse(text, number)?.resolve(frame, self.data.previous_accepted)?;
        let effect = self.point_inner(point.world)?;
        self.data.previous_accepted = Some(point.world);
        Ok(effect)
    }
    fn numeric(&mut self, n: f64) -> Result<Effect, String> {
        let name = match self.phase() {
            2 => "Length", 3 => "Width", 4 => "Height", 7 => "Diameter", _ => "Radius",
        };
        let n = dimension(n).map_err(|error| format!("{name}: {error}"))?;
        match self.phase() {
            2 => {
                if self.kind == Kind::Box && self.data.mode != Mode::Center {
                    return Err("Pick the end of the edge; Length input is available only in Center".into());
                }
                if matches!(self.data.mode, Mode::Diagonal | Mode::Cube) {
                    return Err("Pick the opposite diagonal corner".into());
                }
                if n < 0. {
                    return Err("Length must be positive".into());
                }
                self.data.length = n;
                self.data.last = add(self.data.first, scale(self.data.axes[0], n));
                self.data.phase = 3;
                Ok(Effect::Waiting)
            }
            3 => {
                self.data.width = n;
                self.data.phase = 4;
                Ok(Effect::Waiting)
            }
            4 => self.finish_box(n),
            5 => {
                if n < 0. {
                    return Err("Radius must be positive".into());
                }
                self.finish_sphere(n)
            }
            7 => {
                if n <= 0. {
                    return Err("Diameter must be positive".into());
                }
                let c = add(self.data.first, scale(self.data.axes[0], n * 0.5));
                self.data.first = c;
                self.finish_sphere(n * 0.5)
            }
            12 => {
                if n <= 0.
                    || n < dot(
                        sub(self.data.points[1], self.data.points[0]),
                        sub(self.data.points[1], self.data.points[0]),
                    )
                    .sqrt()
                        * 0.5
                {
                    return Err("Radius must be at least half the chord".into());
                }
                self.data.radius = n;
                self.data.phase = 11;
                Ok(Effect::Waiting)
            }
            22 => {
                if n <= 0. {
                    return Err("Radius must be positive".into());
                }
                self.data.radius = n;
                self.data.phase = 21;
                Ok(Effect::Waiting)
            }
            _ => Err("Pick or enter the first point".into()),
        }
    }
    pub fn point(&mut self, p: Point) -> Result<Effect, String> {
        let before = self.data.clone();
        match self.point_inner(p) {
            Ok(e) => {
                self.data.previous_accepted = Some(p);
                if e == Effect::Waiting {
                    self.history.push(before);
                }
                Ok(e)
            }
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    fn point_inner(&mut self, p: Point) -> Result<Effect, String> {
        if self.data.accepted_points.len()>=crate::fit_point_input::MAX_POINTS{return Err("Accepted point capture budget".into());}
        let effect=self.point_geometry(p)?;
        self.data.accepted_points.push(p);Ok(effect)
    }
    fn point_geometry(&mut self, p: Point) -> Result<Effect, String> {
        if self.kind == Kind::Box && !self.data.mode_selected {
            return Err("Choose Diagonal, 3Point, Vertical or Center before picking a point".into());
        }
        if !finite(p) {
            return Err("Invalid point coordinates".into());
        }
        match self.phase() {
            1 => {
                self.data.first = p;
                self.data.last = p;
                self.data.axes = self.frame.1;
                if self.data.mode == Mode::Vertical {
                    self.data.axes = [
                        self.frame.1[0],
                        self.frame.1[2],
                        scale(self.frame.1[1], -1.),
                    ];
                }
                self.data.points = vec![p];
                self.data.phase = if self.kind == Kind::Sphere {
                    match self.data.mode {
                        Mode::TwoPoint => 7,
                        Mode::ThreePoint | Mode::FourPoint => 8,
                        _ => 5,
                    }
                } else {
                    2
                };
                Ok(Effect::Waiting)
            }
            2 => {
                let d = sub(p, self.data.first);
                if self.data.mode == Mode::Cube {
                    let (axes, len) = geometry::cube_axes(self.data.axes, d)?;
                    self.data.axes = axes;
                    self.data.length = len;
                    self.data.width = len;
                    return self.finish_box(len);
                }
                let z = self.data.axes[2];
                if matches!(self.data.mode, Mode::ThreePoint | Mode::Vertical) {
                    let plane_normal = if self.data.mode == Mode::Vertical {
                        self.data.axes[1]
                    } else {
                        z
                    };
                    let edge = sub(d, scale(plane_normal, dot(d, plane_normal)));
                    let length = dimension(dot(edge, edge).sqrt())?;
                    self.data.axes[0] = scale(edge, 1. / length);
                    if self.data.mode == Mode::Vertical {
                        self.data.axes[2] = cross(self.data.axes[0], self.data.axes[1]);
                    } else {
                        self.data.axes[1] = cross(z, self.data.axes[0]);
                    }
                    self.data.length = length;
                    self.data.phase = 3;
                } else {
                    let k = if self.data.mode == Mode::Center {
                        2.
                    } else {
                        1.
                    };
                    self.data.length = dimension(k * dot(d, self.data.axes[0]))?;
                    self.data.width = dimension(k * dot(d, self.data.axes[1]))?;
                    self.data.phase = 4;
                }
                self.data.last = p;
                Ok(Effect::Waiting)
            }
            3 => {
                let factor = if self.data.mode == Mode::Center {
                    2.
                } else {
                    1.
                };
                let effect =
                    self.numeric(factor * dot(sub(p, self.data.first), self.data.axes[1]))?;
                self.data.last = p;
                Ok(effect)
            }
            4 => self.finish_box(dot(sub(p, self.data.first), self.data.axes[2])),
            5 => self.finish_sphere(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt()),
            7 => {
                let radius =
                    dimension(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt() * 0.5)?;
                self.data.first = scale(add(self.data.first, p), 0.5);
                self.finish_sphere(radius)
            }
            8 => {
                dimension(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt())?;
                self.data.points.push(p);
                self.data.last = p;
                self.data.phase = 9;
                Ok(Effect::Waiting)
            }
            9 => {
                let circle = geometry::circle(self.data.points[0], self.data.points[1], p)?;
                self.circle_result(circle)
            }
            10 => {
                let (c, r) = geometry::fourth(self.data.circle.unwrap(), p)?;
                self.data.first = c;
                self.finish_sphere(r)
            }
            11 => {
                let circle = geometry::radius_circle(
                    self.data.points[0],
                    self.data.points[1],
                    self.data.radius,
                    p,
                )?;
                self.circle_result(circle)
            }
            15 => {
                if self.data.points.len() >= crate::fit_point_input::MAX_POINTS {
                    return Err("FitPoints supports up to 1024 points".into());
                }
                self.data.points.push(p);
                self.data.last = p;
                Ok(Effect::Waiting)
            }
            20 => {
                if !self.data.point_constraint {
                    return Err("Select a curve or choose Point first".into());
                }
                self.push_constraint(p, true)
            }
            _ => Err("No editable solid command".into()),
        }
    }
    fn push_constraint(&mut self, p: Point, is_point: bool) -> Result<Effect, String> {
        if self.data.points.len() >= 3 {
            return Err("Three constraints already selected".into());
        }
        if self.data.points.is_empty() {
            self.data.first = p;
            self.data.axes = self.frame.1;
        }
        self.data.points.push(p);
        self.data.constraint_types.push(is_point);
        self.data.last = p;
        self.data.point_constraint = false;
        if self.data.points.len() == 3 {
            self.data.phase = 21;
        }
        Ok(Effect::Waiting)
    }
    pub fn reference_mode(&self) -> u32 {
        match self.phase() {
            16 => 1,
            17 => 2,
            20 if !self.data.point_constraint => 3,
            15 => 4,
            _ => 0,
        }
    }
    pub fn accept_reference(&mut self, p: Point, tangent: Point) -> Result<Effect, String> {
        let before = self.data.clone();
        let effect = (|| {
            if self.data.accepted_points.len()>=crate::fit_point_input::MAX_POINTS{return Err("Accepted point capture budget".into());}
            if !finite(p) {
                return Err("Invalid reference point".into());
            }
            match self.phase() {
                16 => {
                    self.data.phase = 17;
                    self.data.last = p;
                    Ok(Effect::Waiting)
                }
                17 => {
                    let z = geometry::unit(tangent)?;
                    let candidate = if z[0].abs() < 0.8 {
                        [1., 0., 0.]
                    } else {
                        [0., 1., 0.]
                    };
                    let x = geometry::unit(cross(candidate, z))?;
                    self.data.axes = [x, cross(z, x), z];
                    self.data.first = p;
                    self.data.last = p;
                    self.data.phase = 5;
                    Ok(Effect::Waiting)
                }
                20 if !self.data.point_constraint => self.push_constraint(p, false),
                _ => Err("No curve selection is expected".into()),
            }
        })();
        match effect {
            Ok(e) => {
                self.data.accepted_points.push(p);
                self.data.previous_accepted = Some(p);
                self.history.push(before);
                Ok(e)
            }
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    pub fn resolve(&mut self, center: Point, radius: f64) -> Result<Effect, String> {
        if self.phase() != 21 || !finite(center) {
            return Err("No valid tangent solution is expected".into());
        }
        let before = self.data.clone();
        self.data.first = center;
        match self.finish_sphere(radius) {
            Ok(e) => Ok(e),
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    pub fn append_fit_points(&mut self, points: &[Point]) -> Result<Effect, String> {
        if self.phase() != 15
            || !crate::fit_point_input::admit_count(self.data.points.len(),points.len())
            || points.iter().any(|&p| !finite(p))
        {
            return Err("Invalid FitPoints batch or point limit exceeded".into());
        }
        if points.is_empty() {
            return Ok(Effect::Waiting);
        }
        self.history.push(self.data.clone());
        self.data.accepted_points.extend_from_slice(points);
        self.data.points.extend_from_slice(points);
        self.data.last = *points.last().unwrap();
        self.data.previous_accepted = Some(self.data.last);
        Ok(Effect::Waiting)
    }
    fn circle_result(&mut self, circle: (Point, f64, Point)) -> Result<Effect, String> {
        self.data.circle = Some(circle);
        if self.data.mode == Mode::FourPoint {
            self.data.phase = 10;
            Ok(Effect::Waiting)
        } else {
            self.data.first = circle.0;
            self.finish_sphere(circle.1)
        }
    }
    fn finish_box(&mut self, height: f64) -> Result<Effect, String> {
        let dims = [
            dimension(self.data.length)?,
            dimension(self.data.width)?,
            dimension(height)?,
        ];
        let mut origin = self.data.first;
        if self.data.mode == Mode::Center {
            origin = sub(
                sub(origin, scale(self.data.axes[0], dims[0] * 0.5)),
                scale(self.data.axes[1], dims[1] * 0.5),
            );
        }
        for (i, &n) in dims.iter().enumerate() {
            if n < 0. {
                origin = add(origin, scale(self.data.axes[i], n));
            }
        }
        let mut b = [0.; 15];
        b[..3].copy_from_slice(&origin);
        for i in 0..3 {
            b[3 + i * 3..6 + i * 3].copy_from_slice(&self.data.axes[i]);
            b[12 + i] = dims[i].abs();
        }
        // Check all extrema, not only the nominal origin.
        for bits in 0..8 {
            let mut p = origin;
            for i in 0..3 {
                if bits & (1 << i) != 0 {
                    p = add(p, scale(self.data.axes[i], b[12 + i]));
                }
            }
            if !finite(p) {
                return Err("Box exceeds supported coordinate range".into());
            }
        }
        self.data.output = Some(b);
        self.data.phase = 6;
        Ok(Effect::Commit)
    }
    fn finish_sphere(&mut self, radius: f64) -> Result<Effect, String> {
        let radius = dimension(radius)?;
        if radius < 0. || self.data.first.iter().any(|x| x.abs() + radius > 1e9) {
            return Err("Invalid sphere radius or coordinate range".into());
        }
        let mut b = [0.; 15];
        b[..3].copy_from_slice(&self.data.first);
        for i in 0..3 {
            b[3 + i * 3..6 + i * 3].copy_from_slice(&self.data.axes[i]);
        }
        b[12] = radius;
        self.data.output = Some(b);
        self.data.phase = 6;
        Ok(Effect::Commit)
    }
    pub fn preview(&self, p: Point, ortho: bool) -> Option<[f64; 15]> {
        if self.kind == Kind::Box && !self.data.mode_selected { return None; }
        let mut candidate = Self {
            kind: self.kind,
            data: self.data.clone(),
            frame: self.frame,
            history: Vec::new(),
        };
        candidate.point(candidate.constrain(p, ortho)).ok()?;
        if candidate.data.mode == Mode::FitPoints && candidate.data.points.len() >= 3 {
            candidate.input("").ok()?;
        }
        candidate.output()
    }
    /// Scene-only construction outline. A base is drawable before there is a
    /// complete solid; hover never adds an accepted point or an Undo checkpoint.
    pub fn wire_preview(&self, p: Point, ortho: bool) -> Option<WirePreview> {
        if self.kind!=Kind::Box || !self.data.mode_selected || !matches!(self.phase(),2|3|4) || !finite(p) {return None;}
        let mut candidate=Self{kind:self.kind,data:self.data.clone(),frame:self.frame,history:Vec::new()};
        let height=dot(sub(p,self.data.first),self.data.axes[2]);
        if !(self.phase()==4 && height.abs()<1e-7) {candidate.point(candidate.constrain(p,ortho)).ok()?;}
        if candidate.phase()==3 {
            let end=add(candidate.data.first,scale(candidate.data.axes[0],candidate.data.length));
            return Some(WirePreview{segments:vec![[candidate.data.first,end]],markers:[candidate.data.first,end,end]});
        }
        let (origin,axes,dims,full)=if let Some(b)=candidate.output(){
            ([b[0],b[1],b[2]],[[b[3],b[4],b[5]],[b[6],b[7],b[8]],[b[9],b[10],b[11]]],[b[12],b[13],b[14]],true)
        }else if candidate.phase()==4 {
            let mut origin=candidate.data.first;let axes=candidate.data.axes;
            let length=dimension(candidate.data.length).ok()?;let width=dimension(candidate.data.width).ok()?;
            if candidate.data.mode==Mode::Center {origin=sub(sub(origin,scale(axes[0],length*0.5)),scale(axes[1],width*0.5));}
            if length<0. {origin=add(origin,scale(axes[0],length));}
            if width<0. {origin=add(origin,scale(axes[1],width));}
            (origin,axes,[length.abs(),width.abs(),0.],false)
        }else{return None;};
        let mut vertices=[[0.;3];8];
        for (i,v) in vertices.iter_mut().enumerate(){
            *v=add(add(add(origin,scale(axes[0],if i&1!=0{dims[0]}else{0.})),scale(axes[1],if i&2!=0{dims[1]}else{0.})),scale(axes[2],if i&4!=0{dims[2]}else{0.}));
            if !finite(*v){return None;}
        }
        let pairs=[(0,1),(1,3),(3,2),(2,0),(4,5),(5,7),(7,6),(6,4),(0,4),(1,5),(2,6),(3,7)];
        let segments=pairs[..if full{12}else{4}].iter().map(|&(a,b)|[vertices[a],vertices[b]]).collect();
        // Mark the construction base, not the normalized lower face: negative
        // Height moves the BRep origin but must not shift these markers twice.
        let factor=if candidate.data.mode==Mode::Center{0.5}else{1.};
        let opposite=add(add(candidate.data.first,scale(candidate.data.axes[0],candidate.data.length*factor)),scale(candidate.data.axes[1],candidate.data.width*factor));
        let signed_height=if candidate.data.mode==Mode::Cube{candidate.data.length}else{height};
        let top=if full{add(opposite,scale(candidate.data.axes[2],signed_height))}else{opposite};
        Some(WirePreview{segments,markers:[self.data.first,opposite,top]})
    }
    fn height_anchor_for(&self, state:&Self)->Point {
        add(add(state.data.first,scale(state.data.axes[0],state.data.length)),scale(state.data.axes[1],state.data.width))
    }
    pub fn height_anchor(&self)->Point {
        if self.kind==Kind::Box && self.data.mode==Mode::Corners {self.height_anchor_for(self)}else{self.data.first}
    }
    /// Screen-up drag when a top/bottom view ray is parallel to the height axis.
    /// Native code supplies the hit on the base plane and camera's world up.
    pub fn height_drag(&self, hit:Point, up:Point)->Option<Point> {
        if self.kind!=Kind::Box || self.phase()!=4 || !finite(hit) || !finite(up) {return None;}
        let n=self.data.axes[2];let tangent=sub(up,scale(n,dot(up,n)));
        let norm=dot(tangent,tangent).sqrt();if norm<1e-9{return None;}
        let anchor=self.height_anchor();let height=dot(sub(hit,anchor),scale(tangent,1./norm));
        let result=add(anchor,scale(n,height));finite(result).then_some(result)
    }
    pub fn constrain(&self, p: Point, ortho: bool) -> Point {
        if !ortho
            || self.phase() != 2
            || !matches!(self.data.mode, Mode::ThreePoint | Mode::Vertical)
        {
            return p;
        }
        let delta = sub(p, self.data.first);
        let x = dot(delta, self.data.axes[0]);
        let y_axis = if self.data.mode == Mode::Vertical {
            scale(self.data.axes[2], -1.)
        } else {
            self.data.axes[1]
        };
        let y = dot(delta, y_axis);
        add(
            self.data.first,
            if x.abs() >= y.abs() {
                scale(self.data.axes[0], x)
            } else {
                scale(y_axis, y)
            },
        )
    }
    pub fn cancel(&mut self) {
        self.data.phase = 0;
        self.data.output = None;
        self.history.clear();
    }
}
#[derive(Default)]
struct State {
    session: Option<Session>,
    message: String,
    generation: u64,
    input_revision: u64,
    error: String,
}
pub(crate) fn command_option_state(scale: f64) -> Option<(u64,u64,crate::command_options::OptionSession)> {
    let locked=state().lock().ok()?;
    let session=locked.session.as_ref()?;
    let model=session.command_options(locked.generation,scale).ok()?;
    Some((locked.generation,locked.input_revision,model))
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::default()))
}
pub(crate) fn current_construction_capability(input:crate::geometry_contract::RepresentationKind,stage:crate::native_solid_capability::Stage)
    ->Option<Result<crate::geometry_contract::OperationCapability,crate::geometry_contract::CapabilityError>>{
    use crate::geometry_contract::{CapabilityError,OperationId};
    let locked=state().lock().ok()?;
    Some(locked.session.as_ref().map_or(Err(CapabilityError::UnsupportedOperation(OperationId::ConstructSolid)),
        |s|s.construction_capability(input,stage)))
}
fn string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .ok()
            .filter(|s| s.len() <= 1024)
            .map(str::to_owned)
    }
}
fn change(f: impl FnOnce(&mut Session) -> Result<Effect, String>) -> u32 {
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    let result = state
        .session
        .as_mut()
        .ok_or("No active solid command".to_owned())
        .and_then(f);
    state.input_revision = state.input_revision.saturating_add(1);
    state.error = result.as_ref().err().cloned().unwrap_or_default();
    state.message = match &result {
        Ok(_) => state
            .session
            .as_ref()
            .map_or(String::new(), Session::prompt),
        Err(e) if state.session.as_ref().is_some_and(|session|session.kind == Kind::Box) => e.clone(),
        Err(e) => format!(
            "{e}. {}",
            state
                .session
                .as_ref()
                .map_or(String::new(), Session::prompt)
        ),
    };
    match result {
        Ok(Effect::Waiting) => 1,
        Ok(Effect::Commit) => 2,
        Ok(Effect::Cancelled) => {
            state.generation = state.generation.saturating_add(1);
            3
        },
        Err(_) => 0,
    }
}
/// # Safety
/// Non-null input is a live NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_kind(p: *const c_char) -> u32 {
    string(p)
        .and_then(|s| Kind::from_name(&s))
        .map_or(0, |k| k as u32)
}
/// # Safety
/// Non-null input is a live NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_start(p: *const c_char) -> bool {
    let Some(kind) = string(p).and_then(|s| Kind::from_name(&s)) else {
        return false;
    };
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    let session = Session::new(kind);
    if session.construction_capability(crate::geometry_contract::RepresentationKind::Point,crate::native_solid_capability::Stage::Declare).is_err(){return false;}
    let Some(generation) = state.generation.checked_add(1) else { return false; };
    state.generation = generation;
    state.message = session.prompt();
    state.input_revision = 0;
    state.error.clear();
    state.session = Some(session);
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_cancel() {
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    let generation = state.generation.saturating_add(1);
    *state = State { generation, ..State::default() };
}
/// Scalar request identity; no native owner or pointer crosses this boundary.
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_generation() -> u64 {
    state().lock().unwrap_or_else(|p| p.into_inner()).generation
}
/// # Safety
/// Caller owns a writable non-overlapping allocation of `capacity` bytes for
/// this call. Null/capacity0 queries size. No pointer retained; Rust panic is
/// contained. Generation is checked under the same lock as capture/copy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_construction_json(generation:u64,output:*mut u8,capacity:usize)
    ->crate::prepared_result::Summary{
    use crate::prepared_result::{Summary,MAX_RESULT_BYTES};
    let rejected=|status,needed|Summary{status,reserved:0,needed};
    if capacity>MAX_RESULT_BYTES||capacity>isize::MAX as usize{return rejected(2,0);}
    if (output.is_null()&&capacity!=0)||(!output.is_null()&&(output as usize).checked_add(capacity).is_none()){
        return rejected(1,0);
    }
    std::panic::catch_unwind(||{
        let state=state().lock().unwrap_or_else(|p|p.into_inner());
        if generation==0||generation!=state.generation{return rejected(5,0);}
        let Some(session)=state.session.as_ref()else{return rejected(5,0);};
        let evidence=match session.construction_evidence(generation){Ok(e)=>e,Err(status)=>return rejected(status,0)};
        let needed=evidence.len();
        if output.is_null(){return rejected(0,needed);}
        if capacity<needed{return rejected(2,needed);}
        unsafe{std::ptr::copy_nonoverlapping(evidence.as_ptr(),output,needed);}
        rejected(0,needed)
    }).unwrap_or_else(|_|rejected(7,0))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_phase() -> u32 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, Session::phase)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_reference_mode() -> u32 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, Session::reference_mode)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_mode_name() -> *const c_char {
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let name = guard.session.as_ref().map_or(c"", |s| if s.kind==Kind::Box&&!s.data.mode_selected {c"ChooseMethod"} else {match s.data.mode {
        Mode::Corners => c"Corners",
        Mode::Diagonal => c"Diagonal",
        Mode::ThreePoint => c"3Point",
        Mode::Vertical => c"Vertical",
        Mode::Center => c"Center",
        Mode::Cube => c"Cube",
        Mode::TwoPoint => c"2Point",
        Mode::FourPoint => c"4Point",
        Mode::FitPoints => c"FitPoints",
        Mode::AroundCurve => c"AroundCurve",
        Mode::Tangent => c"Tangent",
    }});
    name.as_ptr()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_reference(x: f64, y: f64, z: f64, tx: f64, ty: f64, tz: f64) -> u32 {
    change(|s| s.accept_reference([x, y, z], [tx, ty, tz]))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_resolve(x: f64, y: f64, z: f64, r: f64) -> u32 {
    change(|s| s.resolve([x, y, z], r))
}
/// # Safety
/// Non-null output addresses 12 doubles: tangent frame origin and its axes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_reference_frame(p: *mut f64) -> bool {
    if p.is_null() {
        return false;
    }
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &guard.session else {
        return false;
    };
    let mut b = [0.; 12];
    b[..3].copy_from_slice(&s.data.first);
    for i in 0..3 {
        b[3 + i * 3..6 + i * 3].copy_from_slice(&s.data.axes[i]);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(b.as_ptr(), p, 12);
    }
    true
}
/// # Safety
/// Non-null output addresses 12 doubles: up to three x,y,z,is_point constraints.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_constraints(p: *mut f64) -> usize {
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &guard.session else {
        return 0;
    };
    if s.data.mode != Mode::Tangent {
        return 0;
    }
    if !p.is_null() {
        for (i, point) in s.data.points.iter().enumerate() {
            unsafe {
                std::ptr::copy_nonoverlapping(point.as_ptr(), p.add(i * 4), 3);
                *p.add(i * 4 + 3) = if s.data.constraint_types[i] { 1. } else { 0. };
            }
        }
    }
    s.data.points.len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_tangent_radius() -> f64 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0., |s| s.data.radius)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_point_count() -> usize {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, |s| s.data.points.len())
}
/// # Safety
/// Input points addresses count*3 readable doubles; count is at most 1024.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_fit_points(p: *const f64, count: usize) -> u32 {
    if p.is_null() || !crate::fit_point_input::admit_count(0,count) {
        return 0;
    }
    let values = unsafe { std::slice::from_raw_parts(p, count * 3) };
    let points = values.as_chunks::<3>().0.to_vec();
    change(|s| s.append_fit_points(&points))
}
/// # Safety
/// Non-null output addresses 6 doubles: special point-pick plane origin and normal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_pick_plane(p: *mut f64) -> bool {
    if p.is_null() {
        return false;
    }
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &guard.session else {
        return false;
    };
    if !((s.kind == Kind::Box && s.data.mode == Mode::Vertical && s.phase() == 3)
        || (s.kind == Kind::Sphere
            && matches!(s.data.mode, Mode::Vertical | Mode::AroundCurve)
            && s.phase() == 5))
    {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.data.first.as_ptr(), p, 3);
        std::ptr::copy_nonoverlapping(s.data.axes[2].as_ptr(), p.add(3), 3);
    }
    true
}
/// # Safety
/// Non-null input is a live NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_input(p: *const c_char) -> u32 {
    let Some(text) = string(p) else {
        return 0;
    };
    change(|s| s.input(&text))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_point(x: f64, y: f64, z: f64, ortho: bool) -> u32 {
    change(|s| s.point(s.constrain([x, y, z], ortho)))
}
/// # Safety
/// Non-null frame addresses 12 doubles: origin, X, Y, Z.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_frame(p: *const f64) -> bool {
    if p.is_null() {
        return false;
    }
    let p = unsafe { std::slice::from_raw_parts(p, 12) };
    let v = |i| [p[i], p[i + 1], p[i + 2]];
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_mut()
        .is_some_and(|s| s.set_frame(v(0), [v(3), v(6), v(9)]).is_ok())
}
/// # Safety
/// Non-null output addresses 15 writable doubles. Preview uses a copied session.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_geometry(
    p: *mut f64,
    preview: bool,
    x: f64,
    y: f64,
    z: f64,
    ortho: bool,
) -> bool {
  std::panic::catch_unwind(||{
    if p.is_null() {
        return false;
    }
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &state.session else {
        return false;
    };
    use crate::{geometry_contract::*,native_solid_capability::Stage};
    let stage=if preview{Stage::Preview}else{Stage::Construct};
    if s.construction_capability(RepresentationKind::Point,stage).is_err(){return false;}
    let b = if preview {
        s.preview([x, y, z], ortho)
    } else {
        s.output()
    };
    let Some(b) = b else {
        return false;
    };
    let Ok(mut request)=s.capability_request(stage) else{return false;};request.output=Some(b);
    if operation_capability(OperationId::ConstructSolid,RepresentationKind::Point,OperationOptions::NativeSolid(request)).is_err(){return false;}
    unsafe {
        std::ptr::copy_nonoverlapping(b.as_ptr(), p, 15);
    }
    true
  }).unwrap_or(false)
}
/// # Safety
/// Segments addresses 72 writable doubles; markers addresses 9 writable doubles.
/// Return is the bounded segment count (1, 4 or 12), zero for no valid outline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_wire_preview(segments:*mut f64,markers:*mut f64,x:f64,y:f64,z:f64,ortho:bool)->u32 {
    std::panic::catch_unwind(||{
        if segments.is_null()||markers.is_null(){return 0;}
        let locked=state().lock().unwrap_or_else(|p|p.into_inner());let Some(s)=&locked.session else{return 0;};
        if s.construction_capability(crate::geometry_contract::RepresentationKind::Point,crate::native_solid_capability::Stage::Preview).is_err(){return 0;}
        let Some(wire)=s.wire_preview([x,y,z],ortho) else{return 0;};
        if wire.segments.len()>12{return 0;}
        for(i,edge)in wire.segments.iter().enumerate(){for(j,p)in edge.iter().enumerate(){unsafe{std::ptr::copy_nonoverlapping(p.as_ptr(),segments.add(i*6+j*3),3);}}}
        for(i,p)in wire.markers.iter().enumerate(){unsafe{std::ptr::copy_nonoverlapping(p.as_ptr(),markers.add(i*3),3);}}
        wire.segments.len() as u32
    }).unwrap_or(0)
}
/// # Safety
/// Result addresses 3 writable doubles; hit and up each address 3 readable doubles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_height_drag(result:*mut f64,hit:*const f64,up:*const f64)->bool {
    std::panic::catch_unwind(||{
        if result.is_null()||hit.is_null()||up.is_null(){return false;}
        let hit=unsafe{[*hit,*hit.add(1),*hit.add(2)]};let up=unsafe{[*up,*up.add(1),*up.add(2)]};
        let locked=state().lock().unwrap_or_else(|p|p.into_inner());let Some(s)=&locked.session else{return false;};
        let Some(p)=s.height_drag(hit,up)else{return false;};unsafe{std::ptr::copy_nonoverlapping(p.as_ptr(),result,3);}true
    }).unwrap_or(false)
}
/// # Safety
/// Non-null output addresses 6 writable doubles: height anchor and normal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_height_axis(p: *mut f64) -> bool {
    if p.is_null() {
        return false;
    }
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &state.session else {
        return false;
    };
    if s.phase() != 4 {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.height_anchor().as_ptr(), p, 3);
        std::ptr::copy_nonoverlapping(s.data.axes[2].as_ptr(), p.add(3), 3);
    }
    true
}
/// # Safety
/// Non-null buffer must be writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_message(p: *mut c_char, capacity: usize) -> usize {
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let b = state.message.as_bytes();
    if !p.is_null() && capacity > 0 {
        let n = b.len().min(capacity - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(b.as_ptr(), p.cast(), n);
            *p.add(n) = 0;
        }
    }
    b.len()
}
