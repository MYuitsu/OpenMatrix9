use openmatrix9_rust::reference_path::{Plan,Step};
use openmatrix9_rust::reference_path_ffi::{Summary,om9_reference_path_parse};
#[test]
fn collapsed_array_aliases_are_bounded_before_native_numeric_resolution(){
 use openmatrix9_rust::reference_path::array_index;
 for (token,want) in [("1",Some(1)),("001",Some(1)),("A_i1",Some(1)),("$A_i0002",Some(2)),("A_middle_i2",Some(2)),("B_i1",None),("$1",None),("A_i",None),("Member",None)]{assert_eq!(array_index("A",token,true).unwrap(),want, "{token}");}
 for token in ["2047","A_i2047","A_i9999999999999999999999","A_i1\0"]{assert!(array_index("A",token,true).is_err(),"{token}");}
 assert!(array_index("A",&"x".repeat(4097),true).is_err());
 use openmatrix9_rust::reference_path_ffi::{AliasSummary,om9_reference_array_index};
 assert_eq!(std::mem::size_of::<AliasSummary>(),16);
 let call=|o,n,t,m|unsafe{om9_reference_array_index(o,n,t,m,1)};
 let result=call(b"A".as_ptr(),1,b"A_i2".as_ptr(),4);assert_eq!((result.status,result.matched,result.index,result.reserved),(0,1,2,0));
 assert_eq!(call(b"A".as_ptr(),1,b"Member".as_ptr(),6).matched,0);
 for (p,n) in [(std::ptr::null(),1),(1usize as _,4097),(usize::MAX as _,2)]{assert_ne!(call(p,n,b"A_i2".as_ptr(),4).status,0);assert_ne!(call(b"A".as_ptr(),1,p,n).status,0);}
 assert_ne!(call(b"A".as_ptr(),1,b"\xff".as_ptr(),1).status,0);
}
#[test]
fn physical_element_names_and_labels_are_not_collapsed_indices(){
 use openmatrix9_rust::reference_path::array_index;
 for token in ["A_i1","$A_i1","A_middle_i2"]{assert_eq!(array_index("A",token,false).unwrap(),None,"{token}");}
 assert_eq!(array_index("A","001",false).unwrap(),Some(1));
 assert!(array_index("A","9999999999999999999999",false).is_err());
}
#[test]
fn native_array_mode_is_checked_before_alias_classification(){
 use openmatrix9_rust::reference_path_ffi::om9_reference_array_index;
 let call=|token:&[u8],mode|unsafe{om9_reference_array_index(b"A".as_ptr(),1,token.as_ptr(),token.len(),mode)};
 assert_eq!(call(b"$A_i1",0).matched,0);
 assert_eq!((call(b"001",0).status,call(b"001",0).index),(0,1));
 assert_ne!(call(b"A_i1",2).status,0);
}
#[test]
fn native_path_plan_owns_ordered_names_labels_indices_and_selector(){
 let mut bytes=b"Group.$Member label.001.Face3".to_vec();let plan=Plan::parse(&bytes).unwrap();bytes.fill(b'x');
 assert_eq!(plan.path(),"Group.$Member label.001.Face3");assert_eq!(plan.selector(),"Face3");assert_eq!(plan.selector_start(),24);
 assert_eq!(plan.steps(),&[Step{start:0,len:5,kind:0,index:0},Step{start:6,len:13,kind:1,index:0},Step{start:20,len:3,kind:2,index:1}]);
 assert_eq!(Plan::parse(b"").unwrap().steps(),&[]);assert_eq!(Plan::parse(b"Face1").unwrap().selector(),"Face1");assert_eq!(Plan::parse(b"Group.").unwrap().selector(),"");
 let unicode=Plan::parse("$Mặt.2.".as_bytes()).unwrap();assert_eq!(unicode.steps()[0].len,6);assert_eq!(unicode.steps()[1].index,2);
}
#[test]
fn malformed_reference_syntax_and_envelopes_reject_before_ownership(){
 for bytes in [&b"."[..],&b"Group..Member."[..],&b"$."[..],&b"Group.\0"[..],&b"\xff."[..],&b"9999999999999999999999999."[..],&b"2047."[..]]{assert!(Plan::parse(bytes).is_err(),"{bytes:?}");}
 assert!(Plan::parse(&vec![b'x';4097]).is_err());assert!(Plan::parse("a.".repeat(65).as_bytes()).is_err());assert_eq!(Plan::parse("a.".repeat(64).as_bytes()).unwrap().steps().len(),64);
 assert!(Plan::parse(b"2046.").is_ok());
}
#[test]
fn native_reference_abi_preserves_live_output_on_invalid_and_short_calls(){
 assert_eq!(std::mem::size_of::<Step>(),16);assert_eq!(std::mem::align_of::<Step>(),4);assert_eq!(std::mem::size_of::<Summary>(),16);
 let bytes=b"Group.1.Face3";let sentinel=Step{start:77,len:88,kind:99,index:111};let mut rows=[sentinel;3];
 let call=|p,n,out,cap|unsafe{om9_reference_path_parse(p,n,out,cap)};
 let query=call(bytes.as_ptr(),bytes.len(),std::ptr::null_mut(),0);assert_eq!(query.status,0);assert_eq!(query.count,2);assert_eq!(query.selector_start,8);assert_eq!(query.selector_len,5);
 assert_eq!(call(bytes.as_ptr(),bytes.len(),rows.as_mut_ptr(),1).status,1001);assert_eq!(rows,[sentinel;3]);
 assert_eq!(call(bytes.as_ptr(),bytes.len(),rows.as_mut_ptr(),2).status,0);assert_eq!(rows[0],Step{start:0,len:5,kind:0,index:0});assert_eq!(rows[1],Step{start:6,len:1,kind:2,index:1});assert_eq!(rows[2],sentinel);
 let before=rows;
 for (p,n) in [(std::ptr::null(),1),(1usize as _,4097),(usize::MAX as _,2)]{assert_ne!(call(p,n,rows.as_mut_ptr(),2).status,0);assert_eq!(rows,before);}
 assert_ne!(call(bytes.as_ptr(),bytes.len(),(rows.as_mut_ptr() as *mut u8).wrapping_add(1) as _,2).status,0);assert_eq!(rows,before);
 assert_ne!(call(bytes.as_ptr(),bytes.len(),std::ptr::null_mut(),1).status,0);assert_eq!(rows,before);
 assert_ne!(call(bytes.as_ptr(),bytes.len(),rows.as_mut_ptr(),usize::MAX).status,0);assert_eq!(rows,before);
 assert_ne!(call(b"a..".as_ptr(),3,rows.as_mut_ptr(),2).status,0);assert_eq!(rows,before);
 assert_eq!(call(std::ptr::null(),0,std::ptr::null_mut(),0).status,0);
}
