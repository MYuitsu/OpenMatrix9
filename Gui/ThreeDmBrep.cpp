#include "ThreeDmGeometry.h"
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
#include <BRepLib.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <ShapeFix_Shape.hxx>
namespace OpenMatrix9Gui::ThreeDm {
TopoDS_Shape importBrep(const ON_Brep& b,double tolerance){
    if(!b.IsValid())throw ExchangeError("Invalid input BRep topology");
    const double constructionTolerance=std::min(tolerance,1e-7);
    BRepBuilderAPI_Sewing sew(tolerance);int faces=0;
    for(int fi=0;fi<b.m_F.Count();++fi){const auto& f=b.m_F[fi];auto surface=importSurface(*b.m_S[f.m_si]);
        BRepBuilderAPI_MakeFace make(surface,constructionTolerance);if(!make.IsDone())throw ExchangeError("Cannot construct BRep face");
        // Start with no natural boundary: only original trim loops belong to the face.
        TopoDS_Face face;BRep_Builder builder;builder.MakeFace(face,surface,constructionTolerance);
        for(int li=0;li<f.m_li.Count();++li){const auto& loop=b.m_L[f.m_li[li]];if(loop.m_type!=ON_BrepLoop::outer&&loop.m_type!=ON_BrepLoop::inner)throw ExchangeError("Unsupported BRep loop type");BRepBuilderAPI_MakeWire wire;
            for(int ti=0;ti<loop.m_ti.Count();++ti){const auto& trim=b.m_T[loop.m_ti[ti]];
                if(trim.m_type==ON_BrepTrim::singular)continue;
                auto pcurve=curve2d(trim);auto domain=trim.Domain();
                if(trim.m_ei<0||trim.m_ei>=b.m_E.Count())throw ExchangeError("Missing BRep trim edge");
                const auto& sourceEdge=b.m_E[trim.m_ei];auto edgeDomain=sourceEdge.Domain();
                BRepBuilderAPI_MakeEdge edge(curve3d(sourceEdge),edgeDomain.Min(),edgeDomain.Max());
                if(!edge.IsDone())throw ExchangeError("Cannot construct BRep 3D edge");auto e=edge.Edge();
                if(trim.m_bRev3d)pcurve->Reverse();
                builder.UpdateEdge(e,pcurve,surface,TopLoc_Location(),constructionTolerance);
                builder.Range(e,surface,TopLoc_Location(),domain.Min(),domain.Max());
                builder.SameRange(e,false);builder.SameParameter(e,false);BRepLib::SameParameter(e,constructionTolerance);
                if(trim.m_bRev3d)e.Reverse();wire.Add(e);if(!wire.IsDone())throw ExchangeError("Disconnected BRep trim loop");
            }
            if(!wire.IsDone())throw ExchangeError("Empty BRep trim loop");builder.Add(face,wire.Wire());
        }
        if(f.m_bRev)face.Reverse();sew.Add(face);++faces;
    }
    if(!faces)throw ExchangeError("BRep contains no faces");sew.Perform();auto shape=sew.SewedShape();
    ShapeFix_Shape fix(shape);fix.SetPrecision(std::min(tolerance,1e-7));fix.SetMaxTolerance(tolerance);fix.Perform();shape=fix.Shape();
    if(b.IsSolid()){
        if(shape.ShapeType()==TopAbs_FACE){TopoDS_Shell shell;BRep_Builder builder;builder.MakeShell(shell);builder.Add(shell,shape);shape=shell;}
        if(shape.ShapeType()!=TopAbs_SHELL)throw ExchangeError("Solid BRep did not produce one connected shell");auto shell=TopoDS::Shell(shape);if(!BRep_Tool::IsClosed(shell))throw ExchangeError("Imported solid shell is open");auto solid=BRepBuilderAPI_MakeSolid(shell).Solid();BRepLib::OrientClosedSolid(solid);shape=solid;
    }
    if(!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Converted BRep has invalid topology");return shape;
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
    ON_wString log;ON_TextLog errors(log);if(!b->IsValid(&errors))throw ExchangeError(std::string("Exported BRep topology invalid: ")+ON_String(log).Array());return b;
}
}
