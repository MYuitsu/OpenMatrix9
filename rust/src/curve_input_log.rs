// SPDX-License-Identifier: LGPL-2.1-or-later
//! Persistent owned accepted-value branch; checkpoints share immutable nodes.
use crate::{builder_history::Json,prepared_result::{map,string,encode_owned}};
use std::sync::Arc;
pub(super) const MAX_EVENTS:usize=8192;
const MAX_BYTES:usize=8*1024*1024;
const MAX_NODES:usize=500_000;
struct NativeBatch{refs:Vec<(Json,Arc<Json>)>}
struct Node{event:Json,native:Option<NativeBatch>,previous:Option<Arc<Node>>}
impl Drop for Node{
    fn drop(&mut self){
        // An option-only session can be long. Do not recursively destroy its
        // unique chain on Cancel; shared checkpoint branches retain their owner.
        let mut previous=self.previous.take();
        while let Some(owner)=previous{
            match Arc::try_unwrap(owner){Ok(mut node)=>previous=node.previous.take(),Err(_)=>break}
        }
    }
}
#[derive(Default,Clone)]
pub(super) struct InputLog{tail:Option<Arc<Node>>,count:usize,bytes:usize,nodes:usize}
pub(super) struct Prepared{event:Json,bytes:usize,nodes:usize,native:Option<NativeBatch>}
fn nodes(v:&Json)->Result<usize,String>{
    match v{
        Json::Number(n) if !n.is_finite()=>Err("NonFiniteGeometry: Curve accepted input contains a nonfinite number".into()),
        Json::Array(a)=>a.iter().try_fold(1usize,|count,v|count.checked_add(nodes(v)?).ok_or_else(||"InvalidBasis: Curve accepted input node count overflow".into())),
        Json::Object(o)=>o.values().try_fold(1usize,|count,v|count.checked_add(nodes(v)?).ok_or_else(||"InvalidBasis: Curve accepted input node count overflow".into())),
        _=>Ok(1),
    }
}
impl InputLog{
    pub(super) fn native_constraint(&self,index:usize)->Result<(&Json,&Json),u32>{
        let mut node=self.tail.as_deref();let mut found=None;
        while let Some(current)=node{
            if let Some(batch)=&current.native{for(reference,snapshot)in &batch.refs{
                if matches!(reference,Json::Object(o)if matches!(o.get("role"),Some(Json::String(s))if s=="tangent")&&o.get("role_index")==Some(&Json::Number(index as f64))){
                    if found.is_some(){return Err(1);}found=Some((reference,snapshot.as_ref()));
                }
            }}
            node=current.previous.as_deref();
        }
        found.ok_or(1)
    }
    pub(super) fn latest_native_role(&self,role:&str)->Option<(&Json,&Json)>{
        let mut node=self.tail.as_deref();
        while let Some(current)=node{
            if let Some(batch)=&current.native{for(reference,snapshot)in batch.refs.iter().rev(){
                if matches!(reference,Json::Object(o)if matches!(o.get("role"),Some(Json::String(s))if s==role)){return Some((reference,snapshot.as_ref()));}
            }}
            node=current.previous.as_deref();
        }
        None
    }
    pub fn admit_count(&self)->Result<(),String>{
        if self.count>=MAX_EVENTS{Err("InvalidBasis: Curve accepted input event budget exceeded; Undo or restart the command".into())}else{Ok(())}
    }
    pub fn prepare(&self,event:Json)->Result<Prepared,String>{
        self.admit_count()?;
        // encode_owned bounds its output tree but does not validate numbers.
        // Even fields ignored by the native solver are retained in this log.
        let nodes=nodes(&event)?;
        let bytes=encode_owned(&event).map_err(|_|"InvalidBasis: Curve accepted input must be bounded finite data")?.len()+1;
        if self.bytes.checked_add(bytes).is_none_or(|n|n>MAX_BYTES)||self.nodes.checked_add(nodes).is_none_or(|n|n>MAX_NODES){
            return Err("InvalidBasis: Curve accepted input byte/node budget exceeded; Undo or restart the command".into());
        }
        Ok(Prepared{event,bytes,nodes,native:None})
    }
    pub fn prepare_native(&self,event:Json,refs:Vec<Json>,snapshots:Vec<Json>)->Result<Prepared,String>{
        let mut prepared=self.prepare(event)?;
        let mut owned:Vec<Arc<Json>>=Vec::with_capacity(snapshots.len());
        for value in snapshots{
            let mut node=self.tail.as_deref();let mut shared=owned.iter().find(|s|s.as_ref()==&value).cloned();
            while let Some(current)=node{
                if shared.is_some(){break;}
                if let Some(batch)=&current.native{shared=batch.refs.iter().find(|(_,s)|s.as_ref()==&value).map(|(_,s)|s.clone());}
                if shared.is_some(){break;}node=current.previous.as_deref();
            }
            let value=match shared{Some(value)=>value,None=>{
                prepared.nodes=prepared.nodes.checked_add(nodes(&value)?).ok_or("InvalidBasis: Curve native source node overflow")?;
                prepared.bytes=prepared.bytes.checked_add(encode_owned(&value).map_err(|_|"InvalidBasis: Curve native source serialization budget")?.len()+1).ok_or("InvalidBasis: Curve native source byte overflow")?;
                Arc::new(value)
            }};
            owned.push(value);
        }
        let mut native=NativeBatch{refs:Vec::with_capacity(refs.len())};
        for value in refs{
            prepared.nodes=prepared.nodes.checked_add(nodes(&value)?.checked_add(2).ok_or("InvalidBasis: Curve native source reference node overflow")?).ok_or("InvalidBasis: Curve native source reference node overflow")?;
            prepared.bytes=prepared.bytes.checked_add(encode_owned(&value).map_err(|_|"InvalidBasis: Curve native source reference serialization budget")?.len()+64).ok_or("InvalidBasis: Curve native source reference byte overflow")?;
            let Json::Object(o)=&value else{return Err("InvalidBasis: Curve native source reference must be an object".into());};
            let Some(Json::Number(index))=o.get("input_snapshot_index")else{return Err("InvalidBasis: Curve native source snapshot index missing".into());};
            let snapshot=owned.get((*index as usize).checked_sub(1).ok_or("InvalidBasis: Curve native source snapshot index")?).ok_or("InvalidBasis: Curve native source snapshot index")?.clone();
            native.refs.push((value,snapshot));
        }
        if self.bytes.checked_add(prepared.bytes).is_none_or(|n|n>MAX_BYTES)||self.nodes.checked_add(prepared.nodes).is_none_or(|n|n>MAX_NODES){
            return Err("InvalidBasis: Curve accepted input/native source byte/node budget exceeded; Undo or restart the command".into());
        }
        prepared.native=Some(native);Ok(prepared)
    }
    pub fn append(mut self,mut prepared:Prepared,context_after:u64,revision_after:u64)->Self{
        if let Json::Object(o)=&mut prepared.event{o.insert("circle_context_after".into(),string(&context_after.to_string()));o.insert("admission_revision_after".into(),string(&revision_after.to_string()));}
        self.tail=Some(Arc::new(Node{event:prepared.event,native:prepared.native,previous:self.tail.take()}));
        self.count+=1;self.bytes+=prepared.bytes;self.nodes+=prepared.nodes;self
    }
    pub fn json(&self)->Json{
        let mut events=Vec::with_capacity(self.count);let mut node=self.tail.as_deref();
        while let Some(current)=node{events.push(current.event.clone());node=current.previous.as_deref();}
        events.reverse();Json::Array(events)
    }
    pub fn native_json(&self)->(Json,Json){
        let mut chain=Vec::with_capacity(self.count);let mut node=self.tail.as_deref();
        while let Some(current)=node{chain.push(current);node=current.previous.as_deref();}
        let mut refs=Vec::new();let mut snapshots:Vec<Arc<Json>>=Vec::new();
        for(index,current)in chain.iter().rev().enumerate(){if let Some(batch)=&current.native{for(value,snapshot)in &batch.refs{
            let slot=snapshots.iter().position(|s|Arc::ptr_eq(s,snapshot)).unwrap_or_else(||{snapshots.push(snapshot.clone());snapshots.len()-1});
            let mut value=value.clone();if let Json::Object(o)=&mut value{
                o.insert("input_snapshot_index".into(),Json::Number((slot+1)as f64));
                o.insert("accepted_event_index".into(),Json::Number(index as f64));
            }refs.push(value);
        }}}
        (Json::Array(refs),Json::Array(snapshots.into_iter().map(|s|s.as_ref().clone()).collect()))
    }
}
pub(super) fn vector(p:&[f64])->Json{Json::Array(p.iter().map(|&n|Json::Number(n)).collect())}
pub(super) fn points(p:&[[f64;3]])->Json{Json::Array(p.iter().map(|p|vector(p)).collect())}
impl super::CurveSession{
    pub(super) fn input_event(&self,kind:&str,fields:impl IntoIterator<Item=(&'static str,Json)>)->Json{
        let frame=self.frame.unwrap_or_default();let previous=if self.name=="Circle"{self.circle_previous}else{self.points.last().copied()};
        let mut event=map([
            ("kind",string(kind)),("input_mm_per_unit",Json::Number(self.input_mm_per_unit)),
            ("frame_origin",vector(&frame.origin())),("frame_axes",points(&frame.axes())),
            ("previous_point_world",previous.map_or(Json::Other,|p|vector(&p))),
            ("pending_option",self.awaiting_option.as_ref().map_or(Json::Other,|s|string(s))),
            ("normalized_point_count_before",Json::Number(self.points.len() as f64)),
            ("circle_context_before",string(&self.circle_input_context_key().to_string())),
            // Reserve the longest possible decimal identity before accepting.
            ("circle_context_after",string(&u64::MAX.to_string())),
            ("admission_revision_before",string(&self.native_input_revision().to_string())),
            ("admission_revision_after",string(&u64::MAX.to_string())),
            ("circle_size",if self.name=="Circle"{string(match self.circle_options.size{
                crate::circle::Size::Radius=>"Radius",crate::circle::Size::Diameter=>"Diameter",
                crate::circle::Size::Circumference=>"Circumference",crate::circle::Size::Area=>"Area"})}else{Json::Other}),
        ]);
        if let Json::Object(o)=&mut event{for(key,value)in fields{o.insert(key.into(),value);}}
        event
    }
    pub(super) fn text_event(&self,text:&str)->Result<Json,String>{
        self.accepted_input_log.admit_count()?;
        if text.len()>1024{return Err("InvalidBasis: Curve input text exceeds 1024 bytes".into());}
        let previous=if self.name=="Circle"{self.circle_previous}else{self.points.last().copied()};
        // Bare0 is an origin only in point entry. Pending scalar options and
        // length constraints retain their own zero/domain validation.
        let origin=text.trim()=="0"&&self.awaiting_option.is_none()&&!self.awaiting_length;
        let resolved=if origin||super::classify_numeric(text)==super::NumericSyntax::Point{
            super::PointInput::parse(text,|s|self.length_value(s)).and_then(|p|p.resolve(self.frame.unwrap_or_default(),previous)).ok().map(|p|p.world)
        }else{None};
        Ok(self.input_event("text",[("text",string(text)),("resolved_point_world",resolved.map_or(Json::Other,|p|vector(&p)))]))
    }
    pub(super) fn with_input_record(&mut self,event:Json,context_after:Option<u64>,accept:impl FnOnce(&mut Self)->Result<super::Effect,String>)->Result<super::Effect,String>{
        let parent=self.accepted_input_log.clone();let prepared=parent.prepare(event)?;
        let next=self.next_native_input_revision()?;
        let effect=accept(self)?;
        if effect!=super::Effect::Cancelled{self.native_input_revision=next;self.accepted_input_log=parent.append(prepared,context_after.unwrap_or_else(||self.circle_input_context_key()),next);}
        Ok(effect)
    }
}
