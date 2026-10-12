use openmatrix9_rust::{solid::{Kind,Mode},native_solid_capability::{Request,Stage},geometry_contract::*};
fn request(kind:Kind,mode:Mode)->Request{
    let axes=[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]];
    Request{kind,mode,stage:Stage::Capture,phase:1,point_count:0,length:0.,width:0.,radius:0.,
        point_constraint:false,constraint_types:vec![],first:[0.;3],axes,frame_origin:[0.;3],frame_axes:axes,circle:None,output:None}
}
fn cap(input:RepresentationKind,r:Request)->Result<OperationCapability,CapabilityError>{
    operation_capability(OperationId::ConstructSolid,input,OperationOptions::NativeSolid(r))
}
fn output(kind:Kind)->[f64;15]{let mut b=[0.;15];b[3]=1.;b[7]=1.;b[11]=1.;b[12]=2.;if kind==Kind::Box{b[13]=3.;b[14]=4.;}b}
#[test]
fn existing_modes_preserve_options_and_feature_identity(){
    for (kind,modes,id) in [(Kind::Box,vec![Mode::Corners,Mode::Diagonal,Mode::ThreePoint,Mode::Vertical,Mode::Center,Mode::Cube],"OM9-SOLID-012"),
        (Kind::Sphere,vec![Mode::Center,Mode::Vertical,Mode::TwoPoint,Mode::ThreePoint,Mode::FourPoint,Mode::FitPoints,Mode::AroundCurve,Mode::Tangent],"OM9-SOLID-014")]{
        for mode in modes{let r=request(kind,mode);let c=cap(RepresentationKind::Point,r.clone()).unwrap();
            assert_eq!(c.backend,Backend::OcctEdit);assert_eq!(c.evidence_id,id);assert_eq!(c.options,OperationOptions::NativeSolid(r));
            assert!(c.constraints.contains(&OperationConstraint::ValidateNativeInput));}
    }
}
#[test]
fn concrete_reference_permission_depends_on_actual_sphere_mode(){
    use RepresentationKind::*;
    for mode in [Mode::Center,Mode::AroundCurve,Mode::Tangent,Mode::FitPoints]{
        for input in [Curve,Polycurve,TrimmedFace,Shell,Solid,Brep,Mesh,PointCloud]{
            let expected=mode==Mode::FitPoints||(matches!(mode,Mode::AroundCurve|Mode::Tangent)&&matches!(input,Curve|Polycurve));
            assert_eq!(cap(input,request(Kind::Sphere,mode)).is_ok(),expected,"{mode:?}/{input:?}");
        }
        for input in [Unknown,Instance,Group,Annotation,Retained]{assert_eq!(cap(input,request(Kind::Sphere,mode)),Err(CapabilityError::UnsupportedRepresentation(input)));}
    }
    assert!(cap(Curve,request(Kind::Box,Mode::Corners)).is_err());
}
#[test]
fn empty_declaration_can_prompt_but_construct_needs_completed_valid_output(){
    for kind in [Kind::Box,Kind::Sphere]{let mode=if kind==Kind::Box{Mode::Corners}else{Mode::Center};let r=request(kind,mode);
        for stage in [Stage::Declare,Stage::Capture,Stage::Preview]{assert!(cap(RepresentationKind::Point,Request{stage,..r.clone()}).is_ok());}
        assert_eq!(cap(RepresentationKind::Point,Request{stage:Stage::Construct,..r.clone()}),Err(CapabilityError::UnsupportedOptions));
        assert!(cap(RepresentationKind::Point,Request{stage:Stage::Construct,phase:6,output:Some(output(kind)),..r.clone()}).is_ok());
        assert_eq!(cap(RepresentationKind::Point,Request{phase:0,..r}),Err(CapabilityError::UnsupportedOperation(OperationId::ConstructSolid)));
    }
}
#[test]
fn unsupported_modes_counts_dimensions_and_frames_reject(){
    for (kind,mode) in [(Kind::Box,Mode::FitPoints),(Kind::Sphere,Mode::Cube)]{assert_eq!(cap(RepresentationKind::Point,request(kind,mode)),Err(CapabilityError::UnsupportedOptions));}
    let r=request(Kind::Sphere,Mode::FitPoints);
    assert_eq!(cap(RepresentationKind::Point,Request{point_count:1025,..r.clone()}),Err(CapabilityError::UnsupportedOptions));
    for n in [f64::NAN,f64::INFINITY,-1.,1e10]{assert_eq!(cap(RepresentationKind::Point,Request{radius:n,..r.clone()}),Err(CapabilityError::UnsupportedOptions));}
    let mut wrong=r.frame_axes;wrong[2]=[0.,0.,-1.];assert_eq!(cap(RepresentationKind::Point,Request{frame_axes:wrong,..r.clone()}),Err(CapabilityError::UnsupportedOptions));
    let tangent=Request{point_count:2,constraint_types:vec![true],..request(Kind::Sphere,Mode::Tangent)};
    assert_eq!(cap(RepresentationKind::Point,tangent),Err(CapabilityError::UnsupportedOptions));
}
#[test]
fn output_geometry_checks_extrema_orientation_and_sphere_auxiliary_fields(){
    for kind in [Kind::Box,Kind::Sphere]{let mode=if kind==Kind::Box{Mode::Corners}else{Mode::Center};let r=Request{stage:Stage::Construct,phase:6,..request(kind,mode)};
        let b=output(kind);assert!(cap(RepresentationKind::Point,Request{output:Some(b),..r.clone()}).is_ok());
        for (index,value) in [(0,f64::NAN),(3,f64::INFINITY),(11,-1.),(12,0.),(0,1e9)]{let mut bad=b;bad[index]=value;
            assert_eq!(cap(RepresentationKind::Point,Request{output:Some(bad),..r.clone()}),Err(CapabilityError::UnsupportedOptions));}
        if kind==Kind::Sphere{let mut bad=b;bad[13]=1.;assert_eq!(cap(RepresentationKind::Point,Request{output:Some(bad),..r.clone()}),Err(CapabilityError::UnsupportedOptions));}
    }
    assert!(operation_capability(OperationId::Inspect,RepresentationKind::Point,OperationOptions::InspectDescriptor).is_ok());
    assert!(operation_capability(OperationId::Boolean,RepresentationKind::Point,OperationOptions::NativeSolid(request(Kind::Box,Mode::Corners))).is_err());
}
#[test]
fn actual_sessions_snapshot_modes_frame_and_owned_constraint_options(){
    use openmatrix9_rust::solid::Session;
    for (kind,names) in [(Kind::Box,vec!["Diagonal","3Point","Vertical","Center","Cube"]),
        (Kind::Sphere,vec!["Center","Vertical","2Point","3Point","4Point","FitPoints","AroundCurve","Tangent"])]{
        for name in names{let mut s=Session::new(kind);s.input(name).unwrap();
            let r=s.capability_request(Stage::Capture).unwrap();assert_eq!(r.kind,kind);
            assert!(cap(RepresentationKind::Point,r).is_ok());s.cancel();assert!(s.capability_request(Stage::Capture).is_err());}
    }
    let mut s=Session::new(Kind::Sphere);let axes=[[0.,1.,0.],[-1.,0.,0.],[0.,0.,1.]];
    s.set_frame([10.,20.,30.],axes).unwrap();s.input("Tangent").unwrap();
    s.accept_reference([1.,0.,0.],[0.,1.,0.]).unwrap();s.input("Point").unwrap();s.point([0.,1.,0.]).unwrap();
    let old=s.capability_request(Stage::Capture).unwrap();assert_eq!(old.frame_origin,[10.,20.,30.]);assert_eq!(old.axes,axes);
    assert_eq!(old.constraint_types,vec![false,true]);assert_eq!(old.point_count,2);
    s.input("Undo").unwrap();assert_eq!(s.capability_request(Stage::Capture).unwrap().constraint_types,vec![false]);
    assert_eq!(old.constraint_types,vec![false,true]);
}
#[test]
fn actual_signed_box_and_sphere_outputs_share_policy_without_mutating_preview(){
    use openmatrix9_rust::solid::{Session,Effect};
    let mut s=Session::new(Kind::Box);s.input("Diagonal").unwrap();s.point([0.;3]).unwrap();s.point([-2.,-3.,0.]).unwrap();
    let before=s.capability_request(Stage::Preview).unwrap();let preview=s.preview([0.,0.,-4.],false).unwrap();
    assert!(cap(RepresentationKind::Point,Request{output:Some(preview),..before.clone()}).is_ok());
    assert_eq!(s.capability_request(Stage::Preview).unwrap(),before);
    assert_eq!(s.input("-4").unwrap(),Effect::Commit);
    let r=s.capability_request(Stage::Construct).unwrap();let b=r.output.unwrap();
    assert_eq!(&b[..3],&[-2.,-3.,-4.]);assert_eq!(&b[12..],&[2.,3.,4.]);assert!(cap(RepresentationKind::Point,r).is_ok());
    let mut s=Session::new(Kind::Sphere);s.point([0.;3]).unwrap();assert_eq!(s.input("2").unwrap(),Effect::Commit);
    assert!(cap(RepresentationKind::Point,s.capability_request(Stage::Construct).unwrap()).is_ok());
}
