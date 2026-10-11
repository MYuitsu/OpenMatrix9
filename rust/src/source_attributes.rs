// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded independent identity/appearance data. Uninspected attributes stay explicit.
use crate::support_snapshot::{Json,SerializedLimit,numbers};
use std::fmt::{self,Write};
pub const MAX_MATERIALS:usize=4096;
pub const MAX_TEXT_BYTES:usize=65536;
pub const MAX_OWNED_BYTES:usize=1024*1024;
#[derive(Clone,Copy,Debug)]
pub struct Material<'a> {pub colors:[[f64;4];4],pub shininess:f64,pub transparency:f64,pub text:[&'a str;3]}
#[derive(Clone,Copy,Debug)]
pub struct Input<'a> {pub text:[&'a str;6],pub known:u32,pub values:u32,pub materials:&'a [Material<'a>]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum AttributeError {Flags,Identity,Text,Material,Budget}
#[derive(Clone,Debug)]
pub struct SourceAttributes {text:[String;6],known:u32,values:u32,materials:Vec<OwnedMaterial>}
#[derive(Clone,Debug)]
struct OwnedMaterial {colors:[[f64;4];4],shininess:f64,transparency:f64,text:[String;3]}
pub fn counts_allowed(materials:usize,text_bytes:usize)->bool {
    materials<=MAX_MATERIALS && materials.checked_mul(std::mem::size_of::<OwnedMaterial>())
        .and_then(|n|n.checked_add(std::mem::size_of::<SourceAttributes>()))
        .and_then(|n|n.checked_add(text_bytes)).is_some_and(|n|n<=MAX_OWNED_BYTES)
}
// Bit16 means the native ShapeAppearance property was actually inspected.
pub(crate) fn flags_valid(known:u32,values:u32)->bool {known&!31==0 && values&!(known&3)==0 && (known&16==0||known&8!=0)}
fn quoted(j:&mut Json,text:&str)->fmt::Result {
    j.write_char('"')?;
    for c in text.chars() {match c {
        '"'=>j.write_str("\\\"")?, '\\'=>j.write_str("\\\\")?, '\n'=>j.write_str("\\n")?,
        '\r'=>j.write_str("\\r")?, '\t'=>j.write_str("\\t")?,
        c if c<' '=>write!(j,"\\u{:04x}",c as u32)?,c=>j.write_char(c)?
    }}
    j.write_char('"')
}
impl SourceAttributes {
    /// Decode persisted native observations through the same capture policy.
    /// JSON owns its text; the borrowed material view lasts only for this call.
    pub(crate) fn decode(value:&crate::builder_history::Json)->Result<Self,AttributeError>{
        use crate::builder_history::Json as Value;
        let fail=AttributeError::Flags;
        let Value::Object(o)=value else{return Err(fail);};
        let get=|key:&str|o.get(key).ok_or(fail);
        fn text(v:&Value)->Result<&str,AttributeError>{if let Value::String(s)=v{Ok(s.as_str())}else{Err(AttributeError::Text)}}
        let num=|v:&Value|if let Value::Number(n)=v{Ok(*n)}else{Err(AttributeError::Material)};
        if !matches!(get("schema_version")?,Value::Number(1.)){return Err(fail);}
        let mut known=0;let mut values=0;
        for (bit,key) in [(1,"visibility"),(2,"selectable")] {match get(key)?{
            Value::Other=>{},Value::Boolean(b)=>{known|=bit;if *b{values|=bit;}},_=>return Err(fail)}}
        let mut fields=["";6];
        for(i,key)in ["document_uid","object_name","native_type","label","label2","view_provider_type"].iter().enumerate(){
            let v=get(key)?;if i>=4&&matches!(v,Value::Other){continue;}
            fields[i]=text(v)?;if i==4{known|=4;}if i==5{known|=8;}
        }
        let mut materials=Vec::new();
        match get("shape_appearance")?{
            Value::Other=>{},Value::Array(rows)=>{
                if !counts_allowed(rows.len(),0){return Err(AttributeError::Budget);}
                known|=16;materials.reserve(rows.len());
                for row in rows{
                    let Value::Object(m)=row else{return Err(fail);};
                    let get=|key:&str|m.get(key).ok_or(fail);
                    let mut colors=[[0.;4];4];
                    for(i,key)in ["ambient","diffuse","specular","emissive"].iter().enumerate(){
                        let Value::Array(a)=get(key)? else{return Err(fail);};if a.len()!=4{return Err(fail);}
                        for(j,v)in a.iter().enumerate(){colors[i][j]=num(v)?;}
                    }
                    materials.push(Material{colors,shininess:num(get("shininess")?)?,transparency:num(get("transparency")?)?,
                        text:[text(get("uuid")?)?,text(get("image")?)?,text(get("image_path")?)?]});
                }
            },_=>return Err(fail)
        }
        let Value::Object(coverage)=get("coverage")? else{return Err(fail);};
        for key in ["other_app_properties","rhino_user_data"]{
            if text(coverage.get(key).ok_or(fail)?)?!="uninspected"{return Err(fail);}
        }
        Self::capture(Input{text:fields,known,values,materials:&materials})
    }
    pub fn capture(i:Input<'_>)->Result<Self,AttributeError>{
        if !flags_valid(i.known,i.values){return Err(AttributeError::Flags);}
        if !counts_allowed(i.materials.len(),0){return Err(AttributeError::Budget);}
        if i.text[..3].iter().any(|s|s.is_empty()||s.contains('\0')){return Err(AttributeError::Identity);}
        if (i.known&4==0&&!i.text[4].is_empty()) || (i.known&8==0&&!i.text[5].is_empty()) || (i.known&16==0&&!i.materials.is_empty()) {
            return Err(AttributeError::Flags);
        }
        let mut text_bytes=0usize;
        for s in i.text.iter().copied().chain(i.materials.iter().flat_map(|m|m.text)) {
            if s.len()>MAX_TEXT_BYTES{return Err(AttributeError::Text);}
            text_bytes=text_bytes.checked_add(s.len()).ok_or(AttributeError::Budget)?;
            if !counts_allowed(i.materials.len(),text_bytes){return Err(AttributeError::Budget);}
        }
        for m in i.materials {
            if m.colors.iter().flatten().chain([&m.shininess,&m.transparency])
                .any(|v|!v.is_finite()||!(0. ..=1.).contains(v)){return Err(AttributeError::Material);}
        }
        Ok(Self{text:i.text.map(str::to_owned),known:i.known,values:i.values,
            materials:i.materials.iter().map(|m|OwnedMaterial{colors:m.colors,shininess:m.shininess,
                transparency:m.transparency,text:m.text.map(str::to_owned)}).collect()})
    }
    pub fn json(&self)->Result<String,SerializedLimit>{
        let mut j=Json(String::new());
        let result=(||{
            j.write_str("{\"schema_version\":1")?;
            for (index,(key,value)) in ["document_uid","object_name","native_type","label","label2","view_provider_type"].iter().zip(self.text.iter()).enumerate() {
                write!(j,",\"{key}\":")?;
                if (index==4&&self.known&4==0)||(index==5&&self.known&8==0){j.write_str("null")?;}else{quoted(&mut j,value)?;}
            }
            for (bit,key) in [(1,"visibility"),(2,"selectable")] {
                write!(j,",\"{key}\":")?;
                if self.known&bit==0{j.write_str("null")?;}else{write!(j,"{}",self.values&bit!=0)?;}
            }
            j.write_str(",\"shape_appearance\":")?;
            if self.known&16==0 {j.write_str("null")?;}else{
            j.write_char('[')?;
            for (i,m) in self.materials.iter().enumerate(){if i!=0{j.write_char(',')?;}
                j.write_char('{')?;
                for (index,key) in ["ambient","diffuse","specular","emissive"].iter().enumerate(){
                    if index!=0{j.write_char(',')?;}write!(j,"\"{key}\":")?;numbers(&mut j,&m.colors[index])?;
                }
                write!(j,",\"shininess\":{},\"transparency\":{}",m.shininess,m.transparency)?;
                for (key,value) in ["uuid","image","image_path"].iter().zip(&m.text){write!(j,",\"{key}\":")?;quoted(&mut j,value)?;}
                j.write_char('}')?;
            }
            j.write_char(']')?;
            }
            j.write_str(",\"coverage\":{\"other_app_properties\":\"uninspected\",\"rhino_user_data\":\"uninspected\"}}")
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
