// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive commands: one JSON step per user input.
//!
//! A command such as Polyline, Circle, Move or Gem Cutter is a small state machine. OM9 sends
//! an [`Event`] for every input (start, a picked point, the cursor moving, typed text, a clicked
//! command-line option, Enter, Undo) together with the state the add-on returned last time.
//! The add-on answers with a [`Reply`]: the new state, the prompt and the command-line options
//! (the "sub-commands" such as `Close=No  Length  Undo`), a preview, and — when the command
//! finishes — the geometry to create or the transform to apply to the selection.
//!
//! The add-on keeps no state between calls (the host holds it), so a crash or a cancelled
//! command can never leave anything behind, and every step can be tested as a pure function.
//! All coordinates are world millimetres.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type Vec3 = [f64; 3];

/// The active construction plane (origin and unit axes in world coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub origin: Vec3,
    pub x: Vec3,
    pub y: Vec3,
}

impl Default for Plane {
    fn default() -> Self {
        Plane {
            origin: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
        }
    }
}

impl Plane {
    pub fn normal(&self) -> Vec3 {
        unit(cross(self.x, self.y))
    }
    /// World point of plane coordinates (u, v, w along x, y, normal).
    pub fn at(&self, u: f64, v: f64, w: f64) -> Vec3 {
        let n = self.normal();
        add(
            self.origin,
            add(scale(self.x, u), add(scale(self.y, v), scale(n, w))),
        )
    }
    /// Plane coordinates (u, v, w) of a world point.
    pub fn local(&self, p: Vec3) -> Vec3 {
        let d = sub(p, self.origin);
        [dot(d, self.x), dot(d, self.y), dot(d, self.normal())]
    }
    /// The same plane moved to `origin`.
    pub fn moved(&self, origin: Vec3) -> Plane {
        Plane { origin, ..*self }
    }
}

/// One object of the current selection, as the add-on sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Selected {
    pub id: String,
    /// `curve`, `mesh`, `point`, `brep`, `surface`, …
    pub kind: String,
    pub name: String,
    /// World bounding box `[min, max]`.
    pub bbox: [Vec3; 2],
    /// Curves: samples along the curve (closed curves without the repeated end point).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<Vec3>,
    #[serde(default)]
    pub closed: bool,
    /// The `om9.gem` record of a gem (cut, diameter, frame …), when the object is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gem: Option<Value>,
    /// World triangles of a mesh or solid, for commands with `mesh_data = true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<SelectedMesh>,
    /// The exact world definition of a curve or NURBS surface (control points, knots …),
    /// for commands that edit or continue it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<Geo>,
}

/// Triangles of a selected object in world millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SelectedMesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

impl SelectedMesh {
    /// Enclosed volume (mm³) by the divergence theorem; meaningful for closed meshes.
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .filter_map(|t| {
                let p = |i: u32| self.vertices.get(i as usize).copied();
                Some(dot(p(t[0])?, cross(p(t[1])?, p(t[2])?)))
            })
            .sum::<f64>()
            / 6.0
    }
    /// Surface area (mm²).
    pub fn area(&self) -> f64 {
        self.triangles
            .iter()
            .filter_map(|t| {
                let p = |i: u32| self.vertices.get(i as usize).copied();
                let (a, b, c) = (p(t[0])?, p(t[1])?, p(t[2])?);
                Some(norm(cross(sub(b, a), sub(c, a))) / 2.0)
            })
            .sum()
    }
    /// Edges used by one triangle only (holes), and by more than two (non-manifold).
    pub fn open_and_nonmanifold_edges(&self) -> (usize, usize) {
        let mut edges = std::collections::BTreeMap::<(u32, u32), u32>::new();
        for t in &self.triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        (
            edges.values().filter(|&&n| n == 1).count(),
            edges.values().filter(|&&n| n > 2).count(),
        )
    }
}

/// What happened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Input {
    /// The command was started (menu, icon, typed name or shortcut).
    Start,
    /// A point was clicked in a viewport (on the active CPlane).
    Point { point: Vec3 },
    /// The cursor moved over a viewport; answer with a preview only.
    Hover { point: Vec3 },
    /// Text typed at the command line that is not one of the declared options
    /// (coordinates `1,2`, `@3,0`, a number, a word).
    Text { text: String },
    /// A command-line option was clicked or typed (`Close`, `Length=5`).
    Option {
        name: String,
        #[serde(default)]
        value: Option<String>,
    },
    /// Enter / right click: accept.
    Enter,
    /// The `Undo` option of a command that collects points.
    Undo,
    /// The selection changed while the command waits for objects.
    Selection,
}

/// One step sent to the add-on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub command: String,
    pub input: Input,
    /// The state the add-on returned last time (`null` on start).
    #[serde(default)]
    pub state: Value,
    #[serde(default)]
    pub plane: Plane,
    #[serde(default)]
    pub selection: Vec<Selected>,
    /// Where the click that last changed the selection landed (the point under the cursor on
    /// the CPlane, or its snap), world millimetres. Commands that pick a curve by location
    /// (Fillet keeps the picked sides, Extend the picked end) read it with the selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<Vec3>,
    /// The add-on's settings (builder panel values), by key.
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
}

impl Event {
    pub fn number(&self, key: &str, default: f64) -> f64 {
        self.settings
            .get(key)
            .and_then(Value::as_f64)
            .unwrap_or(default)
    }
    pub fn text<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.settings
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or(default)
    }
    pub fn flag(&self, key: &str, default: bool) -> bool {
        self.settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }
}

/// Geometry in a reply (preview or result), world millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Geo {
    Point {
        point: Vec3,
    },
    Line {
        start: Vec3,
        end: Vec3,
    },
    Polyline {
        points: Vec<Vec3>,
        #[serde(default)]
        closed: bool,
    },
    /// Full circle in the plane (center, x, y).
    Circle {
        center: Vec3,
        x: Vec3,
        y: Vec3,
        radius: f64,
    },
    /// Arc from `x` towards `y` over `angle` radians (0 < angle ≤ 2π).
    Arc {
        center: Vec3,
        x: Vec3,
        y: Vec3,
        radius: f64,
        angle: f64,
    },
    /// Clamped NURBS curve: knots has `points + degree + 1` values.
    Nurbs {
        degree: usize,
        points: Vec<Vec3>,
        #[serde(default)]
        weights: Vec<f64>,
        knots: Vec<f64>,
    },
    /// Clamped NURBS surface. Poles are u-major (`index = u * size[1] + v`); each knot
    /// vector has `size + degree + 1` values.
    Surface {
        degree: [usize; 2],
        size: [usize; 2],
        points: Vec<Vec3>,
        #[serde(default)]
        weights: Vec<f64>,
        knots: [Vec<f64>; 2],
    },
    Mesh {
        #[serde(default)]
        name: String,
        vertices: Vec<Vec3>,
        triangles: Vec<[u32; 3]>,
        /// JSON object stored in the object's `om9.gem` slot (gems only).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        meta: Option<Value>,
        /// Layer to put the object on, by name (`Cutting`, `Gem 01` …); the current layer when
        /// absent or unknown.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
    },
}

/// How a command-line option behaves when clicked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OptionKind {
    /// Yes/No.
    Toggle,
    /// One of several values; a click steps to the next one.
    Choice { choices: Vec<String> },
    /// A number (a click puts `Name=` in the command line).
    Number,
    /// Runs at once (`Undo`, `Close`, `Sharp` …).
    Action,
}

/// A command-line option ("sub-command") shown above the prompt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionToken {
    pub name: String,
    /// Current value shown as `Name=value` (none for actions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(flatten)]
    pub kind: OptionKind,
}

impl OptionToken {
    pub fn toggle(name: &str, on: bool) -> Self {
        OptionToken {
            name: name.into(),
            value: Some(if on { "Yes" } else { "No" }.into()),
            kind: OptionKind::Toggle,
        }
    }
    pub fn choice(name: &str, value: &str, choices: &[&str]) -> Self {
        OptionToken {
            name: name.into(),
            value: Some(value.into()),
            kind: OptionKind::Choice {
                choices: choices.iter().map(|c| c.to_string()).collect(),
            },
        }
    }
    pub fn number(name: &str, value: Option<f64>) -> Self {
        OptionToken {
            name: name.into(),
            value: value.map(fmt),
            kind: OptionKind::Number,
        }
    }
    pub fn action(name: &str) -> Self {
        OptionToken {
            name: name.into(),
            value: None,
            kind: OptionKind::Action,
        }
    }
}

/// What the command waits for next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Want {
    /// A point (click or typed coordinates).
    #[default]
    Point,
    /// Objects: the user selects, then presses Enter.
    Objects,
    /// A number typed at the command line (or a point that gives it, such as a radius).
    Number,
    /// Nothing more: Enter accepts.
    Nothing,
}

/// A transform applied to the selected objects. Each matrix is row-major 4×4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformOp {
    pub matrices: Vec<[f64; 16]>,
    /// `false` with one matrix: move the originals (Move, Rotate, Scale, Mirror with Copy=No).
    /// `true`: keep the originals and add one transformed copy per matrix (Copy, Array).
    pub keep_original: bool,
    /// One matrix per selected object (by [`Selected::id`]) instead of `matrices`: each stone
    /// flipped about its own girdle, each ring resized about its own axis …
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub per_object: Vec<ObjectMatrix>,
}

/// A transform for one selected object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectMatrix {
    pub id: String,
    pub matrix: [f64; 16],
}

/// The add-on's answer to one [`Event`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Reply {
    #[serde(default)]
    pub state: Value,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub options: Vec<OptionToken>,
    #[serde(default)]
    pub want: Want,
    /// Preview drawn over the viewports (never a document object).
    #[serde(default)]
    pub preview: Vec<Geo>,
    /// The command follows the pointer (Sketch): the host keeps the `state` of replies to
    /// hover events too, not only to clicks.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub track: bool,
    /// Draw a rubber-band line from this point to the cursor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rubber_from: Option<Vec3>,
    /// Preview of the selection under these transforms.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preview_transforms: Vec<[f64; 16]>,
    /// The command finished: create `create`, apply `transform`, then close.
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub create: Vec<Geo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformOp>,
    /// Lines for the command history.
    #[serde(default)]
    pub messages: Vec<String>,
    /// The input was refused (bad value); the command keeps going.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// With `done`: selected objects (by [`Selected::id`]) to delete in the same transaction
    /// (a stone replaced by another cut, a curve consumed by the command …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delete: Vec<String>,
    /// With `done`: new world geometry for selected objects (Bend, Twist, Taper …). The object
    /// keeps its id, layer and attributes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replace: Vec<Replacement>,
    /// With `done`: new `om9.gem` records (world frame) for selected objects, e.g. a material.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RecordUpdate>,
}

/// New geometry for one selected object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replacement {
    pub id: String,
    pub geo: Geo,
}

/// A new gem record for one selected object, in world millimetres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordUpdate {
    pub id: String,
    pub record: Value,
}

impl Reply {
    pub fn ask(prompt: impl Into<String>, state: Value) -> Reply {
        Reply {
            prompt: prompt.into(),
            state,
            ..Default::default()
        }
    }
    pub fn with_options(mut self, options: Vec<OptionToken>) -> Reply {
        self.options = options;
        self
    }
    pub fn want(mut self, want: Want) -> Reply {
        self.want = want;
        self
    }
    pub fn preview(mut self, geo: Vec<Geo>) -> Reply {
        self.preview = geo;
        self
    }
    pub fn rubber(mut self, from: Option<Vec3>) -> Reply {
        self.rubber_from = from;
        self
    }
    pub fn refuse(mut self, why: impl Into<String>) -> Reply {
        self.error = Some(why.into());
        self
    }
    pub fn finish(create: Vec<Geo>) -> Reply {
        Reply {
            done: true,
            create,
            want: Want::Nothing,
            ..Default::default()
        }
    }
    pub fn finish_transform(op: TransformOp) -> Reply {
        Reply {
            done: true,
            transform: Some(op),
            want: Want::Nothing,
            ..Default::default()
        }
    }
    /// Keep the state of hover replies (see [`Reply::track`]).
    pub fn tracking(mut self) -> Reply {
        self.track = true;
        self
    }
    pub fn say(mut self, message: impl Into<String>) -> Reply {
        self.messages.push(message.into());
        self
    }
}

/// An add-on with interactive commands.
pub trait Interactive {
    /// Catalog codes handled by [`Interactive::step`].
    const SESSION_COMMANDS: &'static [&'static str];
    fn step(event: &Event) -> Result<Reply, String>;
    /// Whether `command` is one of this add-on's interactive commands.
    fn accepts(command: &str) -> bool {
        Self::SESSION_COMMANDS.contains(&command)
    }
}

/// Largest event or reply accepted across the boundary.
pub const MAX_JSON: usize = 64 << 20;

/// Body of the exported `om9_addon_session`: JSON in, JSON out, panics caught.
pub fn session_json<T: Interactive>(input: &[u8]) -> (i32, Vec<u8>) {
    if input.len() > MAX_JSON {
        return (crate::INVOKE_BAD_INPUT, b"\"event too large\"".to_vec());
    }
    let result = std::panic::catch_unwind(|| {
        let event: Event = match serde_json::from_slice(input) {
            Ok(e) => e,
            Err(e) => return Err((crate::INVOKE_BAD_INPUT, format!("event: {e}"))),
        };
        if !T::accepts(&event.command) {
            return Err((
                crate::INVOKE_ERROR,
                format!("lệnh {} không tương tác", event.command),
            ));
        }
        T::step(&event).map_err(|e| (crate::INVOKE_ERROR, e))
    });
    match result {
        Ok(Ok(reply)) => match serde_json::to_vec(&reply) {
            Ok(v) => (crate::INVOKE_OK, v),
            Err(e) => (
                crate::INVOKE_ERROR,
                serde_json::to_vec(&format!("reply: {e}")).unwrap_or_default(),
            ),
        },
        Ok(Err((code, why))) => (code, serde_json::to_vec(&why).unwrap_or_default()),
        Err(_) => (
            crate::INVOKE_PANIC,
            serde_json::to_vec("add-on bị panic; lệnh đã huỷ").unwrap_or_default(),
        ),
    }
}

/// Export `om9_addon_session` for a type implementing [`Interactive`].
#[macro_export]
macro_rules! export_session {
    ($ty:ty) => {
        /// # Safety
        /// Only the OM9 host calls this, with a valid buffer and writer for the call.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn om9_addon_session(
            input: *const u8,
            len: usize,
            write: $crate::Om9Write,
            ctx: *mut ::std::ffi::c_void,
        ) -> i32 {
            if input.is_null() && len > 0 {
                return $crate::INVOKE_BAD_INPUT;
            }
            let bytes = if len == 0 {
                &[][..]
            } else {
                // SAFETY: the host passes `len` readable bytes for this call.
                unsafe { ::std::slice::from_raw_parts(input, len) }
            };
            let (code, out) = $crate::session::session_json::<$ty>(bytes);
            // SAFETY: the writer and its context are valid for this call.
            unsafe { write(ctx, out.as_ptr(), out.len()) };
            code
        }
    };
}

/// Export every symbol of an add-on whose commands are all interactive: the command table comes
/// from [`Interactive::SESSION_COMMANDS`] and `om9_addon_invoke` refuses (no batch commands).
#[macro_export]
macro_rules! export_interactive {
    ($ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn om9_addon_abi_version() -> u32 {
            $crate::ABI_VERSION
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn om9_addon_command_count() -> usize {
            <$ty as $crate::session::Interactive>::SESSION_COMMANDS.len()
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn om9_addon_command_name(index: usize) -> $crate::Om9Str {
            <$ty as $crate::session::Interactive>::SESSION_COMMANDS
                .get(index)
                .map(|s| $crate::Om9Str::new(s))
                .unwrap_or($crate::Om9Str::EMPTY)
        }
        /// # Safety
        /// Only the OM9 host calls this.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn om9_addon_invoke(
            _invocation: *const $crate::Om9Invocation,
        ) -> i32 {
            $crate::INVOKE_ERROR
        }
        $crate::export_session!($ty);
    };
}

// ---- small vector helpers shared by add-ons -------------------------------------------------

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
pub fn unit(a: Vec3) -> Vec3 {
    let n = norm(a);
    if n > 0.0 { scale(a, 1.0 / n) } else { a }
}
pub fn dist(a: Vec3, b: Vec3) -> f64 {
    norm(sub(a, b))
}

/// A number as the command line shows it (`1.5`, `0.25`, `12`).
pub fn fmt(v: f64) -> String {
    let t = format!("{v:.4}");
    let t = t.trim_end_matches('0').trim_end_matches('.');
    if t == "-0" { "0".into() } else { t.into() }
}

/// A typed number (`2`, `0,5`, `-3.25 mm`).
pub fn parse_number(text: &str) -> Option<f64> {
    let t = text.trim().trim_end_matches("mm").trim().replace(',', ".");
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Typed coordinates on the CPlane, Rhino/Matrix style:
/// `x,y` or `x,y,z` (absolute in CPlane coordinates), `@dx,dy[,dz]` (relative to `last`),
/// `w0,0,0` (world). Decimal commas are not accepted here because `,` separates values.
pub fn parse_point(text: &str, plane: &Plane, last: Option<Vec3>) -> Option<Vec3> {
    let t = text.trim();
    let (mode, body) = if let Some(r) = t.strip_prefix('@') {
        ('r', r)
    } else if let Some(r) = t.strip_prefix(['w', 'W']) {
        ('w', r)
    } else {
        ('a', t)
    };
    let parts: Vec<f64> = body
        .split(',')
        .map(|s| s.trim().parse::<f64>())
        .collect::<Result<_, _>>()
        .ok()?;
    if !(2..=3).contains(&parts.len()) || parts.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let (u, v, w) = (parts[0], parts[1], parts.get(2).copied().unwrap_or(0.0));
    Some(match mode {
        'w' => [u, v, w],
        'r' => {
            let base = last?;
            let n = plane.normal();
            add(
                base,
                add(scale(plane.x, u), add(scale(plane.y, v), scale(n, w))),
            )
        }
        _ => plane.at(u, v, w),
    })
}

// ---- 4×4 row-major transforms --------------------------------------------------------------

pub const IDENTITY: [f64; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

pub fn translation(d: Vec3) -> [f64; 16] {
    let mut m = IDENTITY;
    m[3] = d[0];
    m[7] = d[1];
    m[11] = d[2];
    m
}

/// Rotation by `angle` radians about the axis through `center` along `axis`.
pub fn rotation(center: Vec3, axis: Vec3, angle: f64) -> [f64; 16] {
    let [x, y, z] = unit(axis);
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    let r = [
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c],
    ];
    linear_about(center, r)
}

/// Scale about `center` by (sx, sy, sz) along the plane axes (x, y, normal).
pub fn scale_about(center: Vec3, plane: &Plane, s: Vec3) -> [f64; 16] {
    let axes = [plane.x, plane.y, plane.normal()];
    let mut r = [[0.0; 3]; 3];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| axes[k][i] * s[k] * axes[k][j]).sum();
        }
    }
    linear_about(center, r)
}

/// Mirror across the plane through `origin` with unit-ish `normal`.
pub fn mirror(origin: Vec3, normal: Vec3) -> [f64; 16] {
    let n = unit(normal);
    let mut r = [[0.0; 3]; 3];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = if i == j { 1.0 } else { 0.0 } - 2.0 * n[i] * n[j];
        }
    }
    linear_about(origin, r)
}

/// Project onto the plane (flatten along its normal).
pub fn project_to(plane: &Plane) -> [f64; 16] {
    let n = plane.normal();
    let mut r = [[0.0; 3]; 3];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = if i == j { 1.0 } else { 0.0 } - n[i] * n[j];
        }
    }
    linear_about(plane.origin, r)
}

fn linear_about(c: Vec3, r: [[f64; 3]; 3]) -> [f64; 16] {
    let mut m = IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            m[i * 4 + j] = r[i][j];
        }
        m[i * 4 + 3] = c[i] - (0..3).map(|j| r[i][j] * c[j]).sum::<f64>();
    }
    m
}

pub fn apply(m: &[f64; 16], p: Vec3) -> Vec3 {
    [
        m[0] * p[0] + m[1] * p[1] + m[2] * p[2] + m[3],
        m[4] * p[0] + m[5] * p[1] + m[6] * p[2] + m[7],
        m[8] * p[0] + m[9] * p[1] + m[10] * p[2] + m[11],
    ]
}

pub fn compose(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut m = [0.0; 16];
    for i in 0..4 {
        for j in 0..4 {
            m[i * 4 + j] = (0..4).map(|k| a[i * 4 + k] * b[k * 4 + j]).sum();
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        dist(a, b) < 1e-9
    }

    #[test]
    fn events_and_replies_round_trip_as_json() {
        let e = Event {
            command: "CurvePolylinePolyline".into(),
            input: Input::Option {
                name: "Close".into(),
                value: None,
            },
            state: serde_json::json!({"points": [[0, 0, 0]]}),
            plane: Plane::default(),
            selection: vec![],
            pick: None,
            settings: BTreeMap::new(),
        };
        let text = serde_json::to_string(&e).unwrap();
        assert!(text.contains("\"kind\":\"option\""), "{text}");
        assert_eq!(serde_json::from_str::<Event>(&text).unwrap(), e);
        let r = Reply::ask("Next point", Value::Null)
            .with_options(vec![
                OptionToken::toggle("Close", false),
                OptionToken::number("Length", None),
                OptionToken::action("Undo"),
            ])
            .preview(vec![Geo::Line {
                start: [0.0; 3],
                end: [1.0, 0.0, 0.0],
            }]);
        let text = serde_json::to_string(&r).unwrap();
        assert!(text.contains("\"type\":\"line\"") && text.contains("\"kind\":\"toggle\""));
        assert_eq!(serde_json::from_str::<Reply>(&text).unwrap(), r);
        // A minimal event (host-written, new fields omitted) still parses.
        let min: Event =
            serde_json::from_str(r#"{"command":"X","input":{"kind":"enter"}}"#).unwrap();
        assert_eq!(min.input, Input::Enter);
    }

    #[test]
    fn typed_points_follow_the_cplane() {
        let front = Plane {
            origin: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 0.0, 1.0],
        };
        assert!(close(
            parse_point("3,4", &front, None).unwrap(),
            [3.0, 0.0, 4.0]
        ));
        assert!(close(
            parse_point("@1,1", &front, Some([3.0, 0.0, 4.0])).unwrap(),
            [4.0, 0.0, 5.0]
        ));
        assert!(close(
            parse_point("w1,2,3", &front, None).unwrap(),
            [1.0, 2.0, 3.0]
        ));
        assert!(parse_point("@1,1", &front, None).is_none());
        assert!(parse_point("abc", &front, None).is_none());
        assert_eq!(parse_number(" 0,5 mm"), Some(0.5));
        assert_eq!(fmt(1.5000), "1.5");
        assert_eq!(fmt(-0.00001), "0");
    }

    #[test]
    fn transforms_match_the_geometry() {
        let p = [2.0, 0.0, 0.0];
        let r = rotation(
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            std::f64::consts::FRAC_PI_2,
        );
        assert!(close(apply(&r, p), [1.0, 1.0, 0.0]));
        let m = mirror([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(close(apply(&m, [3.0, 2.0, 1.0]), [-3.0, 2.0, 1.0]));
        let s = scale_about([1.0, 1.0, 0.0], &Plane::default(), [2.0, 3.0, 1.0]);
        assert!(close(apply(&s, [2.0, 2.0, 5.0]), [3.0, 4.0, 5.0]));
        let f = project_to(&Plane::default());
        assert!(close(apply(&f, [1.0, 2.0, 7.0]), [1.0, 2.0, 0.0]));
        let t = compose(&translation([0.0, 0.0, 1.0]), &r);
        assert!(close(apply(&t, p), [1.0, 1.0, 1.0]));
    }

    struct Echo;
    impl Interactive for Echo {
        const SESSION_COMMANDS: &'static [&'static str] = &["Echo", "Boom"];
        fn step(e: &Event) -> Result<Reply, String> {
            if e.command == "Boom" {
                panic!("boom");
            }
            Ok(Reply::ask(format!("got {:?}", e.input), e.state.clone()))
        }
    }

    #[test]
    fn the_boundary_catches_bad_input_unknown_commands_and_panics() {
        let (code, out) = session_json::<Echo>(br#"{"command":"Echo","input":{"kind":"start"}}"#);
        assert_eq!(code, crate::INVOKE_OK);
        let r: Reply = serde_json::from_slice(&out).unwrap();
        assert!(r.prompt.contains("Start"));
        assert_eq!(session_json::<Echo>(b"not json").0, crate::INVOKE_BAD_INPUT);
        assert_eq!(
            session_json::<Echo>(br#"{"command":"Other","input":{"kind":"enter"}}"#).0,
            crate::INVOKE_ERROR
        );
        let (code, out) = session_json::<Echo>(br#"{"command":"Boom","input":{"kind":"enter"}}"#);
        assert_eq!(code, crate::INVOKE_PANIC);
        assert!(String::from_utf8_lossy(&out).contains("panic"));
    }
}
