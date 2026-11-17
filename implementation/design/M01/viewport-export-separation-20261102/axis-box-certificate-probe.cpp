// Test-only: conservative recognition of a single axis-aligned box represented by linear B-splines.
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Check.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeWedge.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <gp_Vec2d.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS.hxx>
#include <Precision.hxx>
#include <algorithm>
#include <array>
#include <cmath>
#include <cstdio>
#include <fstream>
#include <set>

#include "../../../../crates/om9-kernel/src/axis_box_certificate.hxx"
using Point=om9_axis_box::Point;
using om9_axis_box::certify;
static TopoDS_Shape read(const char* path) {
 TopoDS_Shape s; BRep_Builder b; std::ifstream f(path); if (f) BRepTools::Read(s,f,b); return s;
}

int main(int argc,char** argv) {
 if (argc!=3 && argc!=4) return 2;
 auto sphere=read(argv[1]),box=read(argv[2]); Point lo,hi; double tol=0.;
 bool positive=certify(box,lo,hi,tol); std::printf("scaled_box=%d tolerance=%.17g\n",positive,tol);
 if (!positive || !om9_axis_box::make_operand(sphere).IsNull()) return 3;
 if (!om9_axis_box::make_operand(BRepPrimAPI_MakeBox(4.,8.,8.).Shape()).IsNull()) return 4;
 if (!om9_axis_box::make_operand(BRepPrimAPI_MakeCylinder(2.,5.).Shape()).IsNull()) return 5;
 if (!om9_axis_box::make_operand(BRepPrimAPI_MakeWedge(4.,8.,6.,2.).Shape()).IsNull()) return 6;
 gp_GTrsf shear; shear.SetValue(1,2,0.2);
 auto sheared=BRepBuilderAPI_GTransform(box,shear,true).Shape();
 if (!om9_axis_box::make_operand(sheared).IsNull()) return 9;
 auto wedge=BRepPrimAPI_MakeWedge(4.,8.,6.,2.).Shape();
 auto spline_wedge=BRepBuilderAPI_GTransform(wedge,shear,true).Shape();
 if (!om9_axis_box::make_operand(spline_wedge).IsNull()) return 10;
 auto altered=read(argv[2]); TopTools_IndexedMapOfShape altered_faces; TopExp::MapShapes(altered,TopAbs_FACE,altered_faces);
 auto surface=occ::handle<Geom_BSplineSurface>::DownCast(BRep_Tool::Surface(TopoDS::Face(altered_faces(1))));
 if (surface.IsNull()) return 11;
 auto pole=surface->Pole(1,1); pole.SetX(pole.X()+1e-9); surface->SetPole(1,1,pole);
 std::printf("shifted_pole_occt_valid=%d\n",BRepCheck_Analyzer(altered).IsValid());
 if (!om9_axis_box::make_operand(altered).IsNull()) return 12;
 auto altered_trim=read(argv[2]); TopTools_IndexedMapOfShape trim_faces,trim_edges;
 TopExp::MapShapes(altered_trim,TopAbs_FACE,trim_faces); auto trim_face=TopoDS::Face(trim_faces(1));
 TopExp::MapShapes(trim_face,TopAbs_EDGE,trim_edges); double first=0.,last=0.; bool stored=false;
 auto trim=BRep_Tool::CurveOnSurface(TopoDS::Edge(trim_edges(1)),trim_face,first,last,&stored);
 if (trim.IsNull() || !stored) return 17; trim->Translate(gp_Vec2d(1e-9,0.));
 std::printf("shifted_trim_occt_valid=%d\n",BRepCheck_Analyzer(altered_trim).IsValid());
 if (!om9_axis_box::make_operand(altered_trim).IsNull()) return 18;
 auto mixed_tolerance=read(argv[2]); TopTools_IndexedMapOfShape mixed_vertices; TopExp::MapShapes(mixed_tolerance,TopAbs_VERTEX,mixed_vertices);
 BRep_Builder change; change.UpdateVertex(TopoDS::Vertex(mixed_vertices(1)),5e-7);
 if (!om9_axis_box::make_operand(mixed_tolerance).IsNull()) return 13;
 auto reversed=read(argv[2]); reversed.Reverse(); if (!om9_axis_box::make_operand(reversed).IsNull()) return 14;
 auto hole_tool=BRepPrimAPI_MakeCylinder(1.,20.).Shape();
 BRepAlgoAPI_Cut cut(box,hole_tool); cut.Build(); if (!cut.IsDone()) return 15;
 if (!om9_axis_box::make_operand(cut.Shape()).IsNull()) return 16;
 std::printf("negative fixtures rejected: analytic box, cylinder, wedge, sheared box, sheared wedge, 1e-9 pole shift, 1e-9 trim shift, mixed tolerances, reversed solid, cut box\n");
 TopoDS_Shape repaired=om9_axis_box::make_operand(box); if (repaired.IsNull()) return 19;
 TopTools_ListOfShape args,tools; args.Append(sphere);tools.Append(repaired);
 BRepAlgoAPI_Common common;common.SetArguments(args);common.SetTools(tools);common.SetUseOBB(true);common.Build();
 if (!common.IsDone()||common.HasErrors()||common.Shape().IsNull()) return 7;
 BRepAlgoAPI_Check check(common.Shape(),true,true); GProp_GProps g; BRepGProp::VolumePropertiesGK(common.Shape(),g,1e-10,false,true);
 std::printf("rebuilt_box checked=%d intersection_volume=%.12f\n",check.IsValid(),g.Mass());
 if (argc==4) {
  auto output=read(argv[3]); if (output.IsNull()) return 22;
  int positive=0,negative=0,wrong=0,unknown=0;
  for (int ix=1;ix<=14;++ix) for (int iy=-14;iy<=14;++iy) for (int iz=-14;iz<=14;++iz) {
   const double x=3.*ix/15.,y=3.*iy/15.,z=3.*iz/15.;
   for (int sign : {-1,1}) {
    const double px=sign*x,dx=px-0.001; if (dx*dx+y*y+z*z>=8.8) continue;
    BRepClass3d_SolidClassifier classifier(output,gp_Pnt(px,y,z),1e-7);
    const TopAbs_State state=classifier.State();
    if (state!=TopAbs_IN && state!=TopAbs_OUT) {++unknown;continue;}
    if (sign>0) {++positive;if (state!=TopAbs_IN) ++wrong;}
    else {++negative;if (state!=TopAbs_OUT) ++wrong;}
   }
  }
  std::printf("product output samples positive=%d negative=%d wrong=%d inconclusive=%d\n",positive,negative,wrong,unknown);
  if (wrong || unknown || positive==0 || negative==0) return 23;
 }
 return !check.IsValid() || std::abs(g.Mass()-18.*std::acos(-1.))>1e-5 ? 8 : 0;
}
