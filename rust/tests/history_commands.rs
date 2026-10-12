// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::ffi::*;
use openmatrix9_rust::history::om9_history_command_kind;
use std::ffi::CStr;
#[test]
fn all_five_native_history_commands_require_document_write_permission(){
    for (icon,kind) in [("RhinoHistoryON",1),("InfoSettingsGVClearHistory",2),("GVHistoryRecordON",3),("GVHistoryUpdateON",4),("gvJoinHistory",5)]{
        let matches:Vec<_>=(0..om9_command_count()).filter(|i|unsafe{CStr::from_ptr(om9_command_icon(*i))}.to_str().unwrap()==icon).collect();
        assert_eq!(matches.len(),1,"{icon} must exist exactly once in the live command catalog");
        let i=matches[0];
        if icon=="gvJoinHistory"{assert_eq!(unsafe{CStr::from_ptr(om9_command_id(i))}.to_str().unwrap(),"OM9_gvJoinHistory","Join History must preserve its existing stable menu command ID");}
        assert_eq!(unsafe{om9_history_command_kind(om9_command_id(i))},kind,"{icon} must route to its native History handler");
        assert_eq!(om9_command_permissions(i),1,"{icon} mutates persistent document geometry or policy and must enforce write permission");
    }
}
