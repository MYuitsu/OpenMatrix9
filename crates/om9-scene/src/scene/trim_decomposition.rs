// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded UV scanline decomposition for standalone trimmed surfaces.
use om9_core::{Error, Result};

type Point = [f64; 2];
// Match the display topology verifier's 523,776 unordered-pair work limit.
// Memory for the quadratic worst case is admitted before publication.
const MAX_EDGES: usize = om9_core::rhino_semantics::TRIM_DISPLAY_MAX_EDGES;
const SUBDIVISIONS: usize = 11;

fn allocated<T>(count: usize) -> Result<Vec<T>> {
    // An impossible vector layout is structural; only allocator failure is a
    // memory refusal eligible for the existing consent-based retry flow.
    std::alloc::Layout::array::<T>(count).map_err(|_| Error::Overflow("trim decomposition allocation"))?;
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(|_| Error::Budget("bytes"))?;
    Ok(result)
}

#[test]
fn allocation_size_overflow_is_not_a_memory_refusal() {
    assert!(matches!(allocated::<[f64;2]>(usize::MAX),Err(Error::Overflow("trim decomposition allocation"))));
}

#[test]
#[cfg(all(windows, target_pointer_width = "64"))]
fn allocator_refusal_is_reported_as_memory() {
    // Valid layout, but far beyond the Windows virtual address space. This
    // checks the allocator error path without exhausting physical memory.
    assert!(matches!(allocated::<u8>(1_usize << 60),Err(Error::Budget("bytes"))));
}

#[test]
fn scanline_retains_adjacent_collinear_vertices_in_both_directions() {
    let y=0.5_f64;let next=f64::from_bits(y.to_bits()+1);
    let plain=[[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]];
    let subdivided=[[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8],[0.2,next],[0.2,y]];
    let hole=[[0.35,0.35],[0.65,0.35],[0.65,0.65],[0.35,0.65]];
    let expected=trapezoids(&[&plain,&hole]).unwrap();
    let actual=trapezoids(&[&subdivided,&hole]).unwrap();
    let reversed:Vec<_>=subdivided.iter().copied().rev().collect();
    let reverse=trapezoids(&[&reversed,&hole]).unwrap();
    let area=|quads:&[[Point;4]]|quads.iter().map(|q|(q[1][0]-q[0][0]+q[2][0]-q[3][0])*(q[3][1]-q[0][1])*0.5).sum::<f64>();
    assert!((area(&expected)-area(&actual)).abs()<1e-12);
    assert!((area(&expected)-area(&reverse)).abs()<1e-12);
    assert!(actual.iter().flatten().any(|p|p[1]==y));
    assert!(actual.iter().flatten().any(|p|p[1]==next));
}

/// Partition noncrossing polygon loops into horizontal trapezoids using the
/// even-odd rule. The caller must validate loop topology before using output.
fn trapezoids(loops: &[&[Point]]) -> Result<Vec<[Point; 4]>> {
    let count = loops.iter().try_fold(0usize, |n, p| n.checked_add(p.len()))
        .ok_or(Error::Overflow("trim edges"))?;
    if count > MAX_EDGES || loops.is_empty() || loops.iter().any(|p| p.len() < 3) {
        return Err(Error::Unsupported("trim decomposition size"));
    }
    if loops.iter().flat_map(|p| p.iter()).flatten().any(|x| !x.is_finite()) {
        return Err(Error::Invalid("trim coordinate"));
    }
    let mut levels = allocated::<f64>(count + SUBDIVISIONS - 1)?;
    for p in loops.iter().flat_map(|p|p.iter()) {levels.push(p[1]);}
    let low = levels.iter().copied().fold(f64::INFINITY,f64::min);
    let high = levels.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    for i in 1..SUBDIVISIONS {
        let t=i as f64/SUBDIVISIONS as f64;
        levels.push(low*(1.-t)+high*t);
    }
    levels.sort_by(f64::total_cmp);
    levels.dedup_by(|a,b| *a == *b);
    // Count actual crossings before allocating: the edge-count product is a
    // safe admission bound, but can greatly exceed the number of cells for a
    // simple many-edge outline.
    let mut capacity=0usize;
    for pair in levels.windows(2) {
        let crossings=loops.iter().map(|polygon| (0..polygon.len()).filter(|&i| {
            let a=polygon[i];let b=polygon[(i+1)%polygon.len()];
            a[1]!=b[1] && a[1].min(b[1])<=pair[0] && pair[1]<=a[1].max(b[1])
        }).count()).try_fold(0usize,usize::checked_add).ok_or(Error::Overflow("trim crossings"))?;
        if crossings%2!=0 {return Err(Error::Invalid("trim crossing parity"));}
        capacity=capacity.checked_add((crossings/2).checked_mul(SUBDIVISIONS)
            .ok_or(Error::Overflow("trim trapezoids"))?).ok_or(Error::Overflow("trim trapezoids"))?;
    }
    let mut result = allocated::<[Point; 4]>(capacity)?;
    let mut cuts = allocated::<(f64, f64, f64)>(count)?;
    for pair in levels.windows(2) {
        let [lo, hi] = [pair[0],pair[1]];
        cuts.clear();
        for polygon in loops {
            for i in 0..polygon.len() {
                let a=polygon[i];let b=polygon[(i+1)%polygon.len()];
                if a[1]!=b[1] && a[1].min(b[1])<=lo && hi<=a[1].max(b[1]) {
                    let x = |y: f64| a[0] + ((y-a[1])/(b[1]-a[1]))*(b[0]-a[0]);
                    let (left,right)=(x(lo),x(hi));
                    let c=(left*0.5+right*0.5,left,right);
                    if !c.0.is_finite() || !c.1.is_finite() || !c.2.is_finite() {
                        return Err(Error::Unsupported("ambiguous trim intersection"));
                    }
                    cuts.push(c);
                }
            }
        }
        cuts.sort_by(|a,b| a.0.total_cmp(&b.0));
        if cuts.len()%2 != 0 {return Err(Error::Invalid("trim crossing parity"));}
        for pair in cuts.chunks_exact(2) {
            let (left,right)=(pair[0],pair[1]);
            if left.0 >= right.0 || left.1 > right.1 || left.2 > right.2 {
                return Err(Error::Unsupported("ambiguous trim crossings"));
            }
            for i in 0..SUBDIVISIONS {
                let t=i as f64/SUBDIVISIONS as f64;
                let s=(i+1) as f64/SUBDIVISIONS as f64;
                let lerp=|a:f64,b:f64,t:f64| a*(1.-t)+b*t;
                result.push([[lerp(left.1,right.1,t),lo],[lerp(left.1,right.1,s),lo],
                    [lerp(left.2,right.2,s),hi],[lerp(left.2,right.2,t),hi]]);
            }
        }
    }
    Ok(result)
}

fn validated_trapezoids(trim: &om9_core::rhino_semantics::TrimmedSurface,
    limits: &om9_core::budget::Limits) -> Result<Vec<[Point; 4]>> {
    trim.validate(limits)?;
    om9_core::rhino_semantics::validate_trim_loop_topology_for_display(trim)?;
    let count=trim.inners.len().checked_add(1).ok_or(Error::Overflow("trim loops"))?;
    let mut loops=allocated::<&[Point]>(count)?;
    loops.push(&trim.outer);
    loops.extend(trim.inners.iter().map(Vec::as_slice));
    trapezoids(&loops)
}

/// Experimental display mesh; source geometry remains untouched.
pub(super) fn surface_trim_buffers(trim: &om9_core::rhino_semantics::TrimmedSurface,
    usage: &mut om9_core::budget::Usage, limits: &om9_core::budget::Limits) -> Result<super::RenderBuffers> {
    use om9_core::{budget::Usage, geometry::Point3, surface_kernel};
    let parts=validated_trapezoids(trim,limits)?;
    let edge_count=std::iter::once(&trim.outer).chain(&trim.inners)
        .try_fold(0usize,|n,p|n.checked_add(p.len())).ok_or(Error::Overflow("trim edges"))?;
    let boundary_vertices=edge_count.checked_mul(SUBDIVISIONS+1).ok_or(Error::Overflow("trim boundary"))?;
    let points=parts.len().checked_mul(4).and_then(|n|n.checked_add(boundary_vertices))
        .ok_or(Error::Overflow("trim display"))?;
    let triangles=parts.len().checked_mul(2).ok_or(Error::Overflow("trim display"))?;
    let edges=edge_count.checked_mul(SUBDIVISIONS).ok_or(Error::Overflow("trim boundary"))?;
    usage.add(Usage {points:points as u64,..Usage::array::<Point3>(points)?},limits)?;
    usage.add(Usage::array::<([u32;3],u32)>(triangles)?,limits)?;
    usage.add(Usage::array::<[u32;2]>(edges)?,limits)?;
    let mut vertices=allocated::<Point3>(points)?;
    let mut faces=allocated::<([u32;3],u32)>(triangles)?;
    let mut lines=allocated::<[u32;2]>(edges)?;
    for part in parts {
        let base=u32::try_from(vertices.len()).map_err(|_|Error::Overflow("trim indices"))?;
        for uv in part {vertices.push(surface_kernel::evaluate(&trim.underlying,uv[0],uv[1],limits)?);}
        faces.push(([base,base+1,base+2],0)); faces.push(([base,base+2,base+3],0));
    }
    for polygon in std::iter::once(&trim.outer).chain(&trim.inners) {
        for i in 0..polygon.len() {
            let a=polygon[i];let b=polygon[(i+1)%polygon.len()];
            let base=u32::try_from(vertices.len()).map_err(|_|Error::Overflow("trim indices"))?;
            for j in 0..=SUBDIVISIONS {
                let t=j as f64/SUBDIVISIONS as f64;
                let uv=[a[0]*(1.-t)+b[0]*t,a[1]*(1.-t)+b[1]*t];
                vertices.push(surface_kernel::evaluate(&trim.underlying,uv[0],uv[1],limits)?);
                if j>0 {lines.push([base+j as u32-1,base+j as u32]);}
            }
        }
    }
    Ok(super::RenderBuffers::Mesh {vertices,triangles:faces,edges:lines})
}

pub(super) fn decode(entity: &om9_core::model::Entity, limits: &om9_core::budget::Limits)
    -> Result<Option<om9_core::rhino_semantics::TrimmedSurface>> {
    let Some(ext)=entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) else {return Ok(None)};
    if ext.payload.len()>om9_core::model::trim_surface_payload_limit(limits) {return Err(Error::Unsupported("trim metadata exceeds supported size"));}
    let trim=om9_core::rhino_semantics::TrimmedSurface::from_extension(ext,limits)?;
    if !matches!(&entity.geometry,om9_core::geometry::Geometry::Surface(surface) if *surface==trim.underlying) {
        return Err(Error::Invalid("trimmed surface underlying mismatch"));
    }
    Ok(Some(trim))
}

/// Conservative simultaneous output and scratch bound based on validated edge count.
pub(super) fn bound(trim: &om9_core::rhino_semantics::TrimmedSurface,
    payload_bytes: usize, limits: &om9_core::budget::Limits) -> Result<om9_core::budget::Usage> {
    let edges=std::iter::once(&trim.outer).chain(&trim.inners)
        .try_fold(0usize,|n,p|n.checked_add(p.len())).ok_or(Error::Overflow("trim edges"))?;
    if edges>MAX_EDGES {return Err(Error::Unsupported("trim decomposition size"));}
    // The planner already holds decoded metadata and its scratch reservation.
    // Count the same scanline crossings as the builder with fixed stack space;
    // no temporary mesh is built before the scene reservation.
    let mut levels=[0_f64;MAX_EDGES+SUBDIVISIONS-1];let mut len=0usize;
    for point in std::iter::once(&trim.outer).chain(&trim.inners).flatten() {
        levels[len]=point[1];len+=1;
    }
    if len==0 {return Err(Error::Invalid("trim loops"));}
    let low=levels[..len].iter().copied().fold(f64::INFINITY,f64::min);
    let high=levels[..len].iter().copied().fold(f64::NEG_INFINITY,f64::max);
    for i in 1..SUBDIVISIONS {let t=i as f64/SUBDIVISIONS as f64;levels[len]=low*(1.-t)+high*t;len+=1;}
    levels[..len].sort_by(f64::total_cmp);
    let mut quads=0usize;
    for pair in levels[..len].windows(2) {
        if pair[0]==pair[1] {continue;}
        let crossings=std::iter::once(&trim.outer).chain(&trim.inners).map(|p|(0..p.len()).filter(|&i| {
            let a=p[i];let b=p[(i+1)%p.len()];
            a[1]!=b[1] && a[1].min(b[1])<=pair[0] && pair[1]<=a[1].max(b[1])
        }).count()).try_fold(0usize,usize::checked_add).ok_or(Error::Overflow("trim crossings"))?;
        if crossings%2!=0 {return Err(Error::Invalid("trim crossing parity"));}
        quads=quads.checked_add((crossings/2).checked_mul(SUBDIVISIONS).ok_or(Error::Overflow("trim trapezoids"))?)
            .ok_or(Error::Overflow("trim trapezoids"))?;
    }
    // Headroom for allocation capacity above logical length.
    let quads=quads.checked_mul(2).and_then(|n|n.checked_add(16)).ok_or(Error::Overflow("trim trapezoids"))?;
    let mut result=bound_with_quads(edges,quads,payload_bytes,limits)?;
    // Ambiguous topology may still take the underlying-surface fallback.
    let mut fallback=om9_core::budget::Usage::default();super::surface_buffer_usage(&mut fallback,limits)?;
    result.bytes=result.bytes.max(fallback.bytes);result.points=result.points.max(fallback.points);
    Ok(result)
}

pub(super) fn payload_bound(payload: &str, limits: &om9_core::budget::Limits) -> Result<om9_core::budget::Usage> {
    if payload.len()>om9_core::model::trim_surface_payload_limit(limits) {return Err(Error::Unsupported("trim metadata exceeds supported size"));}
    if let Some((edges,quads))=raw_loop_crossings(payload.as_bytes()) {
        let quads=quads.checked_mul(2).and_then(|n|n.checked_add(16)).ok_or(Error::Overflow("trim trapezoids"))?;
        let mut result=bound_with_quads(edges,quads,payload.len(),limits)?;
        let mut fallback=om9_core::budget::Usage::default();super::surface_buffer_usage(&mut fallback,limits)?;
        result.bytes=result.bytes.max(fallback.bytes);result.points=result.points.max(fallback.points);
        return Ok(result);
    }
    // Count arrays within the two top-level trim-loop fields without parsing
    // the potentially large NURBS control net or allocating. If field names
    // are escaped, duplicated or structurally uncertain, count every array.
    let all=||payload.bytes().filter(|b|*b==b'[').count();
    let edges=trim_loop_array_upper(payload.as_bytes()).unwrap_or_else(all).min(MAX_EDGES).max(3);
    bound_count(edges,payload.len(),limits)
}

fn trim_loop_array_upper(bytes:&[u8]) -> Option<usize> {
    trim_loop_array_info(bytes).map(|info|info.0)
}

// Offset lookup is deliberately conservative: escaped or repeated top-level
// names fall back to the existing worst-case estimate.
fn trim_loop_array_info(bytes:&[u8]) -> Option<(usize,(usize,usize),(usize,usize))> {
    fn array_end(bytes:&[u8],start:usize)->Option<(usize,usize)> {
        let mut depth=0usize;let mut count=0usize;let mut string=false;let mut escaped=false;
        for (i,&b) in bytes.iter().enumerate().skip(start) {
            if string {if escaped {escaped=false;} else if b==b'\\' {escaped=true;} else if b==b'"' {string=false;} continue;}
            match b {
                b'"'=>string=true,
                b'['=>{depth=depth.checked_add(1)?;count=count.checked_add(1)?;},
                b']'=>{depth=depth.checked_sub(1)?;if depth==0{return Some((i,count));}},
                _=>{}
            }
        }
        None
    }
    let(mut i,mut objects,mut arrays,mut seen,mut count)=(0usize,0usize,0usize,0u8,0usize);
    let(mut outer,mut inners)=(None,None);
    while i<bytes.len() {
        match bytes[i] {
            b'{' => objects=objects.checked_add(1)?,
            b'}' => objects=objects.checked_sub(1)?,
            b'[' => arrays=arrays.checked_add(1)?,
            b']' => arrays=arrays.checked_sub(1)?,
            b'"' => {
                let start=i+1;let mut end=start;let mut escaped=false;
                while end<bytes.len() && bytes[end]!=b'"' {
                    if bytes[end]==b'\\' {escaped=true;end=end.checked_add(1)?;}
                    end=end.checked_add(1)?;
                }
                if end>=bytes.len(){return None;}
                if objects==1 && arrays==0 && !escaped {
                    let flag=match &bytes[start..end] {b"outer"=>1,b"inners"=>2,_=>0};
                    if flag!=0 {
                        let mut value=end+1;while bytes.get(value).is_some_and(u8::is_ascii_whitespace){value+=1;}
                        if bytes.get(value)==Some(&b':') {
                            value+=1;while bytes.get(value).is_some_and(u8::is_ascii_whitespace){value+=1;}
                            if bytes.get(value)!=Some(&b'[') || seen&flag!=0{return None;}
                            let(last,extra)=array_end(bytes,value)?;count=count.checked_add(extra)?;seen|=flag;
                            if flag==1 {outer=Some((value,last));}else{inners=Some((value,last));}i=last;
                        } else {i=end;}
                    } else {i=end;}
                } else {i=end;}
            }
            _=>{}
        }
        i+=1;
    }
    if seen==3 && objects==0 && arrays==0 {Some((count,outer?,inners?))}else{None}
}

// Exact scanline counts from canonical polygon arrays, using fixed stack
// storage and no deserialization allocation before scene admission. Any
// uncertain syntax falls back to the conservative count above.
fn raw_loop_crossings(bytes:&[u8]) -> Option<(usize,usize)> {
    struct Cursor<'a> { bytes:&'a [u8], pos:usize, end:usize }
    impl Cursor<'_> {
        fn whitespace(&mut self) {while self.pos<self.end && self.bytes[self.pos].is_ascii_whitespace(){self.pos+=1;}}
        fn take(&mut self,c:u8)->Option<()> {self.whitespace();if self.pos>=self.end || self.bytes[self.pos]!=c{return None;}self.pos+=1;Some(())}
        fn peek(&mut self)->Option<u8>{self.whitespace();(self.pos<self.end).then(||self.bytes[self.pos])}
        fn number(&mut self)->Option<f64>{self.whitespace();let start=self.pos;
            while self.pos<self.end && !matches!(self.bytes[self.pos],b','|b']'|b' '|b'\t'|b'\r'|b'\n'){self.pos+=1;}
            let value=std::str::from_utf8(&self.bytes[start..self.pos]).ok()?.parse::<f64>().ok()?;
            value.is_finite().then_some(value)}
    }
    fn loop_points(c:&mut Cursor<'_>,points:&mut [Point;MAX_EDGES],len:&mut usize)->Option<(usize,usize)> {
        c.take(b'[')?;let start=*len;
        if c.peek()?!=b']' {loop {c.take(b'[')?;let x=c.number()?;c.take(b',')?;let y=c.number()?;c.take(b']')?;
            if *len>=MAX_EDGES{return None;}points[*len]=[x,y];*len+=1;
            match c.peek()? {b','=>{c.take(b',')?;},b']'=>break,_=>return None}}}
        c.take(b']')?;if *len-start<3{return None;}Some((start,*len))
    }
    let(_,outer,inners)=trim_loop_array_info(bytes)?;
    let mut points=[[0.;2];MAX_EDGES];let mut ranges=[(0usize,0usize);MAX_EDGES];let(mut count,mut loops)=(0usize,0usize);
    let mut c=Cursor {bytes,pos:outer.0,end:outer.1+1};ranges[0]=loop_points(&mut c,&mut points,&mut count)?;loops+=1;
    c.whitespace();if c.pos!=c.end{return None;}
    let mut c=Cursor {bytes,pos:inners.0,end:inners.1+1};c.take(b'[')?;
    if c.peek()?!=b']' {loop {if loops>=MAX_EDGES{return None;}ranges[loops]=loop_points(&mut c,&mut points,&mut count)?;loops+=1;
        match c.peek()? {b','=>{c.take(b',')?;},b']'=>break,_=>return None}}}
    c.take(b']')?;c.whitespace();if c.pos!=c.end{return None;}
    let mut levels=[0_f64;MAX_EDGES+SUBDIVISIONS-1];for i in 0..count {levels[i]=points[i][1];}
    let low=levels[..count].iter().copied().fold(f64::INFINITY,f64::min);
    let high=levels[..count].iter().copied().fold(f64::NEG_INFINITY,f64::max);
    for i in 1..SUBDIVISIONS {let t=i as f64/SUBDIVISIONS as f64;levels[count+i-1]=low*(1.-t)+high*t;}
    let total=count+SUBDIVISIONS-1;levels[..total].sort_by(f64::total_cmp);
    let mut quads=0usize;
    for pair in levels[..total].windows(2) {if pair[0]==pair[1]{continue;}let mut crossings=0usize;
        for &(start,end) in &ranges[..loops] {for i in start..end {let a=points[i];let b=points[if i+1==end {start}else{i+1}];
            if a[1]!=b[1] && a[1].min(b[1])<=pair[0] && pair[1]<=a[1].max(b[1]) {crossings+=1;}}}
        if crossings%2!=0{return None;}quads=quads.checked_add((crossings/2).checked_mul(SUBDIVISIONS)?)?;
    }
    Some((count,quads))
}

#[test]
fn trim_array_preflight_counts_only_loop_arrays_and_falls_back_on_uncertainty() {
    let json=br#"{"underlying":{"poles":[[1,2,3],[4,5,6]],"note":"[escaped]"},"outer":[[0,0],[1,0],[0,1]],"inners":[[[0.1,0.1],[0.2,0.1],[0.1,0.2]]],"reversed":false}"#;
    assert_eq!(trim_loop_array_upper(json),Some(9));
    assert!(trim_loop_array_upper(br#"{"\u006futer":[[0,0],[1,0],[0,1]],"inners":[]}"#).is_none());
    assert!(trim_loop_array_upper(br#"{"outer":[],"outer":[],"inners":[]}"#).is_none());
    assert!(trim_loop_array_upper(br#"{"outer":[[0,0]],"inners":[}"#).is_none());
}

#[test]
fn raw_loop_preflight_matches_actual_trapezoid_count_and_rejects_uncertainty() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},model::Entity,geometry::Geometry};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer:Vec<_>=(0..501).map(|i|{let a=std::f64::consts::TAU*(i as f64)/501.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let hole=vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]];
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![hole],reversed:false};
    let mut entity=Entity::new("raw trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let payload=&entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload;
    let (edges,quads)=raw_loop_crossings(payload.as_bytes()).unwrap();
    assert_eq!(edges,505);assert_eq!(quads,validated_trapezoids(&trim,&limits).unwrap().len());
    assert!(raw_loop_crossings(br#"{"outer":[[0,0],[1,0],[0,1]],"outer":[],"inners":[]}"#).is_none());
    assert!(raw_loop_crossings(br#"{"\u006futer":[[0,0],[1,0],[0,1]],"inners":[]}"#).is_none());
    assert!(raw_loop_crossings(br#"{"outer":[[0,0],[1,0],[0]],"inners":[]}"#).is_none());
}

fn bound_count(edges: usize, payload_bytes: usize, limits: &om9_core::budget::Limits)
    -> Result<om9_core::budget::Usage> {
    let quads=edges.checked_add(SUBDIVISIONS-2).and_then(|n|n.checked_mul(edges/2))
        .and_then(|n|n.checked_mul(SUBDIVISIONS)).ok_or(Error::Overflow("trim trapezoids"))?;
    bound_with_quads(edges,quads,payload_bytes,limits)
}

fn bound_with_quads(edges:usize,quads:usize,payload_bytes:usize,limits:&om9_core::budget::Limits)
    -> Result<om9_core::budget::Usage> {
    use om9_core::{budget::Usage,geometry::Point3};
    let points=quads.checked_mul(4).and_then(|n|n.checked_add(edges*(SUBDIVISIONS+1)))
        .ok_or(Error::Overflow("trim vertices"))?;
    let mut usage=Usage {points:points as u64,..Usage::array::<Point3>(points)?};
    usage.add(Usage::array::<([u32;3],u32)>(quads*2)?,limits)?;
    usage.add(Usage::array::<[u32;2]>(edges*SUBDIVISIONS)?,limits)?;
    usage.add(Usage::array::<[Point;4]>(quads)?,limits)?;
    usage.add(Usage::array::<(f64,f64,f64)>(edges)?,limits)?;
    usage.add(Usage::array::<f64>(edges+SUBDIVISIONS-1)?,limits)?;
    let scratch=payload_bytes.checked_mul(64).and_then(|n|n.checked_add(4096))
        .ok_or(Error::Overflow("trim metadata scratch"))?;
    usage.add(Usage {bytes:scratch as u64,..Usage::default()},limits)?;
    Ok(usage)
}

#[test]
fn many_edge_outline_keeps_area_without_worst_case_scratch_allocation() {
    let outer:Vec<_>=(0..129).map(|i| {let a=std::f64::consts::TAU*i as f64/129.;[0.5+0.25*a.cos(),0.5+0.25*a.sin()]}).collect();
    let expected=(0..outer.len()).map(|i| {let a=outer[i];let b=outer[(i+1)%outer.len()];a[0]*b[1]-a[1]*b[0]}).sum::<f64>()*0.5;
    let quads=trapezoids(&[&outer]).unwrap();
    let actual:f64=quads.iter().map(|q|(q[1][0]-q[0][0]+q[2][0]-q[3][0])*(q[3][1]-q[0][1])*0.5).sum();
    assert!((actual-expected).abs()<1e-10);
    let prior_bound=(outer.len()+SUBDIVISIONS-2)*(outer.len()/2)*SUBDIVISIONS;
    assert!(quads.capacity()<prior_bound/4,"unexpected scratch capacity {}",quads.capacity());
}

#[test]
fn many_edge_surface_with_hole_builds_trimmed_display() {
    use om9_core::{budget::{Limits,Usage},geometry::Geometry,model::Entity,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    for count in [708,1020] {
    let outer=(0..count).map(|i| {let a=std::f64::consts::TAU*i as f64/count as f64;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("many edges",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let payload=&entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload;
    let predicted=payload_bound(payload,&limits).unwrap();
    let mut usage=Usage::default();
    let super::RenderBuffers::Mesh {vertices,triangles,edges}=surface_trim_buffers(&trim,&mut usage,&limits).unwrap() else {panic!("mesh")};
    assert!(usage.bytes<=predicted.bytes && usage.points<=predicted.points);
    assert_eq!(edges.len(),(count+4)*SUBDIVISIONS);
    for (tri,_) in triangles {
        let p=tri.map(|i|vertices[i as usize]);let x=(p[0][0]+p[1][0]+p[2][0])/3.;let y=(p[0][1]+p[1][1]+p[2][1])/3.;
        assert!(!(4.<x && x<6. && 4.<y && y<6.));
    }
    }
}

#[test]
fn candidate_mesh_respects_a_hole_and_source_surface() {
    use om9_core::{budget::{Limits,Usage},rhino_semantics::TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut usage=Usage::default();
    let super::RenderBuffers::Mesh {vertices,triangles,edges}=surface_trim_buffers(&trim,&mut usage,&limits).unwrap() else {panic!("mesh")};
    assert_eq!(usage.points,vertices.len() as u64);
    assert_eq!(edges.len(),8*SUBDIVISIONS);
    for (tri,_) in triangles {
        let p=tri.map(|i|vertices[i as usize]);
        let x=(p[0][0]+p[1][0]+p[2][0])/3.;let y=(p[0][1]+p[1][1]+p[2][1])/3.;
        assert!((1.-1e-10..=9.+1e-10).contains(&x) && (1.-1e-10..=9.+1e-10).contains(&y));
        assert!(!(4.<x && x<6. && 4.<y && y<6.));
    }
}

#[test]
fn candidate_mesh_samples_a_cylindrical_patch() {
    use om9_core::{budget::{Limits,Usage},rhino_semantics::TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.2],[0.8,0.2],[0.8,0.8],[0.1,0.8]],inners:vec![],reversed:false};
    let super::RenderBuffers::Mesh {vertices,triangles,edges}=surface_trim_buffers(&trim,&mut Usage::default(),&limits).unwrap() else {panic!("mesh")};
    assert!(!triangles.is_empty());assert_eq!(edges.len(),4*SUBDIVISIONS);
    for p in vertices {
        assert!((p[0].hypot(p[1])-2.).abs()<1e-10);
        assert!((1.-1e-10..=4.+1e-10).contains(&p[2]));
    }
}

#[test]
fn decomposes_concave_outer_and_hole_without_filling_the_hole() {
    let outer=[[0.,0.],[6.,0.],[6.,5.],[4.,5.],[4.,2.],[2.,2.],[2.,5.],[0.,5.]];
    let hole=[[0.5,0.5],[1.5,0.5],[1.5,1.5],[0.5,1.5]];
    let parts=trapezoids(&[&outer,&hole]).unwrap();
    let area: f64=parts.iter().map(|q| (q[1][0]-q[0][0]+q[2][0]-q[3][0])*(q[3][1]-q[0][1])*0.5).sum();
    assert!((area-23.).abs()<1e-12);
    for q in parts {let x=(q[0][0]+q[1][0]+q[2][0]+q[3][0])*0.25;let y=(q[0][1]+q[3][1])*0.5;
        assert!(!(0.5<x && x<1.5 && 0.5<y && y<1.5));
        assert!(!(2.<x && x<4. && 2.<y && y<5.));}
}

#[test]
fn handles_diagonal_boundaries_and_rejects_too_many_edges() {
    let triangle=[[0.,0.],[4.,0.],[0.,4.]];
    let parts=trapezoids(&[&triangle]).unwrap();
    assert_eq!(parts.len(),SUBDIVISIONS*SUBDIVISIONS);
    let area: f64=parts.iter().map(|q| (q[1][0]-q[0][0]+q[2][0]-q[3][0])*(q[3][1]-q[0][1])*0.5).sum();
    assert!((area-8.).abs()<1e-12);
    let too_many=vec![[0.,0.];MAX_EDGES+1];
    assert!(trapezoids(&[&too_many]).is_err());
}

#[test]
fn topology_preflight_rejects_crossings_and_nested_holes() {
    use om9_core::{budget::Limits, rhino_semantics::{TrimmedSurface, validate_trim_loop_topology}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],1.,1.,&limits).unwrap();
    let outer=vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]];
    let hole=vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]];
    let nested=vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]];
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![hole,nested],reversed:false};
    assert!(validate_trim_loop_topology(&trim).is_err());
    assert!(validated_trapezoids(&trim,&limits).is_err());
    let crossed=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,1.],[0.,1.],[1.,0.]],inners:vec![],reversed:false};
    assert!(validate_trim_loop_topology(&crossed).is_err());
    assert!(validated_trapezoids(&crossed,&limits).is_err());
    let valid=TrimmedSurface {underlying:crossed.underlying,outer,inners:vec![vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]]],reversed:false};
    valid.validate(&limits).unwrap(); validate_trim_loop_topology(&valid).unwrap();
    assert!(!validated_trapezoids(&valid,&limits).unwrap().is_empty());
}
