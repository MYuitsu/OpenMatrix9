//! WHAT THE PROPERTIES SAY OF AN EDGE AND OF A CORNER taken with nothing in hand: its length (and radius, for a
//! circle or an arc) and where its ends stand; the corner's coordinates. In the body's own frame, as the face says
//! its centre.
use egui_phosphor::regular as ph;
use qymcad_core::model::Id;
use qymcad_ui_state::{props_header, NameSlot, PropsCtx};

/// The edge `id` of `body` as the model knows it, asking the kernel for the edges of the body once.
fn edge_of(pr: &mut PropsCtx, body: Id, id: u32) -> Option<qymcad_core::geom::MeshEdge> {
    qymcad_ui_state::ensure_model_edges(&mut pr.rebuild(), body);
    pr.project.regen_edges.get(&body)?.iter().find(|e| e.id == id).cloned()
}

fn xyz(p: [f64; 3]) -> String {
    format!("{}, {}, {}", crate::i18n::num(p[0], 3), crate::i18n::num(p[1], 3), crate::i18n::num(p[2], 3))
}

/// The length of an edge: a circle is 2 pi r, an arc r x its angle (the longer way round when its polyline is over
/// half a turn), anything else the length of its polyline - exact for a straight edge.
fn length_of(pr: &mut PropsCtx, body: Id, e: &qymcad_core::geom::MeshEdge) -> f64 {
    let poly_len = qymcad_pick::body_edges_cached(pr.cache, pr.live, pr.regen, body)
        .and_then(|polys| {
            polys.ids.iter().position(|i| *i == e.id).map(|k| polys.polys[k].windows(2).map(|w| (0..3).map(|i| (w[1][i] - w[0][i]) as f64).map(|d| d * d).sum::<f64>().sqrt()).sum::<f64>())
        })
        .unwrap_or(0.0);
    if e.radius <= 0.0 {
        return poly_len;
    }
    let chord = (0..3).map(|i| (e.b[i] - e.a[i]).powi(2)).sum::<f64>().sqrt();
    if chord < 1e-9 {
        return std::f64::consts::TAU * e.radius; // a whole circle
    }
    let short = 2.0 * (chord / (2.0 * e.radius)).clamp(-1.0, 1.0).asin();
    let angle = if poly_len > std::f64::consts::PI * e.radius { std::f64::consts::TAU - short } else { short };
    e.radius * angle
}

pub(crate) fn edge_props(pr: &mut PropsCtx, ui: &mut egui::Ui, body: Id, id: u32) {
    let lin = qymcad_ui_state::lineage_of(pr.project, Some(body));
    props_header(ui, ph::LINE_SEGMENT, "edge-props-title", NameSlot::None, &lin);
    let Some(e) = edge_of(pr, body, id) else { return };
    let len = length_of(pr, body, &e);
    ui.label(crate::i18n::tr1("edge-length", "v", &crate::i18n::num(len, 3)));
    if e.radius > 0.0 {
        ui.label(crate::i18n::tr1("edge-radius", "v", &crate::i18n::num(e.radius, 3)));
        ui.label(crate::i18n::tr1("edge-centre", "v", &xyz(e.center)));
    }
    ui.label(crate::i18n::tr2("edge-ends", "a", &xyz(e.a), "b", &xyz(e.b)));
}

pub(crate) fn vertex_props(pr: &mut PropsCtx, ui: &mut egui::Ui, body: Id, id: u32, far: bool) {
    let lin = qymcad_ui_state::lineage_of(pr.project, Some(body));
    props_header(ui, ph::DOT_OUTLINE, "vertex-props-title", NameSlot::None, &lin);
    let Some(e) = edge_of(pr, body, id) else { return };
    ui.label(crate::i18n::tr1("vertex-at", "v", &xyz(if far { e.b } else { e.a })));
}
