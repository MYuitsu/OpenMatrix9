// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned normalized constructor state, distinct from original input/source history.
use super::CurveSession;
use crate::{builder_history::Json, prepared_result::{map,string,encode_owned}};
fn vector(values:&[f64])->Json{Json::Array(values.iter().map(|&n|Json::Number(n)).collect())}
fn points(values:&[[f64;3]])->Json{Json::Array(values.iter().map(|p|vector(p)).collect())}
fn optional(value:Option<f64>)->Json{value.map_or(Json::Other,Json::Number)}
fn axes(value:Option<[[f64;3];3]>)->Json{value.map_or(Json::Other,|a|points(&a))}
impl CurveSession {
    /// This value owns its bytes and retains no host owner. It describes the
    /// normalized completed session, not a GeometryResult or live-source proof.
    pub fn construction_evidence(&self,generation:u64)->Result<String,u32>{
        use crate::{geometry_contract::RepresentationKind,native_curve_capability::{Options,Stage}};
        if generation==0{return Err(5);}
        if self.active{return Err(2);}
        self.construction_capability(RepresentationKind::Point,Stage::Construct).map_err(|_|2u32)?;
        let request=self.capability_request(Stage::Construct).ok_or(2u32)?;
        let(source_refs,source_snapshots)=self.accepted_input_log.native_json();
        // Admission already enforces the input boundary. Generated constructor
        // points and display samples may extend beyond that input boundary.
        if self.points.iter().chain(&self.output).flatten().any(|n|!n.is_finite()){return Err(4);}
        let options=match &request.options{
            Options::Line{both_sides,length}=>map([("both_sides",Json::Boolean(*both_sides)),("length",optional(*length))]),
            Options::Polyline{close,length}=>map([("close",Json::Boolean(self.output.first()==self.output.last())),
                ("persistent_close",Json::Boolean(*close)),("length",optional(*length))]),
            Options::InterpCrv{degree,knots,close}=>map([("degree",Json::Number(*degree as f64)),
                ("close",Json::Boolean(self.spline.as_ref().is_some_and(|s|s.periodic)||self.output.first()==self.output.last())),
                ("persistent_close",Json::Boolean(*close)),
                ("knots",string(match knots{crate::spline::Knots::Uniform=>"Uniform",crate::spline::Knots::Chord=>"Chord",crate::spline::Knots::SqrtChord=>"SqrtChord"}))]),
            Options::Rectangle(r)=>map([("mode",string(match r.mode{crate::rectangle::Mode::Corners=>"Corners",crate::rectangle::Mode::Center=>"Center",crate::rectangle::Mode::ThreePoint=>"3Point",crate::rectangle::Mode::Vertical=>"Vertical"})),
                ("length",optional(r.length)),("width",optional(r.width)),("axes",axes(r.axes))]),
            Options::Circle(c)=>map([
                ("mode",string(match c.mode{crate::circle::Mode::Center=>"Center",crate::circle::Mode::TwoPoint=>"2Point",crate::circle::Mode::ThreePoint=>"3Point",crate::circle::Mode::Vertical=>"Vertical",crate::circle::Mode::AroundCurve=>"AroundCurve",crate::circle::Mode::FitPoints=>"FitPoints",crate::circle::Mode::Tangent=>"Tangent"})),
                ("size",string(match c.size{crate::circle::Size::Radius=>"Radius",crate::circle::Size::Diameter=>"Diameter",crate::circle::Size::Circumference=>"Circumference",crate::circle::Size::Area=>"Area"})),
                ("axes",axes(c.axes)),("normal",c.normal.map_or(Json::Other,|p|vector(&p))),
                ("orientating",Json::Boolean(c.orientating)),("oriented",Json::Boolean(c.oriented)),
                ("radius_constraint",optional(c.radius_constraint)),("radius_pending",Json::Boolean(c.radius_pending)),
                ("path_selected",Json::Boolean(c.path_selected)),("point_mode",Json::Boolean(c.point_mode)),
                ("from_first",Json::Boolean(c.from_first)),("solution",c.solution.map_or(Json::Other,|n|Json::Number(n as f64))),
                ("deformable",Json::Boolean(c.deformable)),("degree",Json::Number(c.degree as f64)),
                ("point_count",Json::Number(c.point_count as f64)),("fit_deviation",Json::Number(c.fit_deviation)),
                ("tangent_vertical",Json::Boolean(c.tangent_vertical)),("history",Json::Boolean(c.history)),
                ("constraints",Json::Array(c.constraints.iter().map(|(p,is_point)|map([("point",vector(p)),("is_point",Json::Boolean(*is_point))])).collect())),
            ]),
            Options::Ellipse(e)=>map([("mode",string(match e.mode{crate::ellipse::Mode::Center=>"Center",crate::ellipse::Mode::Diameter=>"Diameter",crate::ellipse::Mode::Corner=>"Corner",crate::ellipse::Mode::Vertical=>"Vertical",crate::ellipse::Mode::FromFoci=>"FromFoci",crate::ellipse::Mode::AroundCurve=>"AroundCurve"})),
                ("deformable",Json::Boolean(e.deformable)),("degree",Json::Number(e.degree as f64)),
                ("point_count",Json::Number(e.point_count as f64)),("mark_foci",Json::Boolean(e.mark_foci)),("axes",points(&e.axes))]),
        };
        let spline=if let Some(s)=&self.spline{
            if s.poles.len()>crate::spline::MAX_POLES||s.degree>s.poles.len()
                ||s.knots.len()>2*crate::spline::MAX_POLES+24{return Err(2);}
            let (knots,multiplicities)=s.host_knots();
            crate::spline::Basis{poles:&s.poles,weights:None,knots:&knots,multiplicities:&multiplicities,
                degree:s.degree,periodic:s.periodic,domain:[0.,1.]}.validate().map_err(|_|2u32)?;
            map([("poles",points(&s.poles)),("weights",Json::Other),("degree",Json::Number(s.degree as f64)),
                ("periodic",Json::Boolean(s.periodic)),("domain",vector(&[0.,1.])),("knots",vector(&knots)),
                ("multiplicities",Json::Array(multiplicities.iter().map(|&n|Json::Number(n as f64)).collect())),
                ("expanded_evaluation_knots",vector(&s.knots)),("basis_role",string("nonrational_native_constructor_basis"))])
        }else{Json::Other};
        let circle=self.circle_plan.as_ref().map_or(Json::Other,|p|map([("center",vector(&p.center)),("normal",vector(&p.normal)),("radius",Json::Number(p.radius))]));
        let ellipse=self.ellipse_plan.as_ref().map_or(Json::Other,|p|map([("center",vector(&p.center)),("major_direction",vector(&p.major_direction)),("normal",vector(&p.normal)),("major",Json::Number(p.major)),("minor",Json::Number(p.minor))]));
        let frame=self.frame.unwrap_or_default();
        encode_owned(&map([
            ("schema_version",Json::Number(1.)),("operation",string("ConstructCurve")),
            ("feature_id",string(request.options.feature_id())),("command",string(self.name)),
            ("generation",string(&generation.to_string())),("coordinates",string("document_world_mm")),
            ("input_mm_per_unit",Json::Number(self.input_mm_per_unit)),("frame_origin",vector(&frame.origin())),
            ("frame_axes",points(&frame.axes())),("frame_role",string("session_input_frame_at_capture")),
            ("normalized_points",points(&self.points)),("output_points",points(&self.output)),
            ("accepted_input_events",self.accepted_input_log.json()),
            ("accepted_input_role",string("entrypoint_values_before_normalization_not_complete_native_sources")),
            ("accepted_input_undo_role",string("circle_checkpoint_restores_branch_other_commands_retain_explicit_undo_events")),
            ("input_role",string("normalized_session_state_not_complete_accepted_input_history")),
            ("output_points_role",string(if matches!(self.name,"Circle"|"Ellipse"){"display_outline_only"}else{"native_constructor_points"})),
            ("options",options),("circle",circle),("ellipse",ellipse),("spline",spline),
            ("circle_approx_deviation",Json::Number(self.circle_approx_deviation)),
            ("ellipse_approx_deviation",Json::Number(self.ellipse_approx_deviation)),
            ("lifetime",string("immutable_construction_input_evidence_not_current_validity")),
            ("source_provenance",string("native_owners_and_original_inputs_not_complete_in_this_value")),
            ("source_refs",source_refs),("source_snapshots",source_snapshots),
            ("source_snapshot_indexing",string("source_snapshots_one_based")),
        ]))
    }
}
