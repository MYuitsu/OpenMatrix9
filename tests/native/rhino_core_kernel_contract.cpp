// SPDX-License-Identifier: LGPL-2.1-or-later
// RCORE-01 exact fixtures using existing converters; no production solver changes.
#include "ThreeDmGeometry.h"
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <GeomConvert.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom2d_Curve.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Circ.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <QCryptographicHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <algorithm>
#include <cmath>
#include <fstream>
#include <filesystem>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <vector>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
constexpr double lengthBound=1e-7, areaBound=1e-7;
QJsonObject report{{"candidate_status","unrun_until_executed"},
                   {"units","mm"},{"length_bound_mm",lengthBound},
                   {"area_bound_mm2",areaBound}};
QJsonArray checks;
void check(const char* clause,bool condition){
    checks.append(QJsonObject{{"clause",clause},{"passed",condition}});
    if(!condition)throw std::runtime_error(clause);
}
QJsonArray xyz(const gp_Pnt& p){return {p.X(),p.Y(),p.Z()};}
QString fingerprint(const TopoDS_Shape& shape){
    std::ostringstream stream;BRepTools::Write(shape,stream);
    const auto data=stream.str();return QCryptographicHash::hash(
        QByteArray(data.data(),static_cast<qsizetype>(data.size())),
        QCryptographicHash::Sha256).toHex();
}
QJsonObject curveBasis(const Handle(Geom_BSplineCurve)& c,double a,double b){
    QJsonArray poles,weights,knots,mults;
    for(int i=1;i<=c->NbPoles();++i){poles.append(xyz(c->Pole(i)));weights.append(c->Weight(i));}
    for(int i=1;i<=c->NbKnots();++i){knots.append(c->Knot(i));mults.append(c->Multiplicity(i));}
    return {{"degree",c->Degree()},{"poles_xyz",poles},{"weights",weights},
            {"knots",knots},{"multiplicities",mults},{"domain",QJsonArray{a,b}},
            {"periodic",c->IsPeriodic()},{"closed",c->IsClosed()},{"rational",c->IsRational()}};
}
QJsonObject onBasis(const ON_NurbsCurve& c){
    QJsonArray poles,weights,knots;
    for(int i=0;i<c.CVCount();++i){ON_3dPoint p;c.GetCV(i,p);poles.append(QJsonArray{p.x,p.y,p.z});weights.append(c.Weight(i));}
    for(int i=0;i<c.KnotCount();++i)knots.append(c.Knot(i));
    return {{"degree",c.Degree()},{"poles_xyz",poles},{"weights",weights},
            {"opennurbs_stored_knots",knots},{"domain",QJsonArray{c.Domain()[0],c.Domain()[1]}},
            {"periodic",c.IsPeriodic()},{"closed",c.IsClosed()},{"rational",c.IsRational()}};
}
QJsonObject surfaceBasis(const Handle(Geom_Surface)& surface){
    double u0,u1,v0,v1;surface->Bounds(u0,u1,v0,v1);
    QJsonObject data{{"u_periodic",surface->IsUPeriodic()},{"v_periodic",surface->IsVPeriodic()},
                     {"u_closed",surface->IsUClosed()},{"v_closed",surface->IsVClosed()},
                     {"domain",QJsonArray{u0,u1,v0,v1}}};
    auto s=Handle(Geom_BSplineSurface)::DownCast(surface);
    if(s.IsNull()){data["basis_kind"]="analytic";return data;}
    data["basis_kind"]="nurbs";data["u_degree"]=s->UDegree();data["v_degree"]=s->VDegree();
    QJsonArray poles,weights,uk,vk,um,vm;
    for(int u=1;u<=s->NbUPoles();++u){QJsonArray row,wrow;
        for(int v=1;v<=s->NbVPoles();++v){row.append(xyz(s->Pole(u,v)));wrow.append(s->Weight(u,v));}
        poles.append(row);weights.append(wrow);}
    for(int i=1;i<=s->NbUKnots();++i){uk.append(s->UKnot(i));um.append(s->UMultiplicity(i));}
    for(int i=1;i<=s->NbVKnots();++i){vk.append(s->VKnot(i));vm.append(s->VMultiplicity(i));}
    data["poles_u_major"]=poles;data["weights_u_major"]=weights;
    data["u_knots"]=uk;data["v_knots"]=vk;data["u_multiplicities"]=um;data["v_multiplicities"]=vm;
    return data;
}
TopoDS_Face oneFace(const TopoDS_Shape& shape){
    TopTools_IndexedMapOfShape faces;TopExp::MapShapes(shape,TopAbs_FACE,faces);
    check("one native face",faces.Extent()==1);return TopoDS::Face(faces(1));
}
struct Incidence {QJsonObject json;std::vector<TopoDS_Edge> freeEdges;int pairedSeams=0;};
Incidence incidence(const TopoDS_Face& face){
    TopTools_IndexedMapOfShape unique;TopExp::MapShapes(face,TopAbs_EDGE,unique);
    std::vector<int> count(static_cast<std::size_t>(unique.Extent())+1,0);
    QJsonArray loops;const auto outer=BRepTools::OuterWire(face);int loopId=0;
    for(TopExp_Explorer loop(face,TopAbs_WIRE);loop.More();loop.Next(),++loopId){
        const auto wire=TopoDS::Wire(loop.Current());QJsonArray uses;int order=0;
        for(BRepTools_WireExplorer use(wire,face);use.More();use.Next(),++order){
            const auto edge=use.Current();const int identity=unique.FindIndex(edge);
            check("edge use resolves a unique edge identity",identity>0);++count[identity];
            double first,last;auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);
            check("edge use has a pcurve",!pc.IsNull());auto start=pc->Value(first),end=pc->Value(last);
            uses.append(QJsonObject{{"edge_identity",identity},{"use_order",order},
                {"orientation",int(edge.Orientation())},{"seam",bool(BRep_Tool::IsClosed(edge,face))},
                {"pcurve_domain",QJsonArray{first,last}},
                {"uv_start",QJsonArray{start.X(),start.Y()}},{"uv_end",QJsonArray{end.X(),end.Y()}}});
        }
        loops.append(QJsonObject{{"loop_index",loopId},{"role",wire.IsSame(outer)?"outer":"inner"},{"ordered_edge_uses",uses}});
    }
    Incidence result;QJsonArray edges;int total=0;
    for(int i=1;i<=unique.Extent();++i){auto e=TopoDS::Edge(unique(i));total+=count[i];
        const bool seam=BRep_Tool::IsClosed(e,face);
        if(count[i]==1&&!BRep_Tool::Degenerated(e))result.freeEdges.push_back(e);
        if(seam&&count[i]==2)++result.pairedSeams;
        edges.append(QJsonObject{{"edge_identity",i},{"uses",count[i]},{"seam",seam},
                                {"degenerate",bool(BRep_Tool::Degenerated(e))}});
    }
    result.json={{"unique_edges",unique.Extent()},{"edge_use_count",total},
                 {"paired_seam_identities",result.pairedSeams},{"edges",edges},{"loops",loops}};
    return result;
}
bool shareVertex(const TopoDS_Edge& left,const TopoDS_Edge& right){
    for(TopExp_Explorer a(left,TopAbs_VERTEX);a.More();a.Next())
        for(TopExp_Explorer b(right,TopAbs_VERTEX);b.More();b.Next())if(a.Current().IsSame(b.Current()))return true;
    return false;
}
QJsonArray boundaryComponents(const std::vector<TopoDS_Edge>& edges){
    std::vector<bool> seen(edges.size(),false);QJsonArray components;
    for(std::size_t seed=0;seed<edges.size();++seed)if(!seen[seed]){
        std::vector<std::size_t> group{seed};seen[seed]=true;
        for(std::size_t cursor=0;cursor<group.size();++cursor)for(std::size_t j=0;j<edges.size();++j)
            if(!seen[j]&&shareVertex(edges[group[cursor]],edges[j])){seen[j]=true;group.push_back(j);}
        double length=0,maxRadial=0,minZ=1e100,maxZ=-1e100;TopTools_IndexedMapOfShape vertices;
        std::vector<int> degree(1,0);
        for(auto i:group){GProp_GProps mass;BRepGProp::LinearProperties(edges[i],mass);length+=mass.Mass();
            TopoDS_Vertex a,z;TopExp::Vertices(edges[i],a,z);check("free boundary has endpoint identities",!a.IsNull()&&!z.IsNull());
            for(auto vertex:{a,z}){const int id=vertices.Add(vertex);if(degree.size()<=static_cast<std::size_t>(id))degree.resize(id+1,0);++degree[id];}
            double first,last;auto c=BRep_Tool::Curve(edges[i],first,last);check("free boundary has a 3D curve",!c.IsNull());
            for(int k=0;k<=128;++k){auto p=c->Value(first+(last-first)*k/128.0);
                maxRadial=std::max(maxRadial,std::abs(std::hypot(p.X(),p.Y())-4));minZ=std::min(minZ,p.Z());maxZ=std::max(maxZ,p.Z());}
        }
        const bool closed=std::all_of(degree.begin()+1,degree.end(),[](int n){return n==2;});
        check("RCORE-01.T03 free-boundary component is topologically closed",closed);
        check("RCORE-01.T03 circular boundary R4 residual",maxRadial<=lengthBound);
        check("RCORE-01.T03 circular boundary circumference",std::abs(length-8*ON_PI)<=lengthBound);
        check("RCORE-01.T03 boundary lies at a single end plane",maxZ-minZ<=lengthBound);
        components.append(QJsonObject{{"edge_segments",int(group.size())},{"closed",closed},{"length",length},
            {"radial_sample_max",maxRadial},{"z_min",minZ},{"z_max",maxZ}});
    }
    return components;
}
QJsonObject areaOracle(const TopoDS_Shape& shape,double expected){
    GProp_GProps rough,coarse,a,b;BRepGProp::SurfaceProperties(shape,rough);
    const double coarseError=BRepGProp::SurfaceProperties(shape,coarse,1e-10);
    const double errorA=BRepGProp::SurfaceProperties(shape,a,1e-14);
    const double errorB=BRepGProp::SurfaceProperties(shape,b,1e-16);
    auto diagnostics=report["area_diagnostics"].toArray();diagnostics.append(QJsonObject{
        {"expected",expected},{"eps_1e_10",coarse.Mass()},{"eps_1e_14",a.Mass()},{"eps_1e_16",b.Mass()},
        {"reported_relative_error_1e_10",coarseError},{"reported_relative_error_1e_14",errorA},{"reported_relative_error_1e_16",errorB},
        {"difference",std::abs(a.Mass()-b.Mass())},{"residual",std::abs(b.Mass()-expected)}});
    report["area_diagnostics"]=diagnostics;
    check("adaptive area is within the declared analytic bound",std::abs(b.Mass()-expected)<=areaBound);
    check("adaptive area convergence difference is within the declared bound",std::abs(a.Mass()-b.Mass())<=areaBound);
    return {{"analytic_expected",expected},{"default",rough.Mass()},
            {"adaptive_eps_1e_10",coarse.Mass()},{"adaptive_eps_1e_14",a.Mass()},{"adaptive_eps_1e_16",b.Mass()},
            {"absolute_residual",std::abs(b.Mass()-expected)},{"bound_mm2",areaBound}};
}
void fixtures(){
    ON_ArcCurve analytic(ON_Circle(ON_Plane::World_xy,10));ON_NurbsCurve converted;
    const int form=analytic.GetNurbForm(converted);check("RCORE-01.T01 analytic circle has NURBS form",form==1||form==2);
    check("RCORE-01.T01 conversion clamps through the current adapter convention",converted.ClampEnd(2));
    auto circle=importCurve(analytic,lengthBound);double first,last;
    auto curve=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(TopoDS::Edge(circle),first,last));
    check("RCORE-01.T01 native curve remains rational NURBS",!curve.IsNull()&&curve->IsRational());
    double minW=1e100,maxW=0,radial=0,planar=0;
    for(int i=1;i<=curve->NbPoles();++i){minW=std::min(minW,curve->Weight(i));maxW=std::max(maxW,curve->Weight(i));}
    check("RCORE-01.T01 weights are positive and not replaced by all ones",minW>0&&maxW-minW>1e-12);
    for(int i=0;i<=1024;++i){auto p=curve->Value(first+(last-first)*i/1024.0);
        radial=std::max(radial,std::abs(std::hypot(p.X(),p.Y())-10));planar=std::max(planar,std::abs(p.Z()));}
    check("RCORE-01.T01 sampled native radius and plane residual",radial<=lengthBound&&planar<=lengthBound);
    GProp_GProps length;BRepGProp::LinearProperties(circle,length);
    check("RCORE-01.T01 circumference analytic oracle",std::abs(length.Mass()-20*ON_PI)<=lengthBound);
    check("conversion does not mutate analytic circle input",std::abs(analytic.m_arc.Radius()-10)<=lengthBound);
    // Independently authored rational quadratic quarter circle, not exporter output.
    const double w=std::sqrt(0.5);ON_NurbsCurve quarter(3,true,3,3);
    quarter.SetCV(0,ON_4dPoint(10,0,0,1));quarter.SetCV(1,ON_4dPoint(10*w,10*w,0,w));quarter.SetCV(2,ON_4dPoint(0,10,0,1));
    for(int i=0;i<4;++i)quarter.SetKnot(i,i<2?0:1);auto quarterShape=importCurve(quarter,lengthBound);
    auto quarterCurve=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(TopoDS::Edge(quarterShape),first,last));
    check("independent rational quarter-circle middle weight survives",!quarterCurve.IsNull()&&std::abs(quarterCurve->Weight(2)-w)<1e-12);
    report["T01"]=QJsonObject{{"analytic_to_nurbs_status",form},{"source_nurbs_basis",onBasis(converted)},
        {"native_basis",curveBasis(curve,curve->FirstParameter(),curve->LastParameter())},
        {"independent_quarter_basis",curveBasis(quarterCurve,first,last)},
        {"radius_sample_max_mm",radial},{"planarity_sample_max_mm",planar},{"circumference",length.Mass()},
        {"parameter_map","not_queried_or_certified"},{"global_deviation_certificate","unavailable; sampled observation only"}};

    const auto outer=BRepBuilderAPI_MakePolygon(gp_Pnt(-10,-10,0),gp_Pnt(10,-10,0),gp_Pnt(10,10,0),gp_Pnt(-10,10,0),true).Wire();
    auto hole=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),3)).Edge()).Wire();
    BRepBuilderAPI_MakeFace make(outer);make.Add(TopoDS::Wire(hole.Reversed()));const auto square=make.Face();
    const auto beforeSquare=fingerprint(square);auto onSquare=exportBrep(square,lengthBound);auto importedSquare=importBrep(*onSquare,lengthBound);
    check("RCORE-01.T02 BRep conversion leaves source unchanged",fingerprint(square)==beforeSquare);
    check("RCORE-01.T02 native face remains valid",BRepCheck_Analyzer(importedSquare).IsValid());auto face=oneFace(importedSquare);auto trim=incidence(face);
    check("RCORE-01.T02 retains outer and hole loop",trim.json["loops"].toArray().size()==2);
    BRepClass_FaceClassifier center(face,gp_Pnt(0,0,0),lengthBound),inside(face,gp_Pnt(5,0,0),lengthBound);
    check("RCORE-01.T02 hole center is outside trimmed face",center.State()==TopAbs_OUT);
    check("RCORE-01.T02 non-hole point is inside trimmed face",inside.State()==TopAbs_IN);
    report["T02"]=QJsonObject{{"topology",trim.json},{"native_surface_basis",surfaceBasis(BRep_Tool::Surface(face))},
        {"area",areaOracle(importedSquare,400-9*ON_PI)},{"hole_center_state",int(center.State())},
        {"source_sha256",beforeSquare},{"converted_sha256",fingerprint(importedSquare)}};

    const auto cylinder=BRepPrimAPI_MakeCylinder(4,7).Face();auto sourceSurface=BRep_Tool::Surface(cylinder);
    check("RCORE-01.T03 independent source is periodic U",sourceSurface->IsUPeriodic());
    const auto beforeCylinder=fingerprint(cylinder);auto onCylinder=exportBrep(cylinder,lengthBound);auto importedCylinder=importBrep(*onCylinder,lengthBound);
    check("RCORE-01.T03 conversion leaves source unchanged",fingerprint(cylinder)==beforeCylinder);
    check("RCORE-01.T03 converted surface valid and open",BRepCheck_Analyzer(importedCylinder).IsValid()&&!BRep_Tool::IsClosed(importedCylinder));
    TopTools_IndexedMapOfShape solids;TopExp::MapShapes(importedCylinder,TopAbs_SOLID,solids);
    check("RCORE-01.T03 periodic surface is never classified as solid",solids.IsEmpty()&&!onCylinder->IsSolid());
    auto lateral=oneFace(importedCylinder);auto uses=incidence(lateral);auto boundaries=boundaryComponents(uses.freeEdges);
    check("RCORE-01.T03 retains a paired seam identity used twice",uses.pairedSeams>=1);
    check("RCORE-01.T03 identifies exactly two free-boundary components",boundaries.size()==2);
    std::vector<double> ends;for(const auto& item:boundaries)ends.push_back(item.toObject()["z_min"].toDouble());std::sort(ends.begin(),ends.end());
    check("RCORE-01.T03 free boundaries at z0 and z7",std::abs(ends[0])<=lengthBound&&std::abs(ends[1]-7)<=lengthBound);
    report["T03"]=QJsonObject{{"source_surface",surfaceBasis(sourceSurface)},
        {"native_surface",surfaceBasis(BRep_Tool::Surface(lateral))},{"topology",uses.json},
        {"free_boundary_components",boundaries},{"area",areaOracle(importedCylinder,56*ON_PI)},
        {"source_sha256",beforeCylinder},{"converted_sha256",fingerprint(importedCylinder)}};

    // A direct periodic NURBS owner and sibling handle exercise the converter's
    // mutation boundary separately from analytic cylinders and document copies.
    auto periodic=GeomConvert::SurfaceToBSplineSurface(new Geom_RectangularTrimmedSurface(
        sourceSurface,0.0,2*ON_PI,0.0,7.0,true,true));
    if(!periodic->IsUPeriodic())periodic->SetUPeriodic();
    auto sibling=periodic;
    const auto periodicBefore=QJsonDocument(surfaceBasis(periodic)).toJson(QJsonDocument::Compact);
    check("periodic NURBS ownership source is U periodic",periodic->IsUPeriodic());
    auto exportedPeriodic=exportSurface(periodic);
    check("periodic NURBS export preserves source and shared sibling basis",
        periodic->IsUPeriodic()&&sibling->IsUPeriodic()&&
        QJsonDocument(surfaceBasis(periodic)).toJson(QJsonDocument::Compact)==periodicBefore&&
        QJsonDocument(surfaceBasis(sibling)).toJson(QJsonDocument::Compact)==periodicBefore);
    check("periodic NURBS export remains a valid archive surface",exportedPeriodic->IsValid());
}
// Optional GUI-produced sphere input: validate the exact imported CAD bytes
// independently. Default nonadaptive volume is diagnostic only on rational faces.
void sphereVolumeOracle(const char* path){
    constexpr double volumeBound=1e-5;
    const std::filesystem::path input(path);
    check("T04 sphere oracle input is bounded",std::filesystem::file_size(input)<=4*1024*1024);
    std::ifstream bytesFile(input,std::ios::binary);
    const std::string bytes((std::istreambuf_iterator<char>(bytesFile)),{});
    check("T04 sphere oracle input is readable",bool(bytesFile)&&!bytes.empty());
    TopoDS_Shape sphere;BRep_Builder builder;std::istringstream stream(bytes);
    BRepTools::Read(sphere,stream,builder);
    check("T04 exact imported CAD bytes form a valid solid",
        !sphere.IsNull()&&sphere.ShapeType()==TopAbs_SOLID&&BRepCheck_Analyzer(sphere).IsValid());
    GProp_GProps rough,a,b;BRepGProp::VolumeProperties(sphere,rough);
    const double estimateA=BRepGProp::VolumeProperties(sphere,a,1e-14);
    const double estimateB=BRepGProp::VolumeProperties(sphere,b,1e-16);
    const double expected=36*ON_PI;
    report["T04_imported_sphere_volume"]=QJsonObject{{"input_path",QString::fromLocal8Bit(path)},
        {"input_sha256",QString::fromLatin1(QCryptographicHash::hash(QByteArray(bytes.data(),static_cast<qsizetype>(bytes.size())),QCryptographicHash::Sha256).toHex())},
        {"expected_mm3",expected},{"absolute_bound_mm3",volumeBound},{"default_nonadaptive_mm3",rough.Mass()},
        {"adaptive_eps_1e_14_mm3",a.Mass()},{"adaptive_eps_1e_16_mm3",b.Mass()},
        {"reported_relative_error_1e_14",estimateA},{"reported_relative_error_1e_16",estimateB},
        {"absolute_residual_mm3",std::abs(b.Mass()-expected)},{"convergence_difference_mm3",std::abs(a.Mass()-b.Mass())}};
    check("T04 adaptive sphere volume within unchanged analytic bound",
        std::isfinite(b.Mass())&&std::abs(b.Mass()-expected)<=volumeBound);
    check("T04 adaptive sphere volume pair converges within unchanged bound",
        std::isfinite(a.Mass())&&std::abs(a.Mass()-b.Mass())<=volumeBound);
}
}
int main(int argc,char** argv){
    ON::Begin();report["candidate_status"]="executed_partial_fixture_observations";int result=0;try{fixtures();if(argc>2)sphereVolumeOracle(argv[2]);report["ok"]=true;}
    catch(const std::exception& e){report["ok"]=false;report["error"]=e.what();result=1;}
    report["checks"]=checks;report["unfulfilled"]=QJsonArray{
        "Shared Rust descriptor/capability/conversion-report contract is not exercised by this candidate",
        "T01 sampled residual is not a certified global deviation bound or a T10 parameter map",
        "T07 malformed/allocation/transaction rejection is not covered",
        "Native executable has no FreeCAD document: Undo, ownership lifecycle, cold FCStd and asynchronous T08 are not covered",
        "No Rhino runtime or cross-backend parity evidence"};
    const auto bytes=QJsonDocument(report).toJson(QJsonDocument::Indented);
    if(argc>1){std::ofstream out(argv[1],std::ios::binary);out.write(bytes.constData(),bytes.size());if(!out)result=2;}
    std::cout<<bytes.constData();ON::End();return result;
}
