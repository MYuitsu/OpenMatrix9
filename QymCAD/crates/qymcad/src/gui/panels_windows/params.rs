//! THE TABLE OF PARAMETERS: its rows, its window, an edit of a parameter.

use super::*;

/// THE ROWS OF THE PARAMETER TABLE. They were lifted out of the window into a METHOD OF THEIR OWN, because a
/// field's width has to be checked in a REAL frame rather than against a number some function computed. The
/// first attempt at making the fields elastic was checked by a test on the function and by a test that the
/// call appeared in the source - both green, while the fields in the window stayed narrow. Now the test draws
/// THIS method into a ui of a known width and measures how much the frame grew.
///
/// Returns (whether anything was edited, what to delete).
pub(crate) fn params_rows_ui(wc: &mut qymcad_ui_state::WinCtx, ui: &mut egui::Ui) -> super::ParamRowsOut {
    let mut out = super::ParamRowsOut::default();
    let (dirty, remove) = (&mut out.dirty, &mut out.remove);
    // THE FIELD WIDTHS ARE ELASTIC. They used to be a hard 90 and 120 points: the window stretched and the
    // fields did not, so a long variable name COULD NOT BE TYPED - the text crawled under the edge and one had
    // to type blind. The width comes from the ui being drawn into and is handed to EVERY field.
    let (w_name, w_expr) = super::param_field_widths(ui.available_width());
    let mut name_w = 0.0_f32;
    // THE LIST SCROLLS. Asked for: with some 150 variables and drivers, is there vertical scrolling? There
    // was not: the table simply grew, and at a hundred and fifty parameters the window ran off the edge of the
    // screen along with the Add button. The height is now capped and the contents scroll; when there is less
    // room than the cap, whatever there is gets used.
    let max_h = ui.available_height().clamp(120.0, super::PARAM_ROWS_MAX_H);
    // WHAT WAS ASKED FOR. The edits are gathered and applied AFTER the drawing: during it the document is lent
    // to the driver list (which reads the whole project), and changing the model mid-frame is the very thing
    // being moved away from.
    enum Act {
        Rename(usize, String),
        SetExpr(usize, String),
        /// A driving dimension's value (or a feature's parameter) is edited straight from the table.
        SetDriver(usize, f64),
        DropDriver(usize),
        /// Go to whatever was named: the dimension's sketch, or the feature.
        GoTo(usize),
    }
    let mut acts: Vec<Act> = Vec::new();
    // WHERE THE PATHS WERE DRAWN, for the tests: they move the mouse over the REAL coordinates.
    let mut paths: Vec<(usize, egui::Rect)> = Vec::new();
    // WHO HOLDS A NAME IS SAID IN WORDS, UNDER THE TABLE. What was asked for: either forbid identical names,
    // or make it clear which sketch, body or assembly each of them comes from.
    let mut refusal: Option<String> = None;
    let names: Vec<String> = wc.project.parameters.iter().map(|p| p.name.clone()).collect();
    let exprs: Vec<String> = wc.project.parameters.iter().map(|p| p.expr.clone()).collect();
    // A SEARCH ABOVE THE TABLE. At a hundred and fifty names the list cannot be read by eye. It searches both
    // the name and the path: a person remembers either what they called it or where it sits.
    ui.horizontal(|ui| {
        ui.label(ph::MAGNIFYING_GLASS);
        ui.add(egui::TextEdit::singleline(&mut *wc.par_search).desired_width(160.0).hint_text(crate::i18n::tr("par-search")));
    });
    let q = wc.par_search.trim().to_lowercase();
    let hit = |name: &str, path: &str| q.is_empty() || name.to_lowercase().contains(&q) || path.to_lowercase().contains(&q);
    let scrolled = egui::ScrollArea::vertical().max_height(max_h).auto_shrink([false, true]).show(ui, |ui| {
        egui::Grid::new("params_grid").num_columns(4).spacing([8.0, 4.0]).striped(true).show(ui, |ui| {
            ui.label(egui::RichText::new(crate::i18n::tr("pp-name")).strong());
            ui.label(egui::RichText::new(crate::i18n::tr("par-expression")).strong());
            ui.label(egui::RichText::new(crate::i18n::tr("par-value")).strong());
            ui.label("");
            ui.end_row();
            for i in 0..names.len() {
                if !hit(&names[i], "") {
                    continue;
                }
                // THE NAME. It is edited in A BUFFER and goes into the document on Enter - it used to be written
                // into the model on every letter, and every formula referring to it broke on the very first one.
                let own = names[i].clone();
                let taken = |nm: &str| nm != own && wc.project.name_owner(nm).is_some();
                let ok = |nm: &str| qymcad_core::drivers::check_ident(nm).is_ok() && !taken(nm);
                let id = egui::Id::new(("par_name", i));
                let r = super::expr_field::name_field(ui, &*wc.project, id, &names[i], w_name, "w", &ok);
                name_w = name_w.max(r.resp.rect.width());
                if r.committed && r.text.trim() != names[i] {
                    acts.push(Act::Rename(i, r.text.trim().to_string()));
                }
                // THE EXPLANATION STAYS FOR AS LONG AS THE NAME IS BAD rather than flashing for one frame on
                // refusal. Measured: the first edition showed the line only in the frame where Enter was pressed -
                // that is, never. It is computed from the CURRENT text.
                let nm = r.text.trim().to_string();
                if !nm.is_empty() && !ok(&nm) {
                    refusal = Some(match wc.project.name_owner(&nm) {
                        Some(o) if nm != own => {
                            // A GLOBAL PARAMETER HAS NO PATH, and giving it a "where this dimension sits" would be
                            // a lie: it is not a dimension and it sits nowhere.
                            let where_ = if o.path.is_empty() { crate::i18n::tr("par-owner-project") } else { o.path.clone() };
                            crate::i18n::tr2("par-name-taken", "name", &nm, "where", &where_)
                        }
                        _ => crate::i18n::tr1("par-name-bad", "name", &nm),
                    });
                }

                let id = egui::Id::new(("par_expr", i));
                let r = super::expr_field::expr_field(ui, &*wc.project, id, &exprs[i], w_expr, &crate::i18n::tr("par-example"));
                if r.committed && r.text != exprs[i] {
                    acts.push(Act::SetExpr(i, r.text.clone()));
                }

                // THE VALUE IS COMPUTED FROM WHAT IS IN THE FIELD RIGHT NOW rather than from what was recorded: the
                // answer is visible while typing, and it costs the document nothing.
                match wc.project.eval_expr(&r.text) {
                    Ok(v) => {
                        ui.label(format!("{v:.3}"));
                    }
                    Err(e) => {
                        // THE REASON IN WORDS RATHER THAN AN ICON (see the history in expr_errors.rs).
                        //
                        // IN THE CELL ONLY A MARK: the value column is the narrowest of the four, and the reason
                        // does not fit it. Written into the cell as a plain label it ran off the edge of the
                        // window and was cut mid-word ("...a number or a na"); wrapped, it broke every second
                        // word onto its own line. The words themselves go below the table, across its whole
                        // width, where they can be read at a glance - see `params_window`.
                        out.errors.push((nm.clone(), crate::gui::error_words::expr_error_text(&e)));
                        ui.label(egui::RichText::new("!").color(wc.scheme.pal.error_mild()).small()).on_hover_text(crate::gui::error_words::expr_error_text(&e));
                    }
                }
                if ui.button(ph::TRASH).on_hover_text(crate::i18n::tr("par-delete")).clicked() {
                    *remove = Some(i);
                }
                ui.end_row();
            }
            // THE DRIVERS LIVE IN THE SAME TABLE, AND THEIR VALUES ARE EDITABLE THERE.
            //
            // They used to hang in a separate look-but-do-not-touch list: a name, a path, a number. This table is
            // the one place where the project's WHOLE set of numbers is both visible and editable.
            let drv: Vec<(usize, String, String, Option<f64>, bool)> = wc
                .project
                .named_dims
                .iter()
                .enumerate()
                .map(|(k, n)| {
                    let dup = wc.project.named_dims.iter().filter(|m| m.name == n.name).count() > 1;
                    (k, n.name.clone(), wc.project.driver_path(&n.target), wc.project.named_dim_value(n), dup)
                })
                .filter(|(_, nm, path, _, _)| hit(nm, path))
                .collect();
            for (k, nm, path, val, dup) in drv {
                // THE NAME AND THE PATH. The path answers "which of the identically named ones"; for namesakes it
                // also carries the warning colour - a bare name in a formula is ambiguous for them.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&nm).strong());
                    // A PATH IS A ROAD, NOT A CAPTION. A click leads to that very sketch or feature: the question
                    // "which sketch is this dimension from" gets answered by arriving there, rather than by hunting
                    // through the tree by hand.
                    let mut t = egui::RichText::new(&path).small();
                    t = if dup { t.color(ui.visuals().warn_fg_color) } else { t.weak() };
                    let r = ui.add(egui::Label::new(t).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(crate::i18n::tr(if dup {
                        "par-driver-ambiguous"
                    } else {
                        "par-driver-goto"
                    }));
                    paths.push((k, r.rect));
                    if r.clicked() {
                        acts.push(Act::GoTo(k));
                    }
                });
                match val {
                    Some(v) => {
                        let id = egui::Id::new(("drv_val", k));
                        let shown = qymcad_core::expr::fmt_num(v);
                        let r = super::expr_field::expr_field(ui, &*wc.project, id, &shown, w_expr, &crate::i18n::tr("par-example"));
                        if r.committed && r.text.trim() != shown {
                            if let Ok(nv) = wc.project.eval_expr(&r.text) {
                                acts.push(Act::SetDriver(k, nv));
                            }
                        }
                        ui.label(format!("{v:.3}"));
                    }
                    None => {
                        // THE DIMENSION IS GONE (its sketch or its constraint was deleted) - that is said out loud
                        // rather than shown as a blank: the driver is still in the formulas and has no value.
                        ui.label(egui::RichText::new(crate::i18n::tr1("par-dim-missing", "name", &nm)).weak());
                        ui.label("");
                    }
                }
                if ui.button(ph::TRASH).on_hover_text(crate::i18n::tr("par-drop-driver")).clicked() {
                    acts.push(Act::DropDriver(k));
                }
                ui.end_row();
            }

            // A FEATURE'S UNNAMED NUMBERS DO NOT GET IN HERE.
            //
            // They were all shown at first, following the convention where a parameter table also holds the
            // model's parameters. What was asked for: remove from the global parameters the empty entries where no
            // driver name was given. The parameter list is what a person HAS NAMED; a hundred foreign rows with
            // empty names turn it into a dump. There is a place to name a feature's number: the feature's
            // properties in the right panel.
        });
    });
    wc.tree.drv_path_rects = paths;
    if let Some(msg) = &refusal {
        ui.label(egui::RichText::new(msg).color(wc.scheme.pal.warning()).small());
    }
    for a in acts {
        match a {
            Act::Rename(i, nm) => {
                let old = wc.project.parameters[i].name.clone();
                // ONE OPERATION, ONE UNDO STEP, AND THE REFERENCES FOLLOW THE NAME - the name of a row just added
                // being part of adding it
                let done = if fresh_row(wc, i) {
                    let done = wc.project.rename_driver(&old, &nm);
                    qymcad_ui_state::fold_into_last_step(wc.edits, wc.project);
                    done
                } else {
                    let mut ed = qymcad_ui_state::edit_over(wc.rebuild(), crate::i18n::tr("par-rename-step"));
                    let done = ed.project().rename_driver(&old, &nm);
                    drop(ed);
                    done
                };
                if done.is_err() {
                    *wc.status = crate::i18n::tr1("par-name-bad", "name", &nm);
                }
            }
            Act::SetExpr(i, e) => {
                if fresh_row(wc, i) {
                    wc.project.parameters[i].expr = e; // the first value of a row just added is part of adding it
                    qymcad_ui_state::fold_into_last_step(wc.edits, wc.project);
                } else {
                    let mut ed = qymcad_ui_state::edit_over(wc.rebuild(), crate::i18n::tr("par-edit-step"));
                    ed.project().parameters[i].expr = e;
                    drop(ed);
                }
                *dirty = true;
            }
            Act::SetDriver(k, v) => {
                let Some(target) = wc.project.named_dims.get(k).map(|n| n.target.clone()) else { continue };
                let mut ed = qymcad_ui_state::edit_over(wc.rebuild(), crate::i18n::tr("par-edit-step"));
                let done = ed.project().set_dim_target_value(&target, v);
                // THE SKETCH IS RESOLVED EXACTLY ONCE - here, on commit - AND THE BODIES STANDING ON IT ARE
                // MARKED FOR REBUILD.
                //
                // Without that second step came exactly the trouble that was reported: a driver was changed
                // from 90 to 300, the green circle grew, and the part behind it never rebuilt. The general
                // parameter pass (`apply_param_edit`) touches only the sketches that contain EXPRESSIONS - and
                // a driving dimension is an ordinary NUMBER, so its turn never came. A silent answer: 300 in
                // the table, 90 on the screen.
                if let qymcad_core::model::DimTarget::Sketch { sketch, .. } = &target {
                    if let Some(si) = ed.project().sketch_index(*sketch) {
                        ed.project().solve_sketch(si);
                    }
                    ed.project().mark_sketch_dirty(*sketch);
                }
                drop(ed);
                if done {
                    *dirty = true;
                }
            }
            Act::GoTo(k) => {
                // NAVIGATING IS NOT EDITING THE DOCUMENT: there must be no undo step here.
                let Some(target) = wc.project.named_dims.get(k).map(|n| n.target.clone()) else { continue };
                // THE ORDER MATTERS AND IT IS ONE REQUEST, NOT TWO. Stepping into a component clears the
                // selection, so a selection made here and a step deferred to the end of the frame would
                // undo each other: the jump landed nowhere.
                match target {
                    qymcad_core::model::DimTarget::Sketch { sketch, .. } => {
                        let owner = wc.project.sketch_owner(sketch);
                        if let Some(si) = wc.project.sketch_index(sketch) {
                            wc.ask.push(qymcad_ui_state::WinAsk::GoTo { owner, sel: Sel::Sketch(si) });
                        }
                    }
                    qymcad_core::model::DimTarget::Feature { node, .. } => {
                        let owner = wc.project.timeline.iter().find(|n| n.id == node).and_then(|n| n.parent);
                        if let Some(ti) = wc.project.timeline.iter().position(|n| n.id == node) {
                            wc.ask.push(qymcad_ui_state::WinAsk::GoTo { owner, sel: Sel::Feature(ti) });
                        }
                    }
                }
            }
            Act::DropDriver(k) => {
                if k < wc.project.named_dims.len() {
                    let mut ed = qymcad_ui_state::edit_over(wc.rebuild(), crate::i18n::tr("par-edit-step"));
                    ed.project().named_dims.remove(k);
                    drop(ed);
                    *dirty = true;
                }
            }
        }
    }
    out.name_w = name_w;
    out.height = scrolled.inner_rect.height();
    out.content_h = scrolled.content_size.y;
    out
}

pub(crate) fn params_window(wc: &mut qymcad_ui_state::WinCtx, ctx: &egui::Context) {
    use qymcad_core::model::Param;
    if !wc.win.is(WinKind::Params) {
        return;
    }
    let mut open = true;
    let mut dirty = false;
    let mut remove: Option<usize> = None;
    egui::Window::new(format!("{} {}", ph::FUNCTION, crate::i18n::tr("win-params"))).id(egui::Id::new("win_params")).open(&mut open).resizable(true).default_width(360.0).show(ctx, |ui| {
        ui.label(egui::RichText::new(crate::i18n::tr("par-hint")).weak().small());
        ui.separator();
        // THE FIELD WIDTHS ARE ELASTIC. They used to be a hard 90 and 120 points: the window stretched and the
        // fields did not, so a long variable name COULD NOT BE TYPED - the text crawled under the edge and one
        // had to type blind. The fields now share the window's width, and the hard numbers became a lower bound.
        let rows = params_rows_ui(wc, ui);
        dirty |= rows.dirty;
        remove = rows.remove;
        // THE REASONS, BELOW THE TABLE AND ACROSS ITS WHOLE WIDTH. A cause that has to be hunted for by
        // hovering is a cause nobody reads, and the narrowest column of the table is no place for a sentence.
        for (name, msg) in &rows.errors {
            ui.add(egui::Label::new(egui::RichText::new(format!("{name}: {msg}")).color(wc.scheme.pal.error_mild()).small()).wrap());
        }
        ui.separator();
        if ui.button(format!("{} {}", ph::PLUS, crate::i18n::tr("win-add-param"))).clicked() {
            let mut ed = qymcad_ui_state::edit_over(wc.rebuild(), crate::i18n::tr("win-add-param"));
            ed.project().parameters.push(Param { name: String::new(), expr: String::new(), value: 0.0 });
            drop(ed);
            dirty = true;
        }
        // THE DRIVERS LIVE IN THE TABLE ITSELF (see `params_rows_ui`) rather than in a separate list below.
        //
        // That separate list was look-but-do-not-touch: a name, a path, a number, and nothing more. This table
        // is the one place where the project's WHOLE set of numbers is visible and editable; two different
        // lists with different rules have no business here.
    });
    // a step of undo like every edit of the table: a parameter deleted by mistake comes back with Ctrl+Z
    if let Some(i) = remove {
        let mut ed = qymcad_ui_state::edit_over(wc.rebuild(), crate::i18n::tr("par-delete-step"));
        ed.project().parameters.remove(i);
        drop(ed);
        dirty = true;
    }
    if dirty {
        apply_param_edit(wc);
    }
    if !open {
        wc.win.close(WinKind::Params);
    }
}

/// A ROW BEING ADDED: the last step of the history added it and it has no value yet, so its name and its first value
/// finish that step rather than making two more. Reported behaviour: one Ctrl+Z after adding "w = 40" left "w" with
/// no value - adding, naming and valuing were three steps.
pub(crate) fn fresh_row(wc: &qymcad_ui_state::WinCtx, i: usize) -> bool {
    wc.project.parameters.get(i).is_some_and(|p| p.expr.is_empty()) && wc.edits.undo.last().is_some_and(|s| s.name == crate::i18n::tr("win-add-param"))
}

/// A GLOBAL PARAMETER EDIT HAS BEEN APPLIED. A method of its own, because this path has to be TESTABLE: the
/// parameter window draws itself, and a test must pull the same handle rather than a similar one of its own.
pub(crate) fn apply_param_edit(wc: &mut qymcad_ui_state::WinCtx) {
    wc.project.eval_parameters();
    // THE REBUILD GRAPH: editing a parameter touches only the sketches that actually mention it, not every one
    // of them. The features with expressions over that name are marked by `mark_changed_params_dirty` (by
    // comparing against a snapshot of the values), so there is nothing to duplicate here.
    for si in 0..wc.project.sketches.len() {
        let uses = wc.project.sketches[si].constraints.iter().any(|c| c.expr().is_some());
        if uses {
            wc.project.solve_sketch(si);
            let sid = wc.project.sketches[si].id;
            wc.project.mark_sketch_dirty(sid);
        }
    }
    // THE SKETCHES SOLVED BY THE NEW VALUES ARE PART OF THE EDIT that changed them: solved after its step closed, they
    // were a step of their own ("Edit"), and a change of two parameters took four presses of Ctrl+Z to undo
    qymcad_ui_state::fold_into_last_step(wc.edits, wc.project);
    wc.ask.push(qymcad_ui_state::WinAsk::RegenerateAll); // the bodies rebuild associatively from the new parameters
}
