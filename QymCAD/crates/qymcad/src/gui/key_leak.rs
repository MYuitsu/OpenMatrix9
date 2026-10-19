//! A KEY OF THE CATALOGUE ON THE SCREEN INSTEAD OF A WORD, recognised by its shape in every frame the window of the
//! checks draws.
//!
//! Reported behaviour: here and there the names of labels are seen rather than their translations. A check that
//! looks only for keys the catalogue holds misses the worst case - a key the catalogue lacks is drawn as it is, and it
//! is no key of the catalogue. So a word is taken for a key by its shape: its part before the first dash is the part a
//! key of the catalogue begins with (`f-`, `feat-name-`, `sk-`), and no translation in any language holds such a word
//! (the catalogue itself writes words like `point-on-line`, and those are words).

use std::collections::HashSet;

/// The key prefixes of the catalogue, and the dashed words its translations hold.
struct Shapes {
    prefixes: HashSet<String>,
    words: HashSet<String>,
}

fn shapes() -> &'static Shapes {
    static SHAPES: std::sync::OnceLock<Shapes> = std::sync::OnceLock::new();
    SHAPES.get_or_init(|| {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n");
        let (mut prefixes, mut words) = (HashSet::new(), HashSet::new());
        for lang in std::fs::read_dir(&root).into_iter().flatten().flatten() {
            for file in std::fs::read_dir(lang.path()).into_iter().flatten().flatten() {
                if file.path().extension().is_none_or(|x| x != "ftl") {
                    continue;
                }
                for line in std::fs::read_to_string(file.path()).unwrap_or_default().lines() {
                    let Some((key, value)) = line.split_once(" = ") else { continue };
                    if let Some((head, _)) = key.split_once('-') {
                        if !key.contains(' ') {
                            prefixes.insert(head.to_string());
                        }
                    }
                    for w in tokens(value) {
                        words.insert(w);
                    }
                }
            }
        }
        Shapes { prefixes, words }
    })
}

/// The dashed lowercase words of `text`: `f-nominal-d`, `name-body#3`.
fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '#' || c == '_'))
        .map(|t| t.trim_matches('-'))
        .filter(|t| t.contains('-') && t.starts_with(|c: char| c.is_ascii_lowercase()))
        .map(str::to_string)
        .collect()
}

/// The words of `text` shaped as keys of the catalogue. A path or a file name is no key, however it is spelled: the
/// checks save documents named `sketch-line-saved.qcad`, and the window shows that name as it was given.
pub(crate) fn leaks(text: &str) -> Vec<String> {
    let s = shapes();
    let file = |w: &str| {
        w.contains('/')
            || w.contains('\\')
            || w.rsplit_once('.').is_some_and(|(_, ext)| (2..=5).contains(&ext.trim_end_matches(|c: char| !c.is_ascii_alphanumeric()).len()) && ext.starts_with(|c: char| c.is_ascii_alphanumeric()))
    };
    // a word the layout cut short ends in an ellipsis, and what was cut - an extension, the rest of a name - cannot be
    // told from a key any more: a file name elided in a tab ("sketch-con-coincident-round-trip.q...") was one
    let cut = |w: &str| w.ends_with('\u{2026}');
    text.split_whitespace().filter(|w| !file(w) && !cut(w)).flat_map(tokens).filter(|t| t.split('-').next().is_some_and(|h| s.prefixes.contains(h)) && !s.words.contains(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::leaks;

    /// A KEY IS KNOWN BY ITS SHAPE even when the catalogue lacks it - the case no other check sees - and a word of a
    /// translation, a path or a file name is not taken for one.
    #[test]
    fn a_key_is_known_by_its_shape_and_a_file_name_is_not() {
        assert_eq!(leaks("panel-joint_tool_bar"), ["panel-joint_tool_bar"], "a key the catalogue lacks was not seen");
        assert_eq!(leaks("edge 40.00 mm: m3-length 40.000 mm"), ["m3-length"], "a key inside a sentence was not seen");
        assert!(leaks("Saving: /tmp/qymcad-contract-1/menu-undo-saved.qcad…").is_empty(), "a path was taken for a key");
        assert!(leaks("mesh-gone.3mf").is_empty(), "a file name was taken for a key");
        // the layout cut a file name short where the tab ran out: the extension is gone with the rest
        assert!(leaks("sketch-con-coincident-round-trip.q…").is_empty(), "a file name cut short was taken for a key");
        assert!(leaks("sketch-circular-pattern-round-tri…").is_empty(), "a file name cut short was taken for a key");
        assert!(leaks("a point-on-edge constraint, a double-click").is_empty(), "a dashed word of a translation was taken for a key");
    }
}
