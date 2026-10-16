//! TAKING ONE TOOL RELEASES THE PREVIOUS ONE.
//!
//! Two tools at once means ambiguity under the cursor: the click goes to whichever handler stands
//! higher in the code, while the person is certain they are working with the one picked last. The
//! error is quiet, and people blame themselves for it.
//!
//! EVERY pair is checked against the single list (`AssemblyTool::ALL`): take the first, take the second,
//! and exactly one must be left in hand.
//!
//! IT USED TO SAY THAT AND NOT DO IT. The doors were a list of SEVEN written by hand here, while the
//! enumeration has NINE: the axis pick and the anchor re-pick were missing, and the rule was unchecked for
//! them - they are taken from a button in the joint's own popup rather than from the toolbar, so nobody
//! noticed. The comment claimed the sweep was exhaustive, which is the worst kind of wrong: a promise that
//! stops anyone from looking. The doors now come from `assembly_tools::doors`, shared with the F1 check,
//! and the sweep really does walk `AssemblyTool::ALL`.
#[cfg(test)]
mod tests {
    use super::super::assembly_tools::doors::arm;
    use super::super::assembly_tools::AssemblyTool;
    use super::super::App;
    use qymcad_core::model::Id;

    fn two_parts(app: &mut App) -> Vec<Id> {
        let before: Vec<Id> = app.project.bodies.iter().map(|b| b.id).collect();
        super::super::joint_flow::tests::add_part_at(app, 0.0);
        super::super::joint_flow::tests::add_part_at(app, 60.0);
        let root = app.project.root;
        app.enter_component(root);
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
        crate::gui::commands::refresh_edges(&mut app.part_ctx());
        app.viewing.mode_3d = true;
        app.project.bodies.iter().map(|b| b.id).filter(|b| !before.contains(b)).collect()
    }

    /// TAKING A TOOL CLOSES THE EDIT OF A MATE. Reported behaviour: the first hinge of an assembly was made, its
    /// popup stayed open over the canvas, and when the second mate was started the corner of the third part lay
    /// under the popup's fields - the click went to them. The axis pick and the anchor re-pick are taken FROM that
    /// popup, so they keep it.
    #[test]
    fn taking_a_tool_closes_the_edit_of_a_mate() {
        let mut open: Vec<String> = Vec::new();
        for t in AssemblyTool::ALL {
            if matches!(t, AssemblyTool::Axis | AssemblyTool::Repick) {
                continue;
            }
            let mut app = App::default();
            let (jid, _) = super::super::assembly_tools::doors::a_joint(&mut app);
            crate::gui::enter_joint_edit(&mut app.side.joint, &mut app.chosen.sel, &mut app.status, jid);
            arm(&mut app, t);
            if app.side.joint.edit.is_some() {
                open.push(format!("{t:?}"));
            }
        }
        assert!(open.is_empty(), "the edit of a mate stayed open over the canvas after taking {open:?}");
        // the tools taken from the popup itself keep it
        for t in [AssemblyTool::Axis, AssemblyTool::Repick] {
            let mut app = App::default();
            arm(&mut app, t);
            let jid = app.project.joints.first().map(|j| j.id).expect("the joint");
            app.side.joint.edit = Some(jid);
            crate::gui::assembly_tools::drop_assembly_tools(&mut app.joint_ctx());
            assert!(app.side.joint.edit.is_some(), "putting down {t:?} closed the popup it was taken from");
        }
    }

    #[test]
    fn taking_a_tool_releases_the_previous_one() {
        let mut both: Vec<String> = Vec::new();
        for first in AssemblyTool::ALL {
            for second in AssemblyTool::ALL {
                if first == second {
                    continue; // the same door is a toggle rather than a change of tool
                }
                let (first_name, second_name) = (first.help_mode(), second.help_mode());
                let mut app = App::default();
                let mine = two_parts(&mut app);
                assert_eq!(mine.len(), 2, "setup: there should be two bodies of our own, and there are {}", mine.len());
                app.workbench = super::super::Workbench::Assembly;

                arm(&mut app, first);
                // GUARD AGAINST A VACUOUS CHECK: the first tool really was taken, otherwise there is no change to check.
                assert!(!app.armed_assembly_tools().is_empty(), "GUARD: \"{first_name}\" was not taken, so there is nothing to check the change on");
                arm(&mut app, second);

                let armed = app.armed_assembly_tools().len();
                if armed != 1 {
                    both.push(format!("\"{first_name}\" -> \"{second_name}\": tools left in hand: {armed}"));
                }
            }
        }
        assert!(both.is_empty(), "two tools at once: the click goes to the wrong one while the person is certain they work with the last taken:\n{}", both.join("\n"));
    }
}
