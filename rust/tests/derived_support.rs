use openmatrix9_rust::analytic_support::AnalyticInput;
use openmatrix9_rust::derived_support::{DerivedNode,LeafInput,Tree};
use openmatrix9_rust::derived_support_ffi::{NativeSupportTree,om9_support_tree_json};
use openmatrix9_rust::nurbs_basis::{Axis,CurveBasis};

fn line(dim:u32)->AnalyticInput {
    AnalyticInput{kind:1,dimension:dim,origin:[2.,3.,0.],axes:[[1.,0.,0.],[0.;3],[0.;3]],radii:[0.;2],extra:0.}
}
fn node(kind:u32)->DerivedNode {
    let mut n=DerivedNode{kind,dimension:3,direction:[0.;3],origin:[0.;3],offset:0.,bounds:[0.,1.,0.,1.],infinite_mask:0,reserved:0};
    if matches!(kind,1|3|4){n.direction=[0.,0.,1.];}
    if matches!(kind,1|5){n.bounds[2..].fill(0.);}
    n
}
#[test]
fn nested_signed_offset_extrusion_keeps_owned_basis_and_trim_order() {
    let mut nodes=[node(2),node(3),node(5)];nodes[0].offset=-2.;nodes[1].infinite_mask=12;nodes[1].bounds[2..].fill(0.);
    nodes[2].bounds=[0.2,0.8,0.,0.];
    let mut p=[[2.,0.,0.],[4.,3.,0.]];
    let a=Axis{degree:1,knots:&[0.,1.],multiplicities:&[2,2],periodic:false,domain:[0.,1.]};
    let tree=Tree::capture(LeafInput::BezierCurve(CurveBasis{poles:&p,weights:None,axis:a}),3,&nodes).unwrap();
    p[0][0]=91.;assert_eq!(p[0][0],91.);
    nodes[0].offset=88.;assert_eq!(nodes[0].offset,88.);
    let json=tree.json().unwrap();
    assert!(json.starts_with("{\"schema_version\":1,\"kind\":\"offset_surface\""));
    assert!(json.contains("\"offset\":-2"));assert!(!json.contains("\"offset\":88"));
    let ex=json.find("\"kind\":\"linear_extrusion\"").unwrap();let trim=json.find("\"kind\":\"trimmed_curve\"").unwrap();let basis=json.find("\"kind\":\"bezier_curve\"").unwrap();
    assert!(ex<trim && trim<basis);assert!(json.contains("\"natural_domains\":[[0,1],\"unbounded\"]"));
    assert!(json.contains("\"natural_domains\":[[0.2,0.8]]"));assert!(json.contains("[[2,0,0],[4,3,0]]"));
}

#[test]
fn abi_typed_basis_leaves_retain_controls_and_reject_invalid_basis_atomically() {
    use openmatrix9_rust::nurbs_basis_ffi::{NativeAxis,NativeCurve,NativeSurface};
    let knots=[0.,1.];let mult=[2,2];let poles=[[2.,0.,0.],[4.,3.,0.]];
    let grid=[[0.,0.,0.],[0.,1.,0.],[1.,0.,0.],[1.,1.,1.]];
    let a=NativeAxis{degree:1,knots:knots.as_ptr(),knot_count:2,multiplicities:mult.as_ptr(),multiplicity_count:2,periodic:0,domain:[0.,1.]};
    let c=NativeCurve{poles:poles.as_ptr(),pole_count:2,weights:std::ptr::null(),weight_count:0,axis:a};
    let s=NativeSurface{poles:grid.as_ptr(),pole_count:4,weights:std::ptr::null(),weight_count:0,u_count:2,v_count:2,axes:[a,a]};
    for (kind,n,want) in [(2,node(1),"nurbs_curve"),(4,node(1),"bezier_curve"),(3,node(2),"nurbs_surface"),(5,node(2),"bezier_surface")] {
        let mut input=NativeSupportTree::analytic(line(3),&n,1);input.leaf_kind=kind;input.curve=c;input.surface=s;
        let call=|i,out,cap|unsafe{om9_support_tree_json(i,out,cap)};
        let q=call(input,std::ptr::null_mut(),0);assert_eq!(q.status,0);
        let mut bytes=vec![0x5a;q.needed+1];assert_eq!(call(input,bytes.as_mut_ptr(),q.needed).status,0);assert_eq!(bytes[q.needed],0x5a);
        let json=std::str::from_utf8(&bytes[..q.needed]).unwrap();assert!(json.contains(want));assert!(json.contains(if kind==2||kind==4{"[[2,0,0],[4,3,0]]"}else{"[[0,0,0],[0,1,0],[1,0,0],[1,1,1]]"}));
        let saved=bytes.clone();input.curve.axis.domain=[1.,0.];input.surface.axes[0].domain=[1.,0.];
        assert_ne!(call(input,bytes.as_mut_ptr(),q.needed).status,0);assert_eq!(saved,bytes);
    }
}
#[test]
fn semi_infinite_domains_and_two_dimensional_basis_are_explicit() {
    let mut n=node(1);n.infinite_mask=1;n.bounds=[0.,5.,0.,0.];
    assert!(Tree::capture(LeafInput::Analytic(line(3)),3,&[n]).unwrap().json().unwrap().contains("[\"-infinity\",5]"));
    n.infinite_mask=2;n.bounds=[-5.,0.,0.,0.];
    assert!(Tree::capture(LeafInput::Analytic(line(3)),3,&[n]).unwrap().json().unwrap().contains("[-5,\"infinity\"]"));
    let a=Axis{degree:1,knots:&[0.,1.],multiplicities:&[2,2],periodic:false,domain:[0.,1.]};
    n.dimension=2;n.direction=[0.;3];n.infinite_mask=0;n.bounds=[0.,1.,0.,0.];
    let b=CurveBasis{poles:&[[0.,0.,1.],[1.,1.,0.]],weights:None,axis:a};
    assert!(Tree::capture(LeafInput::NurbsCurve(b),2,&[n]).is_err());
}

#[test]
fn inner_surface_trim_can_keep_the_other_native_axis_unbounded() {
    let offset=node(2);let mut trim=node(6);trim.bounds=[0.2,0.8,0.,0.];trim.infinite_mask=12;
    let mut extrusion=node(3);extrusion.infinite_mask=12;extrusion.bounds[2..].fill(0.);
    let json=Tree::capture(LeafInput::Analytic(line(3)),3,&[offset,trim,extrusion]).unwrap().json().unwrap();
    assert!(json.contains("\"kind\":\"trimmed_surface\""));assert!(json.contains("\"natural_domains\":[[0.2,0.8],\"unbounded\"]"));
}
#[test]
fn invalid_category_dimension_direction_and_domain_cannot_publish() {
    assert!(Tree::capture(LeafInput::Analytic(line(3)),3,&[node(2)]).is_err());
    assert!(Tree::capture(LeafInput::Analytic(line(2)),2,&[node(3)]).is_err());
    for mutate in 0..9 {
        let mut n=node(1);
        match mutate {0=>n.direction=[0.,0.,2.],1=>n.offset=f64::NAN,2=>n.origin[0]=1.,3=>n.bounds=[1.,0.,0.,0.],4=>n.reserved=1,5=>n.infinite_mask=16,6=>{n.infinite_mask=1;n.bounds[0]=2.;},7=>n.bounds[2]=1.,_=>n.kind=99}
        assert!(Tree::capture(LeafInput::Analytic(line(3)),3,&[n]).is_err(),"{mutate}");
    }
    let mut trim=node(5);trim.infinite_mask=3;trim.bounds=[0.;4];
    assert!(Tree::capture(LeafInput::Analytic(line(3)),3,&[trim]).is_err());
    assert!(Tree::capture(LeafInput::Analytic(line(3)),3,&[node(1);17]).is_err());
    let mut uv=node(1);uv.dimension=2;uv.direction=[0.;3];uv.offset=-3.;
    assert!(Tree::capture(LeafInput::Analytic(line(2)),2,&[uv]).unwrap().json().unwrap().contains("\"dimension\":2"));
    uv.direction[2]=1.;assert!(Tree::capture(LeafInput::Analytic(line(2)),2,&[uv]).is_err());
}
#[test]
fn ffi_query_copy_is_atomic_and_rejects_wrapper_envelopes_before_leaf() {
    assert_eq!((std::mem::size_of::<DerivedNode>(),std::mem::align_of::<DerivedNode>()),(104,8));
    assert_eq!((std::mem::size_of::<NativeSupportTree>(),std::mem::align_of::<NativeSupportTree>()),(424,8));
    let n=[node(4)];let mut i=NativeSupportTree::analytic(line(3),n.as_ptr(),1);
    let call=|i,out,cap|unsafe{om9_support_tree_json(i,out,cap)};
    let q=call(i,std::ptr::null_mut(),0);assert_eq!(q.status,0);
    let mut out=vec![0x5a;q.needed+1];assert_eq!(call(i,out.as_mut_ptr(),q.needed).status,0);assert_eq!(out[q.needed],0x5a);
    assert!(std::str::from_utf8(&out[..q.needed]).unwrap().contains("\"kind\":\"revolution\""));let before=out.clone();
    assert_eq!(call(i,out.as_mut_ptr(),q.needed-1).status,1001);assert_eq!(out,before);
    assert_eq!(call(i,std::ptr::null_mut(),1).status,1002);
    i.nodes=std::ptr::null();assert_ne!(call(i,out.as_mut_ptr(),q.needed).status,0);assert_eq!(out,before);
    i.nodes=1usize as *const DerivedNode;i.node_count=17;assert_ne!(call(i,out.as_mut_ptr(),q.needed).status,0);assert_eq!(out,before);
    i.node_count=1;assert_ne!(call(i,out.as_mut_ptr(),q.needed).status,0);assert_eq!(out,before);
    // An invalid wrapper must reject before inspecting a selected pointer-bearing
    // leaf. These deliberately unusable envelopes may only enter rejected paths.
    let mut invalid=node(1);invalid.direction=[0.;3];
    i.nodes=&invalid;i.leaf_kind=2;i.curve.poles=1usize as *const [f64;3];i.curve.pole_count=2;
    assert_eq!(call(i,out.as_mut_ptr(),q.needed).status,4002);assert_eq!(out,before);
    invalid=node(2);i.nodes=&invalid; // Surface offset cannot have a curve basis.
    assert_eq!(call(i,out.as_mut_ptr(),q.needed).status,4002);assert_eq!(out,before);
}
