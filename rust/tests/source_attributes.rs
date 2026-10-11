use openmatrix9_rust::source_attributes::{Input,Material,SourceAttributes};
use openmatrix9_rust::source_attributes_ffi::{AttributesInput,Text,om9_source_attributes_json};
fn input<'a>(materials:&'a [Material<'a>])->Input<'a> {Input{text:["doc","Object","Part::Feature","Nhãn \"đá\"\n","ghi chú","Gui::ViewProviderPart"],known:31,values:1,materials}}
fn material()->Material<'static> {Material{colors:[[0.1,0.2,0.3,1.];4],shininess:0.5,transparency:0.25,text:["uuid","image-data","texture.png"]}}
#[test]
fn attributes_own_unicode_labels_and_full_ordered_material_values() {
    let mut m=[material()];let data=SourceAttributes::capture(input(&m)).unwrap();m[0].shininess=0.9;assert_eq!(m[0].shininess,0.9);
    let json=data.json().unwrap();assert!(json.contains("Nhãn \\\"đá\\\"\\n"));assert!(json.contains("\"shininess\":0.5"));assert!(json.contains("\"image\":\"image-data\""));assert!(json.contains("\"selectable\":false"));assert!(json.contains("\"other_app_properties\":\"uninspected\""));
}
#[test]
fn invalid_or_oversized_metadata_rejects_before_owning() {
    let mut m=material();m.colors[0][0]=f64::NAN;assert!(SourceAttributes::capture(input(&[m])).is_err());
    m=material();m.transparency=2.;assert!(SourceAttributes::capture(input(&[m])).is_err());
    let mut i=input(&[]);i.values=16;assert!(SourceAttributes::capture(i).is_err());
    let text="x".repeat(65537);let mut i=input(&[]);i.text[3]=&text;assert!(SourceAttributes::capture(i).is_err());
    assert!(SourceAttributes::capture(input(&vec![material();4097])).is_err());
}
#[test]
fn abi_rejects_invalid_utf8_envelopes_and_short_output_atomically() {
    let texts=["doc","Object","Part::Feature","đá","",""];
    let text=|s:&str|Text{bytes:s.as_ptr(),len:s.len()};
    let mut i=AttributesInput{text:texts.map(text),known:1,values:1,materials:std::ptr::null(),material_count:0};
    let call=|i,out,cap|unsafe{om9_source_attributes_json(i,out,cap)};
    let q=call(i,std::ptr::null_mut(),0);assert_eq!(q.status,0);
    let mut bytes=vec![0x5a;q.needed+1];assert_eq!(call(i,bytes.as_mut_ptr(),q.needed).status,0);assert_eq!(bytes[q.needed],0x5a);
    let before=bytes.clone();assert_eq!(call(i,bytes.as_mut_ptr(),q.needed-1).status,1001);assert_eq!(bytes,before);
    let bad=[0xff];i.text[3]=Text{bytes:bad.as_ptr(),len:1};assert_ne!(call(i,bytes.as_mut_ptr(),q.needed).status,0);assert_eq!(bytes,before);
    i.text[3]=Text{bytes:std::ptr::null(),len:1};assert_ne!(call(i,bytes.as_mut_ptr(),q.needed).status,0);assert_eq!(bytes,before);
    i.material_count=usize::MAX;i.materials=1usize as _;assert_ne!(call(i,bytes.as_mut_ptr(),q.needed).status,0);assert_eq!(bytes,before);
}
#[test]
fn unavailable_native_appearance_is_unknown_not_an_empty_inspection() {
    let mut i=input(&[]);i.known=1|4|8;
    let json=SourceAttributes::capture(i).unwrap().json().unwrap();
    assert!(json.contains("\"shape_appearance\":null"));
}
#[test]
fn owned_budget_counts_rust_string_headers_before_clone() {
    let text="x".repeat(65536);
    let mut materials=vec![material();4096];
    for m in &mut materials {m.text=["","",""];}
    materials[0].text[0]=&text;
    let mut i=input(&materials);i.text[3]=&text;i.text[4]=&text;
    // 3*64KiB plus actual owned material/root headers exceed 1MiB. ABI
    // descriptors are smaller and must not be mistaken for retained Rust data.
    assert!(SourceAttributes::capture(i).is_err());
}
#[test]
fn material_abi_owns_ordered_values_and_rejects_invalid_nested_envelopes() {
    use openmatrix9_rust::source_attributes_ffi::NativeMaterial;
    let make_text=|s:&str|Text{bytes:s.as_ptr(),len:s.len()};
    let mut materials=[NativeMaterial{colors:[[0.1,0.2,0.3,1.];4],shininess:0.2,transparency:0.3,text:[make_text("first"),make_text("pixels"),make_text("path")]}];
    let mut i=AttributesInput{text:["doc","Object","Part::Feature","Label","","ViewProvider"].map(make_text),known:1|8|16,values:0,materials:materials.as_ptr(),material_count:1};
    let call=|i,out,cap|unsafe{om9_source_attributes_json(i,out,cap)};
    let query=call(i,std::ptr::null_mut(),0);assert_eq!(query.status,0);
    let mut bytes=vec![0x5a;query.needed+1];assert_eq!(call(i,bytes.as_mut_ptr(),query.needed).status,0);
    let before=bytes.clone();assert!(std::str::from_utf8(&bytes[..query.needed]).unwrap().contains("\"uuid\":\"first\""));
    materials[0].colors[0][0]=f64::INFINITY;
    assert_ne!(call(i,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,before);
    materials[0].colors[0][0]=0.1;materials[0].text[1]=Text{bytes:std::ptr::null(),len:1};
    assert_ne!(call(i,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,before);
    materials[0].text[1]=Text{bytes:1usize as _,len:65537};
    assert_ne!(call(i,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,before);
    i.materials=(materials.as_ptr() as *const u8).wrapping_add(1) as _;
    assert_ne!(call(i,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,before);
    let mut label=String::from("owned label");let mut first=material();first.text=["first","image","path"];let mut second=material();second.text=["second","image2","path2"];
    let ordered=[first,second];let mut input=input(&ordered);input.text[3]=&label;
    let owned=SourceAttributes::capture(input).unwrap();label.clear();let json=owned.json().unwrap();
    assert!(json.contains("owned label")&&json.find("\"uuid\":\"first\"").unwrap()<json.find("\"uuid\":\"second\"").unwrap());
}
