// Diagnostic only: compare OCCT Common configurations on one analytic sphere/box fixture.
#include <BRep_Builder.hxx>
#include <BRepTools.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Check.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRep_Tool.hxx>
#include <GeomLib_IsPlanarSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TopoDS_Vertex.hxx>
#include <GProp_GProps.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <gp_Pnt.hxx>
#include <cstdio>
#include <fstream>

static TopoDS_Shape read(const char* path) {
 TopoDS_Shape shape; BRep_Builder builder; std::ifstream in(path);
 if (in) BRepTools::Read(shape,in,builder); return shape;
}
int main(int argc, char** argv) {
 if (argc!=3) return 2;
 for (int input=1;input<=2;++input) {
  TopoDS_Shape s=read(argv[input]); if (s.IsNull()) return 3;
  TopTools_IndexedMapOfShape faces; TopExp::MapShapes(s,TopAbs_FACE,faces);
  std::printf("input=%d shape_type=%d\n",input,int(s.ShapeType()));
  for (int i=1;i<=faces.Extent();++i) {
   BRepAdaptor_Surface surface(TopoDS::Face(faces(i)));
   GeomLib_IsPlanarSurface planar(BRep_Tool::Surface(TopoDS::Face(faces(i))),1e-9);
   std::printf("input=%d face=%d surface_type=%d planar_at_1e-9=%d tolerance=%.17g\n",input,i,int(surface.GetType()),planar.IsPlanar(),BRep_Tool::Tolerance(TopoDS::Face(faces(i))));
   if (surface.GetType()==GeomAbs_BSplineSurface) {
    const auto spline=surface.BSpline();
    std::printf("  degree=%d,%d poles=%d,%d rational=%d,%d periodic=%d,%d\n",spline->UDegree(),spline->VDegree(),spline->NbUPoles(),spline->NbVPoles(),spline->IsURational(),spline->IsVRational(),spline->IsUPeriodic(),spline->IsVPeriodic());
   }
  }
  if (input==2) {
   TopTools_IndexedMapOfShape vertices; TopExp::MapShapes(s,TopAbs_VERTEX,vertices);
   for (int i=1;i<=vertices.Extent();++i) {const auto v=TopoDS::Vertex(vertices(i)); const auto p=BRep_Tool::Pnt(v);
    std::printf("  vertex=%d p=%.17g,%.17g,%.17g tolerance=%.17g\n",i,p.X(),p.Y(),p.Z(),BRep_Tool::Tolerance(v));}
   TopTools_IndexedMapOfShape edges; TopExp::MapShapes(s,TopAbs_EDGE,edges);
   for (int i=1;i<=edges.Extent();++i) {
    const auto edge=TopoDS::Edge(edges(i)); BRepAdaptor_Curve curve(edge);
    std::printf("  edge=%d curve_type=%d",i,int(curve.GetType()));
    if (curve.GetType()==GeomAbs_BSplineCurve) {const auto spline=curve.BSpline();
     std::printf(" degree=%d poles=%d rational=%d periodic=%d",spline->Degree(),spline->NbPoles(),spline->IsRational(),spline->IsPeriodic());}
    std::printf(" tolerance=%.17g\n",BRep_Tool::Tolerance(edge));
   }
  }
 }
 {
  TopoDS_Shape a=read(argv[1]); TopoDS_Shape b=BRepPrimAPI_MakeBox(gp_Pnt(0.,-4.,-4.),4.,8.,8.).Shape();
  TopoDS_Shape spline_box=read(argv[2]); int agree=0,disagree=0,inconclusive=0;
  for (double x : {-1.,0.5,2.,3.5,5.}) for (double y : {-5.,-2.,0.,2.,5.}) for (double z : {-5.,-2.,0.,2.,5.}) {
   BRepClass3d_SolidClassifier one(b,gp_Pnt(x,y,z),1e-7),two(spline_box,gp_Pnt(x,y,z),1e-7);
   if ((one.State()!=TopAbs_IN && one.State()!=TopAbs_OUT) || (two.State()!=TopAbs_IN && two.State()!=TopAbs_OUT)) ++inconclusive;
   else if (one.State()==two.State()) ++agree; else ++disagree;
  }
  std::printf("box_classification agree=%d disagree=%d inconclusive=%d\n",agree,disagree,inconclusive);
  TopTools_ListOfShape arguments,tools; arguments.Append(a); tools.Append(b);
  BRepAlgoAPI_Common op; op.SetArguments(arguments); op.SetTools(tools); op.SetNonDestructive(true); op.SetUseOBB(true); op.Build();
  if (!op.IsDone() || op.HasErrors() || op.Shape().IsNull()) return 4;
  BRepAlgoAPI_Check check(op.Shape(),true,true); GProp_GProps volume;
  double error=BRepGProp::VolumePropertiesGK(op.Shape(),volume,1e-10,false,true);
  BRepClass3d_SolidClassifier result(op.Shape(),gp_Pnt(.1875,1.875,-1.875),1e-7);
  std::printf("analytic_box valid=%d vol=%.9f err=%.3g witness=%d\n",check.IsValid(),volume.Mass(),error,int(result.State()));
 }
 for (bool non_destructive : {false,true}) for (bool obb : {false,true})
  for (double fuzzy : {0.,1e-7,1e-6,1e-5}) {
   TopoDS_Shape a=read(argv[1]),b=read(argv[2]); if (a.IsNull()||b.IsNull()) return 3;
   const gp_Pnt witness(.1875,1.875,-1.875);
   BRepClass3d_SolidClassifier before(a,witness,1e-7);
   TopTools_ListOfShape arguments,tools; arguments.Append(a); tools.Append(b);
   BRepAlgoAPI_Common op; op.SetArguments(arguments); op.SetTools(tools);
   op.SetNonDestructive(non_destructive); op.SetUseOBB(obb); op.SetFuzzyValue(fuzzy); op.Build();
   if (!op.IsDone() || op.HasErrors() || op.Shape().IsNull()) {std::printf("nd=%d obb=%d fuzzy=%.0e failed\n",non_destructive,obb,fuzzy); continue;}
   BRepAlgoAPI_Check check(op.Shape(),true,true);
   BRepClass3d_SolidClassifier after(a,witness,1e-7),result(op.Shape(),witness,1e-7);
   GProp_GProps volume; double error=BRepGProp::VolumePropertiesGK(op.Shape(),volume,1e-10,false,true);
   std::printf("nd=%d obb=%d fuzzy=%.0e valid=%d vol=%.9f err=%.3g input=%d>%d result=%d\n",
     non_destructive,obb,fuzzy,check.IsValid(),volume.Mass(),error,int(before.State()),int(after.State()),int(result.State()));
  }
}
