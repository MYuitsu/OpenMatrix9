//! Guard on the IDENTITY OF WINDOWS: every `egui::Window` is given one of its own with `.id(...)`.
//!
//! Without it, `Window::new` takes the identity from the title (`Id::new(title.text())`, egui 0.35
//! `containers/window.rs:103`), and egui keeps the place, size and folding of a window under that identity, in
//! memory and on disk between runs. A title is translated, so a window without an identity of its own was a
//! different window in every language: it was placed anew when the language changed, and every language used left
//! its own entries behind. A title that names what it holds (the format of a mesh export) multiplied it again.
//!
//! The rule is checked by its shape: between the opening of a window and the `.show(` that ends its chain, an `.id(`
//! call.

#[cfg(test)]
mod tests {
    const OPENING: &str = "egui::Window::new(";

    /// The byte offsets of every window of `text` whose chain reaches `.show(` without an `.id(`.
    fn windows_without_id(text: &str) -> Vec<usize> {
        let mut out = Vec::new();
        let mut from = 0usize;
        while let Some(k) = text[from..].find(OPENING) {
            let at = from + k;
            from = at + OPENING.len();
            let chain = text[from..].find(".show(").map_or(&text[from..], |end| &text[from..from + end]);
            if !chain.contains(".id(") {
                out.push(at);
            }
        }
        out
    }

    /// The line `at` falls on, counted from one.
    fn line_of(text: &str, at: usize) -> usize {
        text[..at].matches('\n').count() + 1
    }

    /// The signal itself: a chain with an identity passes, one without is caught, wherever the call is wrapped.
    #[test]
    fn the_signal_catches_a_window_named_only_by_its_title() {
        let with = "egui::Window::new(tr(\"win-a\")).id(egui::Id::new(\"win_a\")).open(&mut open).show(ctx, |ui| {});\n";
        let wrapped = "egui::Window::new(format!(\"{} {}\", ph::GEAR, tr(\"win-b\")))\n    .id(egui::Id::new(\"win_b\"))\n    .show(ctx, |ui| {});\n";
        let without = "egui::Window::new(tr(\"win-c\")).open(&mut open).show(ctx, |ui| { ui.label(\"x\"); });\n";
        assert!(windows_without_id(with).is_empty(), "a window with an identity of its own passes");
        assert!(windows_without_id(wrapped).is_empty(), "the identity is found across the lines of a wrapped chain");
        assert_eq!(windows_without_id(without), vec![0], "a window named only by its title is caught");
        // the `.id(` of a widget inside the window of a later chain does not stand for this one
        let next = format!("{without}egui::TextEdit::singleline(&mut q).id(egui::Id::new(\"field\"));\n");
        assert_eq!(windows_without_id(&next), vec![0], "an identity after `.show(` belongs to something else");
    }

    /// The sweep over every crate: no window is named only by its title.
    #[test]
    fn every_window_has_an_identity_of_its_own() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("the crates directory").to_path_buf();
        let mut offenders: Vec<String> = Vec::new();
        let mut windows = 0usize;
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).expect("sources are readable").flatten() {
                let p = e.path();
                if p.is_dir() {
                    if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                        stack.push(p);
                    }
                    continue;
                }
                // THIS FILE ITSELF holds the samples the signal is tried on, written as strings, not as code
                if p.extension().and_then(|x| x.to_str()) != Some("rs") || p.file_name().and_then(|n| n.to_str()) == Some("window_identity_guard.rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&p).expect("file is readable");
                windows += text.matches(OPENING).count();
                let rel = p.strip_prefix(&root).unwrap_or(&p).display().to_string();
                offenders.extend(windows_without_id(&text).into_iter().map(|at| format!("{rel}:{}", line_of(&text, at))));
            }
        }
        offenders.sort();
        assert!(windows > 0, "GUARD: the sweep must find the windows of the program, and it found none under {}", root.display());
        assert!(
            offenders.is_empty(),
            "a window is named only by its title, so it moves and loses its size when the language changes. \
             Give it an identity that no language changes: `.id(egui::Id::new(\"win_...\"))`. {windows} windows scanned.\n{}",
            offenders.join("\n")
        );
    }
}
