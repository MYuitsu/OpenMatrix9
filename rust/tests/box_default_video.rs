use openmatrix9_rust::solid::{Effect,Kind,Session};

#[test]
fn box_starts_default_three_picks_without_selecting_an_option() {
    let mut s=Session::new(Kind::Box);
    assert_eq!(s.command_options(901,1.).unwrap().snapshot().text,"First corner of base ( Diagonal 3Point Vertical Center ): ");
    assert_eq!(s.input("0").unwrap(),Effect::Waiting);
    assert_eq!(s.command_options(901,1.).unwrap().snapshot().text,"Other corner of base: ");
    for invalid in ["10","Length=10"] {assert!(s.input(invalid).is_err());assert_eq!(s.phase(),2);}
    assert_eq!(s.input("10,20,0").unwrap(),Effect::Waiting);
    assert_eq!(s.phase(),4);assert!(s.output().is_none());
    assert_eq!(s.input("5").unwrap(),Effect::Commit);
    assert_eq!(s.output().unwrap(),[0.,0.,0.,1.,0.,0.,0.,1.,0.,0.,0.,1.,10.,20.,5.]);
}

#[test]
fn explicit_three_point_remains_an_edge_width_height_branch() {
    let mut s=Session::new(Kind::Box);s.input("3Point").unwrap();
    s.input("0").unwrap();s.input("3,4,0").unwrap();
    assert_eq!(s.phase(),3);s.input("2").unwrap();s.input("3").unwrap();
    assert_eq!(&s.output().unwrap()[12..],&[5.,2.,3.]);
}

#[test]
fn hovering_builds_base_then_twelve_wire_edges_without_mutation() {
    let mut s=Session::new(Kind::Box);s.input("0").unwrap();
    let base=s.wire_preview([10.,20.,0.],false).unwrap();
    assert_eq!(base.segments.len(),4);assert_eq!(base.markers[0],[0.,0.,0.]);assert_eq!(base.markers[1],[10.,20.,0.]);
    assert_eq!(s.phase(),2);assert!(s.output().is_none());
    s.point([10.,20.,0.]).unwrap();
    let wire=s.wire_preview([10.,20.,5.],false).unwrap();
    assert_eq!(wire.segments.len(),12);assert_eq!(wire.markers[2],[10.,20.,5.]);
    assert_eq!(s.phase(),4);assert!(s.output().is_none());
    let preview=s.preview([10.,20.,5.],false).unwrap();
    assert_eq!(s.point([10.,20.,5.]).unwrap(),Effect::Commit);assert_eq!(s.output().unwrap(),preview);
}

#[test]
fn default_height_drag_in_top_view_uses_opposite_corner_and_world_view_up() {
    let mut s=Session::new(Kind::Box);s.input("0").unwrap();s.input("10,20,0").unwrap();
    assert_eq!(s.height_drag([10.,25.,0.],[0.,1.,0.]),Some([10.,20.,5.]));
    assert_eq!(s.height_drag([10.,15.,0.],[0.,1.,0.]),Some([10.,20.,-5.]));
    assert!(s.height_drag([10.,25.,0.],[0.,0.,1.]).is_none());
    assert_eq!(s.phase(),4);assert!(s.output().is_none());
}

#[test]
fn default_undo_restores_first_corner_and_accepts_another_point() {
    let mut s=Session::new(Kind::Box);s.input("0").unwrap();s.input("10,20,0").unwrap();
    s.input("Undo").unwrap();assert_eq!(s.phase(),2);
    s.input("Undo").unwrap();assert_eq!(s.phase(),1);
    assert!(s.prompt().starts_with("First corner of base"));assert!(s.input("0").is_ok());
}

#[test]
fn negative_height_markers_stay_on_original_base_and_actual_top() {
    for mode in ["3Point","Diagonal","Center"] {
        let mut s=Session::new(Kind::Box);s.input(mode).unwrap();s.input("0").unwrap();
        if mode=="3Point" {s.input("10,0,0").unwrap();s.input("20").unwrap();}
        else if mode=="Center" {s.input("5,10,0").unwrap();}
        else {s.input("10,20,0").unwrap();}
        let wire=s.wire_preview([10.,20.,-5.],false).unwrap();
        assert_eq!(wire.markers[1][2],0.);assert_eq!(wire.markers[2][2],-5.);
        assert!(wire.segments.iter().flatten().all(|p|p[2]>=-5.&&p[2]<=0.));
        assert_eq!(s.phase(),4);assert!(s.output().is_none());
    }
}
