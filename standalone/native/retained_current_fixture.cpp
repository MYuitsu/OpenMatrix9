// SPDX-License-Identifier: LGPL-2.1-or-later
// Authored cache/userdata and valid-but-stale bbox negatives. No host capability.
#define wmain om9_retained_current_fixture_unused_entry
#include "native_exchange.cpp"
#undef wmain

class AuthoredCurrentCacheZero : public ON_UserData {
  ON_OBJECT_DECLARE(AuthoredCurrentCacheZero);
public:
  AuthoredCurrentCacheZero(){m_userdata_uuid=ON_UuidFromString("20000000-0000-4000-8000-0000000004ed");m_application_uuid=ON_UuidFromString("20000000-0000-4000-8000-0000000004ee");m_userdata_copycount=0;}
  bool Archive() const override{return true;}
  bool Write(ON_BinaryArchive& a) const override{return a.WriteString(payload);}
  bool Read(ON_BinaryArchive& a) override{return a.ReadString(payload);}
  ON_wString payload=L"Authored cache-only copycount-zero payload";
};
ON_OBJECT_IMPLEMENT(AuthoredCurrentCacheZero,ON_UserData,"e1392c63-40b5-4f49-8e6f-c0a9e2b0f117");
namespace {
ON_UUID cache_id(unsigned n){char value[37]{};std::snprintf(value,sizeof(value),"20000000-0000-4000-8000-%012x",n);return ON_UuidFromString(value);}
}
int wmain(int argc,wchar_t** argv) {
  try {
    require(argc==4,"cache fixture mode source output");Library library;auto source=NativeSourceArchive::open(std::filesystem::path(argv[2]));const std::wstring mode(argv[1]);J result;
    if(mode==L"oracle") {
      result=J{{"status","authored_current_cache_oracle"},{"records",J::array()}};
      for(const auto number:{18u,41u,30u}){const unsigned type=number==30?11:12;const auto& r=source->record(type,cache_id(number));const auto* ref=ON_InstanceRef::Cast(r.object.get());const auto* def=ON_InstanceDefinition::Cast(r.object.get());const auto* data=AuthoredCurrentCacheZero::Cast(r.object->FirstUserData());require(data,"authored original cache-only userdata missing");result["records"].push_back(J{{"id",id_string(r.id)},{"copy_count",data->m_userdata_copycount},{"payload",bounded_native_text(data->payload)},{"stored_bbox",retained_box(ref?ref->m_bbox:def->BoundingBox())}});}
    } else {
      require(mode==L"cache-copycount-zero"||mode==L"tamper-cache"||mode==L"tamper-zero-definition"||mode==L"tamper-collapse-definition","cache fixture mode");std::map<ON__UINT32,std::vector<unsigned char>> replacements;
      for(const auto type:{11u,12u})replacements.emplace(retained_table_code(type),retained_table_fragment(retained_table_code(type),[&](ON_BinaryArchive& output){for(const auto& record:source->records())if(record.native_component_type==type){
        const bool selected=record.id==cache_id(18)||record.id==cache_id(41)||record.id==cache_id(30);
        if(mode==L"cache-copycount-zero"&&selected){require(record.object->AttachUserData(new AuthoredCurrentCacheZero),"authored copycount-zero native cache attachment");require(output.BeginWrite3dmChunk(record.span.type,0),"authored attached record begin");
          if(type==12){require(output.BeginWrite3dmChunk(TCODE_OBJECT_RECORD_TYPE,record.object->ObjectType())&&output.EndWrite3dmChunk()&&output.WriteObject(*record.object),"authored attached Iref original serializer");for(size_t i=2;i<record.children.size();++i)retained_replay_child(output,*source,record.children[i]);}
          else require(output.WriteObject(*record.object),"authored original Definition class serializer");require(output.EndWrite3dmChunk(),"authored attached record end");
        } else if(mode==L"tamper-cache"&&record.id==cache_id(18)){auto* ref=ON_InstanceRef::Cast(record.object.get());require(ref,"authored actual Iref");ref->m_bbox=ON_BoundingBox(ON_3dPoint(0,0,0),ON_3dPoint(1,1,1));RetainedLive row;row.identity=retained_output(record);row.original=&record;row.source=source.get();row.master=ref;row.attributes=record.attributes.get();row.action="retain";row.refresh=true;row.changed=J{{"geometry",false},{"placement",false},{"attributes",false},{"label",false}};retained_write_row(output,row);}
        else if((mode==L"tamper-zero-definition"||mode==L"tamper-collapse-definition")&&record.id==cache_id(30)){auto* def=ON_InstanceDefinition::Cast(record.object.get());require(def,"authored actual Definition");const auto point=mode==L"tamper-zero-definition"?ON_3dPoint(0,0,0):def->BoundingBox().m_min;def->SetBoundingBox(ON_BoundingBox(point,point));RetainedLive row;row.identity=retained_output(record);row.original=&record;row.source=source.get();row.master=def;row.action="retain";row.refresh=true;row.changed=J{{"geometry",false},{"placement",false},{"attributes",false},{"label",false}};retained_write_row(output,row);}
        else retained_replay_child(output,*source,record.span);
      }}));
      retained_write_assembled(*source,replacements,std::filesystem::path(argv[3]));result=J{{"status","authored_current_cache_fixture"},{"mode",native_text(ON_wString(mode.c_str()))}};
    }
    std::cout<<result.dump()<<'\n';return 0;
  }catch(const std::exception& e){std::cerr<<"OM9 authored retained current cache: "<<e.what()<<'\n';return 1;}
}
