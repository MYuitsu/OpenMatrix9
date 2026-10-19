//! THE AUR PACKAGE POINTS AT A RELEASE THAT EXISTS, AND SAYS SO IN THREE PLACES THAT AGREE.
//!
//! The package is built from the published AppImage, and three things have to match: the version pacman
//! sees (`pkgver`), the name the file really has on the release page (`_relver`), and the checksum of that
//! file. They live on three different lines and are edited by hand unless something stops that.
//!
//! WHAT GOES WRONG WITHOUT THIS. A hyphen inside `pkgver` is not a syntax error - pacman reads everything
//! after it as `pkgrel`, so `0.1.0-dev.20260828` silently becomes version `0.1.0` release `dev.20260828`,
//! and every later build compares as older. A stale checksum is worse: it fails at install time, on
//! somebody else's machine, with a message about a corrupted download.
//!
//! WHAT IS NOT CHECKED HERE. Whether the checksum is the RIGHT one for that release - that would mean
//! reaching the network from a test. `packaging/aur/update-pkgbuild.sh` takes it from the release itself,
//! and this checks that the file was written by that road rather than by hand.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn pkgbuild() -> String {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/aur/PKGBUILD");
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("the AUR package description must be readable: {e}"))
    }

    /// The value of a top-level assignment, unquoted.
    fn field(src: &str, name: &str) -> String {
        let line = src.lines().find(|l| l.starts_with(&format!("{name}="))).unwrap_or_else(|| panic!("PKGBUILD has no {name}"));
        line[name.len() + 1..].trim().trim_matches(|c| c == '\'' || c == '"' || c == '(' || c == ')').to_string()
    }

    /// THE VERSION PACMAN SEES CARRIES NO HYPHEN, and it is the release version with the hyphens replaced.
    #[test]
    fn the_two_versions_are_the_same_version() {
        let src = pkgbuild();
        let (pkgver, relver) = (field(&src, "pkgver"), field(&src, "_relver"));
        assert!(!pkgver.contains('-'), "pkgver is \"{pkgver}\": a hyphen there is read as the boundary before pkgrel, so this version would compare as older than itself");
        assert_eq!(pkgver, relver.replace('-', "."), "pkgver \"{pkgver}\" and the release name \"{relver}\" are two different versions");
    }

    /// THE CHECKSUM IS A REAL ONE.
    ///
    /// `SKIP` turns the check off, which for a downloaded binary means installing whatever arrives.
    #[test]
    fn the_checksum_is_not_switched_off() {
        let sum = field(&pkgbuild(), "sha256sums");
        assert_ne!(sum, "SKIP", "the checksum is switched off: the package would install whatever the download turned out to be");
        assert_eq!(sum.len(), 64, "\"{sum}\" is not a sha256 - 64 hexadecimal characters");
        assert!(sum.chars().all(|c| c.is_ascii_hexdigit()), "\"{sum}\" is not hexadecimal");
    }

    /// THE FILE IT ASKS FOR IS THE FILE THE BUILD PRODUCES.
    ///
    /// The name is decided in `packaging/linux/build-appimage.sh` and repeated here; the two are in
    /// different languages and cannot share a constant, so they are compared instead.
    #[test]
    fn the_name_it_downloads_is_the_name_the_build_writes() {
        let src = pkgbuild();
        let sh = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/linux/build-appimage.sh")).expect("the AppImage build script reads");
        assert!(sh.contains("-x86_64.AppImage"), "the build no longer writes a file ending in -x86_64.AppImage, and the AUR package still asks for one");
        // The package writes the architecture as `$CARCH` - namcap asks for that, and with arch=('x86_64')
        // it is the same word. The two spellings are both accepted here; what must not change is the shape
        // of the name and the fact that the package is declared for the architecture the build writes.
        assert!(src.contains("qymcad-${_relver}-${CARCH}.AppImage") || src.contains("qymcad-${_relver}-x86_64.AppImage"), "the source line does not name the file the build writes");
        assert!(field(&src, "arch").contains("x86_64"), "the build writes an x86_64 file and the package is not declared for x86_64");
        assert!(!field(&src, "_relver").is_empty(), "the release version is empty, so the source line points at nothing");
    }

    /// THE PACKAGE UNPACKS THE APPIMAGE INSTEAD OF INSTALLING IT WHOLE.
    ///
    /// Installed whole, an AppImage needs FUSE to mount itself at every start - one more moving part
    /// between a person and the program, and one that is missing on plenty of machines. Unpacked, the
    /// desktop entry and the icons also land where the desktop environment looks for them, which is what
    /// the package is for.
    #[test]
    fn the_package_unpacks_and_lays_the_files_out() {
        let src = pkgbuild();
        assert!(src.contains("--appimage-extract"), "the package does not unpack the AppImage, so it would need FUSE at every start");
        for what in ["usr/share/applications/qymcad.desktop", "usr/share/icons/hicolor", "usr/share/licenses"] {
            assert!(src.contains(what), "the package does not lay out {what}");
        }
    }

    /// THE INSTALLED FILES CAN BE READ BY EVERYBODY, not by root alone.
    ///
    /// Reported behaviour: after installing the package, `qymcad` said "permission denied", and so did `cd
    /// /opt/qymcad/usr`. The AppImage holds its directories as 755, but `--appimage-extract` writes every
    /// directory it makes as 0700, and `cp -a` carried that into /opt, where pacman installs as root - a
    /// program only root could reach. The package sets the modes of what it copies itself, after the copy.
    #[test]
    fn the_installed_files_can_be_read_by_everybody() {
        let src = pkgbuild();
        let copy = src.find("cp -a usr \"$pkgdir/opt/qymcad/\"").expect("the package copies the AppImage's usr into /opt/qymcad");
        let modes = src
            .find("chmod -R u=rwX,go=rX \"$pkgdir/opt/qymcad\"")
            .unwrap_or_else(|| panic!("the package keeps the modes the unpacking left - its directories are 0700, so only root could open them"));
        assert!(modes > copy, "the modes are set before the copy that brings the unpacked 0700 directories in");
    }

    /// AN UPGRADE MENDS THE MODES THE FIRST RELEASE LEFT.
    ///
    /// Reported behaviour: upgrading from the release that installed /opt/qymcad as 0700 to the one that packs it as
    /// 755 still answered "permission denied" - pacman does not change the modes of a directory already on disk, it
    /// only warns "directory permissions differ". Only removing the package and installing it again helped. The
    /// package carries an install script whose `post_upgrade` sets the modes itself.
    #[test]
    fn an_upgrade_mends_the_modes_the_first_release_left() {
        let src = pkgbuild();
        assert_eq!(field(&src, "install"), "qymcad-bin.install", "the package carries no install script, so an upgrade keeps the 0700 directories already on disk");
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/aur/qymcad-bin.install");
        let script = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("the install script the PKGBUILD names must be beside it: {e}"));
        let upgrade = script.split("post_upgrade()").nth(1).unwrap_or_else(|| panic!("the install script has no post_upgrade"));
        assert!(upgrade.contains("chmod -R u=rwX,go=rX /opt/qymcad"), "post_upgrade does not set the modes of /opt/qymcad");
    }

    /// AND .SRCINFO SAYS THE SAME AS THE PKGBUILD BESIDE IT.
    ///
    /// The AUR reads `.SRCINFO`, not the PKGBUILD: it is what the site shows and what dependency resolvers
    /// go by. It is generated, never edited - and generated files rot exactly when somebody edits the other
    /// half and forgets. The visible half of that rot is a page advertising a version the package does not
    /// build.
    #[test]
    fn the_generated_srcinfo_agrees_with_the_pkgbuild() {
        let src = pkgbuild();
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/aur/.SRCINFO");
        let info = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!(".SRCINFO must be there - the AUR reads it rather than the PKGBUILD: {e}"));

        let mut apart = Vec::new();
        for (name, key) in [("pkgver", "pkgver"), ("pkgrel", "pkgrel"), ("pkgname", "pkgname")] {
            let want = field(&src, name);
            if !info.lines().any(|l| l.trim() == format!("{key} = {want}")) {
                apart.push(format!("{key}: the PKGBUILD says \"{want}\" and .SRCINFO does not"));
            }
        }
        let sum = field(&src, "sha256sums");
        if !info.contains(&sum) {
            apart.push(format!("the checksum {sum} is not in .SRCINFO"));
        }
        assert!(apart.is_empty(), "the generated .SRCINFO has drifted from the PKGBUILD - regenerate it with packaging/aur/update-pkgbuild.sh:\n{}", apart.join("\n"));
    }
}
