// SPDX-License-Identifier: LGPL-2.1-or-later
// Capability-scoped, synchronous read-only OCCT measurements. No native handle escapes.
#include "brep_snapshot.hxx"
#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <Geom_Circle.hxx>
#include <Standard_Failure.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_GTrsf.hxx>

#include <cmath>
#include <cstdio>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

struct Om9MeasurePart {
 int kind; int degree; int periodic;
 const char* source; size_t source_len;
 const double* points; size_t point_count;
 const double* weights;
 const double* knots; size_t knot_count;
 double domain[2]; double radius; double matrix[16];
 int size[2]; int degree_v; const double* knots_v; size_t knot_count_v; double domain_v[2];
};

namespace {
TopoDS_Shape read_measure_source(const char* source, size_t length) {
 if (!source || !length) throw std::invalid_argument("missing canonical measurement source");
 TopoDS_Shape shape; BRep_Builder builder;
 std::istringstream input(std::string(source, length)); BRepTools::Read(shape, input, builder);
 if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
  throw std::invalid_argument("invalid canonical measurement source");
 return shape;
}
TopoDS_Shape placed(const TopoDS_Shape& shape, const double* matrix) {
 gp_GTrsf transform;
 for (int row=0;row<3;++row) for(int col=0;col<4;++col) {
  const double value=matrix[row*4+col];
  if (!std::isfinite(value)) throw std::invalid_argument("measurement transform");
  transform.SetValue(row+1,col+1,value);
 }
 BRepBuilderAPI_GTransform operation(shape,transform,true);
 if (!operation.IsDone()) throw std::runtime_error("measurement transform failed");
 return operation.Shape();
}
gp_Pnt point(const Om9MeasurePart& p,size_t index) {
 if(!p.points || index>=p.point_count) throw std::invalid_argument("measurement point array");
 return gp_Pnt(p.points[index*3],p.points[index*3+1],p.points[index*3+2]);
}
TopoDS_Shape part(const Om9MeasurePart& p) {
 TopoDS_Shape shape;
 if(p.kind==0) shape=read_measure_source(p.source,p.source_len);
 else if(p.kind==1) shape=BRepBuilderAPI_MakeVertex(point(p,0)).Shape();
 else if(p.kind==2) shape=BRepBuilderAPI_MakeEdge(point(p,0),point(p,1)).Shape();
 else if(p.kind==3) {
  const gp_Pnt x=point(p,1),y=point(p,2);
  gp_Dir xd(x.X(),x.Y(),x.Z()),yd(y.X(),y.Y(),y.Z());
  const Handle(Geom_Circle) curve=new Geom_Circle(gp_Ax2(point(p,0),xd.Crossed(yd),xd),p.radius);
  shape=BRepBuilderAPI_MakeEdge(curve,p.domain[0],p.domain[1]).Shape();
 } else if(p.kind==4) {
  if(!p.weights || !p.knots || !p.point_count || !p.knot_count)
   throw std::invalid_argument("measurement NURBS arrays");
  std::vector<double> knots;std::vector<int> multiplicities;
  for(size_t i=0;i<p.knot_count;++i) {
   if(knots.empty() || p.knots[i]!=knots.back()){knots.push_back(p.knots[i]);multiplicities.push_back(1);}
   else ++multiplicities.back();
  }
  TColgp_Array1OfPnt poles(1,int(p.point_count));TColStd_Array1OfReal weights(1,int(p.point_count));
  for(size_t i=0;i<p.point_count;++i){poles.SetValue(int(i+1),point(p,i));weights.SetValue(int(i+1),p.weights[i]);}
  TColStd_Array1OfReal k(1,int(knots.size()));TColStd_Array1OfInteger m(1,int(knots.size()));
  for(size_t i=0;i<knots.size();++i){k.SetValue(int(i+1),knots[i]);m.SetValue(int(i+1),multiplicities[i]);}
  const Handle(Geom_BSplineCurve) curve=new Geom_BSplineCurve(poles,weights,k,m,p.degree,p.periodic!=0);
  shape=BRepBuilderAPI_MakeEdge(curve,p.domain[0],p.domain[1]).Shape();
 } else if(p.kind==5) {
  BRepBuilderAPI_MakePolygon polygon;for(size_t i=0;i<p.point_count;++i)polygon.Add(point(p,i));
  polygon.Close();shape=BRepBuilderAPI_MakeFace(polygon.Wire()).Shape();
 } else if(p.kind==6) {
  if(!p.weights || !p.knots || !p.knots_v || p.size[0]<2 || p.size[1]<2 || size_t(p.size[0])*size_t(p.size[1])!=p.point_count)throw std::invalid_argument("measurement surface arrays");
  TColgp_Array2OfPnt poles(1,p.size[0],1,p.size[1]);TColStd_Array2OfReal weights(1,p.size[0],1,p.size[1]);
  for(int u=0;u<p.size[0];++u)for(int v=0;v<p.size[1];++v){const size_t i=size_t(u)*size_t(p.size[1])+size_t(v);poles.SetValue(u+1,v+1,point(p,i));weights.SetValue(u+1,v+1,p.weights[i]);}
  std::vector<double> uk,vk;std::vector<int> um,vm;
  auto compress=[](const double* source,size_t count,std::vector<double>& k,std::vector<int>& m){for(size_t i=0;i<count;++i){if(k.empty()||k.back()!=source[i]){k.push_back(source[i]);m.push_back(1);}else ++m.back();}};
  compress(p.knots,p.knot_count,uk,um);compress(p.knots_v,p.knot_count_v,vk,vm);
  TColStd_Array1OfReal uknots(1,int(uk.size())),vknots(1,int(vk.size()));TColStd_Array1OfInteger umults(1,int(um.size())),vmults(1,int(vm.size()));
  for(size_t i=0;i<uk.size();++i){uknots.SetValue(int(i+1),uk[i]);umults.SetValue(int(i+1),um[i]);}
  for(size_t i=0;i<vk.size();++i){vknots.SetValue(int(i+1),vk[i]);vmults.SetValue(int(i+1),vm[i]);}
  const Handle(Geom_BSplineSurface) surface=new Geom_BSplineSurface(poles,weights,uknots,vknots,umults,vmults,p.degree,p.degree_v,(p.periodic&1)!=0,(p.periodic&2)!=0);
  shape=BRepBuilderAPI_MakeFace(surface,p.domain[0],p.domain[1],p.domain_v[0],p.domain_v[1],1e-9).Shape();
 } else throw std::invalid_argument("measurement operand kind");
 return placed(shape,p.matrix);
}
TopoDS_Shape operand(const Om9MeasurePart* parts,size_t count) {
 if(!parts || !count)throw std::invalid_argument("empty measurement operand");
 BRep_Builder builder;TopoDS_Compound compound;builder.MakeCompound(compound);
 for(size_t i=0;i<count;++i)builder.Add(compound,part(parts[i]));
 return compound;
}
void failure(char* message,int capacity,const char* text){if(message && capacity>0)std::snprintf(message,capacity,"%s",text);}
}

extern "C" int om9_occt_measure_distance(const Om9MeasurePart* a,size_t ac,
 const Om9MeasurePart* b,size_t bc,double* value,double* witnesses,char* message,int capacity) {
 try {
  if(!value || !witnesses)throw std::invalid_argument("measurement outputs");
  const TopoDS_Shape first=operand(a,ac),second=operand(b,bc);
  BRepExtrema_DistShapeShape measure(first,second,1e-9);
  if(!measure.IsDone() || measure.NbSolution()<1)throw std::runtime_error("object distance did not converge");
  *value=measure.Value();const gp_Pnt p=measure.PointOnShape1(1),q=measure.PointOnShape2(1);
  witnesses[0]=p.X();witnesses[1]=p.Y();witnesses[2]=p.Z();
  witnesses[3]=q.X();witnesses[4]=q.Y();witnesses[5]=q.Z();
  if(!std::isfinite(*value))throw std::runtime_error("nonfinite object distance");
  return 0;
 }catch(const Standard_Failure& e){failure(message,capacity,e.what());return 2;}
 catch(const std::exception& e){failure(message,capacity,e.what());return 1;}
 catch(...){failure(message,capacity,"native distance failure");return 2;}
}

extern "C" int om9_occt_measure_edge_length(const char* source,size_t length,int index,
 const double* matrix,double* value,char* message,int capacity) {
 try {
  if(!matrix || !value)throw std::invalid_argument("edge measurement outputs");
  const TopoDS_Shape shape=read_measure_source(source,length);
  TopTools_IndexedMapOfShape edges;TopExp::MapShapes(shape,TopAbs_EDGE,edges);
  if(index < -1 || index>=edges.Extent())throw std::invalid_argument("edge measurement index");
  *value=0.;
  for(int i=1;i<=edges.Extent();++i)if(index==-1 || i==index+1){
   const TopoDS_Edge edge=TopoDS::Edge(placed(edges(i),matrix));
   if(BRep_Tool::Degenerated(edge))continue;
   BRepAdaptor_Curve curve(edge);*value+=GCPnts_AbscissaPoint::Length(curve,1e-9);
  }
  if(!std::isfinite(*value))throw std::runtime_error("nonfinite edge length");return 0;
 }catch(const Standard_Failure& e){failure(message,capacity,e.what());return 2;}
 catch(const std::exception& e){failure(message,capacity,e.what());return 1;}
 catch(...){failure(message,capacity,"native edge measurement failure");return 2;}
}
