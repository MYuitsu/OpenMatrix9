// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CorePreparedSnapshot.h"
#include "CorePreparedCad.h"
#include "NativeInputTree.h"
#include <BRepBuilderAPI_Copy.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <BRepTools.hxx>
#include <Standard_Version.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonParseError>
#include <QCryptographicHash>
#include <cstdint>
#include <locale>
#include <limits>
#include <ostream>
#include <streambuf>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
struct Summary {std::uint32_t status,reserved;std::size_t needed;};
static_assert(sizeof(Summary)==16&&alignof(Summary)==8);
extern "C" Summary om9_prepared_result_json(const std::uint8_t*,std::size_t,std::uint8_t*,std::size_t);
extern "C" std::size_t om9_prepared_master_limit();
extern "C" unsigned om9_prepared_cad_budget(std::size_t,std::size_t,std::size_t,std::size_t);
class BoundedBuffer final:public std::streambuf {
    std::string bytes;
    std::size_t limit;
protected:
    std::streamsize xsputn(const char* data,std::streamsize count) override {
        if(count<0||static_cast<std::size_t>(count)>limit-bytes.size())
            throw std::runtime_error("Native master serialization exceeds Rust byte budget");
        bytes.append(data,static_cast<std::size_t>(count));return count;
    }
    int_type overflow(int_type value) override {
        if(traits_type::eq_int_type(value,traits_type::eof()))return traits_type::not_eof(value);
        const auto c=traits_type::to_char_type(value);xsputn(&c,1);return value;
    }
public:
    explicit BoundedBuffer(std::size_t maximum):limit(maximum){}
    const std::string& value()const{return bytes;}
};
QString nativeMaster(const TopoDS_Shape& shape){
    BoundedBuffer buffer(om9_prepared_master_limit());std::ostream stream(&buffer);
    stream.imbue(std::locale::classic());stream.exceptions(std::ios::badbit|std::ios::failbit);
    // Existing OCCT serializer retains the complete native CAD tree. Mesh cache
    // and normals are intentionally excluded; this is not a 3DM conversion.
    BRepTools::Write(shape,stream,false,false,TopTools_FormatVersion_VERSION_3);
    const auto& bytes=buffer.value();return QString::fromUtf8(bytes.data(),static_cast<qsizetype>(bytes.size()));
}
QJsonArray nativeLocation(const TopLoc_Location& location){
    const auto& transform=location.Transformation();QJsonArray matrix;
    for(int row=1;row<=3;++row){QJsonArray values;for(int column=1;column<=4;++column)values.append(transform.Value(row,column));matrix.append(values);}
    matrix.append(QJsonArray{0,0,0,1});return matrix;
}
}
QByteArray preparedCadNativeMasterDigest(const TopoDS_Shape& shape){
    // Checked is a mutable OCCT validation cache, not geometry. Native Union
    // publication clears it on a face without changing any CAD data. Normalize
    // only in privately copied topology; shared curve/surface data are read-only.
    const auto summary=ThreeDm::InputTree::validate(shape);
    if(summary.nodes>std::numeric_limits<std::size_t>::max()/2||summary.bytes>std::numeric_limits<std::size_t>::max()/2
        ||om9_prepared_cad_budget(0,0,summary.nodes*2,summary.bytes*2))throw std::runtime_error("InvalidBasis: native CAD master normalization budget");
    // Strip root Location on a value wrapper before copying. OCCT Copy expands
    // cumulative locations; stripping afterwards retains datum-chain identity
    // from the old root in descendant locations despite equal numeric frames.
    auto local=shape;local.Location(TopLoc_Location());
    BRepBuilderAPI_Copy copy(local,false,false);if(!copy.IsDone()||copy.Shape().IsNull())throw std::runtime_error("KernelFailure: native CAD master topology copy failed");
    auto canonical=copy.Shape();
    TopTools_IndexedMapOfShape shapes;TopExp::MapShapes(canonical,shapes);
    for(int index=1;index<=shapes.Extent();++index){auto item=shapes(index);item.Checked(false);}
    return QCryptographicHash::hash(nativeMaster(canonical).toUtf8(),QCryptographicHash::Sha256);
}
QJsonObject preparedCadResult(const TopoDS_Shape& shape,const QJsonObject& topology,
    const QJsonObject& validation,const QJsonObject& attributes,
    const QJsonObject& identity,const std::string& unitContext,const QJsonObject& operationContext){
    const auto operation=operationContext.isEmpty()?QByteArray("CommitImport3dm"):operationContext.value("operation").toString().toUtf8();
    const auto snapshot=preparedCadSnapshot(shape,topology,validation,attributes,unitContext,identity,operation.constData());
    return preparedGeometryResult(snapshot,validation,attributes,identity,unitContext,operationContext);
}
QJsonObject preparedCadSnapshot(const TopoDS_Shape& shape,const QJsonObject& topology,
    const QJsonObject& validation,const QJsonObject& attributes,const std::string& unitContext,
    const QJsonObject& identity,const char* operation){
    const auto kind=static_cast<unsigned>(shape.ShapeType());
    static const char* names[]={"brep","brep","solid","shell","trimmed_face","polycurve","curve","point"};
    static const char* orientations[]={"forward","reversed","internal","external"};
    const auto orientation=static_cast<unsigned>(shape.Orientation());
    if(kind>=std::size(names)||orientation>=std::size(orientations))throw PreparedCadFailure(1,operation,identity,"Unsupported native master representation/orientation");
    QJsonObject master{{"encoding","occt_brep_ascii_v3"},{"sdk_version",OCC_VERSION_COMPLETE},
        {"triangulation_included",false},{"payload",nativeMaster(shape)}};
    QJsonObject geometry{{"representation",names[kind]},{"coordinates","native_local_with_locations"},
        {"native_root_orientation",orientations[orientation]},{"native_root_location",nativeLocation(shape.Location())},
        {"native_master",master}};
    auto ownedTopology=topology;ownedTopology["validation"]=validation;
    ownedTopology["native_shape_is_solid"]=shape.ShapeType()==TopAbs_SOLID;
    QJsonObject snapshot{{"schema_version",1},{"ownership","private_native_copy"},{"native_shape_type",static_cast<int>(kind)},
        {"unit_context_snapshot",QString::fromUtf8(unitContext.data(),unitContext.size())},
        {"geometry_data",geometry},{"topology_data",ownedTopology},{"source_attributes",attributes}};
    return snapshot;
}
QJsonObject preparedGeometryResult(QJsonObject snapshot,const QJsonObject& validation,
    const QJsonObject& attributes,const QJsonObject& identity,const std::string& unitContext,const QJsonObject& operationContext){
    const auto operation=operationContext.isEmpty()?QByteArray("CommitImport3dm"):operationContext.value("operation").toString().toUtf8();
    snapshot["schema_version"]=1;snapshot["source_attributes"]=attributes;
    snapshot["unit_context_snapshot"]=QString::fromUtf8(unitContext.data(),unitContext.size());
    QJsonObject request{{"snapshot",snapshot},{"input_identity",identity},{"validation",validation}};
    if(!operationContext.isEmpty())request["operation_context"]=operationContext;
    const auto raw=QJsonDocument(request).toJson(QJsonDocument::Compact);
    const auto query=om9_prepared_result_json(reinterpret_cast<const std::uint8_t*>(raw.constData()),raw.size(),nullptr,0);
    if(query.status||query.reserved||!query.needed)throw PreparedCadFailure(query.status?query.status:7,operation.constData(),identity,"Rust rejected owned prepared CAD snapshot");
    QByteArray output(static_cast<qsizetype>(query.needed),'\0');
    const auto copied=om9_prepared_result_json(reinterpret_cast<const std::uint8_t*>(raw.constData()),raw.size(),reinterpret_cast<std::uint8_t*>(output.data()),output.size());
    if(copied.status||copied.reserved||copied.needed!=query.needed)throw PreparedCadFailure(copied.status?copied.status:7,operation.constData(),identity,"Rust rejected prepared result output buffer");
    QJsonParseError error;const auto parsed=QJsonDocument::fromJson(output,&error);
    if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw PreparedCadFailure(7,operation.constData(),identity,"Invalid Rust prepared result JSON");
    return parsed.object();
}
}
