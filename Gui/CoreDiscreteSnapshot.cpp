// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreDiscreteSnapshot.h"
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/ComplexGeoData.h>
#include <Mod/Mesh/App/Mesh.h>
#include <Mod/Points/App/Points.h>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonParseError>
#include <array>
#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <vector>
#include <limits>
namespace OpenMatrix9Gui {
namespace {
using Point=std::array<double,3>;
using Triangle=std::array<std::uint64_t,3>;
struct Input {std::uint32_t representation,reserved;const Point* points;std::size_t point_count;const Triangle* faces;std::size_t face_count;};
struct JsonSummary {std::uint32_t status,reserved;std::size_t needed;};
static_assert(sizeof(Input)==40&&alignof(Input)==8&&offsetof(Input,points)==8&&offsetof(Input,faces)==24);
static_assert(sizeof(Point)==24&&sizeof(Triangle)==24&&sizeof(JsonSummary)==16);
extern "C" std::uint32_t om9_discrete_counts(std::uint32_t,std::size_t,std::size_t);
extern "C" JsonSummary om9_discrete_json(Input,std::uint8_t*,std::size_t);
extern "C" std::uint32_t om9_prepared_cad_budget(std::size_t,std::size_t,std::size_t,std::size_t);
extern "C" std::uint32_t om9_prepared_number(double);
extern "C" std::uint32_t om9_prepared_binding_field(std::uint32_t,std::uint32_t,double,std::size_t);
void counts(std::uint32_t kind,std::size_t points,std::size_t faces){if(om9_discrete_counts(kind,points,faces))throw std::runtime_error("Native discrete count/retained-data budget exceeded");}
bool exact(const Base::BaseClass& value,const char* name){const auto type=Base::Type::fromName(name);return !type.isBad()&&value.getTypeId()==type;}
QJsonObject capture(const Data::ComplexGeoData* native,bool mesh) {
    const auto kind=mesh?8u:9u;
    // Points does not register PointKernel in its module initializer. The exact
    // builtin PropertyPointKernel checked above owns a concrete PointKernel;
    // its getComplexData() returns that allocation, not a replaceable subclass.
    // Mesh registers MeshObject and additionally permits its exact type check.
    std::vector<Point> points;std::vector<Triangle> faces;
    if(mesh){
        // Exact builtin native type checked above; these inline getters retain
        // module-owned initialized storage. No generic unbounded copy/getter.
        const auto& kernel=static_cast<const Mesh::MeshObject*>(native)->getKernel();
        const auto& vertices=kernel.GetPoints();const auto& triangles=kernel.GetFacets();
        counts(kind,vertices.size(),triangles.size());points.reserve(vertices.size());faces.reserve(triangles.size());
        for(const auto& p:vertices){if(p.IsFlag(MeshCore::MeshPoint::INVALID))throw std::runtime_error("Invalid native mesh vertex owner");points.push_back({p.x,p.y,p.z});}
        for(const auto& f:triangles){if(f.IsFlag(MeshCore::MeshFacet::INVALID))throw std::runtime_error("Invalid native mesh facet owner");faces.push_back({f._aulPoints[0],f._aulPoints[1],f._aulPoints[2]});}
    }else{
        const auto& vertices=static_cast<const Points::PointKernel*>(native)->getBasicPoints();
        counts(kind,vertices.size(),0);points.reserve(vertices.size());
        for(const auto& p:vertices)points.push_back({p.x,p.y,p.z});
    }
    Input input{kind,0,points.data(),points.size(),faces.data(),faces.size()};
    const auto query=om9_discrete_json(input,nullptr,0);
    if(query.status)throw std::runtime_error("Rust rejected discrete geometry values/serialized budget (status "+std::to_string(query.status)+")");
    QByteArray raw(static_cast<int>(query.needed),'\0');
    const auto copied=om9_discrete_json(input,reinterpret_cast<std::uint8_t*>(raw.data()),query.needed);
    if(copied.status||copied.needed!=query.needed)throw std::runtime_error("Rust could not copy owned discrete geometry atomically");
    QJsonParseError error;const auto parsed=QJsonDocument::fromJson(raw,&error);
    if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw std::runtime_error("Invalid Rust discrete geometry JSON");
    auto data=parsed.object();data["native_storage_precision"]="float32";data["bounds_coordinates"]="native_local";
    // Virtual native API avoids linking/loading extra modules at plugin startup.
    const auto transform=native->getTransform();QJsonArray matrix;
    for(int r=0;r<4;++r)matrix.append(QJsonArray{transform[r][0],transform[r][1],transform[r][2],transform[r][3]});
    return {{"geometry_data",data},{"geometry_data_status","copied_validated_discrete_geometry"},{"native_discrete_location",matrix}};
}
}
QJsonObject meshSnapshot(const Mesh::MeshObject& native){
    if(!exact(native,"Mesh::MeshObject"))throw std::runtime_error("Unsupported discrete native kernel owner");
    return capture(&native,true);
}
MeshInputBudget meshInputBudget(const Mesh::MeshObject& owner){
    if(!exact(owner,"Mesh::MeshObject"))throw MeshInputFailure(1,"Unsupported native mesh owner");
    const auto& points=owner.getKernel().GetPoints();const auto& facets=owner.getKernel().GetFacets();
    if(om9_discrete_counts(8,points.size(),facets.size()))throw MeshInputFailure(2,"Native mesh count budget exceeded");
    const std::size_t segments=owner.countSegments();
    if(segments==std::numeric_limits<std::size_t>::max())throw MeshInputFailure(2,"Native mesh owner count overflow");
    MeshInputBudget result{segments+1,0};
    auto check=[&]{if(om9_prepared_cad_budget(0,0,result.nodes,result.bytes))throw MeshInputFailure(2,"Native mesh owner/byte budget exceeded");};check();
    auto account=[&](std::size_t count,std::size_t size){
        if(size&&count>std::numeric_limits<std::size_t>::max()/size)throw MeshInputFailure(2,"Native mesh byte overflow");
        const auto added=count*size;if(added>std::numeric_limits<std::size_t>::max()-result.bytes)throw MeshInputFailure(2,"Native mesh byte overflow");
        result.bytes+=added;check();
    };
    account(1,sizeof(Mesh::MeshObject));account(points.size(),sizeof(MeshCore::MeshPoint));account(facets.size(),sizeof(MeshCore::MeshFacet));
    for(const auto& p:points)for(double n:{p.x,p.y,p.z})if(auto status=om9_prepared_number(n))throw MeshInputFailure(status,"Nonfinite native mesh coordinate");
    account(segments,sizeof(Mesh::Segment));
    for(auto at=owner.segments_begin();at!=owner.segments_end();++at){
        const auto& indices=at->getIndices();account(indices.size(),sizeof(Mesh::FacetIndex));
        account(at->getName().size(),1);account(at->getColor().size(),1);
        for(auto index:indices)if(auto status=om9_prepared_binding_field(9,6,static_cast<double>(index),facets.size()))throw MeshInputFailure(status,"Invalid native mesh segment index");
    }
    return result;
}
QJsonObject discreteSnapshot(App::DocumentObject& object,bool mesh){
    auto* property=object.getPropertyByName<App::PropertyComplexGeoData>(mesh?"Mesh":"Points");
    if(!property||!exact(*property,mesh?"Mesh::PropertyMeshKernel":"Points::PropertyPointKernel"))throw std::runtime_error("Unsupported discrete native property owner");
    const auto* native=property->getComplexData();
    if(!native||(mesh&&!exact(*native,"Mesh::MeshObject")))throw std::runtime_error("Unsupported discrete native kernel owner");
    return capture(native,mesh);
}
}
