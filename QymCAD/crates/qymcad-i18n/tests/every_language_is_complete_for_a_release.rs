//! EVERY LANGUAGE OF THE CATALOGUE HOLDS EVERY KEY OF THE REFERENCE - checked for a release, not for every change.
//!
//! A string missing in a language is shown in English: `tr` falls back to the reference, so the checks that run on
//! every change pass with English alone, and a contributor is asked for no language they do not know. That leaves a
//! missing translation caught by nothing: brought in English only and not filled in after, it would reach a release
//! and a Russian or Kazakh reader would find part of the interface in English. Before a release every language is
//! whole; between releases the gaps are the maintainers' to fill (`python3 tools/i18n.py --check <code>` lists them).
//!
//! Run by `tools/gate.py release` with `QYMCAD_TIER=release`; at any other level it is passed over.

#[test]
fn every_language_holds_every_key_of_the_reference() {
    if std::env::var("QYMCAD_TIER").as_deref() != Ok("release") {
        eprintln!("PASSED OVER: the completeness of every language is checked for a release (QYMCAD_TIER=release)");
        return;
    }
    let reference = qymcad_i18n::reference_keys();
    assert!(reference.len() > 100, "suspiciously few keys in the reference: {}", reference.len());
    let mut holes: Vec<String> = Vec::new();
    for (code, _) in qymcad_i18n::available() {
        if code == qymcad_i18n::FALLBACK {
            continue;
        }
        let have: std::collections::HashSet<String> = qymcad_i18n::keys_of(&code).into_iter().collect();
        holes.extend(reference.iter().filter(|k| !have.contains(*k)).map(|k| format!("{code}: {k}")));
    }
    assert!(holes.is_empty(), "strings missing in a language, shown in English to its readers ({}):\n{}", holes.len(), holes.join("\n"));
}
