// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pointer moves of commands whose preview does not follow the pointer.
//!
//! Many commands show a result that only their state decides — a sweep waiting for Enter, a
//! rebuilt or refitted curve, an unrolled surface, a variable fillet with its handles — yet the
//! host asks for a fresh reply at every pointer move. [`steady_hover`] answers such a move with
//! the reply of the previous move when nothing but the pointer changed: the command, its
//! state, the settings, the construction plane, the pick and the selected objects (their ids
//! and geometry) are the same. The first move after any change computes as before.
//!
//! Only commands whose code never reads the pointer of a `Hover` input may use it: for them
//! the reply to a move is a function of everything above, so the stored reply is exactly the
//! one the command would give again.

use om9_addon_abi::session::{Event, Input, Reply};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

/// The last pointer-move reply and what it was computed from.
static LAST: Mutex<Option<(u64, Reply)>> = Mutex::new(None);

/// `step` for a pointer move, unless the previous move had the same inputs (see the module).
/// Every other input runs `step` as usual.
pub fn steady_hover(
    e: &Event,
    step: impl FnOnce(&Event) -> Result<Reply, String>,
) -> Result<Reply, String> {
    if !matches!(e.input, Input::Hover { .. }) {
        return step(e);
    }
    let key = inputs_key(e);
    if let Ok(last) = LAST.lock()
        && let Some((k, reply)) = last.as_ref()
        && *k == key
    {
        return Ok(reply.clone());
    }
    let reply = step(e)?;
    if let Ok(mut last) = LAST.lock() {
        *last = Some((key, reply.clone()));
    }
    Ok(reply)
}

/// A fingerprint of everything a steady command's reply depends on (the pointer excluded).
fn inputs_key(e: &Event) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    e.command.hash(&mut h);
    e.state.to_string().hash(&mut h);
    serde_json::to_string(&e.settings).unwrap_or_default().hash(&mut h);
    serde_json::to_string(&e.plane).unwrap_or_default().hash(&mut h);
    e.pick.map(|p| p.map(f64::to_bits)).hash(&mut h);
    e.selection.len().hash(&mut h);
    for s in &e.selection {
        s.id.hash(&mut h);
        s.kind.hash(&mut h);
        s.closed.hash(&mut h);
        s.bbox.iter().flatten().for_each(|v| v.to_bits().hash(&mut h));
        s.points.iter().flatten().for_each(|v| v.to_bits().hash(&mut h));
        if let Some(m) = &s.mesh {
            m.vertices.iter().flatten().for_each(|v| v.to_bits().hash(&mut h));
            m.triangles.len().hash(&mut h);
        }
        serde_json::to_string(&s.exact).unwrap_or_default().hash(&mut h);
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::{Plane, Selected};
    use std::cell::Cell;

    fn hover(state: serde_json::Value, at: [f64; 3], points: Vec<[f64; 3]>) -> Event {
        Event {
            command: "MemoTest".into(),
            input: Input::Hover { point: at },
            state,
            plane: Plane::default(),
            selection: vec![Selected { id: "a".into(), kind: "curve".into(), points, ..Default::default() }],
            pick: None,
            settings: Default::default(),
        }
    }

    #[test]
    fn a_move_with_the_same_inputs_reuses_the_reply_and_any_change_recomputes() {
        let calls = Cell::new(0);
        let step = |e: &Event| {
            calls.set(calls.get() + 1);
            Ok(Reply::ask(format!("{}", e.state), e.state.clone()))
        };
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let a = steady_hover(&hover(serde_json::json!({"n": 1}), [0.0; 3], pts.clone()), step).unwrap();
        let b = steady_hover(&hover(serde_json::json!({"n": 1}), [5.0, 5.0, 0.0], pts.clone()), step).unwrap();
        assert_eq!(calls.get(), 1, "only the pointer moved");
        assert_eq!(a, b);
        // Another state, other geometry under the same id, another input: computed again.
        steady_hover(&hover(serde_json::json!({"n": 2}), [0.0; 3], pts.clone()), step).unwrap();
        assert_eq!(calls.get(), 2);
        steady_hover(&hover(serde_json::json!({"n": 2}), [0.0; 3], vec![[0.0; 3], [2.0, 0.0, 0.0]]), step).unwrap();
        assert_eq!(calls.get(), 3);
        let mut click = hover(serde_json::json!({"n": 2}), [0.0; 3], pts);
        click.input = Input::Point { point: [0.0; 3] };
        steady_hover(&click, step).unwrap();
        steady_hover(&click, step).unwrap();
        assert_eq!(calls.get(), 5, "clicks always run");
    }
}
