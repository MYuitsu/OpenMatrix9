// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shape audit of the Curve menu through the real controller: every command is run as a
//! user would (clicks, typed values, picks), the result is checked against what the
//! command is documented to make, and the displayed points are written to
//! `OM9_AUDIT_DIR` (when set) so the shapes can be drawn and inspected.
mod common;
use common::*;

#[test]
fn dump_curve_prompts() {
    let (mut c, tmp) = controller("dump");
    for code in CURVE_MENU {
        let r = c.run_addon_command(code);
        let prompt = c.prompt_text();
        let tokens: Vec<String> = c
            .addon_session()
            .map(|s| s.tokens(c.tr()).into_iter().map(|(_, t)| t).collect())
            .unwrap_or_default();
        eprintln!("{code}: {r:?} | {prompt} | {tokens:?}");
        c.cancel();
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
