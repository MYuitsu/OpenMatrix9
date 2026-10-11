// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreSourceAttributes.h"
#include "CoreObjects.h"
#include <App/DocumentObject.h>
#include <App/Material.h>
#include <Gui/Application.h>
#include <Gui/ViewProviderGeometryObject.h>
#include <QJsonDocument>
#include <QJsonParseError>
#include <array>
#include <cstdint>
#include <cstring>
#include <limits>
#include <stdexcept>
#include <string_view>
#include <type_traits>
#include <vector>
namespace OpenMatrix9Gui {
namespace {
struct Text {const std::uint8_t* bytes;std::size_t length;};
struct Material {double colors[4][4],shininess,transparency;Text text[3];};
struct AttributesInput {Text text[6];std::uint32_t known,values;const Material* materials;std::size_t count;};
struct JsonSummary {std::uint32_t status,reserved;std::size_t needed;};
static_assert(sizeof(Text)==16&&alignof(Text)==8);
static_assert(std::is_standard_layout_v<Material>&&sizeof(Material)==192&&alignof(Material)==8&&offsetof(Material,text)==144);
static_assert(std::is_standard_layout_v<AttributesInput>&&sizeof(AttributesInput)==120&&alignof(AttributesInput)==8&&offsetof(AttributesInput,materials)==104);
static_assert(sizeof(JsonSummary)==16&&alignof(JsonSummary)==8);
extern "C" JsonSummary om9_source_attributes_json(AttributesInput,std::uint8_t*,std::size_t);
extern "C" std::uint32_t om9_source_attribute_counts(std::size_t,std::size_t);
Text text(const std::string& s){return {reinterpret_cast<const std::uint8_t*>(s.data()),s.size()};}
Text typeName(std::string_view s){return {reinterpret_cast<const std::uint8_t*>(s.data()),s.size()};}
}
QJsonObject sourceAttributes(App::DocumentObject& object,const ObjectSnapshot& snapshot) {
    AttributesInput input{};
    input.text[0]=text(snapshot.documentUid);input.text[1]=text(snapshot.objectName);
    input.text[2]=typeName(object.getTypeId().getName());input.text[3]=text(object.Label.getStrValue());
    input.text[4]=text(object.Label2.getStrValue());input.known=1|4;input.values=object.Visibility.getValue()?1:0;
    const std::vector<App::Material>* nativeMaterials=nullptr;
    if(auto* vp=Gui::Application::Instance->getViewProvider(&object)) {
        input.known|=8;input.text[5]=typeName(vp->getTypeId().getName());
        if(auto* geometry=dynamic_cast<Gui::ViewProviderGeometryObject*>(vp)) {
            input.known|=2|16;if(geometry->Selectable.getValue())input.values|=2;
            nativeMaterials=&geometry->ShapeAppearance.getValues();
        }
    }
    input.count=nativeMaterials?nativeMaterials->size():0;
    if(om9_source_attribute_counts(input.count,0))throw std::runtime_error("Source appearance exceeds owned attribute budget");
    // Preflight live string lengths before allocating the native ABI vector.
    std::size_t bytes=0;
    auto account=[&](Text t){
        if(t.length>65536||t.length>std::numeric_limits<std::size_t>::max()-bytes)
            throw std::runtime_error("Source attribute text exceeds owned budget");
        bytes+=t.length;
        if(om9_source_attribute_counts(input.count,bytes))throw std::runtime_error("Source attributes exceed owned aggregate budget");
    };
    for(const auto t:input.text)account(t);
    if(nativeMaterials)for(const auto& m:*nativeMaterials){account(text(m.uuid));account(text(m.image));account(text(m.imagePath));}
    std::vector<Material> materials;materials.reserve(input.count);
    if(nativeMaterials)for(const auto& m:*nativeMaterials) {
        Material copy{};
        const std::array<Base::Color,4> colors{m.ambientColor,m.diffuseColor,m.specularColor,m.emissiveColor};
        for(std::size_t i=0;i<colors.size();++i){const auto& c=colors[i];copy.colors[i][0]=c.r;copy.colors[i][1]=c.g;copy.colors[i][2]=c.b;copy.colors[i][3]=c.a;}
        copy.shininess=m.shininess;copy.transparency=m.transparency;
        copy.text[0]=text(m.uuid);copy.text[1]=text(m.image);copy.text[2]=text(m.imagePath);materials.push_back(copy);
    }
    input.materials=materials.data();
    const auto query=om9_source_attributes_json(input,nullptr,0);
    if(query.status||query.needed>8*1024*1024)throw std::runtime_error("Rust rejected native source attributes (status "+std::to_string(query.status)+")");
    QByteArray json(static_cast<int>(query.needed),'\0');
    const auto copied=om9_source_attributes_json(input,reinterpret_cast<std::uint8_t*>(json.data()),query.needed);
    if(copied.status||copied.needed!=query.needed)throw std::runtime_error("Rust could not copy source attributes atomically");
    QJsonParseError error;const auto parsed=QJsonDocument::fromJson(json,&error);
    if(error.error!=QJsonParseError::NoError||!parsed.isObject())throw std::runtime_error("Invalid Rust source attributes JSON");
    return parsed.object();
}
}
