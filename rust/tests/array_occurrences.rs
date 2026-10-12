use openmatrix9_rust::array_occurrences::{Occurrence,Snapshot,counts_allowed};
use openmatrix9_rust::array_occurrences_ffi::{om9_array_occurrences_json,om9_array_occurrences_counts};
use openmatrix9_rust::core_objects::Stamp;
use openmatrix9_rust::geometry_frame::IDENTITY;
fn stamp(object:u64)->Stamp{Stamp{document:1,object,generation:3,revision:4,epoch:5}}
fn row(index:u64)->Occurrence{Occurrence{index,element_present:0,reserved:0,element:Stamp{document:0,object:0,generation:0,revision:0,epoch:0},definition:stamp(7),object_to_world:IDENTITY}}
#[test]
fn ordered_array_metadata_owns_virtual_and_physical_occurrences(){
 let mut rows=[row(0),row(2)];rows[1].element_present=1;rows[1].element=stamp(9);rows[1].object_to_world[0]=-2.;rows[1].object_to_world[3]=17.;
 let owned=Snapshot::capture(3,&rows).unwrap();rows[1].object_to_world[3]=99.;rows[1].element.object=999;
 assert_eq!(rows[1].element.object,999);
 let json=owned.json().unwrap();assert!(json.contains("\"total_count\":3"));assert!(json.contains("\"native_path\":\"2.\""));assert!(json.contains("\"native_element_ref\":null"));assert!(json.contains("\"object\":\"9\""));assert!(json.contains("\"world_orientation_sign\":-1"));assert!(!json.contains("999"));assert!(!json.contains("99"));
 assert!(Snapshot::capture(3,&[row(1)]).is_ok());
}
#[test]
fn invalid_indices_stamps_flags_frames_and_counts_reject_before_ownership(){
 for (total,count) in [(0,0),(1,0),(3,4),(2048,1),(usize::MAX,1),(1,usize::MAX)]{assert!(!counts_allowed(total,count));}
 assert!(counts_allowed(2047,2047));assert!(Snapshot::capture(3,&[row(0),row(0)]).is_err());assert!(Snapshot::capture(3,&[row(2),row(1)]).is_err());assert!(Snapshot::capture(3,&[row(3)]).is_err());
 let mut bad=row(0);bad.reserved=1;assert!(Snapshot::capture(1,&[bad]).is_err());
 bad=row(0);bad.element_present=2;assert!(Snapshot::capture(1,&[bad]).is_err());
 bad=row(0);bad.element=stamp(1);assert!(Snapshot::capture(1,&[bad]).is_err());
 bad=row(0);bad.element_present=1;assert!(Snapshot::capture(1,&[bad]).is_err());
 for field in 0..5{
  bad=row(0);match field{0=>bad.definition.document=0,1=>bad.definition.object=0,2=>bad.definition.generation=0,3=>bad.definition.revision=0,_=>bad.definition.epoch=0};assert!(Snapshot::capture(1,&[bad]).is_err());
 }
 for value in [f64::NAN,f64::INFINITY,1e151]{bad=row(0);bad.object_to_world[3]=value;assert!(Snapshot::capture(1,&[bad]).is_err());}
 bad=row(0);bad.object_to_world[0]=0.;assert!(Snapshot::capture(1,&[bad]).is_err());
 bad=row(0);bad.object_to_world[12]=1.;assert!(Snapshot::capture(1,&[bad]).is_err());
}
#[test]
fn array_abi_rejections_preserve_live_output_and_no_pointer_is_retained(){
 assert_eq!(std::mem::size_of::<Occurrence>(),224);assert_eq!(std::mem::align_of::<Occurrence>(),8);assert_eq!(std::mem::offset_of!(Occurrence,object_to_world),96);
 assert_eq!(om9_array_occurrences_counts(2047,1),0);assert_ne!(om9_array_occurrences_counts(2048,1),0);
 let rows=[row(0),row(2)];let call=|p,n,total,out,cap|unsafe{om9_array_occurrences_json(p,n,total,out,cap)};
 let query=call(rows.as_ptr(),2,3,std::ptr::null_mut(),0);assert_eq!(query.status,0);assert_eq!(query.reserved,0);
 let mut bytes=vec![0x5a;query.needed+1];assert_eq!(call(rows.as_ptr(),2,3,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes[query.needed],0x5a);
 let before=bytes.clone();assert_eq!(call(rows.as_ptr(),2,3,bytes.as_mut_ptr(),query.needed-1).status,1001);assert_eq!(bytes,before);
 for (p,n,total) in [(std::ptr::null(),1,1),(1usize as _,1,2048),((rows.as_ptr() as *const u8).wrapping_add(1) as _,2,3),(rows.as_ptr(),usize::MAX,usize::MAX),(rows.as_ptr(),2,1)]{
  assert_ne!(call(p,n,total,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,before);
 }
 for mut bad in [row(3),row(0)]{bad.object_to_world[3]=f64::NAN;assert_eq!(call(&bad,1,1,bytes.as_mut_ptr(),query.needed).status,6101);assert_eq!(bytes,before);}
 assert_eq!(call(rows.as_ptr(),2,3,std::ptr::null_mut(),1).status,1002);assert_eq!(call(rows.as_ptr(),2,3,bytes.as_mut_ptr(),usize::MAX).status,1002);assert_eq!(bytes,before);
}
