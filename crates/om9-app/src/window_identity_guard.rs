// SPDX-License-Identifier: LGPL-2.1-or-later
//! Guard on the IDENTITY OF WINDOWS: every `egui::Window` gets an identity of its
//! own with `.id(...)`.
//!
//! Adapted from the pinned QymCAD `window_identity_guard.rs` (746a17f1) after
//! reading it in the source audit of `OM9-L3-01-01-001`. The reason is the same:
//! `Window::new` derives its identity from the title text, so a window named only
//! by its title moves, loses its size and leaves stale entries behind when the
//! title changes (a renamed or translated title, a title that names its content).
//!
//! The rule is checked by its shape: between `egui::Window::new(` and the
//! `.show(` that ends its chain there must be an `.id(` call.
#![cfg(test)]

const OPENING: &str = "egui::Window::new(";

/// Byte offsets of every window whose chain reaches `.show(` without an `.id(`.
fn windows_without_id(text: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(found) = text[from..].find(OPENING) {
        let at = from + found;
        from = at + OPENING.len();
        let chain = text[from..]
            .find(".show(")
            .map_or(&text[from..], |end| &text[from..from + end]);
        if !chain.contains(".id(") {
            out.push(at);
        }
    }
    out
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

#[test]
fn the_signal_catches_a_window_named_only_by_its_title() {
    let with = "egui::Window::new(\"F6\").id(egui::Id::new(\"om9-command-search\")).open(&mut open).show(ctx, |ui| {});\n";
    let wrapped = "egui::Window::new(format!(\"{} {}\", prefix, name))\n    .id(egui::Id::new(\"om9-float-menu\"))\n    .show(ctx, |ui| {});\n";
    let without =
        "egui::Window::new(\"F6\").open(&mut open).show(ctx, |ui| { ui.label(\"x\"); });\n";
    assert!(
        windows_without_id(with).is_empty(),
        "a window with an identity passes"
    );
    assert!(
        windows_without_id(wrapped).is_empty(),
        "the identity is found across the lines of a wrapped chain"
    );
    assert_eq!(
        windows_without_id(without),
        vec![0],
        "a window named only by its title is caught"
    );
    let next =
        format!("{without}egui::TextEdit::singleline(&mut q).id(egui::Id::new(\"field\"));\n");
    assert_eq!(
        windows_without_id(&next),
        vec![0],
        "an identity after `.show(` belongs to something else"
    );
}

#[test]
fn every_window_of_the_application_carries_an_identity() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    let mut windows = 0usize;
    let mut stack = vec![root.clone()];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory)
            .expect("sources are readable")
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("rs")
                || path.file_name().and_then(|value| value.to_str())
                    == Some("window_identity_guard.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("file is readable");
            windows += text.matches(OPENING).count();
            let relative = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .display()
                .to_string();
            offenders.extend(
                windows_without_id(&text)
                    .into_iter()
                    .map(|at| format!("{relative}:{}", line_of(&text, at))),
            );
        }
    }
    offenders.sort();
    assert!(
        windows > 0,
        "the sweep must find the windows of the application under {}",
        root.display()
    );
    assert!(
        offenders.is_empty(),
        "a window is named only by its title, so it moves and loses its size when the title changes. \
         Give it an identity: `.id(egui::Id::new(\"om9-...\"))`. {windows} windows scanned.\n{}",
        offenders.join("\n")
    );
}
