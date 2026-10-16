//! WHERE THE PROGRAM KEEPS ITS OWN FILES.
//!
//! THE APPLICATION ID IS ONE STRING, and this is it. The same name identifies the program to the desktop,
//! to macOS and to Flathub. It was written out in six places in the code, plus the macOS bundle, plus the
//! Flatpak manifest - eight copies of one decision, and three of them had already drifted apart:
//! `tech.qymis.qym-cad` in the code, `tech.qymis.qymcad` on macOS, `tech.qymis.cad` on Flathub.
//!
//! THE FOLDER IS A SECOND DECISION, next to it and not derived from it - see `FOLDER` below for what
//! deriving it cost.
//!
//! CHANGING EITHER LOSES WHAT PEOPLE HAVE. The old directory does not move by itself: settings, schemes,
//! templates and the parts library stay behind under the old name and simply vanish from view. That is the
//! price, it was paid deliberately while the audience is two dev releases small, and it is the reason both
//! names live here rather than in eight files.

/// The reverse-DNS name of the program: `cad.qymis.tech` backwards, the project's own site.
pub const APP_ID: &str = "tech.qymis.cad";

/// THE NAME OF THE FOLDER, and it is NOT the last part of the id.
///
/// Reported behaviour: settings and crash reports turned up in `~/.local/share/cad`.
///
/// `ProjectDirs` is given three parts and on Linux uses ONLY the third. `tech.qymis.cad` split at the
/// dots ends with `cad`, so the program claimed a folder named after a whole field of software - a name
/// any other CAD may take tomorrow - and stopped finding what it had already written under its own.
///
/// These are two decisions, not one. The id says who the program is to the desktop, to macOS and to
/// Flathub; the folder says where a person's settings, schemes, templates and library of parts live.
/// Deriving the second from the first is exactly what put a stranger's name on the folder.
const FOLDER: &str = "qymcad";

/// The two parts above the name, taken from `APP_ID` so the id and the folder stay in one file.
fn parts() -> Option<(&'static str, &'static str)> {
    let mut it = APP_ID.split('.');
    Some((it.next()?, it.next()?))
}

/// The program's own directories, or `None` where the system has no notion of them.
pub fn dirs() -> Option<directories::ProjectDirs> {
    let (tld, org) = parts()?;
    directories::ProjectDirs::from(tld, org, FOLDER)
}

/// A ROOT THAT STANDS IN FOR THE PERSON'S DIRECTORIES, for the whole process once set.
static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// KEEP EVERYTHING THE PROGRAM WRITES UNDER `root`, for the rest of this process: settings, schemes,
/// templates, the parts library, crash and problem reports.
///
/// A program driven from outside - a run of acceptance checks - is still the whole program, and a whole
/// program writes where a person keeps their files. One such run on a working machine replaced a person's
/// settings and filled their crash folder. Set once: a second root would split one process between two
/// homes, so a different root after the first is refused and the answer is `false`.
pub fn keep_under(root: std::path::PathBuf) -> bool {
    ROOT.get_or_init(|| root.clone()) == &root
}

/// THE ONE FOLDER A PERSON'S FILES ARE IN - settings, schemes, templates, parts, reports: under the root when one was
/// given, the system's data folder of the program otherwise (`~/.local/share/qymcad`, `%APPDATA%\qymis\qymcad\data`,
/// `~/Library/Application Support/tech.qymis.qymcad`). One folder on every system, not a "config" beside a "data".
/// Reported behaviour: half a person's files were in `~/.config/qymcad` and half in `~/.local/share/qymcad`.
pub fn data_root() -> Option<std::path::PathBuf> {
    if let Some(root) = ROOT.get() {
        return Some(root.clone());
    }
    dirs().map(|d| d.data_dir().to_path_buf())
}

/// THE FILE THE SETTINGS ARE KEPT IN.
///
/// Named here, with the rest of the program's places, because the framework picks one of its own otherwise -
/// derived from the application id, which is a different string. On Linux both happen to land in the same
/// folder and nothing looks wrong; on Windows they came out as `AppData\Roaming\qymcad\data` against
/// `AppData\Roaming\qymis\qymcad\data`, and on macOS as `qymcad` beside `tech.qymis.qymcad` - a person's
/// settings in one place and their schemes, templates, parts and crash reports in another.
pub fn settings_file() -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join("app.ron"))
}

/// A directory of the person's own things that set how the program looks and starts, e.g. `schemes` or `templates` -
/// in the same one folder as the rest.
pub fn config(sub: &str) -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join(sub))
}

/// A directory under the person's data, e.g. `crashes` or `library/parts`.
pub fn data(sub: &str) -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join(sub))
}

#[cfg(test)]
mod tests {
    /// THE FOLDER IS NAMED AFTER THE PROGRAM, and this is asked of the finished path, not of the strings.
    ///
    /// The check that stood here compared the three parts against the id and was green while the program
    /// was writing into `~/.local/share/cad`: it asked whether the folder was DERIVED from the id, which
    /// was true, instead of what the folder came out as. So this one builds the path and reads it.
    #[test]
    fn the_folder_is_named_after_the_program() {
        let Some(d) = super::dirs() else {
            return; // a system without a notion of per-user directories has nothing to check
        };
        for dir in [d.config_dir(), d.data_dir()] {
            assert!(dir.to_string_lossy().to_lowercase().contains(super::FOLDER), "the program's own directory does not carry the program's name: {}", dir.display());
            assert!(!dir.components().any(|c| c.as_os_str() == "cad"), "the folder is named after the last part of the id again, and any other CAD may claim it: {}", dir.display());
        }
    }
}
