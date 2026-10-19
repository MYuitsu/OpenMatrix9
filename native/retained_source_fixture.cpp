// SPDX-License-Identifier: LGPL-2.1-or-later
// Authored native test archives only; no application or merge capability.
#define wmain om9_retained_fixture_unused_exchange_entry
#include "native_exchange.cpp"
#undef wmain

class AuthoredRetainedCopyZeroData : public ON_UserData {
  ON_OBJECT_DECLARE(AuthoredRetainedCopyZeroData);
public:
  AuthoredRetainedCopyZeroData() {
    m_userdata_uuid=ON_UuidFromString("20000000-0000-4000-8000-0000000003ef");
    m_application_uuid=ON_UuidFromString("20000000-0000-4000-8000-0000000003ee");m_userdata_copycount=0;
  }
  bool Archive() const override {return true;}
  bool Write(ON_BinaryArchive& archive) const override {return archive.WriteString(payload);}
  bool Read(ON_BinaryArchive& archive) override {return archive.ReadString(payload);}
  ON_wString payload=L"OM9 authored original copycount-zero attachment";
};
ON_OBJECT_IMPLEMENT(AuthoredRetainedCopyZeroData,ON_UserData,"f0fb2e54-7de9-4dca-8f2c-76535788522f");

namespace {
ON_UUID authored_id(unsigned n){char value[37]{};std::snprintf(value,sizeof(value),"20000000-0000-4000-8000-%012x",n);return ON_UuidFromString(value);}
void authored_prefix(ON_BinaryArchive& archive,const NativeSourceArchive& source) {
  require(archive.Write3dmStartSection(50,"OM9 authored retained preparation fixture")
    &&archive.Write3dmProperties(source.properties())&&archive.Write3dmSettings(source.settings()),"authored fixture prefix");
  archive.SetReferencedComponentIndexMapping(false);archive.SetShouldSerializeUserDataDefault(true);
}
std::vector<unsigned char> authored_fragment(const ON_Write3dmBufferArchive& archive,size_t begin) {
  const auto end=archive.CurrentPosition();require(begin<=end&&end==archive.SizeOfArchive()&&end<=max_archive_bytes,"authored fixture fragment budget");
  const auto* bytes=static_cast<const unsigned char*>(archive.Buffer());return {bytes+begin,bytes+static_cast<size_t>(end)};
}
J authored_zero_oracle(const std::filesystem::path& path) {
  auto source=NativeSourceArchive::open(path);
  const auto& dim=source->record(8,authored_id(6));const auto& point=source->record(12,authored_id(10));
  const auto* style=AuthoredRetainedCopyZeroData::Cast(dim.object->FirstUserData());
  const auto* attributes=AuthoredRetainedCopyZeroData::Cast(point.attributes->FirstUserData());
  require(style&&attributes,"authored original copycount-zero oracle attachments");
  return J{{"status","authored_original_copycount_zero_oracle"},{"dimstyle_copy_count",style->m_userdata_copycount},
    {"attribute_copy_count",attributes->m_userdata_copycount},{"dimstyle_payload",bounded_native_text(style->payload)},
    {"attribute_payload",bounded_native_text(attributes->payload)}};
}
J author_retained(const std::wstring& mode,const std::filesystem::path& path,const std::filesystem::path& destination) {
  require(mode==L"copycount-zero"||mode==L"sparse"||mode==L"sparse-bad"||mode==L"light","authored fixture mode");
  auto source=NativeSourceArchive::open(path);const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();
  std::map<ON__UINT32,std::vector<unsigned char>> replacements;
  if(mode==L"copycount-zero") {
    replacements[TCODE_OBJECT_TABLE]=retained_table_fragment(TCODE_OBJECT_TABLE,[&](ON_BinaryArchive& archive){
      for(const auto& record:source->records())if(record.native_component_type==12) {
        if(record.id==authored_id(10)) {
          require(record.attributes&&record.attributes->AttachUserData(new AuthoredRetainedCopyZeroData),"authored copycount-zero object attributes");
          require(archive.Write3dmObject(*record.object,record.attributes.get()),"authored native copycount-zero object write");
        }else source->replay(archive,record.span);
      }
    });
    // Direct public class serialization of the ORIGINAL legacy style avoids
    // Write3dmDimStyle's internal copy, which discards copycount-zero userdata.
    ON_Write3dmBufferArchive archive(0,max_archive_bytes,50,ON::Version());authored_prefix(archive,*source);const auto begin=static_cast<size_t>(archive.CurrentPosition());
    require(archive.BeginWrite3dmChunk(TCODE_DIMSTYLE_TABLE,0),"authored original dimstyle table");bool found=false;
    for(const auto& record:source->records())if(record.native_component_type==8) {
      if(record.id==authored_id(6)){require(record.object->AttachUserData(new AuthoredRetainedCopyZeroData),"authored copycount-zero original dimstyle");found=true;}
      require(archive.BeginWrite3dmChunk(TCODE_DIMSTYLE_RECORD,0)&&archive.WriteObject(*record.object)&&archive.EndWrite3dmChunk(),"authored original dimstyle class serializer");
    }
    require(found&&archive.BeginWrite3dmChunk(TCODE_ENDOFTABLE,0)&&archive.EndWrite3dmChunk()&&archive.EndWrite3dmChunk(),"authored original dimstyle table end");
    replacements.emplace(TCODE_DIMSTYLE_TABLE,authored_fragment(archive,begin));
  }else if(mode==L"sparse"||mode==L"sparse-bad") {
    replacements[TCODE_LAYER_TABLE]=retained_table_fragment(TCODE_LAYER_TABLE,[&](ON_BinaryArchive& archive){
      for(const auto& record:source->records())if(record.native_component_type==5) {
        if(record.id==authored_id(2)){auto* layer=dynamic_cast<ON_Layer*>(record.object.get());require(layer&&layer->SetIndex(7)&&archive.Write3dmLayer(*layer),"authored serialized layer index7");}
        else source->replay(archive,record.span);
      }
    });
    ON_Write3dmBufferArchive archive(0,max_archive_bytes,50,ON::Version());authored_prefix(archive,*source);const auto begin=static_cast<size_t>(archive.CurrentPosition());
    require(archive.BeginWrite3dmGroupTable(),"authored sparse group table");
    for(const auto& record:source->records())if(record.native_component_type==6){auto* group=dynamic_cast<ON_Group*>(record.object.get());require(group&&group->SetIndex(5)&&archive.Write3dmGroup(*group),"authored serialized group index5");}
    require(archive.EndWrite3dmGroupTable(),"authored sparse group table end");replacements.emplace(TCODE_GROUP_TABLE,authored_fragment(archive,begin));
    replacements[TCODE_OBJECT_TABLE]=retained_table_fragment(TCODE_OBJECT_TABLE,[&](ON_BinaryArchive& objects){
      for(const auto& record:source->records())if(record.native_component_type==12) {
        auto* attributes=record.attributes.get();require(attributes,"authored object attributes");
        if(attributes->GroupCount()){require(attributes->GroupCount()==1&&attributes->GroupList()[0]==0,"authored initial group reference0");attributes->RemoveFromAllGroups();attributes->AddToGroup(5);}
        if(mode==L"sparse-bad"&&record.id==authored_id(10))attributes->m_layer_index=7;
        require(objects.Write3dmObject(*record.object,attributes),"authored actual sparse attribute references");
      }
    });
  }else {
    ON_Write3dmBufferArchive archive(0,max_archive_bytes,50,ON::Version());authored_prefix(archive,*source);const auto begin=static_cast<size_t>(archive.CurrentPosition());
    require(archive.BeginWrite3dmLightTable(),"authored light table");
    for(const auto& record:source->records())if(record.native_component_type==9)source->replay(archive,record.span);
    ON_Light light;light.m_light_id=authored_id(51);light.m_light_index=7;light.m_light_name=L"Authored serialized-index light";
    light.SetStyle(ON::world_directional_light);light.SetLocation(ON_3dPoint(1,2,3));light.SetDirection(ON_3dVector(0,0,-1));
    ON_3dmObjectAttributes attributes;attributes.m_uuid=authored_id(51);attributes.m_name=L"Authored original light attributes";attributes.m_layer_index=0;
    require(light.IsValid()&&archive.Write3dmLight(light,&attributes)&&archive.EndWrite3dmLightTable(),"authored actual light serializer");
    replacements.emplace(TCODE_LIGHT_TABLE,authored_fragment(archive,begin));
  }
  retained_write_assembled(*source,replacements,destination);require(ON_GetErrorCount()==errors&&ON_GetWarningCount()==warnings,"authored fixture native diagnostics");
  return J{{"status","authored_retained_source_fixture"},{"mode",native_text(ON_wString(mode.c_str()))}};
}
}
int wmain(int argc,wchar_t** argv) {
  try {
    require(argc==4,"retained fixture mode source output");Library library;const std::wstring mode(argv[1]);
    J result;
    if(mode==L"oracle-held-source") {
      File primary(std::filesystem::path(argv[2]),true);
      result=inventory_source(std::filesystem::path(argv[3]),nullptr,nullptr,primary.value);
    }else result=mode==L"oracle-copycount-zero"?authored_zero_oracle(std::filesystem::path(argv[2]))
      :author_retained(mode,std::filesystem::path(argv[2]),std::filesystem::path(argv[3]));
    std::cout<<result.dump()<<'\n';return 0;
  }catch(const std::exception& error){std::cerr<<"OM9 retained source fixture ("<<operation_record<<"): "<<error.what()<<'\n';return 1;}
}
