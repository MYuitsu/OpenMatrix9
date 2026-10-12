use openmatrix9_rust::topology_snapshot::*;

fn edge(id:u64, a:u64, b:u64)->Use {
    Use {edge_id:id,pcurve_id:id,start_vertex:a,end_vertex:b,orientation:1,seam:0,degenerate:0,reserved:0,
         domain:[0.,1.],uv_start:[0.,0.],uv_end:[1.,0.]}
}
fn fixture()->(Vec<Face>,Vec<Loop>,Vec<Use>) {
    // One rectangular periodic face: circles at vertices 1 and 2, a paired seam.
    let mut a = edge(2,1,2); a.pcurve_id=2; a.seam=1;
    let mut b = edge(2,2,1); b.pcurve_id=4; b.seam=1; b.orientation=2;
    (vec![Face{surface_id:1,orientation:1,reserved:0,first_loop:0,loop_count:1}],
     vec![Loop{role:1,reserved:0,first_use:0,use_count:4}],
     vec![edge(1,1,1),a,Use{pcurve_id:3,..edge(3,2,2)},b])
}

#[test]
fn ordered_seam_occurrences_remain_owned_and_have_two_free_circles() {
    let (f,l,mut u)=fixture();
    let s = Snapshot::capture(&f,&l,&u).unwrap();
    assert_eq!(s.edge_identity_count,3);
    assert_eq!(s.boundary_components,vec![vec![0],vec![2]]);
    assert!(s.edge_manifold); assert!(!s.edge_closed);
    assert_eq!(s.uses[1].edge_id,s.uses[3].edge_id);
    assert_ne!(s.uses[1].pcurve_id,s.uses[3].pcurve_id);
    u[1].uv_start[0]=42.; assert_eq!(s.uses[1].uv_start,[0.,0.]);
}

#[test]
fn disconnected_or_empty_loop_rejects_instead_of_a_partial_snapshot() {
    let (f,mut l,mut u)=fixture(); u[0].start_vertex=77; u[0].end_vertex=77;
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::DisconnectedLoop);
    l[0].use_count=0;
    assert_eq!(Snapshot::capture(&f,&l,&[]).unwrap_err(),Error::InvalidRange);
}

#[test]
fn overlapping_uncovered_and_overflowing_ranges_reject() {
    let (mut f,mut l,u)=fixture();
    f[0].first_loop=1;
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::InvalidRange);
    f[0].first_loop=0;l[0].first_use=usize::MAX;
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::InvalidRange);
    l[0].first_use=0; l[0].use_count=3;
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::InvalidRange);
}

#[test]
fn ids_flags_uv_and_seam_inconsistencies_reject() {
    let (f,l,u)=fixture();
    for (field,want) in [(0,Error::InvalidIdentity),(1,Error::InvalidFlags),(2,Error::NonFiniteGeometry),
                         (3,Error::InvalidDomain),(4,Error::InvalidSeam),(5,Error::InvalidIdentity)] {
        let mut bad=u.clone();
        match field {0=>bad[0].edge_id=0,1=>bad[0].orientation=3,2=>bad[0].uv_start[0]=f64::NAN,
                    3=>bad[0].domain=[1.,0.],4=>bad[3].seam=0,5=>bad[1].pcurve_id=1,_=>unreachable!()}
        assert_eq!(Snapshot::capture(&f,&l,&bad).unwrap_err(),want);
    }
}

#[test]
fn degenerate_pole_uses_are_not_free_boundaries() {
    let (f,l,mut u)=fixture(); u[0].degenerate=1;u[2].degenerate=1;
    let s=Snapshot::capture(&f,&l,&u).unwrap();
    assert!(s.edge_closed);assert!(s.boundary_components.is_empty());
    assert_eq!(s.degenerate_use_count,2);
}

#[test]
fn missing_outer_loop_and_duplicate_seam_side_reject() {
    let (f,mut l,mut u)=fixture(); l[0].role=2;
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::InvalidLoopRole);
    l[0].role=1;
    for edge in &mut u {edge.start_vertex=1;edge.end_vertex=1;}
    u[3].orientation=1;
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::InvalidSeam);
}

#[test]
fn checked_record_budget_rejects_overflow_before_extraction() {
    assert!(counts_allowed(1,1,4));
    assert!(!counts_allowed(usize::MAX,1,1));
    assert!(!counts_allowed(1,1,16384));
}

#[test]
fn opposite_use_orientation_requires_opposite_directed_endpoints() {
    let (f,l,mut u)=fixture();
    u[0].start_vertex=2;u[0].end_vertex=1;
    u[2].start_vertex=2;u[2].end_vertex=1;
    u[3].start_vertex=1;u[3].end_vertex=2;
    // The wire remains connected, but the reversed seam has wrong endpoints.
    assert_eq!(Snapshot::capture(&f,&l,&u).unwrap_err(),Error::InvalidIdentity);
}

#[test]
fn common_descriptor_retains_occurrences_without_certifying_solid_or_manifold() {
    use openmatrix9_rust::geometry_contract::{Fact,LoopRole};
    let (f,l,u)=fixture();let d=Snapshot::capture(&f,&l,&u).unwrap().descriptor();
    assert_eq!(d.edge_use_count(),Fact::Known(4));
    assert_eq!(d.edge_identity_count(),Fact::Known(3));
    assert_eq!(d.manifold,Fact::Unknown);assert_eq!(d.solid,Fact::Unknown);
    let Fact::Known(faces)=d.faces else {panic!("missing owned faces")};
    let Fact::Known(loops)=&faces[0].loops else {panic!("missing owned loops")};
    assert_eq!(loops[0].role,LoopRole::Outer);
    assert_eq!(loops[0].edge_uses[1].edge_id,loops[0].edge_uses[3].edge_id);
}
