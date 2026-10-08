#include "ThreeDmNativeReferences.h"
#include "ThreeDmCurveOnSurface.h"
#include "opennurbs_polyedgecurve.h"
#include <set>
#include <cmath>
#include <limits>
#include <functional>
#include <QRegularExpression>
namespace OpenMatrix9Gui::ThreeDm {
namespace {
QString uuid(ON_UUID id){char text[37]{};ON_UuidToString(id,text);return QString::fromLatin1(text);}
struct Unavailable:ExchangeError{using ExchangeError::ExchangeError;};
struct Limit:ExchangeError{using ExchangeError::ExchangeError;};
struct CachedOwner {
    std::shared_ptr<ON_Geometry> geometry;
    std::map<QString,std::shared_ptr<ON_Geometry>> closure;
    QJsonArray references;
};
struct SharedOwners {
    std::map<QString,CachedOwner> entries;
    NativeReferenceLimits limits;
    size_t chargedBytes=0,cloneAttempts=0,visitedNodes=0,cacheHits=0;
    void reserve(const ON_Geometry& source){
        const size_t bytes=std::max<size_t>(source.SizeOf(),sizeof(ON_Geometry));
        if(cloneAttempts>=limits.owners)throw Limit("Native reference clone-count limit exceeded before allocation");
        if(bytes>limits.cloneBytes-chargedBytes)throw Limit("Native reference SDK-accounted byte limit exceeded before allocation");
        // Charge every attempt, including later failed roots. Sharing validated
        // geometry avoids repeated charges; released failures do not reset work.
        chargedBytes+=bytes;++cloneAttempts;
    }
};
void interval(ON_Interval value,const char* what){if(!value.IsIncreasing()||!std::isfinite(value[0])||!std::isfinite(value[1]))throw ExchangeError(what);}
QJsonObject trimDomainAnalysis(const ON_PolyEdgeSegment& segment,const ON_BrepTrim& trim,const ON_BrepEdge& edge,const ON_Surface& surface,double sourceTolerance){
    if(!ON_IsValid(sourceTolerance)||sourceTolerance<0)throw ExchangeError("Invalid source model tolerance for native trim domain validation");
    QJsonArray endpoints;double maximum=0,effective=sourceTolerance;
    // These are the two topological endpoints of the referenced portion. An
    // interior trim/edge parameter map need not be affine; do not invent one.
    for(int i=0;i<2;++i){
        const double edgeParameter=segment.m_edge_domain[i];
        const double trimParameter=segment.m_trim_domain[trim.m_bRev3d?1-i:i];
        const auto uv=trim.PointAt(trimParameter),edgePoint=edge.PointAt(edgeParameter);
        if(!uv.IsValid()||!edgePoint.IsValid())throw ExchangeError("Invalid native trim endpoint evaluation");
        const auto surfacePoint=surface.PointAt(uv.x,uv.y);
        if(!surfacePoint.IsValid())throw ExchangeError("Invalid native trim surface endpoint evaluation");
        const double magnitude=std::max({1.,std::abs(edgePoint.x),std::abs(edgePoint.y),std::abs(edgePoint.z),std::abs(surfacePoint.x),std::abs(surfacePoint.y),std::abs(surfacePoint.z)});
        double tolerance=std::max(sourceTolerance,64*std::numeric_limits<double>::epsilon()*magnitude);
        if(ON_IsValid(edge.m_tolerance)&&edge.m_tolerance>=0)tolerance=std::max(tolerance,edge.m_tolerance);
        const double deviation=edgePoint.DistanceTo(surfacePoint);
        if(!std::isfinite(deviation)||deviation>tolerance)throw ExchangeError("Native trim subdomain endpoints do not correspond to edge portion within source tolerance");
        maximum=std::max(maximum,deviation);effective=std::max(effective,tolerance);
        endpoints.append(QJsonObject{{"edge_parameter",edgeParameter},{"trim_parameter",trimParameter},{"uv",QJsonArray{uv.x,uv.y}},{"surface_point",QJsonArray{surfacePoint.x,surfacePoint.y,surfacePoint.z}},{"edge_point",QJsonArray{edgePoint.x,edgePoint.y,edgePoint.z}},{"deviation",deviation}});
    }
    QJsonArray interior;double interiorMaximum=0;double previous=trim.m_bRev3d?segment.m_trim_domain[1]:segment.m_trim_domain[0];
    for(int i=1;i<16;++i){
        const double edgeParameter=segment.m_edge_domain.ParameterAt(i/16.);
        const auto mapping=nativeTrimParameterAt(trim,edge,surface,segment.m_trim_domain,edgeParameter,effective);
        if(trim.m_bRev3d?mapping.parameter>=previous:mapping.parameter<=previous)throw ExchangeError("Native trim interior parameter mapping is not monotone");
        previous=mapping.parameter;interiorMaximum=std::max(interiorMaximum,mapping.deviation);
        const auto uv=trim.PointAt(mapping.parameter);
        interior.append(QJsonObject{{"edge_parameter",edgeParameter},{"trim_parameter",mapping.parameter},{"uv",QJsonArray{uv.x,uv.y}},{"deviation",mapping.deviation}});
    }
    return QJsonObject{{"status","endpoints_and_interior_samples_verified"},{"interior_mapping","projected_samples_verified"},{"source_tolerance",sourceTolerance},{"effective_tolerance",effective},{"maximum_endpoint_deviation",maximum},{"maximum_interior_sample_deviation",interiorMaximum},{"trim_reversed_3d",trim.m_bRev3d},{"endpoints",endpoints},{"interior_samples",interior},{"mapping_scope","bounded OCCT projection with native physical evaluation; not certified global correspondence"}};
}
struct Resolver {
    std::shared_ptr<NativeReferenceGraph> graph;
    std::shared_ptr<SharedOwners> owners;
    std::set<QString> active,done;
    std::vector<QString> stack;
    std::map<QString,std::set<QString>> dependencies;
    std::set<const ON_Geometry*> tree;
    QJsonArray references;
    const ON_Geometry* sourceGeometry(const QString& id){
        auto component=ON_ModelGeometryComponent::Cast(graph->source->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(id.toLatin1().constData())).ModelComponent());
        auto geometry=component?component->Geometry(nullptr):nullptr;
        if(!geometry)throw ExchangeError("Missing native reference geometry ["+id.toStdString()+"]");return geometry;
    }
    ON_Curve* ownedCurve(const QString& id,unsigned depth){
        if(depth>=owners->limits.depth)throw Limit("Native reference graph depth limit exceeded");
        if(active.contains(id))throw ExchangeError("Cyclic native reference graph ["+id.toStdString()+"]");
        if(done.contains(id))return ON_Curve::Cast(graph->geometry.at(id).get());
        if(auto found=owners->entries.find(id);found!=owners->entries.end()){
            ++owners->cacheHits;
            for(const auto& [key,geometry]:found->second.closure)graph->geometry[key]=geometry;
            for(auto value:found->second.references)references.append(value);
            done.insert(id);return ON_Curve::Cast(found->second.geometry.get());
        }
        auto source=ON_Curve::Cast(sourceGeometry(id));if(!source)throw ExchangeError("Native curve reference targets a non-curve ["+id.toStdString()+"]");
        // Inspect all stable fields and limits before cloning. Unknown/plugin
        // adapters are a retained capability gap, not invalid geometry.
        nativeCurveTreeFields(*source);active.insert(id);
        const auto firstReference=references.size();stack.push_back(id);
        owners->reserve(*source);
        auto copy=std::shared_ptr<ON_Geometry>(ON_Geometry::Cast(source->Duplicate()));auto result=ON_Curve::Cast(copy.get());
        if(!result||result->ClassId()!=source->ClassId())throw ExchangeError("Native reference clone changed class");
        graph->geometry[id]=copy;walk(*result,depth);
        if(!result->IsValid())throw ExchangeError("Linked native curve is invalid ["+id.toStdString()+"]");
        CachedOwner cached;cached.geometry=copy;
        cached.closure[id]=copy;
        for(const auto& dependency:dependencies[id])for(const auto& [key,geometry]:owners->entries.at(dependency).closure)cached.closure[key]=geometry;
        for(qsizetype i=firstReference;i<references.size();++i)cached.references.append(references[i]);
        owners->entries[id]=std::move(cached);stack.pop_back();active.erase(id);done.insert(id);return result;
    }
    void link(ON_PolyEdgeSegment& segment,unsigned depth){
        auto id=uuid(segment.m_object_id);if(ON_UuidIsNil(segment.m_object_id))throw ExchangeError("Native reference has nil owner UUID");
        auto target=sourceGeometry(id);const ON_Curve* curve=nullptr;const ON_Brep* brep=nullptr;const ON_BrepEdge* edge=nullptr;const ON_BrepTrim* trim=nullptr;const ON_BrepFace* face=nullptr;
        const auto ci=segment.m_component_index;QString kind="curve";
        if(ci.m_type==ON_COMPONENT_INDEX::invalid_type&&ci.m_index==-1){dependencies[stack.back()].insert(id);curve=ownedCurve(id,depth+1);}
        else{
            brep=ON_Brep::Cast(target);if(!brep||!brep->IsValid())throw ExchangeError("Native component reference requires a valid Brep");
            if(ci.m_type==ON_COMPONENT_INDEX::brep_edge){if(ci.m_index<0||ci.m_index>=brep->m_E.Count())throw ExchangeError("Native reference edge index out of bounds");edge=&brep->m_E[ci.m_index];kind="brep_edge";}
            else if(ci.m_type==ON_COMPONENT_INDEX::brep_trim){if(ci.m_index<0||ci.m_index>=brep->m_T.Count())throw ExchangeError("Native reference trim index out of bounds");trim=&brep->m_T[ci.m_index];edge=trim->Edge();face=trim->Face();kind="brep_trim";if(!face||!face->SurfaceOf())throw ExchangeError("Native trim has no owning surface");}
            else throw ExchangeError("Unsupported native reference component type");
            curve=edge?edge->EdgeCurveOf():nullptr;if(!curve)throw ExchangeError("Native reference component has no3D edge curve");
            interval(segment.m_edge_domain,"Missing native edge subdomain");if(!edge->Domain().Includes(segment.m_edge_domain))throw ExchangeError("Native edge subdomain is outside target");
            if(trim){interval(segment.m_trim_domain,"Missing native trim subdomain");if(!trim->Domain().Includes(segment.m_trim_domain))throw ExchangeError("Native trim subdomain is outside target");}
            nativeCurveTreeFields(*curve);
        }
        QJsonObject trimAnalysis;
        const auto evaluation=segment.Domain(),proxy=segment.ProxyCurveDomain();const bool reversed=segment.ProxyCurveIsReversed();
        interval(evaluation,"Invalid native reference evaluation domain");interval(proxy,"Invalid native reference proxy domain");
        if(!curve->Domain().Includes(proxy))throw ExchangeError("Native reference proxy domain is outside target curve");
        if(edge){
            double first=edge->RealCurveParameter(segment.m_edge_domain[0]),last=edge->RealCurveParameter(segment.m_edge_domain[1]);
            if(first>last)std::swap(first,last);
            const auto equal=[](double a,double b){return std::isfinite(a)&&std::isfinite(b)&&std::abs(a-b)<=64*std::numeric_limits<double>::epsilon()*std::max({1.,std::abs(a),std::abs(b)});};
            if(!equal(first,proxy[0])||!equal(last,proxy[1]))throw ExchangeError("Native edge/proxy parameter intervals are inconsistent");
        }
        if(trim)trimAnalysis=trimDomainAnalysis(segment,*trim,*edge,*face->SurfaceOf(),graph->source->m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);
        // SetProxyCurve resets direction/domain; restore both from archive fields.
        segment.SetProxyCurve(curve,proxy);if(reversed&&!segment.ON_CurveProxy::Reverse())throw ExchangeError("Cannot restore native reference direction");
        if(!segment.SetDomain(evaluation)||segment.ProxyCurveDomain()!=proxy||segment.ProxyCurveIsReversed()!=reversed)throw ExchangeError("Native reference linking changed serialized parameter fields");
        segment.m_brep=brep;segment.m_edge=edge;segment.m_trim=trim;segment.m_face=face;segment.m_surface=face?face->SurfaceOf():nullptr;segment.DestroyRuntimeCache();
        if(!segment.IsValid())throw ExchangeError("Linked native reference segment invalid");
        QJsonArray samples;for(int i=0;i<5;++i){auto p=segment.PointAt(evaluation.ParameterAt(i/4.));if(!p.IsValid())throw ExchangeError("Native reference produced invalid evaluation");samples.append(QJsonArray{p.x,p.y,p.z});}
        QJsonObject reference{{"owner_uuid",id},{"target_kind",kind},{"component_index",QJsonArray{static_cast<int>(ci.m_type),ci.m_index}},{"reversed",reversed},{"evaluation_domain",QJsonArray{evaluation[0],evaluation[1]}},{"proxy_domain",QJsonArray{proxy[0],proxy[1]}},{"samples",samples}};
        if(trim)reference["trim_domain_analysis"]=trimAnalysis;
        references.append(reference);
    }
    void walk(ON_Geometry& geometry,unsigned depth){
        if(depth>=owners->limits.depth||owners->visitedNodes>=owners->limits.nodes)throw Limit("Native reference aggregate node/depth limit exceeded");
        ++owners->visitedNodes;
        if(!tree.insert(&geometry).second)throw ExchangeError("Native reference owning alias/cycle detected");
        const auto id=geometry.ClassId();
        if(id==&ON_CLASS_RTTI(ON_PolyEdgeSegment))link(static_cast<ON_PolyEdgeSegment&>(geometry),depth);
        else if(id==&ON_CLASS_RTTI(ON_CurveOnSurface)){
            auto& c=static_cast<ON_CurveOnSurface&>(geometry);if(!c.m_c2||!c.m_s)throw ExchangeError("Missing native surface-curve child");walk(*c.m_c2,depth+1);if(c.m_c3)walk(*c.m_c3,depth+1);walk(*c.m_s,depth+1);
        }else if(id==&ON_CLASS_RTTI(ON_PolyCurve)||id==&ON_CLASS_RTTI(ON_PolyEdgeCurve)){
            auto& c=static_cast<ON_PolyCurve&>(geometry);for(int i=0;i<c.Count();++i){auto child=c.SegmentCurve(i);if(!child)throw ExchangeError("Missing native polycurve segment");walk(*child,depth+1);}c.DestroyRuntimeCache();
        }else if(id==&ON_CLASS_RTTI(ON_RevSurface)){auto& s=static_cast<ON_RevSurface&>(geometry);if(!s.m_curve)throw ExchangeError("Missing native revolution curve");walk(*s.m_curve,depth+1);}
        else if(id==&ON_CLASS_RTTI(ON_SumSurface)){auto& s=static_cast<ON_SumSurface&>(geometry);for(auto c:s.m_curve){if(!c)throw ExchangeError("Missing native sum curve");walk(*c,depth+1);}}
        else if(id==&ON_CLASS_RTTI(ON_Extrusion)){auto& s=static_cast<ON_Extrusion&>(geometry);if(!s.m_profile)throw ExchangeError("Missing native extrusion profile");walk(*s.m_profile,depth+1);}
        else if(id!=&ON_CLASS_RTTI(ON_NurbsCurve)&&id!=&ON_CLASS_RTTI(ON_LineCurve)&&id!=&ON_CLASS_RTTI(ON_ArcCurve)&&id!=&ON_CLASS_RTTI(ON_PolylineCurve)&&id!=&ON_CLASS_RTTI(ON_NurbsSurface)&&id!=&ON_CLASS_RTTI(ON_PlaneSurface))throw Unavailable("Native reference class adapter unavailable ["+std::string(id->ClassName())+"]");
        geometry.DestroyRuntimeCache();tree.erase(&geometry);
    }
};
}
void remapDeferredNativeReferences(ON_Curve& curve,const std::map<QString,ON_UUID>& aliases,NativeReferenceLimits limits){
    if(limits.nodes>16384||limits.depth>64)throw ExchangeError("Native remap limits exceed supported ceilings");
    // This validates numeric/text sizes before traversal or metadata conversion.
    nativeCurveTreeFields(curve);
    std::set<ON_Geometry*> active,visited;
    std::vector<std::pair<ON_PolyEdgeSegment*,ON_UUID>> ownerSlots;
    size_t nodes=0;
    const QRegularExpression pattern("[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}");
    std::function<void(ON_Geometry&,unsigned)> preflight=[&](ON_Geometry& geometry,unsigned depth){
        if(depth>=limits.depth||++nodes>limits.nodes||!active.insert(&geometry).second||!visited.insert(&geometry).second)
            throw ExchangeError("Native remap depth/node/cycle/shared-child limit exceeded");
        for(auto data=geometry.FirstUserData();data;data=data->Next())if(data->ClassId()!=&ON_CLASS_RTTI(ON_UserStringList))
            throw ExchangeError("Opaque native child userdata prevents safe UUID remapping");
        ON_ClassArray<ON_UserString> strings;geometry.GetUserStrings(strings);
        for(int i=0;i<strings.Count();++i)for(auto text:{strings[i].m_key,strings[i].m_string_value}){
            ON_String utf8(text);auto matches=pattern.globalMatch(QString::fromUtf8(utf8.Array()));
            while(matches.hasNext()){auto identity=matches.next().captured().toLower();auto found=aliases.find(identity);
                if(found!=aliases.end()&&uuid(found->second)!=identity)throw ExchangeError("Native child user text identity reference requires explicit remapping");}
        }
        auto id=geometry.ClassId();
        if(id==&ON_CLASS_RTTI(ON_PolyEdgeSegment)){
            auto& segment=static_cast<ON_PolyEdgeSegment&>(geometry);
            if(segment.ProxyCurve()||segment.m_brep||segment.m_edge||segment.m_trim||segment.m_face||segment.m_surface)
                throw ExchangeError("Native UUID remap requires a detached deferred tree, not live proxy pointers");
            auto found=aliases.find(uuid(segment.m_object_id));
            if(ON_UuidIsNil(segment.m_object_id)||found==aliases.end()||ON_UuidIsNil(found->second))throw ExchangeError("Missing or nil native owner UUID mapping");
            ownerSlots.emplace_back(&segment,found->second);
        }else if(id==&ON_CLASS_RTTI(ON_PolyCurve)||id==&ON_CLASS_RTTI(ON_PolyEdgeCurve)){
            auto& poly=static_cast<ON_PolyCurve&>(geometry);for(int i=0;i<poly.Count();++i){auto child=poly.SegmentCurve(i);if(!child)throw ExchangeError("Missing native remap polycurve child");preflight(*child,depth+1);}
        }else if(id==&ON_CLASS_RTTI(ON_CurveOnSurface)){
            auto& c=static_cast<ON_CurveOnSurface&>(geometry);if(!c.m_c2||!c.m_s)throw ExchangeError("Missing native remap surface-curve child");preflight(*c.m_c2,depth+1);if(c.m_c3)preflight(*c.m_c3,depth+1);preflight(*c.m_s,depth+1);
        }else if(id==&ON_CLASS_RTTI(ON_RevSurface)){
            auto& s=static_cast<ON_RevSurface&>(geometry);if(!s.m_curve)throw ExchangeError("Missing native remap revolution child");preflight(*s.m_curve,depth+1);
        }else if(id==&ON_CLASS_RTTI(ON_SumSurface)){
            auto& s=static_cast<ON_SumSurface&>(geometry);for(auto c:s.m_curve){if(!c)throw ExchangeError("Missing native remap sum child");preflight(*c,depth+1);}
        }else if(id==&ON_CLASS_RTTI(ON_Extrusion)){
            auto& s=static_cast<ON_Extrusion&>(geometry);if(!s.m_profile)throw ExchangeError("Missing native remap extrusion child");preflight(*s.m_profile,depth+1);
        }else if(id!=&ON_CLASS_RTTI(ON_NurbsCurve)&&id!=&ON_CLASS_RTTI(ON_LineCurve)&&id!=&ON_CLASS_RTTI(ON_ArcCurve)&&id!=&ON_CLASS_RTTI(ON_PolylineCurve)&&id!=&ON_CLASS_RTTI(ON_NurbsSurface)&&id!=&ON_CLASS_RTTI(ON_PlaneSurface))
            throw ExchangeError("Native UUID remap class adapter unavailable");
        active.erase(&geometry);
    };
    preflight(curve,0);
    // No mutation occurs until every owner slot and child dependency is safe.
    for(auto [segment,target]:ownerSlots)segment->m_object_id=target;
    for(auto geometry:visited)geometry->DestroyRuntimeCache();
}
std::shared_ptr<NativeReferenceResolution> resolveNativeReferences(std::shared_ptr<const ONX_Model> source,const QJsonArray& records,NativeReferenceLimits limits){
    if(limits.cloneBytes>512ULL*1024*1024||limits.owners>16384||limits.nodes>16384||limits.depth>64)throw ExchangeError("Native reference limits exceed supported ceilings");
    auto result=std::make_shared<NativeReferenceResolution>();
    auto owners=std::make_shared<SharedOwners>();owners->limits=limits;
    for(auto value:records){auto row=value.toObject();const bool surfaceCurve=row["class_name"]=="ON_CurveOnSurface";
        const bool referenceCurve=row.contains("poly_edge_native");
        if((!surfaceCurve&&!referenceCurve)||row[surfaceCurve?"curve_on_surface_native":"poly_edge_native"].toObject()["object_references"].toArray().isEmpty())continue;
        auto id=row["source_uuid"].toString();Resolver resolver;resolver.owners=owners;resolver.graph=std::make_shared<NativeReferenceGraph>();resolver.graph->source=source;resolver.graph->rootUuid=id;
        try{auto root=resolver.ownedCurve(id,0);auto c=ON_CurveOnSurface::Cast(root);if(surfaceCurve&&(!c||!c->IsValid()))throw ExchangeError("Resolved CurveOnSurface has invalid parameter/approximation dimensions");
            result->diagnostics[id]=QJsonObject{{"status","linked"},{"native_valid",true},{"references",resolver.references}};result->graphs[id]=std::move(resolver.graph);
        }catch(const Limit& e){result->diagnostics[id]=QJsonObject{{"status","limit"},{"message",e.what()}};}
        catch(const Unavailable& e){result->diagnostics[id]=QJsonObject{{"status","unavailable"},{"message",e.what()}};}
        catch(const ExchangeError& e){result->diagnostics[id]=QJsonObject{{"status","invalid"},{"message",e.what()}};}
    }result->statistics=QJsonObject{{"sdk_accounted_clone_bytes",static_cast<double>(owners->chargedBytes)},{"clone_attempts",static_cast<double>(owners->cloneAttempts)},{"visited_nodes",static_cast<double>(owners->visitedNodes)},{"shared_cache_hits",static_cast<double>(owners->cacheHits)}};return result;
}
}
