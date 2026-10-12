// SPDX-License-Identifier: LGPL-2.1-or-later
#[path = "../src/history.rs"]
mod history;
use history::*;

#[test]
fn chain_and_diamond_schedule_once_in_dependency_order() {
    let mut g = HistoryGraph::default();
    g.record(2, &[1], true).unwrap();
    g.record(3, &[1], true).unwrap();
    g.record(4, &[2, 3], true).unwrap();
    assert_eq!(g.schedule(&[1], Policy::default()).unwrap(), vec![2, 3, 4]);
}
#[test]
fn editing_middle_detaches_incoming_only_and_undo_restores() {
    let mut g = HistoryGraph::default();
    g.record(2, &[1], true).unwrap();
    g.record(3, &[2], true).unwrap();
    let saved = g.clone();
    g.detach(2);
    assert!(g.schedule(&[1], Policy::default()).unwrap().is_empty());
    assert_eq!(g.schedule(&[2], Policy::default()).unwrap(), vec![3]);
    g = saved;
    assert_eq!(g.schedule(&[1], Policy::default()).unwrap(), vec![2, 3]);
}
#[test]
fn update_suspension_preserves_edges_and_record_off_is_never_retroactive() {
    let mut g = HistoryGraph::default();
    g.record(2, &[1], true).unwrap();
    g.record(3, &[1], false).unwrap();
    assert!(g.schedule(&[1], Policy {update:false, ..Policy::default()}).unwrap().is_empty());
    assert_eq!(g.schedule(&[1], Policy {record:false, ..Policy::default()}).unwrap(), vec![2]);
    assert_eq!(g.schedule(&[1], Policy::default()).unwrap(), vec![2]);
}

#[test]
fn disabled_updates_do_not_hide_corrupt_graphs_or_null_object_ids() {
    unsafe {
        let mut output = [77; 2];
        for (parents, children) in [([1, 2], [2, 1]), ([0, 2], [2, 3]), ([1, 1], [2, 2])] {
            assert_eq!(om9_history_schedule(parents.as_ptr(), children.as_ptr(), 2,
                [1].as_ptr(), 1, false, false, output.as_mut_ptr(), 2), usize::MAX);
            assert_eq!(output, [77; 2]);
        }
    }
}

#[test]
fn oversized_ffi_lengths_are_rejected_before_dereferencing_input() {
    unsafe {
        let input = [1];
        assert_eq!(om9_history_schedule(input.as_ptr(), input.as_ptr(), usize::MAX,
            std::ptr::null(), 0, true, true, std::ptr::null_mut(), 0), usize::MAX);
    }
}
#[test]
fn reject_cycles_self_links_and_duplicate_inputs_without_mutation() {
    let mut g=HistoryGraph::default();
    g.record(2,&[1],true).unwrap();
    assert!(g.record(1,&[2],true).is_err());
    assert!(g.record(3,&[3],true).is_err());
    assert!(g.record(3,&[1,1],true).is_err());
    assert_eq!(g.schedule(&[1],Policy::default()).unwrap(),vec![2]);
}
#[test]
fn clear_is_selected_only_and_lock_policy_has_no_effect_on_parent_edit() {
    let mut g=HistoryGraph::default();g.record(2,&[1],true).unwrap();g.record(3,&[2],true).unwrap();
    g.detach(2);
    assert_eq!(g.parents(3), vec![2]);
    let p=Policy {lock:true,..Policy::default()};
    assert!(p.can_edit(false));assert!(!p.can_edit(true));
}
#[test]
fn ffi_commands_options_and_boolean_values_are_strict() {
    use std::ffi::CString;
    unsafe {
        assert_eq!(om9_history_command_kind(CString::new("gvJoinHistory").unwrap().as_ptr()),5);
        assert_eq!(om9_history_command_kind(CString::new("OM9_gvJoinHistory").unwrap().as_ptr()),5);
        assert_eq!(om9_history_command_kind(CString::new("OM9_RhinoHistoryON").unwrap().as_ptr()),1);
        assert_eq!(om9_history_command_kind(CString::new("Join").unwrap().as_ptr()),0);
        assert_eq!(om9_history_option(CString::new("BrokenHistoryWarning").unwrap().as_ptr()),4);
        assert_eq!(om9_history_parse_boolean(CString::new("maybe").unwrap().as_ptr()),-1);
        assert_eq!(om9_history_parse_boolean(CString::new("No").unwrap().as_ptr()),0);
    }
}
#[test]
fn ffi_native_document_scheduler_is_topological_and_validates_buffers(){
    unsafe{
        let p=[1,1,2,3];let c=[2,3,4,4];let mut out=[0;4];
        assert_eq!(om9_history_schedule(p.as_ptr(),c.as_ptr(),4,[1].as_ptr(),1,true,true,out.as_mut_ptr(),4),3);
        assert_eq!(&out[..3],&[2,3,4]);
        assert_eq!(om9_history_schedule(p.as_ptr(),c.as_ptr(),4,[1].as_ptr(),1,true,false,out.as_mut_ptr(),4),0);
        assert_eq!(om9_history_schedule(std::ptr::null(),c.as_ptr(),4,[1].as_ptr(),1,true,true,out.as_mut_ptr(),4),usize::MAX);
        assert_eq!(om9_history_schedule([1,2].as_ptr(),[2,1].as_ptr(),2,[1].as_ptr(),1,true,true,out.as_mut_ptr(),4),usize::MAX);
    }
}
