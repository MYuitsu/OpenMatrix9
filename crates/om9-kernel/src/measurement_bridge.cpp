// SPDX-License-Identifier: LGPL-2.1-or-later
// Capability-scoped, synchronous read-only OCCT measurements. No native handle escapes.
#include "brep_snapshot.hxx"
#include <BRepAdaptor_Curve.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRepTools_Modification.hxx>
#include <BRepTools_Modifier.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Surface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom2d_Curve.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <Geom_Circle.hxx>
#include <GProp_GProps.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>
#include <Standard_Failure.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_MapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Shell.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Lin.hxx>
#include <gp_Vec.hxx>

#include <algorithm>
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
 const double identity[16]={1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1};
 if (std::equal(matrix,matrix+16,identity)) return shape;
 gp_GTrsf transform;
 for (int row=0;row<3;++row) for(int col=0;col<4;++col) {
  const double value=matrix[row*4+col];
  if (!std::isfinite(value)) throw std::invalid_argument("measurement transform");
  transform.SetValue(row+1,col+1,value);
 }
 transform.SetForm();
 if (transform.Form()!=gp_Other) {
  gp_Trsf similarity;
  similarity.SetValues(matrix[0],matrix[1],matrix[2],matrix[3],
                       matrix[4],matrix[5],matrix[6],matrix[7],
                       matrix[8],matrix[9],matrix[10],matrix[11]);
  BRepBuilderAPI_Transform operation(shape,similarity,true);
  if (!operation.IsDone()) throw std::runtime_error("measurement transform failed");
  return operation.Shape();
 }
 BRepBuilderAPI_GTransform operation(shape,transform,true);
 if (!operation.IsDone()) throw std::runtime_error("measurement transform failed");
 return operation.Shape();
}
gp_Pnt point(const Om9MeasurePart& p,size_t index) {
 if(!p.points || index>=p.point_count) throw std::invalid_argument("measurement point array");
 return gp_Pnt(p.points[index*3],p.points[index*3+1],p.points[index*3+2]);
}
TopoDS_Shape part(const Om9MeasurePart& p, int certify_periodic=0) {
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
  if (certify_periodic&1) surface->SetUPeriodic();
  if (certify_periodic&2) surface->SetVPeriodic();
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

// Replace supporting surfaces while keeping pcurves and shared topology.
class ShrinkFaceModification : public BRepTools_Modification {
public:
 bool NewSurface(const TopoDS_Face& face, Handle(Geom_Surface)& result, TopLoc_Location& location,
                 double& tolerance, bool& reverse_wires, bool& reverse_face) override {
  Handle(Geom_Surface) surface=BRep_Tool::Surface(face,location);
  if (surface.IsNull() || surface->IsUPeriodic() || surface->IsVPeriodic())
   throw std::invalid_argument("shrink face: periodic or missing surface");
  double u0,u1,v0,v1; BRepTools::UVBounds(face,u0,u1,v0,v1);
  if (!std::isfinite(u0) || !std::isfinite(u1) || !std::isfinite(v0) || !std::isfinite(v1) || u0>=u1 || v0>=v1)
   throw std::invalid_argument("shrink face: invalid UV bounds");
  Handle(Geom_BSplineSurface) bspline=Handle(Geom_BSplineSurface)::DownCast(surface->Copy());
  if (!bspline.IsNull()) {
   bspline->Segment(u0,u1,v0,v1); result=bspline;
  } else {
   result=new Geom_RectangularTrimmedSurface(surface,u0,u1,v0,v1);
  }
  tolerance=BRep_Tool::Tolerance(face); reverse_wires=false; reverse_face=false;
  return true;
 }
 bool NewCurve(const TopoDS_Edge&, Handle(Geom_Curve)&, TopLoc_Location&, double&) override { return false; }
 bool NewPoint(const TopoDS_Vertex&, gp_Pnt&, double&) override { return false; }
 bool NewCurve2d(const TopoDS_Edge& edge, const TopoDS_Face& face,
                 const TopoDS_Edge&, const TopoDS_Face&, Handle(Geom2d_Curve)& result, double& tolerance) override {
  double first,last; result=BRep_Tool::CurveOnSurface(edge,face,first,last);
  if (result.IsNull()) throw std::runtime_error("shrink face: missing pcurve");
  tolerance=BRep_Tool::Tolerance(edge); return true;
 }
 bool NewParameter(const TopoDS_Vertex&, const TopoDS_Edge&, double&, double&) override { return false; }
 GeomAbs_Shape Continuity(const TopoDS_Edge& edge, const TopoDS_Face& first, const TopoDS_Face& second,
                          const TopoDS_Edge&, const TopoDS_Face&, const TopoDS_Face&) override {
  return BRep_Tool::HasContinuity(edge,first,second) ? BRep_Tool::Continuity(edge,first,second) : GeomAbs_C0;
 }
};
}

extern "C" int om9_occt_surface_to_face(const Om9MeasurePart* source, int periodic_axes, double deflection, double angular_deflection,
 size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
 void* context, char* message, int capacity) {
 try {
  if (!source || source->kind != 6) throw std::invalid_argument("surface face: missing surface");
  if (periodic_axes & ~3) throw std::invalid_argument("surface face: invalid periodic axes");
  const TopoDS_Shape face=part(*source,periodic_axes);
  if (face.IsNull() || !BRepCheck_Analyzer(face).IsValid())
   throw std::invalid_argument("surface face: invalid shape");
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(face,TopAbs_FACE,faces);
  if (faces.Extent()!=1) throw std::invalid_argument("surface face: expected one face");
  return om9_emit_brep_snapshot(face,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity,angular_deflection);
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown surface face failure"); return 2; }
}

// Export-only conversion to canonical OCCT source without tessellation.
extern "C" int om9_occt_surface_to_face_source(const Om9MeasurePart* source, int periodic_axes,
 size_t max_bytes, int (*sink)(void*, const unsigned char*, size_t), void* context, char* message, int capacity) {
 try {
  if (!source || source->kind != 6 || !sink || (periodic_axes & ~3)) throw std::invalid_argument("surface face: invalid input");
  const TopoDS_Shape face = part(*source, periodic_axes);
  if (face.IsNull() || !BRepCheck_Analyzer(face).IsValid()) throw std::invalid_argument("surface face: invalid shape");
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(face, TopAbs_FACE, faces);
  if (faces.Extent() != 1) throw std::invalid_argument("surface face: expected one face");
  std::ostringstream output;
  BRepTools::Write(face, output, false, false, TopTools_FormatVersion_CURRENT);
  const std::string text = output.str();
  if (text.empty() || text.size() > max_bytes) throw std::length_error("surface face: source budget");
  if (sink(context, reinterpret_cast<const unsigned char*>(text.data()), text.size()) != 0)
   throw std::runtime_error("surface face: Rust refused source");
  return 0;
 } catch (const Standard_Failure& e) { failure(message, capacity, e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message, capacity, e.what()); return 1;
 } catch (...) { failure(message, capacity, "Unknown surface face source failure"); return 2; }
}

extern "C" int om9_occt_split_surface_parts(const Om9MeasurePart* target,
 const Om9MeasurePart* tools, size_t tool_count, int* pieces, double deflection,
 size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
 void* context, char* message, int capacity) {
 try {
  if (!target || !tools || !tool_count || !pieces) throw std::invalid_argument("split surface: missing input");
  TopoDS_Shape original=part(*target);
  TopTools_IndexedMapOfShape before, solids;
  TopExp::MapShapes(original,TopAbs_FACE,before);
  TopExp::MapShapes(original,TopAbs_SOLID,solids);
  if (before.Extent()==0) throw std::invalid_argument("split surface: target must contain faces");
  if (solids.Extent()!=0) {
   BRep_Builder builder; TopoDS_Compound boundary; builder.MakeCompound(boundary);
   for (int i=1;i<=before.Extent();++i) builder.Add(boundary,before(i));
   original=boundary;
  }
  BRepAlgoAPI_Splitter splitter;
  NCollection_List<TopoDS_Shape> arguments, cutting;
  arguments.Append(original);
  for (size_t i=0;i<tool_count;++i) cutting.Append(part(tools[i]));
  splitter.SetArguments(arguments); splitter.SetTools(cutting); splitter.Build();
  if (!splitter.IsDone() || splitter.HasErrors()) throw std::runtime_error("split surface failed");
  const TopoDS_Shape result=splitter.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid()) throw std::runtime_error("split surface: invalid result");
  TopTools_IndexedMapOfShape after;
  TopExp::MapShapes(result,TopAbs_FACE,after);
  if (after.Extent()<=before.Extent()) throw std::invalid_argument("cutters do not split surface");
  GProp_GProps a,b; BRepGProp::SurfaceProperties(original,a,1e-10); BRepGProp::SurfaceProperties(result,b,1e-10);
  if (std::abs(a.Mass()-b.Mass())>std::abs(a.Mass())*1e-6+1e-9)
   throw std::runtime_error("split surface changed total area: " + std::to_string(a.Mass()) + " -> " + std::to_string(b.Mass()));
  *pieces=after.Extent();
  return om9_emit_brep_snapshot(result,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown split surface failure"); return 2; }
}

extern "C" int om9_occt_split_face_isocurve(const Om9MeasurePart* target,
 const double* point_xyz, int direction, double tolerance, int* pieces, double deflection,
 size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
 void* context, char* message, int capacity) {
 try {
  if (!target || !point_xyz || !pieces || direction<0 || direction>2 || !std::isfinite(tolerance))
   throw std::invalid_argument("split isocurve: invalid inputs");
  for (int i=0;i<3;++i) if (!std::isfinite(point_xyz[i])) throw std::invalid_argument("split isocurve: invalid point");
  const TopoDS_Shape source=part(*target);
  TopTools_IndexedMapOfShape faces, solids;
  TopExp::MapShapes(source,TopAbs_FACE,faces); TopExp::MapShapes(source,TopAbs_SOLID,solids);
  if (faces.Extent()!=1 || solids.Extent()!=0) throw std::invalid_argument("split isocurve needs one open face");
  const TopoDS_Face face=TopoDS::Face(faces(1));
  const TopoDS_Shape vertex=BRepBuilderAPI_MakeVertex(gp_Pnt(point_xyz[0],point_xyz[1],point_xyz[2])).Shape();
  BRepExtrema_DistShapeShape closest(vertex,face); closest.Perform();
  if (!closest.IsDone() || closest.NbSolution()!=1 || closest.SupportTypeShape2(1)!=BRepExtrema_IsInFace)
   throw std::invalid_argument("split isocurve: pick an unambiguous interior point");
  if (tolerance>=0 && closest.Value()>tolerance) throw std::invalid_argument("point must lie on the surface within document tolerance");
  double u=0,v=0; closest.ParOnFaceS2(1,u,v);
  double u0,u1,v0,v1; BRepTools::UVBounds(face,u0,u1,v0,v1);
  if (!std::isfinite(u0) || !std::isfinite(u1) || !std::isfinite(v0) || !std::isfinite(v1))
   throw std::invalid_argument("split isocurve: unbounded face");
  const Handle(Geom_Surface) surface=BRep_Tool::Surface(face);
  if (surface.IsNull()) throw std::runtime_error("split isocurve: missing surface");
  NCollection_List<TopoDS_Shape> arguments, cutting; arguments.Append(face);
  if (direction==0 || direction==2) {
   if (v<=v0+(v1-v0)*1e-9 || v>=v1-(v1-v0)*1e-9) throw std::invalid_argument("split isocurve: boundary V");
   BRepBuilderAPI_MakeEdge edge(surface->VIso(v),u0,u1);
   if (!edge.IsDone()) throw std::runtime_error("split isocurve: U edge failed");
   cutting.Append(edge.Shape());
  }
  if (direction==1 || direction==2) {
   if (u<=u0+(u1-u0)*1e-9 || u>=u1-(u1-u0)*1e-9) throw std::invalid_argument("split isocurve: boundary U");
   BRepBuilderAPI_MakeEdge edge(surface->UIso(u),v0,v1);
   if (!edge.IsDone()) throw std::runtime_error("split isocurve: V edge failed");
   cutting.Append(edge.Shape());
  }
  BRepAlgoAPI_Splitter splitter; splitter.SetArguments(arguments); splitter.SetTools(cutting); splitter.Build();
  if (!splitter.IsDone() || splitter.HasErrors() || splitter.Shape().IsNull() || !BRepCheck_Analyzer(splitter.Shape()).IsValid())
   throw std::runtime_error("split isocurve failed");
  TopTools_IndexedMapOfShape output; TopExp::MapShapes(splitter.Shape(),TopAbs_FACE,output);
  if (output.Extent()<2 || size_t(output.Extent())>max_nodes) throw std::invalid_argument("isocurve does not split face");
  GProp_GProps a,b; BRepGProp::SurfaceProperties(face,a,1e-10); BRepGProp::SurfaceProperties(splitter.Shape(),b,1e-10);
  if (std::abs(a.Mass()-b.Mass())>std::abs(a.Mass())*1e-6+1e-9) throw std::runtime_error("split isocurve changed area");
  *pieces=output.Extent();
  return om9_emit_brep_snapshot(splitter.Shape(),deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown split isocurve failure"); return 2; }
}

extern "C" int om9_occt_surface_ray_hit(const Om9MeasurePart* target,
 const double* origin, const double* direction, const double* reference, double tolerance,
 double* output, char* message, int capacity) {
 try {
  if (!target || !origin || !direction || !reference || !output || !std::isfinite(tolerance) || tolerance<=0.)
   throw std::invalid_argument("surface ray: invalid input");
  for (int i=0;i<3;++i) if (!std::isfinite(origin[i]) || !std::isfinite(direction[i]) || !std::isfinite(reference[i]))
   throw std::invalid_argument("surface ray: non-finite input");
  const gp_Vec vector(direction[0],direction[1],direction[2]);
  if (vector.SquareMagnitude()<=1e-24) throw std::invalid_argument("surface ray: zero direction");
  const TopoDS_Shape shape=part(*target);
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(shape,TopAbs_FACE,faces);
  if (faces.Extent()!=1) throw std::invalid_argument("surface ray needs one face");
  IntCurvesFace_ShapeIntersector intersection; intersection.Load(shape,tolerance);
  intersection.Perform(gp_Lin(gp_Pnt(origin[0],origin[1],origin[2]),gp_Dir(vector)),0.,RealLast());
  if (!intersection.IsDone()) throw std::runtime_error("surface ray intersection failed");
  const gp_Pnt reference_point(reference[0],reference[1],reference[2]);
  int best=0; double best_distance=RealLast();
  for (int i=1;i<=intersection.NbPnt();++i) {
   const double distance=reference_point.SquareDistance(intersection.Pnt(i));
   if (std::isfinite(distance) && distance<best_distance) { best=i; best_distance=distance; }
  }
  if (!best) { failure(message,capacity,"viewport ray misses exact surface"); return 3; }
  for (int i=1;i<=intersection.NbPnt();++i) {
   if (i==best) continue;
   const double distance=reference_point.Distance(intersection.Pnt(i));
   if (std::isfinite(distance) && std::abs(distance-std::sqrt(best_distance))<=tolerance
       && intersection.Pnt(i).Distance(intersection.Pnt(best))>tolerance)
    throw std::invalid_argument("viewport ray has ambiguous surface hits");
  }
  const gp_Pnt point=intersection.Pnt(best);
  output[0]=point.X(); output[1]=point.Y(); output[2]=point.Z();
  return 0;
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown surface ray failure"); return 2; }
}

// Resolve the face in the serialized source before applying placement. Face
// indices use the same TopExp map as the snapshot that created the UUID list.
extern "C" int om9_occt_brep_face_ray_hit(const Om9MeasurePart* target, int face_index,
 const double* origin, const double* direction, const double* reference, double tolerance,
 double* output, char* message, int capacity) {
 try {
  if (!target || target->kind!=0 || face_index<0 || !origin || !direction || !reference || !output || !std::isfinite(tolerance) || tolerance<=0.)
   throw std::invalid_argument("BRep face ray: invalid input");
  for (int i=0;i<3;++i) if (!std::isfinite(origin[i]) || !std::isfinite(direction[i]) || !std::isfinite(reference[i]))
   throw std::invalid_argument("BRep face ray: non-finite input");
  const gp_Vec vector(direction[0],direction[1],direction[2]);
  if (vector.SquareMagnitude()<=1e-24) throw std::invalid_argument("BRep face ray: zero direction");
  const TopoDS_Shape source=read_measure_source(target->source,target->source_len);
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(source,TopAbs_FACE,faces);
  if (face_index>=faces.Extent()) throw std::invalid_argument("BRep face ray: stale face index");
  const TopoDS_Shape face=placed(faces.FindKey(face_index+1),target->matrix);
  IntCurvesFace_ShapeIntersector intersection; intersection.Load(face,tolerance);
  intersection.Perform(gp_Lin(gp_Pnt(origin[0],origin[1],origin[2]),gp_Dir(vector)),0.,RealLast());
  if (!intersection.IsDone()) throw std::runtime_error("BRep face ray intersection failed");
  // A display triangle may approximate the rear of a curved face more closely
  // than its front. Visibility is determined by the first forward ray hit,
  // never by the distance to that approximate reference.
  int best=0; double best_parameter=RealLast();
  for (int i=1;i<=intersection.NbPnt();++i) {
   const double parameter=intersection.WParameter(i);
   if (std::isfinite(parameter) && parameter>=0. && parameter<best_parameter) { best=i; best_parameter=parameter; }
  }
  if (!best) { failure(message,capacity,"viewport ray misses exact BRep face"); return 3; }
  const gp_Pnt point=intersection.Pnt(best);
  output[0]=point.X(); output[1]=point.Y(); output[2]=point.Z();
  return 0;
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown BRep face ray failure"); return 2; }
}

// One immutable source reconstruction per bounded group; no native handle
// escapes the call. Outputs are in the same order as the supplied face indices.
extern "C" int om9_occt_brep_face_ray_hits(const Om9MeasurePart* target,
 const int* face_indices, size_t count, const double* origin, const double* direction,
 double tolerance, double* output, char* message, int capacity) {
 try {
  if (!target || target->kind!=0 || !face_indices || count==0 || count>8 || !origin || !direction || !output || !std::isfinite(tolerance) || tolerance<=0.)
   throw std::invalid_argument("BRep face rays: invalid input");
  for (int i=0;i<3;++i) if (!std::isfinite(origin[i]) || !std::isfinite(direction[i]))
   throw std::invalid_argument("BRep face rays: non-finite input");
  const gp_Vec vector(direction[0],direction[1],direction[2]);
  if (vector.SquareMagnitude()<=1e-24) throw std::invalid_argument("BRep face rays: zero direction");
  const TopoDS_Shape source=read_measure_source(target->source,target->source_len);
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(source,TopAbs_FACE,faces);
  const gp_Lin ray(gp_Pnt(origin[0],origin[1],origin[2]),gp_Dir(vector));
  for (size_t index=0;index<count;++index) {
   const int face_index=face_indices[index];
   if (face_index<0 || face_index>=faces.Extent()) throw std::invalid_argument("BRep face rays: stale face index");
   const TopoDS_Shape face=placed(faces.FindKey(face_index+1),target->matrix);
   IntCurvesFace_ShapeIntersector intersection; intersection.Load(face,tolerance);
   intersection.Perform(ray,0.,RealLast());
   if (!intersection.IsDone()) throw std::runtime_error("BRep face rays: intersection failed");
   int best=0; double best_parameter=RealLast();
   for (int i=1;i<=intersection.NbPnt();++i) {
    const double parameter=intersection.WParameter(i);
    if (std::isfinite(parameter) && parameter>=0. && parameter<best_parameter) { best=i; best_parameter=parameter; }
   }
   if (!best) { failure(message,capacity,"viewport ray misses exact BRep face"); return 3; }
   const gp_Pnt point=intersection.Pnt(best);
   output[index*3]=point.X(); output[index*3+1]=point.Y(); output[index*3+2]=point.Z();
  }
  return 0;
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown BRep face rays failure"); return 2; }
}

extern "C" int om9_occt_shrink_face(const char* source, size_t length, double deflection,
 size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
 void* context, char* message, int capacity) {
 try {
  const TopoDS_Shape shape=read_measure_source(source,length);
  TopTools_IndexedMapOfShape faces, solids; TopExp::MapShapes(shape,TopAbs_FACE,faces); TopExp::MapShapes(shape,TopAbs_SOLID,solids);
  if (faces.Extent()==0 || solids.Extent()!=0) throw std::invalid_argument("shrink needs an open surface or shell");
  Handle(BRepTools_Modification) modification=new ShrinkFaceModification();
  BRepTools_Modifier modifier(shape,modification);
  if (!modifier.IsDone()) throw std::runtime_error("shrink face modification failed");
  const TopoDS_Shape result=modifier.ModifiedShape(shape);
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid())
   throw std::runtime_error("shrink face topology failed");
  for (const TopAbs_ShapeEnum type : {TopAbs_FACE,TopAbs_EDGE,TopAbs_VERTEX,TopAbs_WIRE,TopAbs_SHELL,TopAbs_SOLID}) {
   TopTools_IndexedMapOfShape before,after; TopExp::MapShapes(shape,type,before); TopExp::MapShapes(result,type,after);
   if (before.Extent()!=after.Extent()) throw std::runtime_error("shrink changed topology counts");
  }
  TopTools_IndexedDataMapOfShapeListOfShape before_edges,after_edges;
  TopExp::MapShapesAndAncestors(shape,TopAbs_EDGE,TopAbs_FACE,before_edges);
  TopExp::MapShapesAndAncestors(result,TopAbs_EDGE,TopAbs_FACE,after_edges);
  for (int i=1;i<=before_edges.Extent();++i) {
   const TopoDS_Shape edge=modifier.ModifiedShape(before_edges.FindKey(i));
   if (edge.IsNull() || !after_edges.Contains(edge)) throw std::runtime_error("shrink lost a shared edge");
   TopTools_MapOfShape expected,actual;
   for (const auto& face : before_edges.FindFromIndex(i)) {
    const TopoDS_Shape mapped=modifier.ModifiedShape(face);
    if (mapped.IsNull()) throw std::runtime_error("shrink lost a face");
    expected.Add(mapped);
   }
   for (const auto& face : after_edges.FindFromKey(edge)) actual.Add(face);
   if (expected.Extent()!=actual.Extent()) throw std::runtime_error("shrink changed face-edge adjacency");
   for (const auto& face : before_edges.FindFromIndex(i))
    if (!actual.Contains(modifier.ModifiedShape(face))) throw std::runtime_error("shrink rewired a shared edge");
  }
  GProp_GProps a,b; BRepGProp::SurfaceProperties(shape,a,1e-10); BRepGProp::SurfaceProperties(result,b,1e-10);
  if (std::abs(a.Mass()-b.Mass())>std::abs(a.Mass())*1e-6+1e-9) throw std::runtime_error("shrink face changed area");
  return om9_emit_brep_snapshot(result,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown shrink face failure"); return 2; }
}

extern "C" int om9_occt_face_underlying_bounds(const char* source, size_t length,
 double* bounds, char* message, int capacity) {
 try {
  if (!bounds) throw std::invalid_argument("missing surface bounds output");
  const TopoDS_Shape shape=read_measure_source(source,length);
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(shape,TopAbs_FACE,faces);
  if (faces.Extent()!=1) throw std::invalid_argument("surface bounds need one face");
  const Handle(Geom_Surface) surface=BRep_Tool::Surface(TopoDS::Face(faces(1)));
  if (surface.IsNull()) throw std::runtime_error("missing underlying surface");
  surface->Bounds(bounds[0],bounds[1],bounds[2],bounds[3]); return 0;
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown surface bounds failure"); return 2; }
}

extern "C" int om9_occt_face_shrink_supported(const char* source, size_t length,
 int* supported, char* message, int capacity) {
 try {
  if (!supported) throw std::invalid_argument("missing shrink capability output");
  const TopoDS_Shape shape=read_measure_source(source,length);
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(shape,TopAbs_FACE,faces);
  if (faces.Extent()==0) throw std::invalid_argument("shrink capability needs faces");
  *supported=1;
  for (int i=1;i<=faces.Extent();++i) {
   const Handle(Geom_Surface) surface=BRep_Tool::Surface(TopoDS::Face(faces(i)));
   if (surface.IsNull()) throw std::runtime_error("missing underlying surface");
   if (surface->IsUPeriodic() || surface->IsVPeriodic()) { *supported=0; break; }
  }
  return 0;
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown shrink capability failure"); return 2; }
}

extern "C" int om9_occt_face_periodicity(const char* source, size_t length,
 int* axes, char* message, int capacity) {
 try {
  if (!axes) throw std::invalid_argument("missing periodicity output");
  const TopoDS_Shape shape=read_measure_source(source,length);
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(shape,TopAbs_FACE,faces);
  if (faces.Extent()!=1) throw std::invalid_argument("periodicity needs exactly one face");
  const Handle(Geom_Surface) surface=BRep_Tool::Surface(TopoDS::Face(faces(1)));
  if (surface.IsNull()) throw std::runtime_error("missing underlying surface");
  axes[0]=surface->IsUPeriodic() ? 1 : 0;
  axes[1]=surface->IsVPeriodic() ? 1 : 0;
  return 0;
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown periodicity failure"); return 2; }
}

// Separate boundary regions without severing the target's pre-existing face edges.
extern "C" int om9_occt_split_boundary_components(const Om9MeasurePart* target,
 const Om9MeasurePart* tools, size_t tool_count, int* pieces, double deflection,
 size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
 void* context, char* message, int capacity) {
 try {
  if (!target || !tools || !tool_count || !pieces) throw std::invalid_argument("split boundary: missing input");
  TopoDS_Shape original=part(*target);
  TopTools_IndexedMapOfShape original_faces, original_edges, solids;
  TopExp::MapShapes(original,TopAbs_FACE,original_faces);
  TopExp::MapShapes(original,TopAbs_EDGE,original_edges);
  TopExp::MapShapes(original,TopAbs_SOLID,solids);
  if (original_faces.Extent()==0 || size_t(original_faces.Extent())>max_nodes || size_t(original_edges.Extent())>max_nodes)
   throw std::invalid_argument("split boundary: invalid target or budget");
  // Reject disconnected or non-manifold inputs; otherwise pre-existing separate
  // components could be mistaken for regions created by the cutters.
  std::vector<int> original_parent(size_t(original_faces.Extent()+1));
  for (int i=1;i<=original_faces.Extent();++i) original_parent[size_t(i)]=i;
  auto original_root=[&](int i) { while(original_parent[size_t(i)]!=i) i=original_parent[size_t(i)]; return i; };
  TopTools_IndexedDataMapOfShapeListOfShape original_adjacency;
  TopExp::MapShapesAndAncestors(original,TopAbs_EDGE,TopAbs_FACE,original_adjacency);
  for (int i=1;i<=original_adjacency.Extent();++i) {
   int first=0, count=0;
   for (const auto& adjacent_face : original_adjacency.FindFromIndex(i)) {
    const int face=original_faces.FindIndex(adjacent_face);
    if (!face) throw std::runtime_error("split boundary: inconsistent input topology");
    if (!first) first=face; else if (first!=face) original_parent[size_t(original_root(face))]=original_root(first);
    ++count;
   }
   if (count>2) throw std::invalid_argument("split boundary: non-manifold edge");
  }
  for (int i=2;i<=original_faces.Extent();++i)
   if (original_root(i)!=original_root(1)) throw std::invalid_argument("split boundary: disconnected target");
  if (solids.Extent()!=0) {
   BRep_Builder builder; TopoDS_Compound boundary; builder.MakeCompound(boundary);
   for (int i=1;i<=original_faces.Extent();++i) builder.Add(boundary,original_faces(i));
   original=boundary;
  }
  BRepAlgoAPI_Splitter splitter;
  NCollection_List<TopoDS_Shape> arguments, cutting;
  arguments.Append(original);
  std::vector<TopoDS_Shape> tool_shapes;
  for (size_t i=0;i<tool_count;++i) {
   TopoDS_Shape tool=part(tools[i]);
   TopTools_IndexedMapOfShape tool_faces; TopExp::MapShapes(tool,TopAbs_FACE,tool_faces);
   if (tool_faces.Extent()>0) {
    BRepAlgoAPI_Common overlap(original,tool); overlap.Build();
    if (!overlap.IsDone() || overlap.HasErrors()) throw std::runtime_error("split boundary: overlap check failed");
    GProp_GProps area; BRepGProp::SurfaceProperties(overlap.Shape(),area,1e-10);
    if (std::abs(area.Mass())>1e-9) throw std::invalid_argument("split boundary: overlapping surface cutter");
   }
   tool_shapes.push_back(tool); cutting.Append(tool_shapes.back());
  }
  splitter.SetArguments(arguments); splitter.SetTools(cutting); splitter.Build();
  if (!splitter.IsDone() || splitter.HasErrors()) throw std::runtime_error("split boundary failed");
  const TopoDS_Shape result=splitter.Shape();
  if (result.IsNull() || !BRepCheck_Analyzer(result).IsValid()) throw std::runtime_error("split boundary: invalid result");
  TopTools_IndexedMapOfShape faces;
  TopExp::MapShapes(result,TopAbs_FACE,faces);
  if (size_t(faces.Extent())>max_nodes) throw std::invalid_argument("split boundary: face budget");
  TopTools_MapOfShape retained_edges;
  for (int i=1;i<=original_edges.Extent();++i) {
   retained_edges.Add(original_edges(i));
   const TopTools_ListOfShape& modified=splitter.Modified(original_edges(i));
   for (const auto& edge : modified) retained_edges.Add(edge);
  }
  TopTools_IndexedDataMapOfShapeListOfShape adjacency;
  TopExp::MapShapesAndAncestors(result,TopAbs_EDGE,TopAbs_FACE,adjacency);
  if (size_t(adjacency.Extent())>max_nodes) throw std::invalid_argument("split boundary: edge budget");
  std::vector<int> parent(size_t(faces.Extent()+1));
  for (int i=1;i<=faces.Extent();++i) parent[size_t(i)]=i;
  auto root=[&](int i) { while(parent[size_t(i)]!=i) i=parent[size_t(i)]; return i; };
  size_t edge_comparisons=0;
  for (int i=1;i<=adjacency.Extent();++i) {
   if (!retained_edges.Contains(adjacency.FindKey(i))) continue;
   const TopoDS_Shape& edge=adjacency.FindKey(i);
   int first_face=0; bool shared=false;
   for (const auto& face : adjacency.FindFromIndex(i)) {
    const int index=faces.FindIndex(face);
    if (!first_face) first_face=index; else if (index!=first_face) shared=true;
   }
   GProp_GProps edge_props; BRepGProp::LinearProperties(edge,edge_props);
   const double edge_length=std::abs(edge_props.Mass());
   bool cut_edge=false;
   if (shared && edge_length>1e-9) for (const auto& tool : tool_shapes) {
    if (++edge_comparisons>std::min(max_nodes,size_t(250000))) throw std::invalid_argument("split boundary: edge comparison budget");
    BRepAlgoAPI_Common common(edge,tool); common.Build();
    if (!common.IsDone() || common.HasErrors()) throw std::runtime_error("split boundary: edge comparison failed");
    GProp_GProps overlap; BRepGProp::LinearProperties(common.Shape(),overlap);
    if (std::abs(overlap.Mass()) >= edge_length-std::max(1e-9,edge_length*1e-7)) { cut_edge=true; break; }
   }
   if (cut_edge) continue;
   int first=0;
   for (const auto& adjacent_face : adjacency.FindFromIndex(i)) {
    const int face=faces.FindIndex(adjacent_face);
    if (!face) throw std::runtime_error("split boundary: inconsistent topology");
    if (!first) first=face; else parent[size_t(root(face))]=root(first);
   }
  }
  BRep_Builder builder; TopoDS_Compound output; builder.MakeCompound(output);
  std::vector<TopoDS_Shell> shells;
  std::vector<int> group(size_t(faces.Extent()+1),-1);
  for (int i=1;i<=faces.Extent();++i) {
   const int representative=root(i);
   int& slot=group[size_t(representative)];
   if (slot<0) { slot=int(shells.size()); shells.emplace_back(); builder.MakeShell(shells.back()); }
   builder.Add(shells[size_t(slot)],faces(i));
  }
  for (const auto& shell : shells) {
   if (!BRepCheck_Analyzer(shell).IsValid()) throw std::runtime_error("split boundary: invalid component");
   builder.Add(output,shell);
  }
  const int count=int(shells.size());
  if (count<2) throw std::invalid_argument("cutters do not split surface");
  GProp_GProps a,b; BRepGProp::SurfaceProperties(original,a,1e-10); BRepGProp::SurfaceProperties(output,b,1e-10);
  if (std::abs(a.Mass()-b.Mass())>std::abs(a.Mass())*1e-6+1e-9) throw std::runtime_error("split boundary changed total area");
  *pieces=count;
  return om9_emit_brep_snapshot(output,deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown split boundary failure"); return 2; }
}

extern "C" int om9_occt_split_boundary_component(const char* source, size_t length, int index,
 double deflection, size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
 void* context, char* message, int capacity) {
 try {
  const TopoDS_Shape shape=read_measure_source(source,length);
  TopTools_IndexedMapOfShape shells; TopExp::MapShapes(shape,TopAbs_SHELL,shells);
  if (index<0 || index>=shells.Extent()) throw std::invalid_argument("split boundary component index");
  return om9_emit_brep_snapshot(shells(index+1),deflection,max_points,max_nodes,max_bytes,sink,context,message,capacity);
 } catch (const Standard_Failure& e) { failure(message,capacity,e.GetMessageString()); return 2;
 } catch (const std::exception& e) { failure(message,capacity,e.what()); return 1;
 } catch (...) { failure(message,capacity,"Unknown split boundary component failure"); return 2; }
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
