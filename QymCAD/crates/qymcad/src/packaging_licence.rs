//! EVERY CRATE STATES THE LICENCE OF THE TREE IN ITS OWN MANIFEST.
//!
//! The whole tree is AGPL-3.0-or-later (the root `LICENSE`, `[workspace.package] license`), but a crate only carries
//! that in its metadata when its `Cargo.toml` says `license.workspace = true`. Measured on the tree before this check: 7
//! of the 19 crates said it and 12 had no `license` line at all, so `cargo metadata` gave them `"license": null`.
//!
//! Reported behaviour (issue #99): `cargo deny check licenses` refused the 12 as unlicensed, and the Arch package built
//! from source carried a hand-written licence override for each of them.
//!
//! The line is easy to forget in a new crate and nothing else notices, so it is checked here, over the members the
//! workspace itself lists.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The members of the workspace, as the root manifest lists them in `members = [ ... ]`.
    fn members() -> Vec<String> {
        let text = std::fs::read_to_string(root().join("Cargo.toml")).expect("the root manifest reads");
        let start = text.find("members = [").expect("the root manifest lists its members");
        let list = &text[start..start + text[start..].find(']').expect("the members list closes")];
        list.split('"').skip(1).step_by(2).map(str::to_string).collect()
    }

    /// Whether the `[package]` section of a manifest inherits the licence of the workspace: `license.workspace = true`
    /// or `license = { workspace = true }`. A licence written out (`license = "MIT"`) or a `license-file` does not
    /// count: a crate of this tree has the tree's licence, and readers of the metadata look for the SPDX key.
    fn inherits_the_licence(manifest: &str) -> bool {
        let mut in_package = false;
        for line in manifest.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                in_package = t == "[package]";
                continue;
            }
            let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
            if in_package && (compact == "license.workspace=true" || compact == "license={workspace=true}") {
                return true;
            }
        }
        false
    }

    #[test]
    fn every_member_inherits_the_licence_of_the_tree() {
        let members = members();
        assert!(!members.is_empty(), "the root manifest lists no members");
        let silent: Vec<&String> = members
            .iter()
            .filter(|m| {
                let manifest = std::fs::read_to_string(root().join(m.as_str()).join("Cargo.toml")).unwrap_or_else(|e| panic!("the manifest of member {m} must be readable: {e}"));
                !inherits_the_licence(&manifest)
            })
            .collect();
        assert!(silent.is_empty(), "these crates do not state the licence of the tree; add `license.workspace = true` to their [package]: {silent:?}");
    }

    #[test]
    fn the_reader_tells_the_cases_apart() {
        assert!(inherits_the_licence("[package]\nname = \"a\"\nlicense.workspace = true\n"));
        assert!(inherits_the_licence("[package]\nname = \"a\"\nlicense = { workspace = true }\n"));
        assert!(!inherits_the_licence("[package]\nname = \"a\"\nedition = \"2021\"\n"), "no licence line");
        assert!(!inherits_the_licence("[package]\nname = \"a\"\nlicense = \"MIT\"\n"), "a licence written out is not the tree's");
        assert!(!inherits_the_licence("[package]\nname = \"a\"\n\n[dependencies]\nlicense = \"1\"\n"), "a dependency named license");
        assert!(!inherits_the_licence("[package]\nname = \"a\"\n\n[package.metadata]\nlicense.workspace = true\n"), "a key under [package.metadata]");
        assert!(!inherits_the_licence("[workspace.package]\nlicense = \"AGPL-3.0-or-later\"\n"), "the workspace's own key");
    }
}
