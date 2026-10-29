// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless input replay through the real `MatrixApp::show` widget tree.
//!
//! No window and no GPU surface: an `egui::Context` runs the same frames the application runs,
//! pointer/key events are fed as `RawInput`, and each step returns a typed receipt (view rects,
//! camera revisions, selection, prompt, undo depth, focus, panels). This is how hover, pick,
//! orbit, pan, zoom, rubber-band and keyboard focus are tested without a screenshot.
use crate::ui::MatrixApp;
use eframe::egui::{
    Context, Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect,
    TouchPhase, Vec2,
};
use om9_core::{Error, Result, model::Document};
use serde_json::{Value, json};

/// Runs `steps` and returns one receipt per step, plus the initial state.
pub fn replay(document: Document, size: [f32; 2], dpi: f32, steps: &[Value]) -> Result<Value> {
    if !(size[0] > 0. && size[1] > 0.) {
        return Err(Error::Invalid("ui replay size"));
    }
    let mut app = MatrixApp::new(document)?;
    let context = Context::default();
    context.set_pixels_per_point(dpi.clamp(0.5, 4.0));
    let extent = Vec2::new(size[0], size[1]);
    let mut time = 0.0;
    // Two settling frames: the first builds panels, the second lets focus follow them.
    frame(&mut app, &context, extent, &mut time, Vec::new());
    frame(&mut app, &context, extent, &mut time, Vec::new());
    let mut receipts = vec![receipt(&app, &context, 0)];
    for (index, step) in steps.iter().enumerate() {
        apply(&mut app, &context, extent, &mut time, step)?;
        for _ in 0..4 {
            frame(&mut app, &context, extent, &mut time, Vec::new());
        }
        receipts.push(receipt(&app, &context, index + 1));
    }
    Ok(json!({"size": size, "dpi": dpi, "steps": receipts}))
}

fn frame(app: &mut MatrixApp, context: &Context, size: Vec2, time: &mut f64, events: Vec<Event>) {
    *time += 1. / 30.;
    let raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
        events,
        time: Some(*time),
        focused: false,
        ..Default::default()
    };
    let mut output = context.run_ui(raw, |ui| app.show(ui));
    output.textures_delta.clear();
}

/// The active view follows the pointer, exactly like a user clicking in a viewport.
fn activate_at(app: &mut MatrixApp, point: Pos2) {
    if let Some((id, _)) = app.view_rects.iter().find(|(_, rect)| rect.contains(point)) {
        let _ = app.controller.activate_view(id.title());
    }
}

fn apply(
    app: &mut MatrixApp,
    context: &Context,
    size: Vec2,
    time: &mut f64,
    step: &Value,
) -> Result<()> {
    let button = match step.get("button").and_then(Value::as_str).unwrap_or("left") {
        "left" | "primary" => PointerButton::Primary,
        "right" | "secondary" => PointerButton::Secondary,
        "middle" => PointerButton::Middle,
        other => {
            return Err(Error::Io(format!(
                "ui replay: unknown mouse button `{other}`"
            )));
        }
    };
    if let Some(wait) = step.get("wait").and_then(Value::as_u64) {
        for _ in 0..wait.min(600) {
            frame(app, context, size, time, Vec::new());
        }
        return Ok(());
    }
    if let Some(point) = step.get("move").and_then(position) {
        activate_at(app, point);
        frame(app, context, size, time, vec![Event::PointerMoved(point)]);
        return Ok(());
    }
    if let Some(point) = step.get("click").and_then(position) {
        activate_at(app, point);
        frame(
            app,
            context,
            size,
            time,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        frame(
            app,
            context,
            size,
            time,
            vec![Event::PointerButton {
                pos: point,
                button,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        return Ok(());
    }
    if let Some(drag) = step.get("drag") {
        let from = drag
            .get("from")
            .and_then(position)
            .ok_or(Error::Invalid("ui replay drag from"))?;
        let to = drag
            .get("to")
            .and_then(position)
            .ok_or(Error::Invalid("ui replay drag to"))?;
        if let Some(view) = drag.get("view").and_then(Value::as_str) {
            app.controller
                .activate_view(view)
                .map_err(|error| Error::Io(error.to_string()))?;
        } else {
            activate_at(app, from);
        }
        let count = drag
            .get("steps")
            .and_then(Value::as_u64)
            .unwrap_or(6)
            .clamp(1, 120);
        frame(
            app,
            context,
            size,
            time,
            vec![
                Event::PointerMoved(from),
                Event::PointerButton {
                    pos: from,
                    button,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        for slot in 1..=count {
            let t = slot as f32 / count as f32;
            let point = Pos2::new(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t);
            frame(app, context, size, time, vec![Event::PointerMoved(point)]);
        }
        frame(
            app,
            context,
            size,
            time,
            vec![Event::PointerButton {
                pos: to,
                button,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        return Ok(());
    }
    if let Some(delta) = step.get("wheel").and_then(Value::as_f64) {
        let at = step.get("at").and_then(position);
        let mut events = Vec::new();
        if let Some(point) = at {
            activate_at(app, point);
            events.push(Event::PointerMoved(point));
        }
        events.push(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(0., delta as f32),
            modifiers: Modifiers::NONE,
            phase: TouchPhase::Move,
        });
        frame(app, context, size, time, events);
        return Ok(());
    }
    if let Some(name) = step.get("key").and_then(Value::as_str) {
        let key = match name {
            "Escape" | "Esc" => Key::Escape,
            "Enter" | "Return" => Key::Enter,
            "Tab" => Key::Tab,
            "Backspace" => Key::Backspace,
            "Delete" => Key::Delete,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            "ArrowUp" => Key::ArrowUp,
            "ArrowDown" => Key::ArrowDown,
            other => return Err(Error::Io(format!("ui replay: unknown key `{other}`"))),
        };
        frame(
            app,
            context,
            size,
            time,
            vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        frame(
            app,
            context,
            size,
            time,
            vec![Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        return Ok(());
    }
    if let Some(text) = step.get("text").and_then(Value::as_str) {
        frame(app, context, size, time, vec![Event::Text(text.to_owned())]);
        return Ok(());
    }
    Err(Error::Invalid("ui replay step"))
}

fn position(value: &Value) -> Option<Pos2> {
    let pair = value.as_array()?;
    if pair.len() != 2 {
        return None;
    }
    Some(Pos2::new(
        pair[0].as_f64()? as f32,
        pair[1].as_f64()? as f32,
    ))
}

fn receipt(app: &MatrixApp, context: &Context, step: usize) -> Value {
    let document = app.controller.document();
    let selection: Vec<Value> = app
        .controller
        .selected_ordered()
        .iter()
        .map(|id| {
            json!({
                "id": id.to_string(),
                "label": document.entities.get(id).map(|entity| entity.label.as_str()).unwrap_or(""),
            })
        })
        .collect();
    let cameras: Vec<Value> = app
        .controller
        .view_cameras()
        .into_iter()
        .map(|(title, camera, _)| json!({"view": title, "revision": camera.revision}))
        .collect();
    let views: Vec<Value> = app
        .view_rects
        .iter()
        .map(|(id, rect)| {
            json!({
                "view": id.title(),
                "rect": [rect.min.x, rect.min.y, rect.width(), rect.height()],
            })
        })
        .collect();
    let panels: Vec<Value> = app
        .dock_panel_rects
        .iter()
        .map(|(panel, rect)| {
            json!({
                "panel": panel,
                "rect": [rect.min.x, rect.min.y, rect.width(), rect.height()],
            })
        })
        .collect();
    json!({
        "step": step,
        "focus": context.memory(|memory| memory.focused()).map(|id| format!("{id:?}")),
        "active_view": app.controller.active_view_title(),
        "running_feature": app.controller.running_feature(),
        "prompt": app.controller.prompt_with_options(),
        "selection": selection,
        "undo": app.controller.coordinator().undo_len(),
        "redo": app.controller.coordinator().redo_len(),
        "objects": document.entities.len(),
        "revision": document.revision,
        "views": views,
        "cameras": cameras,
        "panels": panels,
        "navigation": app.navigation_probe(),
        "command": app.transcript.text(),
    })
}
