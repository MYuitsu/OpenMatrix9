// SPDX-License-Identifier: LGPL-2.1-or-later
//! Combines independently observed native and combinatorial facts.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub root_kind: u32,
    pub native_valid: u32,
    pub has_faces: u32,
    pub edge_manifold: u32,
    pub vertex_manifold: u32,
    pub edge_closed: u32,
    pub face_oriented: u32,
    pub loose_geometry: u32,
    pub duplicate_faces: u32,
    pub native_orientation: u32,
    pub shell_count: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub status: u32,
    pub native_valid: u32,
    pub manifold: u32,
    pub closed: u32,
    pub valid_solid: u32,
    pub native_orientation: u32,
    pub world_orientation: u32,
    pub reasons: u32,
}
// Tri-state: 0 unknown, 1 false, 2 true. Orientation: 0 unknown, 1 outward,
// 2 inward. TopAbs kind tags are native representation provenance, not validity.
#[unsafe(no_mangle)]
pub extern "C" fn om9_shape_validation(i: Input, sign: i32) -> Summary {
    let flags=[i.has_faces,i.edge_manifold,i.vertex_manifold,i.edge_closed,
        i.face_oriented,i.loose_geometry,i.duplicate_faces];
    if i.root_kind>7 || i.native_valid>2 || i.native_orientation>2 ||
        flags.iter().any(|&v|v>1) || i.shell_count>16384 ||
        (sign!=1 && sign!=-1) ||
        (i.native_orientation!=0 && (i.root_kind!=2 || i.has_faces==0 || i.shell_count==0)) {
        return Summary{status:1,..Summary::default()};
    }
    let boolean=|b:bool|if b{2}else{1};
    let boundary=|positive:bool| {
        if i.has_faces==0 {0}
        else if !positive || i.loose_geometry!=0 {1}
        else if i.duplicate_faces!=0 {0}
        else {2}
    };
    let manifold=boundary(i.edge_manifold!=0 && i.vertex_manifold!=0);
    let closed=boundary(i.edge_closed!=0);
    let constraints=[boolean(i.root_kind==2 && i.has_faces!=0 && i.shell_count!=0),
        i.native_valid,manifold,closed,boolean(i.face_oriented!=0),
        if i.native_orientation==0{0}else{2}];
    let valid_solid=if constraints.contains(&1){1}else if constraints.contains(&0){0}else{2};
    let reasons=[i.native_valid==1,i.has_faces==0,i.edge_manifold==0,
        i.vertex_manifold==0,i.edge_closed==0,i.face_oriented==0,
        i.loose_geometry!=0,i.duplicate_faces!=0,i.root_kind!=2,
        i.native_valid==0,i.native_orientation==0,i.shell_count==0]
        .iter().enumerate().fold(0,|bits,(n,&present)|bits|if present{1<<n}else{0});
    // Orientation of a rejected/uninspected solid is not a usable certificate.
    let native_orientation=if valid_solid==2{i.native_orientation}else{0};
    Summary{status:0,native_valid:i.native_valid,manifold,closed,valid_solid,
        native_orientation,world_orientation:om9_shape_orientation(native_orientation,sign),reasons}
}

/// Reframe from immutable native orientation; never flip a previous world value.
/// No pointers, allocation, native owners or panic paths cross this ABI.
#[unsafe(no_mangle)]
pub extern "C" fn om9_shape_orientation(native: u32, sign: i32) -> u32 {
    if native>2 || (sign!=1 && sign!=-1) {3}
    else if native==0 || sign==1 {native}
    else {3-native}
}
