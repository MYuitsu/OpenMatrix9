use openmatrix9_rust::geometry_contract::*;
fn capability(kind:u32,input:RepresentationKind,flags:u32,mode:u32)->Result<OperationCapability,CapabilityError>{
 operation_capability(if kind>=4{OperationId::Boolean}else{OperationId::NativeEdit},input,OperationOptions::NativeEdit{kind,flags,mode})
}
#[test]
fn native_edit_keeps_existing_backends_and_supported_inputs(){
 use RepresentationKind::*;
 for kind in [1,3] {for input in [Curve,Polycurve,TrimmedFace,Shell] {
  let c=capability(kind,input,4,0).unwrap();assert_eq!(c.backend,Backend::OcctEdit);
  assert!(!c.evidence_id.is_empty()&&c.constraints.contains(&OperationConstraint::ValidateSourceStamp));
 }}
 for input in [Polycurve,Shell,Solid,Brep,Mesh,Instance,Annotation,Retained] {assert_eq!(capability(2,input,4,0).unwrap().backend,Backend::NativeExplodeAdapters);}
 for kind in 4..=7 {for input in [TrimmedFace,Shell,Solid,Brep] {assert_eq!(capability(kind,input,0,0).unwrap().backend,Backend::OcctEdit);}
  assert_eq!(capability(kind,Mesh,0,0).unwrap().backend,Backend::MeshOcctEdit);
 }
}
#[test]
fn native_edit_rejects_unsupported_types_and_mismatching_operation(){
 use RepresentationKind::*;
 for kind in [1,3] {for input in [Point,PointCloud,Solid,Brep,Mesh,Instance,Annotation,Unknown] {assert_eq!(capability(kind,input,4,0),Err(CapabilityError::UnsupportedRepresentation(input)));}}
 for kind in 4..=7 {for input in [Curve,Polycurve,PointCloud,Annotation,Retained,Unknown] {assert_eq!(capability(kind,input,0,0),Err(CapabilityError::UnsupportedRepresentation(input)));}}
 assert_eq!(capability(2,Curve,4,0),Err(CapabilityError::UnsupportedRepresentation(Curve)));
 assert!(operation_capability(OperationId::Boolean,Curve,OperationOptions::NativeEdit{kind:1,flags:4,mode:0}).is_err());
}
#[test]
fn native_edit_options_match_real_switches_fixed_deletion_and_boolean_modes(){
 use RepresentationKind::*;
 for flags in 4..=7 {assert!(capability(3,Curve,flags,0).is_ok());}
 for flags in [0,1,2,3,8,12] {assert_eq!(capability(3,Curve,flags,0),Err(CapabilityError::UnsupportedOptions));}
 for kind in [1,2] {for flags in [0,1,5,6,7,8] {assert_eq!(capability(kind,Polycurve,flags,0),Err(CapabilityError::UnsupportedOptions));}}
 for kind in 4..=7 {for flags in [0,4] {assert!(capability(kind,Solid,flags,0).is_ok());}
  for flags in [1,2,3,5,6,7,8] {assert_eq!(capability(kind,Solid,flags,0),Err(CapabilityError::UnsupportedOptions));}
 }
 for mode in 0..5 {assert!(capability(7,Solid,0,mode).is_ok());}
 assert_eq!(capability(7,Solid,0,5),Err(CapabilityError::UnsupportedOptions));
 assert_eq!(capability(4,Solid,0,1),Err(CapabilityError::UnsupportedOptions));
}
#[test]
fn scalar_abi_and_declaration_never_authorize_unknown_concrete_input(){
 use openmatrix9_rust::native_edit_capability::{om9_native_edit_capability as gate,om9_native_edit_declared as declared};
 assert_eq!(gate(1,2,4,0),4);assert_eq!(gate(2,8,4,0),6);assert_eq!(gate(4,8,0,0),5);
 assert_eq!(gate(1,8,4,0),101);assert_eq!(gate(3,2,0,0),102);
 for kind in 1..=7{assert!(declared(kind,if kind<=3{4}else{0},0));assert_eq!(gate(kind,0,if kind<=3{4}else{0},0),101);}
 for kind in [0,8,u32::MAX]{assert_eq!(gate(kind,2,4,0),100);assert!(!declared(kind,4,0));}
 assert_eq!(gate(2,13,4,0),6);assert_eq!(gate(4,13,0,0),101);
 assert_eq!(gate(7,6,4,u32::MAX),102);assert_eq!(gate(3,2,u32::MAX,0),102);
}
