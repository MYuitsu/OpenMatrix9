//! A STATE BROUGHT BACK BY UNDO HANDS OUT NO ID THE LIVE DOCUMENT HAS HANDED OUT, and a node laid on what the undo
//! took away says what it reads that is gone.
//!
//! Reported behaviour (found by the contract of every tool): a face picked, Ctrl+Z taking the block away, Enter - the
//! shell came in red on "its own" body: the counter brought back gave the new node the id the picked body had had.
use qymcad_core::model::Project;

#[test]
fn an_id_handed_out_once_is_not_handed_out_again() {
    let mut p = Project::default();
    p.new_document();
    let before = p.clone();
    let block = p.add_box(40.0, 30.0, 10.0);
    let mut restored = before.clone();
    restored.keep_ids_past(&p);
    let next = restored.alloc_id();
    assert!(next > block, "the state brought back handed out {next}, the block the undo took away was {block}");
}

#[test]
fn a_node_on_a_body_that_is_gone_names_it() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let shell = p.add_shell_mode(block, 2.0, vec![1], qymcad_core::feature::ShellSide::default());
    let node = p.timeline.iter().find(|n| n.kind.body() == Some(shell)).map(|n| n.id).expect("the shell node");
    assert!(p.gone_inputs(node).is_empty(), "the shell reads the block, which is there: {:?}", p.gone_inputs(node));
    p.timeline.retain(|n| n.kind.body() != Some(block));
    if let Some(mi) = p.mesh_index(block) {
        p.remove_mesh(mi);
    }
    assert_eq!(p.gone_inputs(node), vec![block], "the block is gone from under the shell and the shell does not say so");
}
