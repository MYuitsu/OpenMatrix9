// SPDX-License-Identifier: LGPL-2.1-or-later
//! Small helpers for writing session commands: state in and out, the stones and curves of the
//! selection, and the common "select objects → preview → Enter" flow.
use crate::gems::PlacedGem;
use crate::mesh::Frame;
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Reply, Selected, Vec3, Want};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

/// The command's state from the last reply (default on start or when unreadable).
pub fn load<T: DeserializeOwned + Default>(e: &Event) -> T {
    serde_json::from_value(e.state.clone()).unwrap_or_default()
}

pub fn save<T: Serialize>(t: &T) -> Value {
    serde_json::to_value(t).unwrap_or(Value::Null)
}

/// The CPlane at `p` as a frame (z = CPlane normal, x = CPlane x).
pub fn plane_frame(e: &Event, p: Vec3) -> Frame {
    Frame::new(p, e.plane.normal(), e.plane.x)
}

/// The stones of the selection, and how many selected objects are not stones.
pub fn selected_gems(e: &Event) -> (Vec<PlacedGem>, usize) {
    let mut gems = Vec::new();
    let mut other = 0;
    for s in &e.selection {
        match PlacedGem::from_selected(s, e.plane.normal()) {
            Some(g) => gems.push(g),
            None => other += 1,
        }
    }
    (gems, other)
}

/// The curves of the selection (samples in world mm).
pub fn selected_curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

/// `(n other objects ignored)` or nothing.
pub fn ignored(n: usize) -> String {
    if n > 0 {
        format!(" ({n} other objects ignored)")
    } else {
        String::new()
    }
}

/// What a selection command builds right now.
pub struct Built {
    pub create: Vec<Geo>,
    /// Shown in the prompt and the history: `12 prongs`.
    pub summary: String,
    pub preview: Vec<Geo>,
}

/// The usual flow of a command that works on the selection: ask for objects until `build`
/// has something to make, preview it, create it on Enter.
pub fn selection_command(
    e: &Event,
    label: &str,
    select_prompt: &str,
    build: impl FnOnce(&Event) -> Result<Built, String>,
) -> Reply {
    let built = match build(e) {
        Ok(b) => b,
        Err(why) => {
            let r = Reply::ask(format!("{label}: {select_prompt}"), json!({})).want(Want::Objects);
            return if matches!(e.input, Input::Enter) {
                r.refuse(why)
            } else {
                r
            };
        }
    };
    if built.create.is_empty() {
        let r = Reply::ask(format!("{label}: {select_prompt}"), json!({})).want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: nothing to create. {}", built.summary))
        } else {
            r
        };
    }
    if matches!(e.input, Input::Enter) {
        return Reply::finish(built.create).say(format!("{label}: {}", built.summary));
    }
    let preview = if built.preview.is_empty() {
        built.create
    } else {
        built.preview
    };
    Reply::ask(
        format!("{label}: {}. Adjust the options, Enter to create", built.summary),
        json!({}),
    )
    .want(Want::Objects)
    .preview(preview)
}

/// A closed polyline through world points (for outlines and guides).
pub fn closed_polyline(points: Vec<Vec3>) -> Geo {
    Geo::Polyline {
        points,
        closed: true,
    }
}

// ---- step-by-step flows --------------------------------------------------------------------

/// One thing a command asks for, in order.
#[derive(Debug, Clone)]
pub enum Need {
    /// A clicked or typed point, with its prompt.
    Point(String),
    /// A typed number (Enter takes the default), with its prompt.
    Number(String, f64),
    /// A curve picked by a click (its id and where on it), with its prompt.
    Curve(String),
    /// Enter to accept what the preview shows (options may change first), with its prompt.
    Confirm(String),
}

/// A collected answer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Answer {
    Point(Vec3),
    Number(f64),
    /// A picked curve and the parameter of the click on it (`None`: preselected).
    Curve(crate::pick::CurveRef, Option<f64>),
    /// Enter at a [`Need::Confirm`].
    Done,
}

/// What a flow has collected so far.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Collected {
    pub selected: bool,
    pub answers: Vec<Answer>,
    pub options: std::collections::BTreeMap<String, String>,
    /// A number option clicked or typed without a value: the next number sets it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<String>,
}

impl Collected {
    /// The k-th picked curve and where it was picked.
    pub fn curve(&self, k: usize) -> Option<(&crate::pick::CurveRef, Option<f64>)> {
        self.answers
            .iter()
            .filter_map(|a| match a {
                Answer::Curve(c, t) => Some((c, *t)),
                _ => None,
            })
            .nth(k)
    }
    /// A pair option's values (`Distances=1,2`); one number gives both.
    pub fn pair(&self, spec: &Flow, name: &str) -> (f64, f64) {
        pair(self.opt(spec, name)).unwrap_or((0.0, 0.0))
    }
    /// The k-th point answer.
    pub fn p(&self, k: usize) -> Option<Vec3> {
        self.answers
            .iter()
            .filter_map(|a| match a {
                Answer::Point(p) => Some(*p),
                _ => None,
            })
            .nth(k)
    }
    /// The k-th number answer.
    pub fn n(&self, k: usize) -> Option<f64> {
        self.answers
            .iter()
            .filter_map(|a| match a {
                Answer::Number(v) => Some(*v),
                _ => None,
            })
            .nth(k)
    }
    pub fn points(&self) -> Vec<Vec3> {
        (0..).map_while(|k| self.p(k)).collect()
    }
    /// An option's value (its default until changed).
    pub fn opt<'a>(&'a self, spec: &'a Flow, name: &str) -> &'a str {
        self.options
            .get(name)
            .map(String::as_str)
            .or_else(|| spec.options.iter().find(|o| o.0 == name).map(|o| o.2.as_str()))
            .unwrap_or("")
    }
    pub fn yes(&self, spec: &Flow, name: &str) -> bool {
        self.opt(spec, name).eq_ignore_ascii_case("yes")
    }
    /// A number option's value (`Radius=1.5`), its default until changed.
    pub fn num(&self, spec: &Flow, name: &str) -> f64 {
        om9_addon_abi::session::parse_number(self.opt(spec, name)).unwrap_or(0.0)
    }
}

/// `a,b` (or one number for both).
fn pair(text: &str) -> Option<(f64, f64)> {
    use om9_addon_abi::session::parse_number;
    match text.split_once(',') {
        Some((a, b)) => Some((parse_number(a.trim())?, parse_number(b.trim())?)),
        None => parse_number(text.trim()).map(|v| (v, v)),
    }
}

/// A command flow: optionally objects first, then points and numbers in order, with
/// command-line options (`(name, choices, default)`; `["Yes", "No"]` for a toggle).
pub struct Flow {
    pub label: String,
    pub select: bool,
    pub needs: Vec<Need>,
    /// `(name, choices, default)`: `["Yes", "No"]` is a toggle, no choices a number option.
    pub options: Vec<(String, Vec<String>, String)>,
    /// The selection prompt as Matrix words it (`Select curves to rebuild`).
    pub select_prompt: Option<String>,
    /// Prompts without the `Label: ` prefix, as on the Matrix command line.
    pub plain: bool,
}

/// What a flow shows while it runs.
#[derive(Default)]
pub struct Show {
    pub geo: Vec<Geo>,
    pub transforms: Vec<[f64; 16]>,
    pub rubber: Option<Vec3>,
}

impl Flow {
    pub fn new(label: &str) -> Flow {
        Flow {
            label: label.to_owned(),
            select: true,
            needs: Vec::new(),
            options: Vec::new(),
            select_prompt: None,
            plain: false,
        }
    }
    /// Matrix-style prompts: `prompt` while selecting, and no `Label: ` prefix.
    pub fn select_prompt(mut self, prompt: &str) -> Flow {
        self.select_prompt = Some(prompt.to_owned());
        self.plain = true;
        self
    }
    /// Prompts without the `Label: ` prefix.
    pub fn plain(mut self) -> Flow {
        self.plain = true;
        self
    }
    /// A number option on the command line (`Radius=1`), typed as `Radius=2`.
    pub fn num(mut self, name: &str, default: f64) -> Flow {
        self.options.push((
            name.to_owned(),
            Vec::new(),
            om9_addon_abi::session::fmt(default),
        ));
        self
    }
    /// A pair of numbers on the command line (`Distances=1,1`), typed as `Distances=2,3`.
    pub fn pair(mut self, name: &str, a: f64, b: f64) -> Flow {
        use om9_addon_abi::session::fmt;
        self.options
            .push((name.to_owned(), Vec::new(), format!("{},{}", fmt(a), fmt(b))));
        self.select = self.select && !self.needs.iter().any(|n| matches!(n, Need::Curve(_)));
        self
    }
    /// A curve picked by clicking it (Matrix: `Select first curve to fillet`). Flows that pick
    /// curves do not select objects first.
    pub fn curve(mut self, prompt: &str) -> Flow {
        self.needs.push(Need::Curve(prompt.to_owned()));
        self.select = false;
        self.plain = true;
        self
    }
    /// Wait for Enter with this prompt (the preview shows the result; options may change).
    pub fn confirm(mut self, prompt: &str) -> Flow {
        self.needs.push(Need::Confirm(prompt.to_owned()));
        self
    }
    pub fn no_selection(mut self) -> Flow {
        self.select = false;
        self
    }
    pub fn point(mut self, prompt: &str) -> Flow {
        self.needs.push(Need::Point(prompt.to_owned()));
        self
    }
    pub fn number(mut self, prompt: &str, default: f64) -> Flow {
        self.needs.push(Need::Number(prompt.to_owned(), default));
        self
    }
    pub fn toggle(mut self, name: &str, on: bool) -> Flow {
        self.options.push((
            name.to_owned(),
            vec!["Yes".into(), "No".into()],
            if on { "Yes" } else { "No" }.into(),
        ));
        self
    }
    pub fn choice(mut self, name: &str, choices: &[&str], default: &str) -> Flow {
        self.options.push((
            name.to_owned(),
            choices.iter().map(|c| c.to_string()).collect(),
            default.to_owned(),
        ));
        self
    }

    /// The command-line options with their current values.
    pub fn tokens(&self, c: &Collected) -> Vec<OptionToken> {
        self.options
            .iter()
            .map(|o| {
                let v = c.opt(self, &o.0);
                if o.1.is_empty() && v.contains(',') {
                    OptionToken {
                        name: o.0.clone(),
                        value: Some(v.to_owned()),
                        kind: om9_addon_abi::session::OptionKind::Number,
                    }
                } else if o.1.is_empty() {
                    OptionToken::number(&o.0, om9_addon_abi::session::parse_number(v))
                } else if o.1.len() == 2 && o.1[0] == "Yes" {
                    OptionToken::toggle(&o.0, v.eq_ignore_ascii_case("yes"))
                } else {
                    let choices: Vec<&str> = o.1.iter().map(String::as_str).collect();
                    OptionToken::choice(&o.0, v, &choices)
                }
            })
            .collect()
    }

    /// Run one step. `show` previews with the hover point as the next point; `finish`
    /// makes the result once every need is answered.
    pub fn step(
        &self,
        e: &Event,
        show: &dyn Fn(&Collected, Option<Vec3>) -> Show,
        finish: &dyn Fn(&Collected) -> Result<Reply, String>,
    ) -> Reply {
        let mut c: Collected = load(e);
        let mut error = None;
        if self.select && !c.selected {
            if e.selection.is_empty() {
                let prompt = match &self.select_prompt {
                    Some(p) => p.clone(),
                    None => format!("{}: select objects, then press Enter", self.label),
                };
                let r = Reply::ask(prompt, save(&c))
                    .with_options(self.tokens(&c))
                    .want(Want::Objects);
                return if matches!(e.input, Input::Enter) {
                    r.refuse(format!("{}: nothing is selected", self.label))
                } else {
                    r
                };
            }
            c.selected = true;
            if matches!(e.input, Input::Start | Input::Selection | Input::Enter) {
                return self.ask(e, c, None, None, false, show, finish);
            }
        }
        let next = self.needs.get(c.answers.len());
        let mut hover = None;
        // A number option waiting for its value (`Radius <1>`): the next number sets it.
        if let Some(name) = c.pending.clone() {
            match &e.input {
                Input::Text { text } => {
                    match self.set_number(&mut c, &name, text) {
                        Ok(()) => c.pending = None,
                        Err(why) => error = Some(why),
                    }
                    return self.ask(e, c, None, error, false, show, finish);
                }
                Input::Enter => {
                    c.pending = None;
                    return self.ask(e, c, None, None, false, show, finish);
                }
                Input::Option { .. } => c.pending = None,
                _ => return self.ask(e, c, None, None, false, show, finish),
            }
        }
        // A curve to pick: the curve just selected (preselected ones at the start).
        if let Some(Need::Curve(_)) = next
            && matches!(e.input, Input::Start | Input::Selection | Input::Point { .. } | Input::Enter)
        {
            let taken: Vec<&str> = (0..)
                .map_while(|k| c.curve(k))
                .map(|(cr, _)| cr.id.as_str())
                .collect();
            if let Some((cr, t)) = crate::pick::CurveRef::picked(e, &taken) {
                let t = if matches!(e.input, Input::Start) { None } else { t };
                c.answers.push(Answer::Curve(cr, t));
                // Preselection may answer several curve needs at once.
                if matches!(e.input, Input::Start) {
                    while let Some(Need::Curve(_)) = self.needs.get(c.answers.len()) {
                        let taken: Vec<String> = (0..)
                            .map_while(|k| c.curve(k))
                            .map(|(cr, _)| cr.id.clone())
                            .collect();
                        let taken: Vec<&str> = taken.iter().map(String::as_str).collect();
                        match crate::pick::CurveRef::picked(e, &taken) {
                            Some((cr, _)) => c.answers.push(Answer::Curve(cr, None)),
                            None => break,
                        }
                    }
                }
                return self.ask(e, c, None, None, false, show, finish);
            }
            if matches!(e.input, Input::Enter) {
                let why = format!("{}: nothing is selected", self.label);
                return self.ask(e, c, None, Some(why), false, show, finish);
            }
        }
        match &e.input {
            Input::Point { point } => match next {
                Some(Need::Point(_)) => c.answers.push(Answer::Point(*point)),
                Some(Need::Number(..)) => {
                    // A click for a number: the distance from the last point.
                    match c.points().last() {
                        Some(&last) => c.answers.push(Answer::Number(om9_addon_abi::session::dist(last, *point))),
                        None => error = Some("type a number".to_owned()),
                    }
                }
                Some(Need::Curve(_) | Need::Confirm(_)) | None => {}
            },
            Input::Hover { point } => hover = Some(*point),
            Input::Undo => {
                c.answers.pop();
            }
            Input::Text { text } => match next {
                Some(Need::Point(_)) => {
                    let number_option = self.options.iter().find(|o| o.1.is_empty()).map(|o| o.0.clone());
                    match om9_addon_abi::session::parse_point(text, &e.plane, c.points().last().copied()) {
                        Some(p) => c.answers.push(Answer::Point(p)),
                        // A plain number sets the first number option (Rhino: `2` at `Side
                        // to offset ( Distance=1 … )` makes Distance=2).
                        None => match (number_option, om9_addon_abi::session::parse_number(text)) {
                            (Some(name), Some(_)) => {
                                if let Err(why) = self.set_number(&mut c, &name, text) {
                                    error = Some(why);
                                }
                            }
                            _ => error = Some(format!("{text}: type a point (x,y or @dx,dy)")),
                        },
                    }
                }
                Some(Need::Number(..)) => match om9_addon_abi::session::parse_number(text) {
                    Some(v) => c.answers.push(Answer::Number(v)),
                    None => error = Some(format!("{text}: type a number")),
                },
                // A number typed while picking sets the first number option (Rhino: typing 2
                // at `Select curve to offset ( Distance=1 … )` makes Distance=2).
                Some(Need::Curve(_)) => match self.options.iter().find(|o| o.1.is_empty()) {
                    Some(o) => {
                        if let Err(why) = self.set_number(&mut c, &o.0.clone(), text) {
                            error = Some(why);
                        }
                    }
                    None => error = Some(format!("{text}: select a curve")),
                },
                Some(Need::Confirm(_)) => error = Some(format!("{text}: press Enter or pick an option")),
                None => {}
            },
            Input::Option { name, value } => {
                if name.eq_ignore_ascii_case("Undo") {
                    c.answers.pop();
                } else if let Some(o) = self
                    .options
                    .iter()
                    .find(|o| o.0.eq_ignore_ascii_case(name) && o.1.is_empty())
                {
                    match value.as_deref() {
                        Some(v) => {
                            if let Err(why) = self.set_number(&mut c, &o.0.clone(), v) {
                                error = Some(why);
                            }
                        }
                        None => c.pending = Some(o.0.clone()),
                    }
                } else if let Some(o) = self.options.iter().find(|o| o.0.eq_ignore_ascii_case(name)) {
                    let current = c.opt(self, &o.0).to_owned();
                    let v = match value {
                        Some(v) => o
                            .1
                            .iter()
                            .find(|x| x.eq_ignore_ascii_case(v))
                            .cloned()
                            .or_else(|| {
                                error = Some(format!("{}: one of {}", o.0, o.1.join(", ")));
                                None
                            }),
                        None => {
                            let i = o.1.iter().position(|x| *x == current).unwrap_or(0);
                            Some(o.1[(i + 1) % o.1.len()].clone())
                        }
                    };
                    if let Some(v) = v {
                        c.options.insert(o.0.clone(), v);
                    }
                } else {
                    error = Some(format!("{name}: unknown option"));
                }
            }
            Input::Enter => match next {
                Some(Need::Number(_, d)) => c.answers.push(Answer::Number(*d)),
                Some(Need::Confirm(_)) => c.answers.push(Answer::Done),
                Some(Need::Point(p) | Need::Curve(p)) => error = Some(format!("{}: {}", self.label, p)),
                None => {}
            },
            _ => {}
        }
        let entered = matches!(e.input, Input::Enter);
        self.ask(e, c, hover, error, entered, show, finish)
    }

    /// Set a number (or pair) option from typed text.
    fn set_number(&self, c: &mut Collected, name: &str, text: &str) -> Result<(), String> {
        use om9_addon_abi::session::{fmt, parse_number};
        let is_pair = self
            .options
            .iter()
            .find(|o| o.0 == name)
            .is_some_and(|o| o.2.contains(','));
        let value = if is_pair {
            pair(text).map(|(a, b)| format!("{},{}", fmt(a), fmt(b)))
        } else {
            parse_number(text).map(fmt)
        };
        match value {
            Some(v) => {
                c.options.insert(name.to_owned(), v);
                Ok(())
            }
            None if is_pair => Err(format!("{name}: type {name}=a,b")),
            None => Err(format!("{name}: type {name}=number")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn ask(
        &self,
        e: &Event,
        c: Collected,
        hover: Option<Vec3>,
        error: Option<String>,
        entered: bool,
        show: &dyn Fn(&Collected, Option<Vec3>) -> Show,
        finish: &dyn Fn(&Collected) -> Result<Reply, String>,
    ) -> Reply {
        // With nothing to ask, Enter finishes; otherwise the last answer does.
        let complete = c.answers.len() >= self.needs.len() && (!self.needs.is_empty() || entered);
        if complete && error.is_none() {
            match finish(&c) {
                Ok(r) => return r,
                Err(why) => {
                    let mut c = c;
                    if !self.needs.is_empty() {
                        c.answers.pop();
                    }
                    return self.ask(e, c, None, Some(why), false, show, finish);
                }
            }
        }
        if let Some(name) = &c.pending {
            let r = Reply::ask(format!("{name} <{}>", c.opt(self, name)), save(&c)).want(Want::Number);
            return match error {
                Some(why) => r.refuse(why),
                None => r,
            };
        }
        let (prompt, want) = match self.needs.get(c.answers.len().min(self.needs.len().saturating_sub(1))) {
            Some(Need::Point(p)) => (p.clone(), Want::Point),
            Some(Need::Curve(p)) => (p.clone(), Want::Objects),
            Some(Need::Confirm(p)) => (p.clone(), Want::Nothing),
            Some(Need::Number(p, d)) => (format!("{p} <{}>", om9_addon_abi::session::fmt(*d)), Want::Number),
            None => match (&self.select_prompt, self.plain) {
                (Some(p), _) => (format!("{p}. Press Enter when done"), Want::Objects),
                (None, true) => ("Press Enter when done".to_owned(), Want::Objects),
                (None, false) => ("press Enter".to_owned(), Want::Objects),
            },
        };
        let mut options = self.tokens(&c);
        if !c.answers.is_empty() {
            options.push(OptionToken::action("Undo"));
        }
        let shown = show(&c, hover);
        let text = if self.plain {
            prompt
        } else {
            format!("{}: {prompt}", self.label)
        };
        let mut r = Reply::ask(text, save(&c))
            .with_options(options)
            .want(want)
            .preview(shown.geo)
            .rubber(shown.rubber.or_else(|| c.points().last().copied()));
        r.preview_transforms = shown.transforms;
        match error {
            Some(why) => r.refuse(why),
            None => r,
        }
    }
}

/// `geo` with every point moved by `f`: curves (control points, or samples of circles and
/// lines refined so bends follow), surfaces (control points) and meshes with their data.
pub fn deformed(sel: &Selected, f: &dyn Fn(Vec3) -> Vec3) -> Option<Geo> {
    use om9_addon_abi::session::{add as v_add, scale as v_scale, sub as v_sub};
    let line_pts = |a: Vec3, b: Vec3| -> Vec<Vec3> {
        (0..=16)
            .map(|i| v_add(a, v_scale(v_sub(b, a), i as f64 / 16.0)))
            .map(f)
            .collect()
    };
    match &sel.exact {
        Some(Geo::Line { start, end }) => crate::interpolate(&line_pts(*start, *end), 3, false),
        Some(Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        }) => {
            // Degree-1 curves (polylines) keep their corners: refine each span.
            if *degree == 1 && points.len() < 400 {
                let mut pts = Vec::new();
                for w in points.windows(2) {
                    let seg = line_pts(w[0], w[1]);
                    pts.extend_from_slice(&seg[..seg.len() - 1]);
                }
                pts.push(f(*points.last()?));
                Some(Geo::Nurbs {
                    degree: 1,
                    knots: crate::clamped_uniform(pts.len(), 1),
                    points: pts,
                    weights: Vec::new(),
                })
            } else {
                Some(Geo::Nurbs {
                    degree: *degree,
                    points: points.iter().map(|&p| f(p)).collect(),
                    weights: weights.clone(),
                    knots: knots.clone(),
                })
            }
        }
        Some(g @ (Geo::Circle { .. } | Geo::Arc { .. })) => {
            let closed = matches!(g, Geo::Circle { .. });
            let n = 64;
            let pts: Vec<Vec3> = (0..if closed { n } else { n + 1 })
                .filter_map(|i| crate::eval_geo(g, i as f64 / n as f64))
                .map(f)
                .collect();
            crate::interpolate(&pts, 3, closed)
        }
        Some(Geo::Surface {
            degree,
            size,
            points,
            weights,
            knots,
        }) => Some(Geo::Surface {
            degree: *degree,
            size: *size,
            points: points.iter().map(|&p| f(p)).collect(),
            weights: weights.clone(),
            knots: knots.clone(),
        }),
        _ => sel.mesh.as_ref().map(|m| Geo::Mesh {
            name: sel.name.clone(),
            vertices: m.vertices.iter().map(|&p| f(p)).collect(),
            triangles: m.triangles.clone(),
            meta: None,
            layer: None,
        }),
    }
}

/// The geometry of selected objects under `f`, as replacements (objects that cannot be
/// deformed are counted).
pub fn deform_selection(
    e: &Event,
    f: &dyn Fn(Vec3) -> Vec3,
) -> (Vec<om9_addon_abi::session::Replacement>, usize) {
    let mut out = Vec::new();
    let mut skipped = 0;
    for s in &e.selection {
        match deformed(s, f) {
            Some(geo) => out.push(om9_addon_abi::session::Replacement {
                id: s.id.clone(),
                geo,
            }),
            None => skipped += 1,
        }
    }
    (out, skipped)
}

// ---- styles ----------------------------------------------------------------------------------

/// Matrix "styles": a stone remembers the builder values of the settings and cutters made on
/// it, under `styles.<command>` in its gem record, so Match Attributes and Save / Load Style
/// can make the same setting on other stones.
pub fn remember_style(e: &Event) -> Vec<om9_addon_abi::session::RecordUpdate> {
    let style: serde_json::Map<String, Value> =
        e.settings.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    e.selection
        .iter()
        .filter_map(|s| {
            let mut record = s.gem.clone()?;
            if !record.is_object() {
                return None;
            }
            if !record.get("styles").is_some_and(Value::is_object) {
                record["styles"] = json!({});
            }
            record["styles"][e.command.as_str()] = Value::Object(style.clone());
            Some(om9_addon_abi::session::RecordUpdate { id: s.id.clone(), record })
        })
        .collect()
}

/// The styles a stone remembers: command code → builder values.
pub fn styles_of(s: &Selected) -> std::collections::BTreeMap<String, Value> {
    s.gem
        .as_ref()
        .and_then(|g| g.get("styles"))
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}
