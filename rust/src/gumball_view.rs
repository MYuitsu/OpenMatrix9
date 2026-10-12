// SPDX-License-Identifier: LGPL-2.1-or-later
//! Value-only Gumball presentation policy. No camera/view or native pointer is retained.
use crate::gumball::{Frame, GumballError, Point};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewSettings {
    pub colors: [f64;9],
    pub move_length: f64, pub move_thickness: f64,
    pub rotate_radius: f64, pub rotate_thickness: f64,
    pub arrow_length: f64, pub arrow_width: f64,
    pub scale_size: f64, pub origin_size: f64,
    pub plane_size: f64, pub plane_offset: f64,
    pub menu_size: f64, pub menu_x: f64, pub menu_y: f64,
    pub pick_radius: f64, pub strength: f64,
    pub snappy: u32, pub orbit: u32,
}
impl Default for ViewSettings {
    fn default() -> Self {
        Self { colors:[1.,0.,0.,0.,1.,0.,0.,0.,1.], move_length:80.,move_thickness:1.5,
            rotate_radius:80.,rotate_thickness:1.5,arrow_length:18.,arrow_width:9.,
            scale_size:8.,origin_size:4.,plane_size:20.,plane_offset:50.,
            menu_size:10.,menu_x:-30.,menu_y:30.,pick_radius:5.,strength:100.,snappy:0,orbit:0 }
    }
}
impl ViewSettings {
    pub fn validate(self) -> Result<(),GumballError> {
        let sizes=[self.move_length,self.move_thickness,self.rotate_radius,self.rotate_thickness,
            self.arrow_length,self.arrow_width,self.scale_size,self.origin_size,self.plane_size,
            self.plane_offset,self.menu_size,self.pick_radius];
        if self.colors.iter().any(|v| !v.is_finite()||!(0. ..=1.).contains(v))
            || sizes.iter().any(|v| !v.is_finite()||!(0. ..=4096.).contains(v))
            || [self.menu_x,self.menu_y].iter().any(|v|!v.is_finite()||v.abs()>4096.)
            || !self.strength.is_finite()||self.strength<=0.||self.snappy>1||self.orbit>1 {
            return Err(GumballError::InvalidInput);
        }
        Ok(())
    }
}
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct ViewProjection {pub right:Point,pub up:Point,pub forward:Point,pub world_per_pixel:f64}
#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct ViewLayout {pub visible:[u32;14],pub anchors:[Point;3],pub reference:Point}
fn dot(a:Point,b:Point)->f64 {a.iter().zip(b).map(|(a,b)|a*b).sum()}
/// Resolve a native Mesh segment only when its facets define one oriented
/// plane. The first facet's first edge fixes the otherwise ambiguous X axis.
pub fn mesh_directions(points:&[Point])->Result<(Point,Point),GumballError> {
    if points.is_empty()||points.len()>131072||points.len()%3!=0{return Err(GumballError::Limit);}
    if points.iter().flatten().any(|v|!v.is_finite()||v.abs()>1e12){return Err(GumballError::InvalidInput);}
    let frame=|t:&[Point]| {
        let a:Point=std::array::from_fn(|i|t[1][i]-t[0][i]);
        let b:Point=std::array::from_fn(|i|t[2][i]-t[0][i]);
        let n=[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
        Frame::from_directions(t[0],a,n)
    };
    let first=frame(&points[..3])?;
    let size=points.iter().map(|p|p.iter().zip(first.origin).map(|(a,b)|(a-b).abs()).fold(0.,f64::max)).fold(0.,f64::max);
    for triangle in points.chunks_exact(3) {
        let other=frame(triangle)?;
        if dot(first.axes[2],other.axes[2])<1.-1e-6{return Err(GumballError::InvalidFrame);}
        for p in triangle {
            let delta=std::array::from_fn(|i|p[i]-first.origin[i]);
            if dot(delta,first.axes[2]).abs()>1e-7_f64.max(size*1e-6){return Err(GumballError::InvalidFrame);}
        }
    }
    Ok((first.axes[0],first.axes[2]))
}
pub fn extents(f:Frame,points:&[Point])->Result<([f64;3],[f64;3]),GumballError> {
    f.validate()?;
    if points.is_empty()||points.len()>131072 {return Err(GumballError::Limit);}
    let mut low=[f64::INFINITY;3];let mut high=[f64::NEG_INFINITY;3];
    for &p in points {
        if p.iter().any(|v|!v.is_finite()||v.abs()>1e12){return Err(GumballError::InvalidInput);}
        let delta=std::array::from_fn(|i|p[i]-f.origin[i]);
        for i in 0..3 {let v=dot(delta,f.axes[i]);low[i]=low[i].min(v);high[i]=high[i].max(v);}
    }
    Ok((low,high))
}
pub fn scale_reference(f:Frame,points:&[Point])->Result<Point,GumballError> {
    let (lo,hi)=extents(f,points)?;
    Ok(std::array::from_fn(|i|if lo[i]< -1e-12 {lo[i]} else {-lo[i].abs().max(hi[i].abs())}))
}
pub fn layout(f:Frame,points:&[Point],p:ViewProjection,s:ViewSettings)->Result<ViewLayout,GumballError> {
    s.validate()?;let (lo,hi)=extents(f,points)?;
    if !p.world_per_pixel.is_finite()||p.world_per_pixel<=0. {return Err(GumballError::InvalidInput);}
    for axis in [p.right,p.up,p.forward] {
        if axis.iter().any(|v|!v.is_finite())||(dot(axis,axis)-1.).abs()>1e-6 {return Err(GumballError::InvalidInput);}
    }
    if [dot(p.right,p.up),dot(p.right,p.forward),dot(p.up,p.forward)].iter().any(|v|v.abs()>1e-6) {return Err(GumballError::InvalidInput);}
    let mut result=ViewLayout {reference:scale_reference(f,points)?,..Default::default()};
    for i in 0..3 {
        let x=dot(f.axes[i],p.right);let y=dot(f.axes[i],p.up);let length=x.hypot(y);
        let usable=length>0.08;
        result.visible[i]=u32::from(usable&&s.move_length>0.&&s.move_thickness>0.);
        result.visible[10+i]=u32::from(usable&&s.scale_size>0.);
        result.visible[7+i]=u32::from(dot(f.axes[i],p.forward).abs()>0.08&&s.rotate_radius>0.&&s.rotate_thickness>0.);
        let negative=if lo[i]<0. {lo[i]}else{0.};
        let inside=lo[i]<0.&&hi[i]>=0.;
        let display=if usable&&!inside {negative.min(-12.*p.world_per_pixel/length)}else{negative};
        result.anchors[i]=std::array::from_fn(|j|f.origin[j]+display*f.axes[i][j]);
    }
    for (handle,normal) in [(3,2),(4,0),(5,1)] {
        result.visible[handle]=u32::from(dot(f.axes[normal],p.forward).abs()>0.08&&s.plane_size>0.);
    }
    result.visible[6]=u32::from(s.origin_size>0.);
    result.visible[13]=u32::from(s.menu_size>0.);
    Ok(result)
}
