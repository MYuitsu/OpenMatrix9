#include "ThreeDmGeometry.h"
#include "ThreeDmSolidShells.h"
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_NurbsConvert.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepLib.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <ShapeFix_Shape.hxx>
#include <map>
#include <limits>
#include <cmath>
#include <vector>
namespace OpenMatrix9Gui::ThreeDm {
static void alignTrimDomain(Handle(Geom2d_Curve)& curve,const ON_Interval& domain,bool reverse){
    auto spline=Handle(Geom2d_BSplineCurve)::DownCast(curve);if(spline.IsNull())throw ExchangeError("Native trim reversal requires the imported NURBS representation");
    // Native reversed proxies may produce a negated NURBS domain. Restore the
    // actual native interval after reversal, rather than assume either kernel's
    // Reverse convention; both seam pcurves must share the same edge range.
    if(reverse)spline->Reverse();TColStd_Array1OfReal knots(1,spline->NbKnots());spline->Knots(knots);
    const double first=spline->FirstParameter(),last=spline->LastParameter();
    if(first!=domain.Min()||last!=domain.Max()){const double scale=domain.Length()/(last-first);for(int i=knots.Lower();i<=knots.Upper();++i)knots(i)=domain.Min()+(knots(i)-first)*scale;spline->SetKnots(knots);}
}
static bool exactPlaneCurveParameters(const ON_Surface& source,const Handle(Geom_Surface)& surface,const TopoDS_Edge& edge,const Handle(Geom2d_Curve)& pcurve,double tolerance){
    if(!std::isfinite(tolerance)||tolerance<=0||(source.ClassId()!=&ON_CLASS_RTTI(ON_PlaneSurface)&&source.ClassId()!=&ON_CLASS_RTTI(ON_NurbsSurface)))return false;
    auto plane=Handle(Geom_BSplineSurface)::DownCast(surface);
    auto uv=Handle(Geom2d_BSplineCurve)::DownCast(pcurve);double first,last;auto xyz=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(edge,first,last));
    if(plane.IsNull()||uv.IsNull()||xyz.IsNull()||plane->UDegree()!=1||plane->VDegree()!=1||plane->NbUPoles()!=2||plane->NbVPoles()!=2||plane->IsURational()||plane->IsVRational())return false;
    // For an affine plane, matching every positive rational control point
    // and the complete spline basis establishes correspondence everywhere.
    // Bound the bilinear cross term and mapped-CV error independently within
    // the existing tolerance. UV control hulls stay in the surface rectangle.
    gp_Vec cross(plane->Pole(1,1),plane->Pole(2,2));cross-=gp_Vec(plane->Pole(1,1),plane->Pole(1,2));cross-=gp_Vec(plane->Pole(1,1),plane->Pole(2,1));if(cross.Magnitude()>tolerance/4)return false;
    if(xyz->Degree()!=uv->Degree()||xyz->NbPoles()!=uv->NbPoles()||xyz->NbPoles()>256||xyz->NbKnots()!=uv->NbKnots()||xyz->FirstParameter()!=first||xyz->LastParameter()!=last)return false;
    bool sameKnots=true,bezier=true;
    for(int i=1;i<=xyz->NbKnots();++i){
        if(xyz->Multiplicity(i)!=uv->Multiplicity(i))return false;
        sameKnots=sameKnots&&xyz->Knot(i)==uv->Knot(i);
        const int multiplicity=(i==1||i==xyz->NbKnots())?xyz->Degree()+1:xyz->Degree();
        bezier=bezier&&xyz->Multiplicity(i)==multiplicity;
    }
    if(!sameKnots&&!bezier)return false;
    const double ratio=xyz->Weight(1)/uv->Weight(1);if(!std::isfinite(ratio)||ratio<=0)return false;
    double weightDeviation=0,hullDiameter=0;std::vector<gp_Pnt> mapped;
    for(int i=1;i<=xyz->NbPoles();++i){
        const double a=xyz->Weight(i),b=ratio*uv->Weight(i);if(!std::isfinite(a)||!std::isfinite(b)||a<=0||b<=0)return false;
        const double relative=std::abs(b/a-1);if(!std::isfinite(relative))return false;weightDeviation=std::max(weightDeviation,relative);
        auto point=uv->Pole(i);if(!std::isfinite(point.X())||!std::isfinite(point.Y())||!source.Domain(0).Includes(point.X())||!source.Domain(1).Includes(point.Y()))return false;
        auto physical=surface->Value(point.X(),point.Y());const double error=xyz->Pole(i).Distance(physical);if(!std::isfinite(error)||error>tolerance/4)return false;
        for(const auto& previous:mapped){const double distance=previous.Distance(physical);if(!std::isfinite(distance))return false;hullDiameter=std::max(hullDiameter,distance);}mapped.push_back(physical);
    }
    // Positive basis weights make each rational point a convex combination.
    // Relative weight perturbations <= d change normalized coefficients by
    // at most 2*d/(1-d) in L1, hence the physical error is bounded by that
    // factor times the control hull diameter. This admits floating roundoff
    // in proportional weights without altering weights or relaxing tolerance.
    if(weightDeviation>=1||2*weightDeviation/(1-weightDeviation)*hullDiameter>tolerance/4)return false;
    if(!sameKnots){
        // Moving breakpoints of independent Bezier spans reparameterizes them
        // without changing their UV loci. Normalize the derived pcurve only;
        // native source domains/knots remain untouched. Other bases are refused.
        TColStd_Array1OfReal knots(1,xyz->NbKnots());xyz->Knots(knots);uv->SetKnots(knots);
    }
    return true;
}
static int classifiedSolidOrientation(const TopoDS_Shape& shape,double tolerance){
    return classifiedBrepSolidOrientation(shape,tolerance);
}
BrepAssembly prepareBrepAssembly(const ON_Brep& source,double tolerance){
    // Const SDK topology/proxy queries may populate serialized caches. Work on
    // an independent native copy so a retained archive remains authoritative.
    ON_Brep native(source);const ON_Brep& b=native;
    if(!b.IsValid())throw ExchangeError("Invalid input BRep topology");
    const double constructionTolerance=std::min(tolerance,1e-7);
    BrepAssembly prepared;prepared.tolerance=tolerance;prepared.solid=b.IsSolid();prepared.orientation=b.SolidOrientation();bool nativeRevolution=false;
    for(int fi=0;fi<b.m_F.Count();++fi){const auto& f=b.m_F[fi];auto surface=importSurface(*b.m_S[f.m_si]);
        bool transposed=false;if(auto rev=ON_RevSurface::Cast(b.m_S[f.m_si])){transposed=rev->m_bTransposed;nativeRevolution=true;}
        BRepBuilderAPI_MakeFace make(surface,constructionTolerance);if(!make.IsDone())throw ExchangeError("Cannot construct BRep face");
        // Start with no natural boundary: only original trim loops belong to the face.
        TopoDS_Face face;BRep_Builder builder;builder.MakeFace(face,surface,constructionTolerance);
        struct SeamEdge{TopoDS_Edge edge;Handle(Geom2d_Curve) pcurve;bool reversed=false;};
        std::map<int,SeamEdge> nativeEdges;std::map<int,TopoDS_Vertex> nativeVertices;
        auto vertex=[&](int index){auto found=nativeVertices.find(index);if(found!=nativeVertices.end())return found->second;
            if(index<0||index>=b.m_V.Count())throw ExchangeError("Missing native singular/seam vertex");auto point=b.m_V[index].point;TopoDS_Vertex result;builder.MakeVertex(result,gp_Pnt(point.x,point.y,point.z),constructionTolerance);nativeVertices[index]=result;return result;};
        for(int li=0;li<f.m_li.Count();++li){const auto& loop=b.m_L[f.m_li[li]];if(loop.m_type!=ON_BrepLoop::outer&&loop.m_type!=ON_BrepLoop::inner)throw ExchangeError("Unsupported BRep loop type");BRepBuilderAPI_MakeWire wire;
            for(int ti=0;ti<loop.m_ti.Count();++ti){const auto& trim=b.m_T[loop.m_ti[ti]];
                try {
                auto pcurve=importedSurfaceTrim(*b.m_S[f.m_si],trim);auto domain=trim.Domain();
                if(trim.m_type==ON_BrepTrim::singular){
                    auto pole=vertex(trim.m_vi[0]);TopoDS_Edge e;builder.MakeEdge(e);builder.Add(e,pole.Oriented(TopAbs_FORWARD));builder.Add(e,pole.Oriented(TopAbs_REVERSED));builder.Degenerated(e,true);
                    builder.UpdateEdge(e,pcurve,surface,TopLoc_Location(),constructionTolerance);builder.Range(e,surface,TopLoc_Location(),domain.Min(),domain.Max());wire.Add(e);if(!wire.IsDone())throw ExchangeError("Disconnected singular trim loop");continue;
                }
                if(trim.m_ei<0||trim.m_ei>=b.m_E.Count())throw ExchangeError("Missing BRep trim edge");
                const auto& sourceEdge=b.m_E[trim.m_ei];auto edgeDomain=sourceEdge.Domain();
                bool seam=false;for(int k=0;k<sourceEdge.m_ti.Count();++k)if(sourceEdge.m_ti[k]!=trim.m_trim_index&&b.m_T[sourceEdge.m_ti[k]].Face()==&f)seam=true;
                if(seam&&nativeEdges.contains(trim.m_ei)){
                    auto& stored=nativeEdges.at(trim.m_ei);auto e=stored.edge;alignTrimDomain(pcurve,edgeDomain,trim.m_bRev3d);
                    builder.UpdateEdge(e,stored.reversed?pcurve:stored.pcurve,stored.reversed?stored.pcurve:pcurve,surface,TopLoc_Location(),constructionTolerance);
                    builder.Range(e,surface,TopLoc_Location(),edgeDomain.Min(),edgeDomain.Max());
                    builder.SameRange(e,false);builder.SameParameter(e,false);try{BRepLib::SameParameter(e,constructionTolerance);}catch(const Standard_Failure& error){throw ExchangeError(std::string("Cannot synchronize paired seam pcurves: ")+error.GetMessageString());}
                    if(trim.m_bRev3d)e.Reverse();wire.Add(e);if(!wire.IsDone())throw ExchangeError("Disconnected BRep seam trim loop");continue;
                }
                BRepBuilderAPI_MakeEdge edge=seam?BRepBuilderAPI_MakeEdge(curve3d(sourceEdge),vertex(sourceEdge.m_vi[0]),vertex(sourceEdge.m_vi[1]),edgeDomain.Min(),edgeDomain.Max()):BRepBuilderAPI_MakeEdge(curve3d(sourceEdge),edgeDomain.Min(),edgeDomain.Max());
                if(!edge.IsDone())throw ExchangeError("Cannot construct BRep 3D edge");auto e=edge.Edge();
                // Edge and trim domains can differ, including a negative edge
                // interval from a reversed closed proxy. Put the pcurve on the
                // edge interval before OCCT synchronizes physical parameters.
                // This preserves its UV locus; SameParameter still resolves
                // any non-affine geometric correspondence.
                alignTrimDomain(pcurve,edgeDomain,trim.m_bRev3d);
                builder.UpdateEdge(e,pcurve,surface,TopLoc_Location(),constructionTolerance);
                builder.Range(e,surface,TopLoc_Location(),edgeDomain.Min(),edgeDomain.Max());
                const bool exact=!seam&&exactPlaneCurveParameters(*b.m_S[f.m_si],surface,e,pcurve,constructionTolerance);
                builder.SameRange(e,exact);builder.SameParameter(e,exact);if(!seam&&!exact)BRepLib::SameParameter(e,constructionTolerance);
                if(seam)nativeEdges[trim.m_ei]=SeamEdge{e,pcurve,trim.m_bRev3d};
                if(trim.m_bRev3d)e.Reverse();wire.Add(e);if(!wire.IsDone())throw ExchangeError("Disconnected BRep trim loop");
                } catch(const Standard_Failure& error) {
                    throw ExchangeError("BRep face "+std::to_string(fi)+" trim "+std::to_string(trim.m_trim_index)+": "+error.GetMessageString());
                }
            }
            if(!wire.IsDone())throw ExchangeError("Empty BRep trim loop");auto boundary=wire.Wire();
            // Swapping native U/V reverses the UV map orientation. Reverse the
            // entire loop so periodic analytic surfaces retain its interior.
            if(transposed)boundary.Reverse();builder.Add(face,boundary);
        }
        if(f.m_bRev!=transposed)face.Reverse();prepared.faces.push_back(face);
    }
    if(prepared.faces.empty())throw ExchangeError("BRep contains no faces");
    prepared.revolution=nativeRevolution;return prepared;
}
TopoDS_Shape assembleBrep(BrepAssembly& prepared){
    const auto tolerance=prepared.tolerance;
    BRepBuilderAPI_Sewing sew(tolerance);for(const auto& face:prepared.faces)sew.Add(face);
    sew.Perform();auto shape=sew.SewedShape();
    ShapeFix_Shape fix(shape);fix.SetPrecision(std::min(tolerance,1e-7));fix.SetMaxTolerance(tolerance);fix.Perform();shape=fix.Shape();
    if(prepared.solid){
        // Retain shell winding; nested odd-depth shells are cavities, even-depth
        // islands are separate material solids. SDK +2 is not presumed outward.
        shape=assembleClosedBrepShells(shape,prepared.orientation,tolerance);
    }
    if(!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Converted BRep has invalid topology");
    // Convert the assembled analytic faces and their mapped pcurves together.
    // The host and export then use the same finite NURBS representation, with
    // consistent trim extrema despite representation-dependent kernel bounds.
    if(prepared.revolution)shape=BRepBuilderAPI_NurbsConvert(shape,true).Shape();
    if(!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Converted NURBS BRep has invalid topology");
    if(prepared.orientation==2)(void)classifiedSolidOrientation(shape,tolerance);
    return shape;
}
TopoDS_Shape importBrep(const ON_Brep& source,double tolerance){auto prepared=prepareBrepAssembly(source,tolerance);return assembleBrep(prepared);}
int resolvedBrepOrientation(const ON_Brep& source,double tolerance){
    // Even const SDK queries lazily update native caches. Classification of a
    // preserved archive must therefore operate on an independent working copy.
    ON_Brep copy(source);int direction=copy.SolidOrientation();
    if(direction!=2)return direction;
    try{return classifiedSolidOrientation(importBrep(copy,tolerance),tolerance);}
    catch(const GeometryRepresentationUnavailable& error){
        // The complete shell forest can establish winding even when OCCT has
        // no valid editable representation of an inward complement with holes.
        if(error.solidOrientation==1||error.solidOrientation==-1)return error.solidOrientation;
        throw;
    }
}
std::unique_ptr<ON_Brep> exportBrep(const TopoDS_Shape& source,double tolerance){
    if(source.IsNull()||!BRepCheck_Analyzer(source).IsValid())throw ExchangeError("Invalid shape for BRep export");
    bool alreadyNurbs=true;
    for(TopExp_Explorer it(source,TopAbs_FACE);it.More();it.Next())if(Handle(Geom_BSplineSurface)::DownCast(BRep_Tool::Surface(TopoDS::Face(it.Current()))).IsNull())alreadyNurbs=false;
    for(TopExp_Explorer it(source,TopAbs_EDGE);it.More();it.Next()){auto edge=TopoDS::Edge(it.Current());if(BRep_Tool::Degenerated(edge))continue;double a,z;if(Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(edge,a,z)).IsNull())alreadyNurbs=false;}
    auto shape=alreadyNurbs?source:BRepBuilderAPI_NurbsConvert(source,true).Shape();auto b=std::make_unique<ON_Brep>();
    TopTools_IndexedMapOfShape vertices,edges;TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);TopExp::MapShapes(shape,TopAbs_EDGE,edges);
    for(int i=1;i<=vertices.Extent();++i){auto p=BRep_Tool::Pnt(TopoDS::Vertex(vertices(i)));b->NewVertex(ON_3dPoint(p.X(),p.Y(),p.Z()),tolerance);}
    std::vector<int> edgeIndex(edges.Extent()+1,-1);
    for(int i=1;i<=edges.Extent();++i){auto edge=TopoDS::Edge(edges(i).Oriented(TopAbs_FORWARD));if(BRep_Tool::Degenerated(edge))continue;auto c=exportCurve(edge,tolerance);int ci=b->m_C3.Count();b->m_C3.Append(c.release());TopoDS_Vertex v1,v2;TopExp::Vertices(edge,v1,v2,true);int a=vertices.FindIndex(v1)-1,z=vertices.FindIndex(v2)-1;if(a<0||z<0)throw ExchangeError("Missing BRep vertex");edgeIndex[i]=b->NewEdge(b->m_V[a],b->m_V[z],ci,nullptr,tolerance).m_edge_index;}
    for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()){
        auto original=TopoDS::Face(it.Current());auto face=TopoDS::Face(original.Oriented(TopAbs_FORWARD));TopLoc_Location loc;auto surface=BRep_Tool::Surface(face,loc);surface=Handle(Geom_Surface)::DownCast(surface->Transformed(loc.Transformation()));
        double u1,u2,v1,v2;BRepTools::UVBounds(face,u1,u2,v1,v2);
        auto ons=exportSurface(Handle(Geom_BSplineSurface)::DownCast(surface).IsNull()?Handle(Geom_Surface)(new Geom_RectangularTrimmedSurface(surface,u1,u2,v1,v2)):surface);
        int si=b->m_S.Count();b->m_S.Append(ons.release());int fi=b->NewFace(si).m_face_index;b->m_F[fi].m_bRev=original.Orientation()==TopAbs_REVERSED;
        auto outer=BRepTools::OuterWire(face);
        for(TopExp_Explorer w(face,TopAbs_WIRE);w.More();w.Next()){
            auto wire=TopoDS::Wire(w.Current());int li=b->NewLoop(wire.IsSame(outer)?ON_BrepLoop::outer:ON_BrepLoop::inner,b->m_F[fi]).m_loop_index;
            for(BRepTools_WireExplorer e(wire,face);e.More();e.Next()){
                auto edge=e.Current();double a,z;auto pc=BRep_Tool::CurveOnSurface(edge,face,a,z);if(pc.IsNull())throw ExchangeError("Missing surface trim curve");auto curve=curveON(pc,a,z);
                auto start=pc->Value(a),end=pc->Value(z);auto onStart=curve->PointAtStart(),onEnd=curve->PointAtEnd();
                if(std::hypot(start.X()-onStart.x,start.Y()-onStart.y)>1e-7||std::hypot(end.X()-onEnd.x,end.Y()-onEnd.y)>1e-7)throw ExchangeError("NURBS trim endpoint conversion mismatch");
                bool reversed=edge.Orientation()==TopAbs_REVERSED;if(reversed)curve->Reverse();int ci=b->m_C2.Count();b->m_C2.Append(curve.release());int ei=edgeIndex.at(edges.FindIndex(edge));
                if(ei<0){auto vertex=e.CurrentVertex();int vi=vertices.FindIndex(vertex)-1;if(vi<0)throw ExchangeError("Missing singular trim vertex");b->NewSingularTrim(b->m_V[vi],b->m_L[li],b->m_S[si]->IsIsoparametric(*b->m_C2[ci]),ci);}
                else {auto& t=b->NewTrim(b->m_E[ei],reversed,b->m_L[li],ci);t.m_tolerance[0]=t.m_tolerance[1]=tolerance;}
            }
        }
    }
    // Adjacent OCC pcurves can differ slightly at a shared vertex. Check the
    // physical gap before matching UV endpoints to Rhino's strict loop closure.
    for(int li=0;li<b->m_L.Count();++li){const auto& loop=b->m_L[li];const auto* surface=b->m_S[b->m_F[loop.m_fi].m_si];
        for(int ti=0;ti<loop.m_ti.Count();++ti){const auto& first=b->m_T[loop.m_ti[ti]];const auto& next=b->m_T[loop.m_ti[(ti+1)%loop.m_ti.Count()]];
            auto a=first.PointAtEnd(),z=next.PointAtStart();
            const auto gap=surface->PointAt(a.x,a.y).DistanceTo(surface->PointAt(z.x,z.y));
            if(gap>tolerance)throw ExchangeError("Export trim gap "+std::to_string(gap)+" exceeds geometric tolerance "+std::to_string(tolerance)+" at loop "+std::to_string(li)+" trim "+std::to_string(ti));
        }
    }
    b->MatchTrimEnds();b->Compact();
    b->SetTrimIsoFlags();b->SetTrimTypeFlags();b->SetTolerancesBoxesAndFlags();
    for(int i=0;i<b->m_E.Count();++i)b->m_E[i].m_tolerance=tolerance;
    for(int i=0;i<b->m_V.Count();++i)b->m_V[i].m_tolerance=tolerance;
    if(b->IsSolid()&&shape.ShapeType()==TopAbs_SOLID){
        BRepClass3d_SolidClassifier classifier(TopoDS::Solid(shape));classifier.PerformInfinitePoint(tolerance);
        if(classifier.State()!=TopAbs_IN&&classifier.State()!=TopAbs_OUT)throw ExchangeError("Cannot determine exported solid orientation");
        b->SetSolidOrientationForExperts(classifier.State()==TopAbs_IN?-1:1);
    }
    ON_wString log;ON_TextLog errors(log);if(!b->IsValid(&errors))throw ExchangeError(std::string("Exported BRep topology invalid: ")+ON_String(log).Array());return b;
}
}
