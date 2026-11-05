// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interpolate Points / `InterpCrv` (Matrix Curve #3): a smooth curve through the picks.
//!
//! - `Start of curve ( Degree Knots PersistentClose StartTangent )`
//! - `Knots` opens its own choice: `Knots <Uniform> ( Uniform Chord SqrtChord )`
//! - `StartTangent`: `Start of curve`, then `Start direction`
//! - `Next point … ( Degree Knots PersistentClose Close Sharp EndTangent Undo )`
//! - `Close` closes smoothly (periodic), `Sharp` with a kink; picking the start closes
//!   smoothly; `PersistentClose` keeps the curve closed while picking.
//! - `EndTangent`: `End of curve`, then `End direction`; the curve is created.
use crate::{AUTO_CLOSE, load, number_of, save, yes};
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Reply, Vec3, dist, norm, sub};
use om9_addon_geom::interp::{Closing, Knots, interpolate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub points: Vec<Vec3>,
    pub degree: usize,
    pub knots: String,
    pub persistent_close: bool,
    pub start_dir: Option<Vec3>,
    pub end_dir: Option<Vec3>,
    /// Sub-step: "knots", "start", "start_dir", "end", "end_dir".
    pub sub: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        State {
            points: Vec::new(),
            degree: 3,
            // Matrix 9 shows Knots=Uniform the first time InterpCrv runs.
            knots: Knots::Uniform.name().into(),
            persistent_close: false,
            start_dir: None,
            end_dir: None,
            sub: None,
        }
    }
}

impl State {
    fn knots(&self) -> Knots {
        Knots::parse(&self.knots).unwrap_or_default()
    }

    fn curve(&self, pts: &[Vec3], closing: Closing) -> Option<Geo> {
        let closing = if closing == Closing::Open && self.persistent_close {
            Closing::Smooth
        } else {
            closing
        };
        let open = closing == Closing::Open;
        interpolate(
            pts,
            self.degree,
            self.knots(),
            self.start_dir.filter(|_| open),
            self.end_dir.filter(|_| open),
            closing,
        )
    }

    fn reply(&self, hover: Option<Vec3>) -> Reply {
        let n = self.points.len();
        let mut pts = self.points.clone();
        let picking_point = matches!(self.sub.as_deref(), None | Some("start" | "end"));
        if let Some(h) = hover.filter(|_| n > 0 && picking_point) {
            pts.push(h);
        }
        let mut preview: Vec<Geo> = self.curve(&pts, Closing::Open).into_iter().collect();
        let mut opts = Vec::new();
        let prompt = match self.sub.as_deref() {
            Some("knots") => {
                for k in [Knots::Uniform, Knots::Chord, Knots::SqrtChord] {
                    opts.push(OptionToken::action(k.name()));
                }
                return Reply::ask(format!("Knots <{}>", self.knots().name()), save(self))
                    .with_options(opts);
            }
            Some("start") => "Start of curve".to_owned(),
            Some("start_dir") => "Start direction".to_owned(),
            Some("end") => "End of curve".to_owned(),
            Some("end_dir") => "End direction".to_owned(),
            _ if n == 0 => "Start of curve".to_owned(),
            _ if n == 1 => "Next point".to_owned(),
            _ => "Next point. Press Enter when done".to_owned(),
        };
        if self.sub.is_none() {
            opts.push(OptionToken::number("Degree", Some(self.degree as f64)));
            opts.push(crate::submenu("Knots", self.knots().name()));
            opts.push(OptionToken::toggle(
                "PersistentClose",
                self.persistent_close,
            ));
            if n == 0 {
                opts.push(OptionToken::action("StartTangent"));
            } else {
                if n >= 3 {
                    opts.push(OptionToken::action("Close"));
                    opts.push(OptionToken::action("Sharp"));
                }
                opts.push(OptionToken::action("EndTangent"));
                opts.push(OptionToken::action("Undo"));
            }
        }
        // Tangent picks show the direction being set.
        let rubber = match self.sub.as_deref() {
            Some("start_dir") => self.points.first().copied(),
            Some("end_dir") => self.points.last().copied(),
            _ => None,
        };
        if rubber.is_some() {
            preview.clear();
        }
        Reply::ask(prompt, save(self))
            .with_options(opts)
            .preview(preview)
            .rubber(rubber)
    }

    fn finish(&self, closing: Closing) -> Reply {
        match self.curve(&self.points, closing) {
            Some(c) if self.points.len() >= 2 => {
                Reply::finish(vec![c]).say(format!("Curve through {} points", self.points.len()))
            }
            _ => Reply::finish(vec![]).say("Curve cancelled: needs two points"),
        }
    }
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    match &e.input {
        Input::Hover { point } => st.reply(Some(*point)),
        Input::Point { point } => match st.sub.take().as_deref() {
            Some("start") => {
                st.points.push(*point);
                st.sub = Some("start_dir".into());
                st.reply(None)
            }
            Some("start_dir") => {
                let d = sub(*point, st.points[0]);
                if norm(d) < 1e-9 {
                    st.sub = Some("start_dir".into());
                    return st
                        .reply(None)
                        .refuse("Pick a direction away from the start");
                }
                st.start_dir = Some(d);
                st.reply(None)
            }
            Some("end") => {
                if st.points.last().is_none_or(|l| dist(*l, *point) > 1e-9) {
                    st.points.push(*point);
                }
                st.sub = Some("end_dir".into());
                st.reply(None)
            }
            Some("end_dir") => {
                let last = *st.points.last().expect("end point");
                let d = sub(*point, last);
                if norm(d) < 1e-9 {
                    st.sub = Some("end_dir".into());
                    return st.reply(None).refuse("Pick a direction away from the end");
                }
                st.end_dir = Some(d);
                st.finish(Closing::Open)
            }
            Some("knots") => {
                st.sub = Some("knots".into());
                st.reply(None)
            }
            _ => {
                if st.points.len() >= 3 && dist(*point, st.points[0]) <= AUTO_CLOSE {
                    return st.finish(Closing::Smooth);
                }
                if st.points.last().is_none_or(|l| dist(*l, *point) > 1e-9) {
                    st.points.push(*point);
                }
                st.reply(None)
            }
        },
        Input::Option { name, value } => {
            if st.sub.as_deref() == Some("knots") {
                if let Some(k) =
                    Knots::parse(name).or_else(|| value.as_deref().and_then(Knots::parse))
                {
                    st.knots = k.name().into();
                    st.sub = None;
                    return st.reply(None);
                }
                return st.reply(None).refuse(format!("Unknown option {name}"));
            }
            match name.as_str() {
                "Degree" => match number_of(value) {
                    Some(d) if (1.0..=11.0).contains(&d) && d.fract() == 0.0 => {
                        st.degree = d as usize;
                        st.reply(None)
                    }
                    _ => st
                        .reply(None)
                        .refuse("Degree is a whole number from 1 to 11"),
                },
                "Knots" => match value.as_deref().and_then(Knots::parse) {
                    Some(k) => {
                        st.knots = k.name().into();
                        st.reply(None)
                    }
                    None => {
                        st.sub = Some("knots".into());
                        st.reply(None)
                    }
                },
                "PersistentClose" => {
                    st.persistent_close = yes(value, st.persistent_close);
                    st.reply(None)
                }
                "StartTangent" if st.points.is_empty() => {
                    st.sub = Some("start".into());
                    st.reply(None)
                }
                "EndTangent" if !st.points.is_empty() => {
                    st.sub = Some("end".into());
                    st.reply(None)
                }
                "Close" if st.points.len() >= 3 => st.finish(Closing::Smooth),
                "Sharp" if st.points.len() >= 3 => st.finish(Closing::Sharp),
                _ => st.reply(None).refuse(format!("Unknown option {name}")),
            }
        }
        Input::Undo => {
            match st.sub.take().as_deref() {
                Some("start_dir") => {
                    st.points.clear();
                }
                Some("end_dir") => {}
                Some(_) => {}
                None => {
                    st.points.pop();
                    if st.points.is_empty() {
                        st.start_dir = None;
                    }
                }
            }
            st.reply(None)
        }
        Input::Enter => match st.sub.as_deref() {
            Some("knots" | "start" | "start_dir" | "end" | "end_dir") => {
                // Enter leaves a sub-step: keep the value, carry on picking points; at the end
                // direction it finishes without an end tangent.
                if st.sub.as_deref() == Some("end_dir") {
                    st.sub = None;
                    return st.finish(Closing::Open);
                }
                st.sub = None;
                st.reply(None)
            }
            _ => st.finish(Closing::Open),
        },
        _ => st.reply(None),
    }
}
