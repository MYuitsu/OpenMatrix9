// Diagnostic only: independent point membership and OCCT volume methods for an authored hemisphere.
#include <BRep_Builder.hxx>
#include <BRepTools.hxx>
#include <BRepGProp.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <fstream>

int main(int argc, char** argv) {
 if (argc != 4) return 2;
 TopoDS_Shape body; BRep_Builder builder; std::ifstream in(argv[1]);
 if (!in) return 3; BRepTools::Read(body,in,builder); if (body.IsNull()) return 3;
 TopoDS_Shape a,b; std::ifstream ia(argv[2]),ib(argv[3]);
 if (!ia || !ib) return 4; BRepTools::Read(a,ia,builder); BRepTools::Read(b,ib,builder);
 Bnd_Box ba,bb; BRepBndLib::AddOptimal(a,ba,false,true); BRepBndLib::AddOptimal(b,bb,false,true);
 double amin[3],amax[3],bmin[3],bmax[3],low[3],span[3];
 ba.Get(amin[0],amin[1],amin[2],amax[0],amax[1],amax[2]);
 bb.Get(bmin[0],bmin[1],bmin[2],bmax[0],bmax[1],bmax[2]);
 for (int i=0;i<3;++i) {low[i]=std::max(amin[i],bmin[i]);span[i]=std::min(amax[i],bmax[i])-low[i];}
 std::printf("common box %.9f %.9f %.9f ; %.9f %.9f %.9f\n",low[0],low[1],low[2],span[0],span[1],span[2]);
 gp_Pnt probe(.1875,1.875,-1.875);
 BRepClass3d_SolidClassifier ca(a,probe,1e-7),cb(b,probe,1e-7),cr(body,probe,1e-7);
 std::printf("probe states a=%d b=%d result=%d\n",int(ca.State()),int(cb.State()),int(cr.State()));
 gp_Pnt candidate(low[0]+span[0]/16.,low[1]+span[1]*13./16.,low[2]+span[2]*3./16.);
 BRepClass3d_SolidClassifier cca(a,candidate,1e-7),ccb(b,candidate,1e-7),ccr(body,candidate,1e-7);
 std::printf("candidate %.12f %.12f %.12f states a=%d b=%d result=%d\n",candidate.X(),candidate.Y(),candidate.Z(),int(cca.State()),int(ccb.State()),int(ccr.State()));
 GProp_GProps standard; BRepGProp::VolumeProperties(body,standard);
 std::printf("standard=%.12f\n",standard.Mass());
 for (double tolerance : {1e-6,1e-9,1e-12}) {
  GProp_GProps adaptive; double error=BRepGProp::VolumePropertiesGK(body,adaptive,tolerance,false,true);
  std::printf("gk tol=%.0e value=%.12f error=%.12g\n",tolerance,adaptive.Mass(),error);
 }
 int expected_in=0, found_in=0, missing=0, boundary=0, input_missing=0, input_uncertain=0;
 for (int ix=1;ix<=14;++ix) for (int iy=-14;iy<=14;++iy) for (int iz=-14;iz<=14;++iz) {
  double x=3.*ix/15.,y=3.*iy/15.,z=3.*iz/15.;
  if (x*x+y*y+z*z>=8.8) continue;
  ++expected_in; gp_Pnt point(x,y,z);
  BRepClass3d_SolidClassifier left(a,point,1e-7),right(b,point,1e-7),classifier(body,point,1e-7);
  if (left.State()==TopAbs_OUT || right.State()==TopAbs_OUT) {
   ++input_missing; if (input_missing<=5) std::printf("input_out x=%.9f y=%.9f z=%.9f a=%d b=%d\n",x,y,z,int(left.State()),int(right.State()));
  } else if (left.State()!=TopAbs_IN || right.State()!=TopAbs_IN) ++input_uncertain;
  if (classifier.State()==TopAbs_IN) ++found_in;
  else if (classifier.State()==TopAbs_OUT) {
   ++missing; if (missing<=5) std::printf("missing x=%.9f y=%.9f z=%.9f\n",x,y,z);
  } else ++boundary;
 }
 std::printf("expected_in=%d found_in=%d missing=%d inconclusive=%d input_out=%d input_inconclusive=%d\n",expected_in,found_in,missing,boundary,input_missing,input_uncertain);
 for (double x : {0.1,0.1875,0.2,0.25,0.3,0.375}) {
  int holes=0;
  for (double y : {1.5,1.8,1.875,2.,2.1,2.2}) for (double z : {-1.5,-1.8,-1.875,-2.,-2.1,-2.2}) {
   if (x*x+y*y+z*z>=8.8) continue;
   BRepClass3d_SolidClassifier classifier(body,gp_Pnt(x,y,z),1e-7);
   if (classifier.State()==TopAbs_OUT) {++holes; if (x==0.1875) std::printf("targeted missing x=%.4f y=%.4f z=%.4f\n",x,y,z);}
  }
  std::printf("targeted x=%.4f missing=%d\n",x,holes);
 }
}
