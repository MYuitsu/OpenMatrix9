//! THE CURSOR NAMES ONE OF THE CORNERS AT A POINT ONLY WHILE IT STANDS NEAR THAT POINT.
//!
//! Where four lines meet at one point there are four corners there, and the cursor says which one is meant by the
//! side of the point it stands on. That is a good answer close to the point and a meaningless one far from it: with
//! the cursor read wherever it happened to be, the corner changed as the pointer crossed the sheet, so the radius
//! being typed in the field would be cut at a corner nobody was looking at - and the sheet shows the corner it is
//! showing, so what Enter cut was not what was on screen.
//!
//! The reach is `Grab::Corner`: thirty-six pixels at the normal precision, three times the aim a point is caught
//! from and then some. Saying WHICH WAY the corner goes among the several at one point is a coarser act than
//! catching the point - the pointer names a sector rather than hitting a mark - and at the aim of a point the choice
//! stopped exactly where the click catches the point, leaving a person who had aimed there with the corner open and
//! no way to move it. It is still a corner of the drawing and not the sheet: the choice must not follow the pointer
//! while the radius is being typed.
use qymcad_core::feature::Purpose;
use qymcad_core::model::Project;
use qymcad_ui_state::grab::{grab, Grab};
use qymcad_ui_state::Settings;

/// A sketch with one line from the origin to (20, 0), and the point at its far end: the point the cursor stands near.
fn a_point_and_its_id() -> (Project, usize, u64) {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("S");
    p.add_line_entity(si, 0.0, 0.0, 20.0, 0.0, Purpose::Real);
    p.regen_sketch(si);
    let pid = p.sketches[si].points.iter().find(|q| (q.x - 20.0).abs() < 1e-9).map(|q| q.id).expect("the far end of the line is a point");
    (p, si, pid)
}

/// STANDING BESIDE THE POINT, THE CURSOR SPEAKS; A POINTER ELSEWHERE ON THE SHEET SAYS NOTHING.
#[test]
fn a_cursor_far_from_the_point_does_not_name_a_corner() {
    let (p, si, pid) = a_point_and_its_id();
    let reach = qymcad_ui_state::corner_reach(&Settings::default());
    let beside = (20.0, reach as f64 / 2.0); // half the reach away: inside it
    assert_eq!(
        qymcad_ui_state::corner_cursor(&p, si, pid, Some(beside), reach, 1.0),
        Some(beside),
        "the cursor standing beside the point was not heard: it is the only thing that says which corner is meant"
    );
    assert_eq!(qymcad_ui_state::corner_cursor(&p, si, pid, Some((20.0 + 80.0, 0.0)), reach, 1.0), None, "a cursor eighty units away named a corner: the corner followed the pointer across the sheet");
    assert_eq!(qymcad_ui_state::corner_cursor(&p, si, pid, None, reach, 1.0), None, "no cursor at all named a corner");
}

/// THE REACH IS THE THIRTY-SIX PIXELS IT WAS DECIDED TO BE - a circle round the point, and not the sheet - and it is
/// the aim of the person, so it narrows and widens with the pick precision.
#[test]
fn the_reach_is_the_aim_it_was_decided_to_be() {
    let mut set = Settings::default();
    let reach = qymcad_ui_state::corner_reach(&set);
    assert_eq!(reach, 36.0, "the reach of the cursor at the normal precision is not the thirty-six pixels it was decided to be ({reach})");
    assert!(reach > 3.0 * grab(&set, Grab::Point), "the reach of the cursor is not three times the aim a point is caught from ({reach} against {})", grab(&set, Grab::Point));
    assert!(reach < grab(&set, Grab::Snap) * 6.0, "the reach of the cursor is not a circle round the point any more ({reach} pixels): the corner would follow the pointer across the drawing");
    for level in [0u8, 1, 2] {
        set.pick_precision = level;
        assert_eq!(qymcad_ui_state::corner_reach(&set), grab(&set, Grab::Corner), "the reach of the cursor is not the aim of the person at pick precision {level}");
        assert!(qymcad_ui_state::corner_reach(&set) > grab(&set, Grab::Point), "at pick precision {level} the cursor reaches less far than the point is caught from");
    }
}

/// THE REACH IS A CIRCLE ON THE SHEET, SO IT IS THE SAME ONE AT EVERY ZOOM.
#[test]
fn the_reach_is_the_same_pixels_at_every_zoom() {
    let (p, si, pid) = a_point_and_its_id();
    let reach = qymcad_ui_state::corner_reach(&Settings::default());
    // EIGHTY UNITS FROM THE POINT: out of reach on a sheet at a pixel to the unit, inside it on a sheet drawn at a
    // tenth of a pixel to the unit. The reach is a circle on the screen - the same one whatever the zoom.
    let away = (100.0, 0.0);
    assert_eq!(qymcad_ui_state::corner_cursor(&p, si, pid, Some(away), reach, 1.0), None, "a cursor eighty pixels away named a corner");
    assert_eq!(
        qymcad_ui_state::corner_cursor(&p, si, pid, Some(away), reach, 0.1),
        Some(away),
        "the same cursor eight pixels away did not name a corner: the reach is a circle on the sheet, not a circle in the drawing"
    );
}
