// SPDX-License-Identifier: LGPL-2.1-or-later
//! Combinatorial surface observations, distinct from native geometric validity.
use crate::topology_snapshot::{Error,Face,Loop,Snapshot,Use};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Facts {
 pub vertex_manifold:bool,
 pub face_oriented:bool,
 pub vertex_count:usize,
 pub boundary_vertex_count:usize,
}
pub fn capture(faces:&[Face],loops:&[Loop],uses:&[Use])->Result<Facts,Error>{
 let snapshot=Snapshot::capture(faces,loops,uses)?;Ok(observe(&snapshot))
}
pub(crate) fn observe(snapshot:&Snapshot)->Facts{
 // The snapshot has validated counts, ranges, directed identities and loop
 // connectivity. At most two graph incidences are added per nondegenerate use.
 // Distinguish both endpoint germs even when an edge starts/ends at one vertex.
 type Germ=(u64,u32);
 let mut links:BTreeMap<u64,BTreeMap<Germ,Vec<Germ>>>=BTreeMap::new();
 let mut directions:BTreeMap<u64,Vec<bool>>=BTreeMap::new();
 let mut collapsed_vertices=BTreeSet::new();
 for u in &snapshot.uses{links.entry(u.start_vertex).or_default();links.entry(u.end_vertex).or_default();}
 for face in &snapshot.faces{
  for l in &snapshot.loops[face.first_loop..face.first_loop+face.loop_count]{
   let uses=&snapshot.uses[l.first_use..l.first_use+l.use_count];
   let chain:Vec<&Use>=uses.iter().filter(|u|u.degenerate==0).collect();
   // A wholly collapsed loop still belongs to its incident vertex. It cannot
   // disappear merely because another face supplies a valid path/cycle there.
   // Mixed loops retain ordinary collapsed poles through their seam germs.
   if chain.is_empty(){for u in uses{collapsed_vertices.insert(u.start_vertex);collapsed_vertices.insert(u.end_vertex);}}
   for u in &chain{directions.entry(u.edge_id).or_default().push((u.orientation==1)==(face.orientation==1));}
   for (a,b) in chain.iter().zip(chain.iter().cycle().skip(1)){
    let incoming=(a.edge_id,u32::from(a.orientation==1));
    let outgoing=(b.edge_id,u32::from(b.orientation!=1));
    let graph=links.entry(a.end_vertex).or_default();
    graph.entry(incoming).or_default().push(outgoing);
    graph.entry(outgoing).or_default().push(incoming);
   }
  }
 }
 let mut facts=Facts{vertex_manifold:!snapshot.faces.is_empty(),face_oriented:!snapshot.faces.is_empty(),vertex_count:links.len(),boundary_vertex_count:0};
 for (vertex,graph) in &links{
  let endpoints=graph.values().filter(|v|v.len()==1).count();
  let mut valid=!collapsed_vertices.contains(vertex)&&!graph.is_empty()&&graph.values().all(|v|(1..=2).contains(&v.len()))&&(endpoints==0||endpoints==2);
  let mut unseen:BTreeSet<Germ>=graph.keys().copied().collect();
  if let Some(seed)=unseen.pop_first(){
   let mut work=vec![seed];
   while let Some(node)=work.pop(){for next in &graph[&node]{if unseen.remove(next){work.push(*next);}}}
   valid &= unseen.is_empty();
  }
  if valid&&endpoints==2{facts.boundary_vertex_count+=1;}
  facts.vertex_manifold &= valid;
 }
 facts.face_oriented &= directions.values().all(|s|s.len()==1||(s.len()==2&&s[0]!=s[1]));
 facts
}
