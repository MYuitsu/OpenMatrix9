// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{ffi::*, gumball::*};
use std::ffi::CStr;
#[test]
fn core_gumball_catalog_routes_views_without_repeat() {
    for (icon, name) in [
        ("InfoSettingsGumballON", "OM9_Gumball"),
        ("InfoSettingsGumballAlignment", "OM9_GumballAlignment"),
        ("InfoSettingsRelocateGumball", "OM9_RelocateGumball"),
    ] {
        let index = (0..om9_command_count())
            .find(|&i| {
                unsafe { CStr::from_ptr(om9_command_icon(i)) }
                    .to_str()
                    .unwrap()
                    == icon
            })
            .unwrap();
        assert_eq!(
            unsafe { CStr::from_ptr(om9_command_id(index)) }
                .to_str()
                .unwrap(),
            name
        );
        assert_eq!(om9_command_permissions(index), 2);
        assert!(!om9_command_repeatable(index));
    }
}
#[test]
fn command_input_recognizes_exact_alias_and_rejects_unknown_option() {
    assert_eq!(
        parse_command("_Gumball On").unwrap(),
        Some(CommandInput {
            kind: CommandKind::Gumball,
            value: Some(1)
        })
    );
    assert_eq!(
        parse_command("GumballAlignment Object").unwrap(),
        Some(CommandInput {
            kind: CommandKind::Alignment,
            value: Some(1)
        })
    );
    assert_eq!(
        parse_command("GumballRelocate").unwrap(),
        Some(CommandInput {
            kind: CommandKind::Relocate,
            value: None
        })
    );
    assert!(parse_command("Gumball Moon").is_err());
    assert_eq!(parse_command("Line").unwrap(), None);
}
