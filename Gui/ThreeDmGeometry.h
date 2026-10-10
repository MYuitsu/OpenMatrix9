#pragma once
// OM9-FILE-012: exact kernel conversion; no tessellation fallback.
#if defined(_WIN64) && defined(WIN32)
#undef WIN32
#endif
#include <opennurbs.h>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <Geom_Surface.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom_Curve.hxx>
#include <array>
#include <vector>
#include <memory>
#include <stdexcept>
namespace OpenMatrix9Gui::ThreeDm {
struct ExchangeError:std::runtime_error {using std::runtime_error::runtime_error;};
struct GeometryRepresentationUnavailable:ExchangeError {
    using ExchangeError::ExchangeError;
    int solidOrientation=0;
    GeometryRepresentationUnavailable(const std::string& message,int orientation):ExchangeError(message),solidOrientation(orientation){}
};
struct MeshData {std::vector<std::array<double,3>> vertices;std::vector<std::array<int,4>> faces;};
Handle(Geom_Curve) curve3d(const ON_Curve&,bool preservePeriodic=false);
Handle(Geom2d_Curve) curve2d(const ON_Curve&);
std::unique_ptr<ON_Curve> curveON(const Handle(Geom_Curve)&,double first,double last);
std::unique_ptr<ON_Curve> curveON(const Handle(Geom2d_Curve)&,double first,double last);
TopoDS_Shape importCurve(const ON_Curve&,double tolerance);
std::unique_ptr<ON_Curve> exportCurve(const TopoDS_Edge&,double tolerance);
Handle(Geom_Surface) importSurface(const ON_Surface&);
std::array<double,2> importedSurfaceParameters(const ON_Surface&,double u,double v);
Handle(Geom2d_Curve) importedSurfaceTrim(const ON_Surface&,const ON_Curve&);
std::unique_ptr<ON_NurbsSurface> exportSurface(const Handle(Geom_Surface)&);
MeshData importMesh(const ON_Mesh&);
std::unique_ptr<ON_Mesh> exportMesh(const MeshData&);
TopoDS_Shape importBrep(const ON_Brep&,double tolerance);
// Detached OCCT faces: no SDK/model state reaches worker threads.
struct BrepAssembly {std::vector<TopoDS_Face> faces;double tolerance=1e-6;int orientation=0;bool solid=false,revolution=false;};
BrepAssembly prepareBrepAssembly(const ON_Brep&,double tolerance);
TopoDS_Shape assembleBrep(BrepAssembly&);
// Resolve standalone SDK +2 on a copy; never normalize or edit source winding.
int resolvedBrepOrientation(const ON_Brep&,double tolerance);
std::unique_ptr<ON_Brep> exportBrep(const TopoDS_Shape&,double tolerance);
}
