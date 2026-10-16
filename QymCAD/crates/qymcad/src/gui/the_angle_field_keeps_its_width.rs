//! THE ANGLE FIELD OF A MATE KEEPS ITS WIDTH.
//!
//! A number field is as wide as its text. Reported behaviour: running the angle of a mate, the field grew and
//! shrank as the angle went from 99 to 100 deg, and everything after it on the line jumped sideways with it. A
//! live frame lasts as long as it lasts, so the run passes angles with decimals ("99.73", "100.12"), and those
//! are wider than the 40 px a field is given at least.
#[cfg(test)]
mod tests {
    use super::super::hand::Hand;
    use super::super::the_mate_window_stands_still_while_it_runs::tests::{a_cylindrical_mate_with_its_window_open, spread};
    use super::super::App;
    use qymcad_core::feature::JointKind;

    /// RUNNING THE ANGLE LEAVES THE ANGLE FIELD OF THE WINDOW AS WIDE AS IT WAS.
    #[test]
    fn running_the_angle_leaves_the_angle_field_as_wide_as_it_was() {
        let mut app = App::default();
        let mut hand = Hand::new(&mut app);
        let (jid, at) = a_cylindrical_mate_with_its_window_open(&mut hand);
        // limits whose range is no whole multiple of a frame's step: the run passes angles with decimals, as a live
        // frame of uneven length makes it do
        if let Some(j) = hand.app.project.joints.iter_mut().find(|j| j.id == jid) {
            j.limit_min[0] = Some(0.37);
            j.limit_max[0] = Some(171.9);
        }
        assert!(hand.press_word(&crate::i18n::tr("j-anim-angle"), at), "the window of a cylindrical mate has a button to run its angle");
        let caption = qymcad_assembly::joint_slot_label(JointKind::Cylindrical, 0).0;
        let mut widths = Vec::new();
        let mut angles = Vec::new();
        // two seconds of whole frames: one way through the range
        for _ in 0..120 {
            let label = hand.written_near(&caption, at).expect("the angle caption is in the window");
            let field = hand.number_near(label.right_center() + egui::vec2(30.0, 0.0)).expect("a number field stands beside its caption");
            widths.push(egui::pos2(field.width(), 0.0));
            angles.extend(hand.app.project.joints.iter().find(|j| j.id == jid).and_then(|j| j.drive[0]));
        }
        // GUARD: the run went through angles of two and of three digits
        let lo = angles.iter().cloned().fold(f64::MAX, f64::min);
        let hi = angles.iter().cloned().fold(f64::MIN, f64::max);
        assert!(lo < 99.0 && hi > 100.0, "GUARD: the run must pass from two-digit to three-digit angles, and it went from {lo:.2} to {hi:.2}");
        let w: Vec<f32> = widths.iter().map(|p| p.x).collect();
        let (narrow, wide) = (w.iter().cloned().fold(f32::MAX, f32::min), w.iter().cloned().fold(f32::MIN, f32::max));
        assert!(spread(&widths) < 0.5, "the angle field must keep its width while the angle runs, and it went from {narrow:.1} to {wide:.1} px");
    }
}
