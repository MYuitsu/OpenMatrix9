// SPDX-License-Identifier: LGPL-2.1-or-later
//! **OM9 Matrix** — one add-on package with every Matrix-style command OM9 add-ons provide,
//! grouped by menu (Curve, Transform, Gems, Settings, Cutters …). Each menu is its own crate
//! under `addons/`; this crate links them into one library (`om9_matrix.dll`) and routes each
//! command to its menu.
//!
//! - Interactive commands (points, command-line options, preview) go through
//!   `om9_addon_session`.
//! - Commands of the older one-shot interface (Gem On Crv, Gem Count On Crv) go through
//!   `om9_addon_invoke`.
use om9_addon_abi::sdk::{Addon, Output, Request};
use om9_addon_abi::session::{Event, Interactive, Reply};

/// One menu of the package: its folder name and the commands it routes.
pub struct Category {
    pub name: &'static str,
    pub commands: &'static [&'static str],
    step: fn(&Event) -> Result<Reply, String>,
}

/// Every menu with interactive commands, in menu order.
pub const CATEGORIES: &[Category] = &[
    Category {
        name: "Top11",
        commands: <om9_top11::Top11 as Interactive>::SESSION_COMMANDS,
        step: <om9_top11::Top11 as Interactive>::step,
    },
    Category {
        name: "Curve",
        commands: <om9_curves::Curves as Interactive>::SESSION_COMMANDS,
        step: <om9_curves::Curves as Interactive>::step,
    },
    Category {
        name: "Curve",
        commands: <om9_curve_tools::CurveTools as Interactive>::SESSION_COMMANDS,
        step: <om9_curve_tools::CurveTools as Interactive>::step,
    },
    Category {
        name: "Transform",
        commands: <om9_transform::Transforms as Interactive>::SESSION_COMMANDS,
        step: <om9_transform::Transforms as Interactive>::step,
    },
    Category {
        name: "Surface",
        commands: <om9_surfaces::Surfaces as Interactive>::SESSION_COMMANDS,
        step: <om9_surfaces::Surfaces as Interactive>::step,
    },
    Category {
        name: "Solid",
        commands: <om9_solid::Solids as Interactive>::SESSION_COMMANDS,
        step: <om9_solid::Solids as Interactive>::step,
    },
    Category {
        name: "Transform",
        commands: <om9_transform_tools::TransformTools as Interactive>::SESSION_COMMANDS,
        step: <om9_transform_tools::TransformTools as Interactive>::step,
    },
    Category {
        name: "Tools",
        commands: <om9_tools::Tools as Interactive>::SESSION_COMMANDS,
        step: <om9_tools::Tools as Interactive>::step,
    },
    Category {
        name: "SubD",
        commands: <om9_subd::SubD as Interactive>::SESSION_COMMANDS,
        step: <om9_subd::SubD as Interactive>::step,
    },
    Category {
        name: "Builder",
        commands: <om9_builders::Builders as Interactive>::SESSION_COMMANDS,
        step: <om9_builders::Builders as Interactive>::step,
    },
    Category {
        name: "Gems",
        commands: <om9_gems::Gems as Interactive>::SESSION_COMMANDS,
        step: <om9_gems::Gems as Interactive>::step,
    },
    Category {
        name: "Gems",
        commands: <om9_gem_tools::GemTools as Interactive>::SESSION_COMMANDS,
        step: <om9_gem_tools::GemTools as Interactive>::step,
    },
    Category {
        name: "Settings",
        commands: <om9_settings::Settings as Interactive>::SESSION_COMMANDS,
        step: <om9_settings::Settings as Interactive>::step,
    },
    Category {
        name: "Settings",
        commands: <om9_setting_tools::SettingTools as Interactive>::SESSION_COMMANDS,
        step: <om9_setting_tools::SettingTools as Interactive>::step,
    },
    Category {
        name: "Cutters",
        commands: <gem_cutter::GemCutter as Interactive>::SESSION_COMMANDS,
        step: <gem_cutter::GemCutter as Interactive>::step,
    },
    Category {
        name: "Cutters",
        commands: <om9_cutters::Cutters as Interactive>::SESSION_COMMANDS,
        step: <om9_cutters::Cutters as Interactive>::step,
    },
    Category {
        name: "Cutters",
        commands: <om9_setting_tools::BooleanTool as Interactive>::SESSION_COMMANDS,
        step: <om9_setting_tools::BooleanTool as Interactive>::step,
    },
];

/// The category crates' manifests (relative to `addons/`), in package order. Their
/// `[[commands]]` and `[[settings]]` make up the package manifest.
pub const SOURCES: &[(&str, &str)] = &[
    ("Top11", "top11"),
    ("Curve", "curves"),
    ("Curve", "curve_tools"),
    ("Surface", "surfaces"),
    ("Solid", "solid"),
    ("Transform", "transform"),
    ("Transform", "transform_tools"),
    ("SubD", "subd"),
    ("Builder", "builders"),
    ("Tools", "tools"),
    ("Gems", "gems"),
    ("Gems", "gem_on_curve"),
    ("Gems", "gem_tools"),
    ("Settings", "settings"),
    ("Settings", "setting_tools"),
    ("Cutters", "gem_cutter"),
    ("Cutters", "cutters"),
    ("Cutters", "setting_tools/boolean"),
];

/// Commands of the one-shot interface and their menu.
pub const INVOKE_COMMANDS: &[(&str, &str)] = &[
    ("BuilderGemOnCrv", "Gems"),
    ("BuilderGemCountOnCrv", "Gems"),
];

pub struct Matrix;

/// Every interactive command of the package.
pub static SESSION: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
    CATEGORIES
        .iter()
        .flat_map(|c| c.commands.iter().copied())
        .collect()
});

/// Every command of the package (interactive and one-shot).
pub static ALL: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
    SESSION
        .iter()
        .copied()
        .chain(INVOKE_COMMANDS.iter().map(|c| c.0))
        .collect()
});

/// The menu a command belongs to.
pub fn category_of(code: &str) -> Option<&'static str> {
    CATEGORIES
        .iter()
        .find(|c| c.commands.contains(&code))
        .map(|c| c.name)
        .or_else(|| INVOKE_COMMANDS.iter().find(|c| c.0 == code).map(|c| c.1))
}

/// Route one session step to its menu.
pub fn step(e: &Event) -> Result<Reply, String> {
    CATEGORIES
        .iter()
        .find(|c| c.commands.contains(&e.command.as_str()))
        .map(|c| (c.step)(e))
        .unwrap_or_else(|| Err(format!("{} không phải lệnh tương tác", e.command)))
}

/// Route one one-shot call.
pub fn invoke(r: &Request, out: &mut dyn Output) -> Result<(), String> {
    if INVOKE_COMMANDS.iter().any(|c| c.0 == r.command) {
        <gem_on_curve::GemOnCurve as Addon>::invoke(r, out)
    } else {
        Err(format!("{} chạy theo từng bước (om9_addon_session)", r.command))
    }
}

// ---- C ABI ---------------------------------------------------------------------------------
// The command table is built once from the menus; names stay valid for the library's life.

#[cfg(feature = "export")]
mod export {
    use super::*;
    use om9_addon_abi::{ABI_VERSION, INVOKE_BAD_INPUT, Om9Invocation, Om9Str, Om9Write};

    #[unsafe(no_mangle)]
    pub extern "C" fn om9_addon_abi_version() -> u32 {
        ABI_VERSION
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn om9_addon_command_count() -> usize {
        ALL.len()
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn om9_addon_command_name(index: usize) -> Om9Str {
        ALL.get(index).map(|s| Om9Str::new(s)).unwrap_or(Om9Str::EMPTY)
    }
    /// # Safety
    /// Only the OM9 host calls this, with a valid invocation for the call.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn om9_addon_invoke(invocation: *const Om9Invocation) -> i32 {
        // SAFETY: forwards the host's contract.
        unsafe { om9_addon_abi::sdk::dispatch::<Matrix>(invocation) }
    }
    /// # Safety
    /// Only the OM9 host calls this, with a valid buffer and writer for the call.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn om9_addon_session(
        input: *const u8,
        len: usize,
        write: Om9Write,
        ctx: *mut std::ffi::c_void,
    ) -> i32 {
        if input.is_null() && len > 0 {
            return INVOKE_BAD_INPUT;
        }
        let bytes = if len == 0 {
            &[][..]
        } else {
            // SAFETY: the host passes `len` readable bytes for this call.
            unsafe { std::slice::from_raw_parts(input, len) }
        };
        let (code, out) = om9_addon_abi::session::session_json::<Matrix>(bytes);
        // SAFETY: the writer and its context are valid for this call.
        unsafe { write(ctx, out.as_ptr(), out.len()) };
        code
    }
}

impl Addon for Matrix {
    /// Only the one-shot commands: the host lists all commands through `ALL`.
    const COMMANDS: &'static [&'static str] = &["BuilderGemOnCrv", "BuilderGemCountOnCrv"];
    fn invoke(request: &Request, out: &mut dyn Output) -> Result<(), String> {
        invoke(request, out)
    }
}

impl Interactive for Matrix {
    /// Not used for routing (see [`step`]); `session_json` checks membership through
    /// [`Interactive::accepts`].
    const SESSION_COMMANDS: &'static [&'static str] = &[];
    fn accepts(command: &str) -> bool {
        SESSION.contains(&command)
    }
    fn step(e: &Event) -> Result<Reply, String> {
        step(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_has_one_menu() {
        let mut seen = std::collections::BTreeSet::new();
        for c in ALL.iter() {
            assert!(seen.insert(*c), "{c} twice");
            assert!(category_of(c).is_some(), "{c}");
        }
        assert_eq!(
            <gem_on_curve::GemOnCurve as Addon>::COMMANDS,
            <Matrix as Addon>::COMMANDS
        );
    }
}
