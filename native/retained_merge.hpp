// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
// Selected SDK infrastructure, included after the strict source scanner. Rust
// supplies the complete action/identity/unit plan and publication authority.
// These bounded adapters do not certify arbitrary opaque remaps or admission.

struct NativeArchiveSpan {
  ON__UINT32 type=0;
  size_t begin=0,end=0;
  size_t size() const { require(begin<=end,"retained span order");return end-begin; }
};
struct NativeArchiveRecord {
  unsigned native_component_type=0;
  ON__UINT32 table_type=0;
  ON_UUID id=ON_nil_uuid;
  int wire_index=ON_UNSET_INT_INDEX,reference_index=ON_UNSET_INT_INDEX,decoded_index=ON_UNSET_INT_INDEX,sdk_index=ON_UNSET_INT_INDEX;
  NativeArchiveSpan span;
  std::unique_ptr<ON_Object> object;
  std::unique_ptr<ON_3dmObjectAttributes> attributes;
  std::vector<NativeArchiveSpan> children,unrepresented_record_chunks;
};

class NativeSourceArchive {
public:
  static std::unique_ptr<NativeSourceArchive> open(const std::filesystem::path& path) {
    auto source=std::unique_ptr<NativeSourceArchive>(new NativeSourceArchive(path));
    operation_record="retained source strict recursive preflight";
    source->inventory_=inventory_source(path,nullptr,&source->reference_map_,source->held_->value);
    require(source->inventory_["complete_read"]==true,"retained source completeness");
    source->read_owned_bytes();source->read_wire_owners();
    require(source_sha256(source->held_->value,source->bytes_.size())==source->sha256_,"retained source changed after owner acquisition");
    return source;
  }
  NativeSourceArchive(const NativeSourceArchive&)=delete;
  NativeSourceArchive& operator=(const NativeSourceArchive&)=delete;
  const J& inventory() const {return inventory_;}
  const std::string& sha256() const {return sha256_;}
  const std::vector<NativeArchiveSpan>& tables() const {return tables_;}
  const std::vector<NativeArchiveRecord>& records() const {return records_;}
  const ON_3dmProperties& properties() const {return properties_;}
  const ON_3dmSettings& settings() const {return settings_;}
  size_t byte_length() const {return bytes_.size();}
  std::string span_sha256(NativeArchiveSpan span) const {
    check_span(span);SourceHash hash;unsigned char buffer[8192]{};
    for(size_t cursor=span.begin;cursor<span.end;) {
      const auto n=std::min<size_t>(8192,span.end-cursor);
      std::copy_n(bytes_.data()+cursor,n,buffer);
      require(BCryptHashData(hash.hash,buffer,static_cast<ULONG>(n),0)>=0,"retained byte-span SHA256 update");cursor+=n;
    }
    unsigned char digest[32]{};require(BCryptFinishHash(hash.hash,digest,sizeof(digest),0)>=0,"retained byte-span SHA256 finish");
    std::string result;result.reserve(64);constexpr char hex[]="0123456789abcdef";
    for(const auto b:digest){result.push_back(hex[b>>4]);result.push_back(hex[b&15]);}return result;
  }
  uintmax_t retained_usage() const {return owned_usage_;}
  void revalidate_bytes() const {require(source_sha256(held_->value,bytes_.size())==sha256_,"retained source held bytes changed");}
  void replay(ON_BinaryArchive& output,NativeArchiveSpan span) const {
    check_span(span);require(output.WriteByte(span.size(),bytes_.data()+span.begin),"retained exact byte replay");
  }
  void append(std::vector<unsigned char>& output,NativeArchiveSpan span) const {
    check_span(span);require(span.size()<=max_archive_bytes-output.size(),"retained assembled byte budget");
    output.insert(output.end(),bytes_.begin()+span.begin,bytes_.begin()+span.end);
  }
  std::vector<NativeArchiveSpan> class_children(NativeArchiveSpan span) const {
    check_span(span);ON_Read3dmBufferArchive a(bytes_.size(),bytes_.data(),false,50,ON::Version());
    require(a.SeekFromStart(span.begin),"retained class span seek");ON__UINT32 code=0;ON__INT64 length=0;
    require(a.BeginRead3dmBigChunk(&code,&length)&&code==TCODE_OPENNURBS_CLASS,"retained selected class framing");
    std::vector<NativeArchiveSpan> children;
    while(a.CurrentPosition()<span.end) {
      NativeArchiveSpan child;child.begin=position(a);require(a.BeginRead3dmBigChunk(&child.type,&length),"retained class child framing");
      require(a.EndRead3dmChunk(),"retained class child CRC/end");child.end=position(a);children.push_back(child);
      require(children.size()<=max_records,"retained class child budget");if(child.type==TCODE_OPENNURBS_CLASS_END)break;
    }
    require(a.EndRead3dmChunk()&&position(a)==span.end&&a.BadCRCCount()==0,"retained selected class end");return children;
  }
  const NativeArchiveRecord& record(unsigned type,ON_UUID id) const {
    for(const auto& r:records_)if(r.native_component_type==type&&r.id==id)return r;
    throw std::runtime_error("retained native record missing");
  }
private:
  explicit NativeSourceArchive(const std::filesystem::path& path):path_(path),held_(std::make_unique<File>(path,true)) {
    const auto n=_filelengthi64(_fileno(held_->value));require(n>0&&static_cast<uintmax_t>(n)<=max_archive_bytes,"retained source file-byte budget");
    sha256_=source_sha256(held_->value,static_cast<uintmax_t>(n));
  }
  size_t position(const ON_BinaryArchive& a) const {const auto p=a.CurrentPosition();require(p<=bytes_.size(),"retained cursor bounds");return static_cast<size_t>(p);}
  void check_span(NativeArchiveSpan span) const {require(span.begin<=span.end&&span.end<=bytes_.size(),"retained byte-span bounds");}
  void read_owned_bytes() {
    const auto n=_filelengthi64(_fileno(held_->value));require(n>0&&static_cast<uintmax_t>(n)<=max_archive_bytes,"retained source byte budget");
    bytes_.resize(static_cast<size_t>(n));owned_usage_=bytes_.size();require(_fseeki64(held_->value,0,SEEK_SET)==0,"retained source rewind");
    require(std::fread(bytes_.data(),1,bytes_.size(),held_->value)==bytes_.size()&&!std::ferror(held_->value),"retained owned source bytes");
  }
  static unsigned table_type(ON__UINT32 code) {
    switch(code) {
      case TCODE_BITMAP_TABLE:return 1;case TCODE_TEXTURE_MAPPING_TABLE:return 2;case TCODE_MATERIAL_TABLE:return 3;
      case TCODE_LINETYPE_TABLE:return 4;case TCODE_LAYER_TABLE:return 5;case TCODE_GROUP_TABLE:return 6;
      case TCODE_FONT_TABLE:return 7;case TCODE_DIMSTYLE_TABLE:return 8;case TCODE_LIGHT_TABLE:return 9;
      case TCODE_HATCHPATTERN_TABLE:return 10;case TCODE_INSTANCE_DEFINITION_TABLE:return 11;
      case TCODE_OBJECT_TABLE:return 12;case TCODE_HISTORYRECORD_TABLE:return 13;
      default:return 0;
    }
  }
  static ON__UINT32 record_code(unsigned type) {
    switch(type) {
      case 1:return TCODE_BITMAP_RECORD;case 2:return TCODE_TEXTURE_MAPPING_RECORD;case 3:return TCODE_MATERIAL_RECORD;
      case 4:return TCODE_LINETYPE_RECORD;case 5:return TCODE_LAYER_RECORD;case 6:return TCODE_GROUP_RECORD;
      case 7:return TCODE_FONT_RECORD;case 8:return TCODE_DIMSTYLE_RECORD;case 9:return TCODE_LIGHT_RECORD;
      case 10:return TCODE_HATCHPATTERN_RECORD;case 11:return TCODE_INSTANCE_DEFINITION_RECORD;
      case 12:return TCODE_OBJECT_RECORD;case 13:return TCODE_HISTORYRECORD_RECORD;
      default:throw std::runtime_error("retained closed SDK table type");
    }
  }
  const J& inventory_record(unsigned type,ON_UUID id) const {
    for(const auto* collection:{"components","objects"})for(const auto& r:inventory_[collection]) {
      const unsigned actual=r.contains("native_component_type")?r["native_component_type"].get<unsigned>()
        :r.contains("component_type")?static_cast<unsigned>(component_type_enum(r["component_type"].get<std::string>())):12;
      if(actual==type&&r["id"]==id_string(id))return r;
    }
    throw std::runtime_error("retained raw/decoded source identity mismatch");
  }
  static Type component_type_enum(const std::string& name) {
    for(unsigned n=1;n<=13;++n){const auto t=ON_ModelComponent::ComponentTypeFromUnsigned(n);if(component_type(t)==name)return t;}
    throw std::runtime_error("retained closed decoded component type");
  }
  void admit_owner(NativeArchiveRecord& record) {
    require(record.object&&records_.size()<max_records,"retained native owner record budget");
    uintmax_t bytes=record.object->SizeOf();if(record.attributes)bytes+=record.attributes->SizeOf();
    add_usage(bytes+2*sizeof(NativeArchiveRecord));
    for(const auto* o:{record.object.get(),static_cast<ON_Object*>(record.attributes.get())})if(o)for(const auto* u=o->FirstUserData();u;u=u->Next())require(++userdata_count_<=max_records,"retained actual userdata budget");
    const auto& decoded=inventory_record(record.native_component_type,record.id);
    if(!decoded["native_index"].is_null())record.decoded_index=decoded["native_index"].get<int>();
    const auto& mapping=reference_map_.MapItemFromSourceId(record.id);
    require(mapping.SourceAndDestinationAreSet()&&static_cast<unsigned>(mapping.ComponentType())==record.native_component_type&&mapping.SourceId()==record.id&&mapping.DestinationId()==record.id,"retained closed source reference identity");
    record.reference_index=mapping.SourceIndex();
    record.sdk_index=mapping.DestinationIndex();
    if(record.native_component_type!=9)require(record.sdk_index==record.decoded_index,"retained source decoded index correspondence");
    require(record.native_component_type!=12||record.wire_index==ON_UNSET_INT_INDEX,"retained geometry has UUID identity, no wire index");
  }
  void add_usage(uintmax_t bytes) {require(bytes<=max_archive_bytes-owned_usage_,"retained bytes plus native owner/metadata budget");owned_usage_+=bytes;}
  void child_span(NativeArchiveRecord& record,NativeArchiveSpan span,bool unrepresented=false) {
    require(++span_nodes_<=max_nodes,"retained aggregate native span budget");add_usage((unrepresented?4:2)*sizeof(NativeArchiveSpan));record.children.push_back(span);if(unrepresented)record.unrepresented_record_chunks.push_back(span);
  }
  NativeArchiveSpan begin_chunk(ON_BinaryArchive& a,ON__INT64& value) const {
    NativeArchiveSpan span;span.begin=position(a);require(a.BeginRead3dmBigChunk(&span.type,&value),"retained native chunk begin");
    if(!(span.type&TCODE_SHORT))require(value>=0&&static_cast<uintmax_t>(value)<=bytes_.size()-position(a),"retained native chunk length budget");
    return span;
  }
  void read_object_record(ON_BinaryArchive& a,NativeArchiveRecord& record) {
    if(record.native_component_type==12) {ON__INT64 value=0;auto kind=begin_chunk(a,value);require(kind.type==TCODE_OBJECT_RECORD_TYPE&&(kind.type&TCODE_SHORT),"retained geometry kind chunk");require(a.EndRead3dmChunk(),"retained geometry kind end");kind.end=position(a);child_span(record,kind);}
    const auto master_start=position(a);ON_Object* object=nullptr;const auto rc=a.ReadObject(&object);record.object.reset(object);
    require(rc==1&&record.object,"retained original registered native master");child_span(record,{TCODE_OPENNURBS_CLASS,master_start,position(a)});
    record.attributes=std::make_unique<ON_3dmObjectAttributes>();bool saw_attributes=false,saw_userdata=false,saw_end=false;
    for(size_t count=0;count<max_records;++count) {
      ON__INT64 value=0;auto child=begin_chunk(a,value);
      const ON__UINT32 attr=record.native_component_type==12?TCODE_OBJECT_RECORD_ATTRIBUTES:TCODE_LIGHT_RECORD_ATTRIBUTES;
      const ON__UINT32 userdata=record.native_component_type==12?TCODE_OBJECT_RECORD_ATTRIBUTES_USERDATA:TCODE_LIGHT_RECORD_ATTRIBUTES_USERDATA;
      const ON__UINT32 end=record.native_component_type==12?TCODE_OBJECT_RECORD_END:TCODE_LIGHT_RECORD_END;
      if(child.type==attr){require(!saw_attributes&&record.attributes->Read(a),"retained original native attributes");saw_attributes=true;}
      else if(child.type==userdata) {
        // Selected public low-level reader: reproduce the SDK record grammar,
        // retaining attachments before Read3dmObject's destructive migrations
        // (e.g. custom mesh data and $temp_object$ removal). No plug-in loads.
        require(saw_attributes&&!saw_userdata&&a.ReadObjectUserData(*record.attributes),"retained original attribute userdata");saw_userdata=true;
      } else if(child.type==end){require((child.type&TCODE_SHORT)&&value==0,"retained record end value");saw_end=true;}
      require(a.EndRead3dmChunk(),"retained native record child CRC/end");child.end=position(a);child_span(record,child,child.type!=attr&&child.type!=userdata&&child.type!=end);
      if(saw_end)break;
    }
    require(saw_end&&saw_attributes&&!ON_UuidIsNil(record.attributes->m_uuid),"retained actual stable native object identity");record.id=record.attributes->m_uuid;
    if(record.native_component_type==9){const auto* light=ON_Light::Cast(record.object.get());require(light&&light->m_light_id==record.id,"retained light identity correspondence");record.wire_index=light->m_light_index;}
  }
  void read_wire_owners() {
    require(_fseeki64(held_->value,0,SEEK_SET)==0,"retained raw native rewind");const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();
    ON_BinaryFile a(ON::archive_mode::read3dm,held_->value);a.SetReferencedComponentIndexMapping(false);a.SetShouldSerializeUserDataDefault(true);
    int version=0;ON_String comments;require(a.Read3dmStartSection(&version,comments)&&version==50,"retained wire-span codec currently requires version50");tables_.push_back({0,0,position(a)});
    auto start=position(a);require(a.Read3dmProperties(properties_),"retained native properties owner");tables_.push_back({TCODE_PROPERTIES_TABLE,start,position(a)});
    start=position(a);require(a.Read3dmSettings(settings_),"retained native settings owner");tables_.push_back({TCODE_SETTINGS_TABLE,start,position(a)});
    bool eof=false;
    while(position(a)<bytes_.size()) {
      require(tables_.size()<=max_records+32,"retained native table-span budget");ON__INT64 value=0;auto table=begin_chunk(a,value);const auto type=table_type(table.type);
      if(type) {
        bool ended=false;for(size_t count=0;count<=max_records;++count) {
          ON__INT64 record_value=0;auto span=begin_chunk(a,record_value);
          if(span.type==TCODE_ENDOFTABLE){require((span.type&TCODE_SHORT)&&record_value==0&&a.EndRead3dmChunk(),"retained table terminator");ended=true;break;}
          operation_record="retained wire table "+std::to_string(type)+" ordinal "+std::to_string(count);
          require(span.type==record_code(type),"retained native table record code");NativeArchiveRecord record;record.native_component_type=type;record.table_type=table.type;record.span=span;
          if(type==9||type==12)read_object_record(a,record);
          else {const auto master_start=position(a);ON_Object* object=nullptr;const int rc=a.ReadObject(&object);record.object.reset(object);require(rc==1&&record.object,"retained actual registered table owner");child_span(record,{TCODE_OPENNURBS_CLASS,master_start,position(a)});
            // The pinned legacy V5x dimstyle C++ base is ON_ModelComponent,
            // but its SDK class registration names ON_Object as the base.
            // Standard C++ RTTI reads the real public base safely; no internal
            // headers, private access or class-name-to-enum guessing is used.
            const auto* component=dynamic_cast<const ON_ModelComponent*>(record.object.get());
            if(!component||static_cast<unsigned>(component->ComponentType())!=type)throw std::runtime_error(operation_record+" native component enum/class mismatch "+native_kind(*record.object));
            record.id=component->Id();record.wire_index=component->Index();}
          require(a.EndRead3dmChunk(),"retained native table record CRC/end");record.span.end=position(a);admit_owner(record);records_.push_back(std::move(record));
        }require(ended,"retained table record count budget");
      } else require(table.type==TCODE_USER_TABLE||table.type==TCODE_ENDOFFILE,"retained unregistered top-level table");
      require(a.EndRead3dmChunk(),"retained native top-level end");table.end=position(a);tables_.push_back(table);
      if(table.type==TCODE_ENDOFFILE){eof=true;break;}
    }
    require(eof&&position(a)==bytes_.size()&&a.BadCRCCount()==0&&a.CriticalErrorCount()==0,"retained exact wire EOF/CRC");
    require(ON_GetErrorCount()==errors&&ON_GetWarningCount()==warnings,"retained original native read warning/error");
    require(records_.size()==inventory_["components"].size()+inventory_["objects"].size(),"retained raw/decoded record coverage");
  }
  std::filesystem::path path_;
  std::unique_ptr<File> held_;
  std::string sha256_;
  std::vector<unsigned char> bytes_;
  J inventory_;
  ON_ManifestMap reference_map_;
  ON_3dmProperties properties_;
  ON_3dmSettings settings_;
  std::vector<NativeArchiveSpan> tables_;
  std::vector<NativeArchiveRecord> records_;
  uintmax_t owned_usage_=0;
  size_t userdata_count_=0;
  size_t span_nodes_=0;
};

// Preparation exposes original byte identities and conservative codec gates.
// It neither chooses output identities/units nor admits any host mutation.
inline J prepare_retained_source(const std::filesystem::path& path) {
  auto source=NativeSourceArchive::open(path);const auto& original=source->inventory();const auto& archive=original["archive"];
  J result{{"schema_version",4},{"status","retained_source_prepared"},{"decoder_policy","om9-retained-source-v1"},
    {"archive",{{"version",archive["version"]},{"opennurbs_version",archive["opennurbs_version"]},
      {"file_bytes",source->byte_length()},{"endmark_bytes",archive["endmark_bytes"]},{"source_sha256",source->sha256()}}},
    {"original_units",{{"model",native_units(source->settings().m_ModelUnitsAndTolerances)},
      {"page",native_units(source->settings().m_PageUnitsAndTolerances)}}},
    {"records",J::array()},{"top_level_spans",J::array()},{"admission_required",true},{"host_ready",false}};
  auto index=[](int value)->J{return value==ON_UNSET_INT_INDEX?J(nullptr):J(value);};source_record_count=0;
  for(const auto& record:source->records()) {
    operation_record="prepare retained native record "+id_string(record.id);
    require(++source_record_count<=max_records,"retained preparation aggregate record budget");
    const auto userdata=native_userdata(*record.object),attribute_userdata=record.attributes?native_userdata(*record.attributes):J::array();
    bool unknown=false;for(const auto* list:{&userdata,&attribute_userdata})for(const auto& data:*list)unknown=unknown||data["unknown"].get<bool>();
    const auto* component=dynamic_cast<const ON_ModelComponent*>(record.object.get());
    const auto* definition=ON_InstanceDefinition::Cast(record.object.get());
    J units=nullptr;if(definition){const auto native=definition_units(definition->UnitSystem());units=J{{"native_enum",native["native_enum"]},{"name",native["name"]},{"declared_mm_per_unit",native["millimeters_per_unit"]}};}
    bool extended=false;
    if(record.attributes) {
      const ON_3dmObjectAttributes defaults;auto actual=attr_values(*record.attributes,J::array());const auto baseline=attr_values(defaults,J::array());
      if(record.attributes->ColorSource()==ON::color_from_layer||record.attributes->ColorSource()==ON::color_from_object) {
        actual["color_source"]=baseline["color_source"];actual["native_color"]=baseline["native_color"];
      }
      extended=actual!=baseline;
    }
    if(const auto* layer=ON_Layer::Cast(record.object.get()))extended=extended||layer_values(*layer,J::array())!=layer_values(ON_Layer(),J::array());
    if(definition)extended=extended||definition_values(*definition,J::array())!=definition_values(ON_InstanceDefinition(),J::array());
    // The selected simple serializers still require a closed Rust action plan.
    // Other table/geometry bodies additionally require an admitted SDK context;
    // opaque attachments/extensions cannot be treated as reference/length-free.
    const bool simple_geometry=ON_Point::Cast(record.object.get())||ON_LineCurve::Cast(record.object.get())
      ||ON_ArcCurve::Cast(record.object.get())||ON_NurbsCurve::Cast(record.object.get())
      ||ON_NurbsSurface::Cast(record.object.get())||ON_Mesh::Cast(record.object.get())||ON_PointCloud::Cast(record.object.get())||ON_InstanceRef::Cast(record.object.get());
    const bool context=record.native_component_type!=5&&record.native_component_type!=11
      &&!(record.native_component_type==12&&simple_geometry);
    // A polycurve has child native owners/attachments, so its compound class
    // remains a context gate even when this top-level userdata list is empty.
    // Nondefault uncovered fields also require admission; a false flag is
    // never certification that an arbitrary UUID/index/length remap is safe.
    const bool opaque=context||extended||!userdata.empty()||!attribute_userdata.empty()||!record.unrepresented_record_chunks.empty();
    result["records"].push_back(J{{"source",{{"native_component_type",record.native_component_type},{"id",id_string(record.id)},{"native_index",index(record.decoded_index)}}},
      {"wire",{{"id",id_string(record.id)},{"serialized_index",index(record.wire_index)},
        {"archive_reference_index",index(record.reference_index)},{"sdk_destination_index",index(record.sdk_index)},
        {"table_type",record.table_type},{"record_type",record.span.type},{"offset",record.span.begin},{"byte_length",record.span.size()},{"sha256",source->span_sha256(record.span)}}},
      {"native_kind",native_kind(*record.object)},{"native_class_uuid",native_class(*record.object)},
      {"name",record.attributes?bounded_native_text(record.attributes->m_name):component?bounded_native_text(component->Name()):std::string()},
      {"userdata",userdata},{"attribute_userdata",attribute_userdata},{"definition_units",units},
      {"preservation_flags",{{"master_userdata_present",!userdata.empty()},{"attribute_userdata_present",!attribute_userdata.empty()},
        {"unknown_userdata_present",unknown},{"unrepresented_record_chunks_present",!record.unrepresented_record_chunks.empty()},
        {"opaque_identity_remap_requires_admission",opaque},{"opaque_length_remap_requires_admission",opaque},{"selected_serializer_context_required",context}}}});
  }
  for(const auto span:source->tables()) {
    const bool anonymous=span.type==TCODE_USER_TABLE;if(anonymous)require(++source_record_count<=max_records,"retained preparation user table aggregate budget");
    result["top_level_spans"].push_back(J{{"table_type",span.type},{"offset",span.begin},{"byte_length",span.size()},
      {"sha256",source->span_sha256(span)},{"anonymous_user_table",anonymous}});
  }
  size_t nodes=0;source_output_tree(result,nodes);operation_record="prepare retained held-byte final revalidation";source->revalidate_bytes();return result;
}

// Context-independent fragment generators support only the public tables for
// which index mapping can be disabled. Annotation/legacy style serialization
// needs a separately admitted context/codec; it is intentionally unavailable.
inline std::vector<unsigned char> retained_table_fragment(ON__UINT32 code,const std::function<void(ON_BinaryArchive&)>& records) {
  ON_Write3dmBufferArchive a(0,max_archive_bytes,50,ON::Version());ON_3dmProperties p;ON_3dmSettings s;
  require(a.Write3dmStartSection(50,"OM9 native retained fragment")&&a.Write3dmProperties(p)&&a.Write3dmSettings(s),"retained fragment required prefix");
  a.SetReferencedComponentIndexMapping(false);a.SetShouldSerializeUserDataDefault(true);const auto start=static_cast<size_t>(a.CurrentPosition());
  bool(ON_BinaryArchive::*begin)()=nullptr;bool(ON_BinaryArchive::*end)()=nullptr;
  switch(code) {
    case TCODE_OBJECT_TABLE:begin=&ON_BinaryArchive::BeginWrite3dmObjectTable;end=&ON_BinaryArchive::EndWrite3dmObjectTable;break;
    case TCODE_LAYER_TABLE:begin=&ON_BinaryArchive::BeginWrite3dmLayerTable;end=&ON_BinaryArchive::EndWrite3dmLayerTable;break;
    case TCODE_INSTANCE_DEFINITION_TABLE:begin=&ON_BinaryArchive::BeginWrite3dmInstanceDefinitionTable;end=&ON_BinaryArchive::EndWrite3dmInstanceDefinitionTable;break;
    default:throw std::runtime_error("retained fragment serializer requires selected table codec");
  }
  const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();require((a.*begin)(),"retained fragment table begin");records(a);require((a.*end)(),"retained fragment table end");
  require(a.CriticalErrorCount()==0&&ON_GetErrorCount()==errors&&ON_GetWarningCount()==warnings,"retained fragment native write diagnostic");
  const auto end_position=static_cast<size_t>(a.CurrentPosition());require(end_position==a.SizeOfArchive()&&start<=end_position&&end_position<=max_archive_bytes,"retained bounded table fragment");const auto* data=static_cast<const unsigned char*>(a.Buffer());return {data+start,data+end_position};
}

inline void retained_replace_geometry_preserving_attributes(ON_BinaryArchive& output,const NativeSourceArchive& source,const NativeArchiveRecord& original,const ON_Geometry& geometry) {
  require(output.Active3dmTable()==Table::object_table&&original.native_component_type==12&&original.attributes&&original.children.size()>=3,"retained geometry replacement record context");
  require(original.unrepresented_record_chunks.empty()&&original.object->FirstUserData()==nullptr&&geometry.FirstUserData()==nullptr,"retained geometry attachments/extensions require explicit codec policy");
  const auto kind=geometry.ObjectType();require(kind==ON::point_object||kind==ON::curve_object||kind==ON::surface_object||kind==ON::mesh_object||kind==ON::pointset_object||kind==ON::instance_reference,"retained replacement family requires selected serializer context");
  require(original.children[0].type==TCODE_OBJECT_RECORD_TYPE&&original.children[1].type==TCODE_OPENNURBS_CLASS,"retained original geometry framing");
  require(output.BeginWrite3dmChunk(TCODE_OBJECT_RECORD,0)&&output.BeginWrite3dmChunk(TCODE_OBJECT_RECORD_TYPE,geometry.ObjectType())&&output.EndWrite3dmChunk()&&output.WriteObject(geometry),"retained replacement native geometry serializer");
  // SDK chunk framing excludes nested chunks from the parent's CRC. Replaying
  // complete validated child chunks must do the same. Their original CRCs and
  // all attribute bytes (including unknown/copycount-zero userdata headers)
  // remain exact. This avoids the SDK's known->unknown 'saved as goo' rewrite.
  for(size_t i=2;i<original.children.size();++i){const bool previous=output.EnableCRCCalculation(false);try{source.replay(output,original.children[i]);}catch(...){output.EnableCRCCalculation(previous);throw;}output.EnableCRCCalculation(previous);}
  require(output.EndWrite3dmChunk(),"retained replacement record CRC/end");
}

inline void retained_write_assembled(const NativeSourceArchive& source,const std::map<ON__UINT32,std::vector<unsigned char>>& replacements,const std::filesystem::path& destination) {
  source.revalidate_bytes();std::vector<unsigned char> assembled;std::set<ON__UINT32> used;
  for(const auto span:source.tables()) {
    if(span.type==TCODE_ENDOFFILE)break;
    if(const auto i=replacements.find(span.type);i!=replacements.end()) {require(used.insert(span.type).second&&!i->second.empty()&&i->second.size()<=max_archive_bytes-assembled.size(),"retained replacement table coverage/budget");assembled.insert(assembled.end(),i->second.begin(),i->second.end());}
    else source.append(assembled,span);
  }
  require(used.size()==replacements.size(),"retained replacement for absent/duplicate table");
  ON_Write3dmBufferArchive final(0,max_archive_bytes,50,ON::Version());ON_3dmProperties p;ON_3dmSettings s;require(final.Write3dmStartSection(50,"")&&final.Write3dmProperties(p)&&final.Write3dmSettings(s),"retained EOF writer required state");
  const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();require(assembled.size()<=max_archive_bytes-20&&final.SeekFromStart(0)&&final.WriteByte(assembled.size(),assembled.data())&&final.Write3dmEndMark(),"retained assembly SDK EOF");
  const auto bytes=final.CurrentPosition();require(bytes==assembled.size()+20&&bytes<=final.SizeOfArchive()&&bytes<=max_archive_bytes,"retained exact EOF/output budget");
  require(final.CriticalErrorCount()==0&&ON_GetErrorCount()==errors&&ON_GetWarningCount()==warnings,"retained assembly native diagnostic");source.revalidate_bytes();
  PrivateFile file(destination);require(std::fwrite(final.Buffer(),1,static_cast<size_t>(bytes),file.value)==bytes&&!std::ferror(file.value)&&std::fflush(file.value)==0&&_commit(_fileno(file.value))==0,"retained private archive write/sync");const int close_result=std::fclose(file.value);file.value=nullptr;require(close_result==0,"retained private archive close");file.complete=true;
  // The caller still owes independent cold structural/current-state/opaque
  // verification. This function never emits a success receipt or publishes.
}

// Closed schema-4 selected adapter. Business actions, identities, frame/unit
// policy and table order originate in Rust; this layer only checks and applies
// that complete plan using public SDK serializers and validated byte owners.
inline J retained_index(int value) {return value==ON_UNSET_INT_INDEX?J(nullptr):J(value);}
inline int retained_index(const J& value) {return value.is_null()?ON_UNSET_INT_INDEX:static_cast<int>(count(value,INT_MAX));}
inline double retained_factor(const J& value) {const auto n=real(value);require(n>0&&std::isfinite(1/n),"retained positive finite unit factor");return n;}
inline void retained_digest(const J& value) {const auto s=text(value);require(s.size()==64&&std::all_of(s.begin(),s.end(),[](char c){return (c>='0'&&c<='9')||(c>='a'&&c<='f');}),"retained SHA256 format");}
inline J retained_wire(const NativeArchiveRecord& r) {return J{{"serialized_index",retained_index(r.wire_index)},{"archive_reference_index",retained_index(r.reference_index)},{"sdk_destination_index",retained_index(r.sdk_index)}};}
inline J retained_output(const NativeArchiveRecord& r) {auto out=retained_wire(r);out["native_component_type"]=r.native_component_type;out["id"]=id_string(r.id);return out;}
inline void retained_identity(const J& id) {
  fields(id,{"native_component_type","id","serialized_index","archive_reference_index","sdk_destination_index"});
  const auto type=count(id["native_component_type"],13);require(type>0,"retained component enum");(void)uuid(id["id"]);
  for(const auto* n:{"serialized_index","archive_reference_index","sdk_destination_index"})(void)retained_index(id[n]);
  if(type==12)require(id["serialized_index"].is_null()&&id["archive_reference_index"].is_null()&&id["sdk_destination_index"].is_null(),"retained geometry has no serialized/reference/SDK index");
  if(type==5)require(!id["serialized_index"].is_null()&&!id["archive_reference_index"].is_null()&&!id["sdk_destination_index"].is_null(),"retained Layer requires three explicit indices");
  if(type==11)require(id["serialized_index"].is_null()&&!id["archive_reference_index"].is_null()&&!id["sdk_destination_index"].is_null(),"retained V5 definition index contexts");
}
inline size_t retained_curve_shape(const J& j,size_t depth=0) {
  require(depth<=max_depth&&j.is_object()&&j.size()==1,"retained Curve shape/depth");const auto key=j.begin().key();const auto& c=j.begin().value();
  if(key=="Line"){fields(c,{"start","end","domain"});require(point(c["start"])!=point(c["end"]),"retained degenerate Line");(void)domain(c["domain"]);return 2;}
  if(key=="Circle") {
    if(c.contains("angle_domain"))fields(c,{"center","x","y","radius","domain","angle_domain"});else fields(c,{"center","x","y","radius","domain"});
    (void)point(c["center"]);const auto xp=point(c["x"]),yp=point(c["y"]);const ON_3dVector x(xp.x,xp.y,xp.z),y(yp.x,yp.y,yp.z);require(std::fabs(x*x-1)<=1e-12&&std::fabs(y*y-1)<=1e-12&&std::fabs(x*y)<=1e-12,"retained Circle orthonormal frame");require(real(c["radius"])>0,"retained Circle radius");const auto parameter=domain(c["domain"]);const auto angles=c.contains("angle_domain")&&!c["angle_domain"].is_null()?domain(c["angle_domain"]):parameter;
    require(angles.Length()==2*ON_PI,"Unsupported: retained Circle angular span must be exact full turn");return 8;
  }
  if(key=="Nurbs"){nurbs_curve_validate(c);return c["poles"].size();}
  require(key=="Polycurve","Unsupported: retained current Curve variant");fields(c,{"segments","reversed"});require(!flag(c["reversed"]),"Unsupported: reversed Polycurve metadata");require(c["segments"].is_array()&&!c["segments"].empty()&&c["segments"].size()<=max_records,"retained Polycurve segment shape/budget");size_t n=0;for(const auto& segment:c["segments"]){const auto m=retained_curve_shape(segment,depth+1);require(m<=max_array-n,"retained recursive Polycurve pole budget");n+=m;}return n;
}
inline size_t retained_geometry_shape(const J& j) {
  require(j.is_object()&&j.size()==1,"retained Geometry tag");const auto key=j.begin().key();const auto& c=j.begin().value();
  if(key=="Point"){(void)point(c);return 1;}if(key=="Instance"){(void)uuid(c);return 0;}if(key=="Curve")return retained_curve_shape(c);
  if(key=="Surface") {fields(c,{"degree","size","poles","weights","knots","domain","periodic","reversed"});array(c["degree"],2);array(c["size"],2);array(c["knots"],2);array(c["domain"],2);array(c["periodic"],2);for(const auto& b:c["periodic"])(void)flag(b);require(!flag(c["reversed"]),"Unsupported: reversed Surface metadata");
    const auto u=count(c["size"][0]),v=count(c["size"][1]);require(u>0&&v>0&&u<=max_array/v,"retained surface pole budget");poles(c["poles"],c["weights"]);require(c["poles"].size()==u*v,"retained Surface grid shape");axis(count(c["degree"][0],16),u,c["knots"][0],c["domain"][0]);axis(count(c["degree"][1],16),v,c["knots"][1],c["domain"][1]);return u*v;}
  require(key=="Mesh"||key=="Cloud","Unsupported: retained current Geometry variant");const bool mesh=key=="Mesh";
  if(mesh)fields(c,{"vertices","faces","normals","colors"});else fields(c,{"points","normals","colors","hidden"});
  const auto& p=c[mesh?"vertices":"points"];require(p.is_array()&&!p.empty()&&p.size()<=max_array,"retained vertices/points budget");for(const auto& q:p)(void)point(q);
  for(const auto* k:{"normals","colors"}){require(c[k].is_array()&&(c[k].empty()||c[k].size()==p.size()),"retained channel count");for(const auto& q:c[k])if(k==std::string("normals"))(void)point(q);else (void)color(q);}
  if(mesh){require(c["faces"].is_array()&&c["faces"].size()<=max_array,"retained mesh face budget");for(const auto& f:c["faces"]){require(f.is_array()&&(f.size()==3||f.size()==4),"retained polygon shape");for(const auto& index:f)require(count(index)<p.size(),"retained face index range");}}
  else {require(c["hidden"].is_array()&&(c["hidden"].empty()||c["hidden"].size()==p.size()),"retained cloud hidden count");for(const auto& b:c["hidden"])require(!flag(b),"Unsupported: true cloud hidden channel is runtime-only");}
  return p.size();
}
inline void retained_current_shape(const J& current,unsigned type) {
  require(current.is_object()&&current.size()==1,"retained current tag");const char* tag=type==5?"Layer":type==11?"Definition":type==12?"Entity":"";
  require(current.contains(tag),"Unsupported: current family requires selected Layer/Definition/Geometry codec");const auto& c=current[tag];
  if(type==5) {fields(c,{"id","name","parent","visible","selectable","locked","color"});(void)text(c["name"]);if(!c["parent"].is_null())(void)uuid(c["parent"]);(void)color(c["color"]);}
  else if(type==11) {fields(c,{"id","label","mm_per_unit","members"});(void)text(c["label"]);(void)retained_factor(c["mm_per_unit"]);require(c["members"].is_array()&&c["members"].size()<=max_records,"retained definition member budget");std::set<std::string> ids;for(const auto& m:c["members"]){(void)uuid(m);require(ids.insert(text(m)).second,"retained duplicate current Definition member");}}
  else {fields(c,{"id","label","geometry","placement","attributes","definition_member"});(void)text(c["label"]);(void)matrix(c["placement"]);(void)flag(c["definition_member"]);(void)retained_geometry_shape(c["geometry"]);
    const auto& a=c["attributes"];fields(a,{"layer_index","visible","selectable","locked","color"});(void)count(a["layer_index"],INT_MAX);if(!a["color"].is_null())(void)color(a["color"]);for(const auto* n:{"visible","selectable","locked"})(void)flag(a[n]);require(flag(a["selectable"]),"Unsupported: independent Entity selectable=false");require(!(flag(a["locked"])&&!flag(a["visible"]))&&!(flag(c["definition_member"])&&(flag(a["locked"])||!flag(a["visible"]))),"Unsupported: current Entity mode combination");}
  (void)uuid(c["id"]);if(type==5){for(const auto* n:{"visible","selectable","locked"})(void)flag(c[n]);require(flag(c["selectable"]),"Unsupported: independent Layer selectable=false");}
}
struct RetainedLive {
  J identity,current,changed;std::string action;
  NativeSourceArchive* source=nullptr;const NativeArchiveRecord* original=nullptr;
  bool primary=false,refresh=false,dirty=false;
  std::unique_ptr<ON_Object> generated;
  std::unique_ptr<ON_3dmObjectAttributes> generated_attributes;
  ON_Object* master=nullptr;ON_3dmObjectAttributes* attributes=nullptr;
  std::vector<std::string> original_members;
  unsigned type() const {return identity["native_component_type"].get<unsigned>();}
  std::string id() const {return identity["id"].get<std::string>();}
};
struct RetainedRequest {
  J value;std::filesystem::path path;std::unique_ptr<File> held;
  std::string request_sha256;size_t request_bytes=0;
  std::vector<std::unique_ptr<NativeSourceArchive>> sources;
  std::vector<RetainedLive> live;std::map<std::string,size_t> by_id;
  std::map<unsigned,std::vector<size_t>> order;
  NativeSourceArchive* primary=nullptr;size_t total_usage=0;
  explicit RetainedRequest(const std::filesystem::path& p):path(std::filesystem::absolute(p)),held(std::make_unique<File>(path,true)) {
    const auto n=_filelengthi64(_fileno(held->value));require(n>0&&static_cast<uintmax_t>(n)<=max_packet_bytes,"retained request byte budget");request_bytes=static_cast<size_t>(n);
    request_sha256=source_sha256(held->value,request_bytes);std::string bytes(request_bytes,'\0');require(std::fread(bytes.data(),1,bytes.size(),held->value)==bytes.size()&&std::fgetc(held->value)==EOF&&!std::ferror(held->value),"retained held request read");
    Preflight preflight;require(J::sax_parse(bytes,&preflight),"retained request preflight");value=J::parse(bytes);revalidate();validate_shape();acquire();validate_order();
  }
  void revalidate() const {require(source_sha256(held->value,request_bytes)==request_sha256,"retained held request changed");for(const auto& s:sources)s->revalidate_bytes();}
  void validate_shape() {
    fields(value,{"schema_version","writer_policy","metadata_policy","bounds_policy","primary_namespace","model_mm_per_unit","document","sources","additions","default_layer","table_order"});
    require(count(value["schema_version"])==4&&value["writer_policy"]=="om9-retained-current-v1"&&value["bounds_policy"]=="cold_leaf_hull_v1","retained request schema/policies");
    require(value["metadata_policy"]=="primary_archive"||value["metadata_policy"]=="new_mm","retained metadata policy");(void)retained_factor(value["model_mm_per_unit"]);
    if(value["metadata_policy"]=="new_mm")require(value["primary_namespace"].is_null()&&real(value["model_mm_per_unit"])==1,"retained new_mm model context");
    else (void)uuid(value["primary_namespace"]);
    const auto& d=value["document"];fields(d,{"id","name","page_unit","current_layer"});(void)uuid(d["id"]);(void)text(d["name"]);(void)unit(d["page_unit"]);
    const auto& c=d["current_layer"];fields(c,{"policy","id","archive_reference_index"});
    if(c["policy"]=="preserve_primary")require(value["metadata_policy"]=="primary_archive"&&c["id"].is_null()&&c["archive_reference_index"].is_null(),"retained preserve_primary current-layer context");
    else {require(c["policy"]=="explicit_output","retained current-layer policy");(void)uuid(c["id"]);require(!c["archive_reference_index"].is_null(),"retained explicit current-layer reference");(void)retained_index(c["archive_reference_index"]);}
    require(value["sources"].is_array()&&value["sources"].size()<=max_records&&value["additions"].is_array(),"retained source/addition arrays");size_t rows=0,projection_points=0;std::set<std::string> namespaces;
    auto projected=[&](const J& current){if(!current.is_null()&&current.contains("Entity")){const auto n=retained_geometry_shape(current["Entity"]["geometry"]);require(n<=max_array-projection_points,"retained aggregate current geometry point budget");projection_points+=n;}};
    for(const auto& s:value["sources"]) {
      fields(s,{"namespace","source_id","sha256","model_mm_per_unit","page_mm_per_unit","records"});(void)uuid(s["namespace"]);(void)uuid(s["source_id"]);retained_digest(s["sha256"]);(void)retained_factor(s["model_mm_per_unit"]);(void)retained_factor(s["page_mm_per_unit"]);
      require(namespaces.insert(text(s["namespace"])).second&&s["records"].is_array(),"retained duplicate namespace/record shape");
      for(const auto& r:s["records"]) {
        require(++rows<=max_records,"retained aggregate record budget");fields(r,{"source","expected_wire","action","changed","needs_cache_refresh","output","definition_mm_per_unit","current"});
        fields(r["source"],{"native_component_type","id","native_index"});const auto type=count(r["source"]["native_component_type"],13);require(type>0,"retained source component enum");(void)uuid(r["source"]["id"]);(void)retained_index(r["source"]["native_index"]);
        const auto& w=r["expected_wire"];fields(w,{"id","serialized_index","archive_reference_index","sdk_destination_index","table_type","record_type","offset","byte_length","sha256"});(void)uuid(w["id"]);retained_digest(w["sha256"]);
        for(const auto* k:{"serialized_index","archive_reference_index","sdk_destination_index"})(void)retained_index(w[k]);for(const auto* k:{"table_type","record_type","offset","byte_length"})(void)count(w[k],UINT_MAX);
        const auto action=text(r["action"]);require(action=="retain"||action=="delete"||action=="omit"||action=="regenerate"||action=="patch_entity"||action=="patch_layer"||action=="patch_definition","retained source action");
        fields(r["changed"],{"geometry","placement","attributes","label"});for(const auto& b:r["changed"])(void)flag(b);(void)flag(r["needs_cache_refresh"]);
        if(!r["definition_mm_per_unit"].is_null()) {require(type==11,"retained nondefinition unit context");(void)retained_factor(r["definition_mm_per_unit"]);}
        if(action=="delete"||action=="omit")require(r["output"].is_null()&&r["current"].is_null()&&!flag(r["needs_cache_refresh"]),"retained removed record carries output/current/cache");
        else {retained_identity(r["output"]);require(count(r["output"]["native_component_type"])==type,"retained source/output component mismatch");if(!r["current"].is_null()){retained_current_shape(r["current"],static_cast<unsigned>(type));projected(r["current"]);require(r["current"].begin().value()["id"]==r["output"]["id"],"retained current/output UUID mismatch");}}
      }
    }
    auto addition=[&](const J& a){require(++rows<=max_records,"retained aggregate addition budget");fields(a,{"output","needs_cache_refresh","current"});retained_identity(a["output"]);(void)flag(a["needs_cache_refresh"]);retained_current_shape(a["current"],static_cast<unsigned>(count(a["output"]["native_component_type"])));projected(a["current"]);require(a["current"].begin().value()["id"]==a["output"]["id"],"retained addition/current identity");};
    for(const auto& a:value["additions"])addition(a);if(!value["default_layer"].is_null()){addition(value["default_layer"]);require(value["default_layer"]["output"]["native_component_type"]==5,"retained default is Layer");}
    require(value["table_order"].is_object()&&value["table_order"].size()<=13,"retained table order shape");
    for(const auto& [key,ids]:value["table_order"].items()){require(key=="1"||key=="2"||key=="3"||key=="4"||key=="5"||key=="6"||key=="7"||key=="8"||key=="9"||key=="10"||key=="11"||key=="12"||key=="13","retained table order enum");require(ids.is_array()&&!ids.empty()&&ids.size()<=max_records,"retained table order UUID budget");for(const auto& id:ids)(void)uuid(id);}
  }
  void add(RetainedLive row) {require(live.size()<max_records&&by_id.emplace(row.id(),live.size()).second,"retained duplicate output UUID");live.push_back(std::move(row));}
  void acquire() {
    total_usage=request_bytes*2;
    for(const auto& s:value["sources"]) {
      operation_record="retained namespace "+text(s["namespace"]);auto source=NativeSourceArchive::open(path.parent_path()/(text(s["namespace"])+".3dm"));
      require(source->sha256()==text(s["sha256"]),"retained source SHA256 mismatch");require(source->retained_usage()<=max_archive_bytes-total_usage,"retained aggregate source owners budget");total_usage+=static_cast<size_t>(source->retained_usage());
      const bool is_primary=value["primary_namespace"]==s["namespace"];if(is_primary){require(!primary,"retained duplicate primary");primary=source.get();require(real(s["model_mm_per_unit"])==real(value["model_mm_per_unit"]),"retained primary/output unit context mismatch");}
      for(const auto* name:{"model","page"}) {const auto declaration=native_units(name==std::string("model")?source->settings().m_ModelUnitsAndTolerances:source->settings().m_PageUnitsAndTolerances);const auto& factor=s[name==std::string("model")?"model_mm_per_unit":"page_mm_per_unit"];if(!declaration["millimeters_per_unit"].is_null())require(real(declaration["millimeters_per_unit"])==real(factor),"retained declared source unit factor conflict");}
      require(s["records"].size()==source->records().size(),"retained source full record coverage");std::set<std::pair<unsigned,std::string>> keys;
      for(const auto& r:s["records"]) {
        const auto type=static_cast<unsigned>(count(r["source"]["native_component_type"]));const auto id=uuid(r["source"]["id"]);const auto& original=source->record(type,id);require(keys.emplace(type,id_string(id)).second,"retained duplicate source key");require(r["source"]["native_index"]==retained_index(original.decoded_index),"retained decoded source key mismatch");
        auto expected=retained_wire(original);expected["id"]=id_string(original.id);expected["table_type"]=original.table_type;expected["record_type"]=original.span.type;expected["offset"]=original.span.begin;expected["byte_length"]=original.span.size();expected["sha256"]=source->span_sha256(original.span);
        require(expected==r["expected_wire"],"retained expected original wire span/identity mismatch");const auto action=text(r["action"]);
        if(action=="delete"||action=="omit") {if(is_primary)require(type==12||type==11,"Unsupported: primary indexed-table deletion requires closed reference/settings remapping");continue;}
        RetainedLive row;row.identity=r["output"];row.current=r["current"];row.changed=r["changed"];row.action=action;row.refresh=flag(r["needs_cache_refresh"]);row.source=source.get();row.original=&original;row.primary=is_primary;
        if(is_primary)require(row.identity==retained_output(original),"Unsupported: primary native identity/index remapping requires an admitted opaque/reference codec");
        if(type!=5&&type!=11&&type!=12)require(is_primary&&action=="retain"&&!row.refresh&&std::none_of(row.changed.begin(),row.changed.end(),[](const J& b){return b.get<bool>();}),"Unsupported: nonselected source table remap/serializer");
        if(!is_primary)require(action=="regenerate"&&!row.current.is_null()&&original.object->FirstUserData()==nullptr&&(!original.attributes||original.attributes->FirstUserData()==nullptr)&&original.unrepresented_record_chunks.empty(),"Unsupported: secondary retained/opaque records require closed remap codecs");
        if(action=="retain")require(std::none_of(row.changed.begin(),row.changed.end(),[](const J& b){return b.get<bool>();}),"retained action has changed fields");
        else require(!row.current.is_null(),"retained patch/regenerate requires current payload");
        if(action=="patch_entity")require(type==12,"retained Entity patch component");if(action=="patch_layer")require(type==5,"retained Layer patch component");if(action=="patch_definition")require(type==11,"retained Definition patch component");
        if(type==11) {const auto* d=ON_InstanceDefinition::Cast(original.object.get());require(d,"retained Definition owner");const auto declared=definition_units(d->UnitSystem())["millimeters_per_unit"];if(action!="regenerate"&&!r["definition_mm_per_unit"].is_null()&&!declared.is_null())require(real(r["definition_mm_per_unit"])==real(declared),"retained Definition factor conflict");if(is_primary&&action=="regenerate")require(!declared.is_null()&&real(declared)==1,"Unsupported: primary Definition unit reframing requires closed incoming Instance codecs");if(!row.current.is_null())require(row.current["Definition"]["mm_per_unit"]==r["definition_mm_per_unit"],"retained current/source Definition unit context");const auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)row.original_members.push_back(id_string(ids[i]));}
        add(std::move(row));
      }
      sources.push_back(std::move(source));
    }
    require((value["metadata_policy"]=="primary_archive")==static_cast<bool>(primary),"retained primary namespace missing");
    auto addition=[&](const J& a){RetainedLive row;row.identity=a["output"];row.current=a["current"];row.action="regenerate";row.refresh=flag(a["needs_cache_refresh"]);row.changed=J{{"geometry",true},{"placement",true},{"attributes",true},{"label",true}};add(std::move(row));};
    for(const auto& a:value["additions"])addition(a);if(!value["default_layer"].is_null())addition(value["default_layer"]);
  }
  void validate_order() {
    std::set<size_t> covered;
    for(const auto& [key,ids]:value["table_order"].items()){const auto type=static_cast<unsigned>(std::stoul(key));size_t ordinal=0;for(const auto& id:ids){require(by_id.contains(text(id)),"retained order missing output UUID");const auto i=by_id.at(text(id));const auto& row=live[i];require(row.type()==type&&covered.insert(i).second,"retained duplicate/wrong table order entry");if(type==5||type==11)require(row.identity["archive_reference_index"]==ordinal&&row.identity["sdk_destination_index"]==ordinal,"retained selected table reference/SDK ordinal mismatch");order[type].push_back(i);++ordinal;}}
    require(covered.size()==live.size()&&order.contains(5),"retained live record/table order coverage or no Layer");
    if(primary)for(unsigned type=1;type<=13;++type)if(type!=12&&type!=11){size_t ordinal=0;for(const auto& r:primary->records())if(r.native_component_type==type){require(order.contains(type)&&ordinal<order.at(type).size(),"Unsupported: primary indexed table removal");const auto& row=live[order.at(type)[ordinal++]];require(row.original==&r,"Unsupported: primary indexed table reorder requires reference/settings codecs");}}
    std::set<int> layers;for(const auto i:order.at(5))require(layers.insert(retained_index(live[i].identity["serialized_index"])).second,"retained duplicate Layer serialized index");
    const auto& c=value["document"]["current_layer"];
    if(c["policy"]=="explicit_output") {require(by_id.contains(text(c["id"])),"retained current Layer absent");const auto& layer=live[by_id.at(text(c["id"]))];require(layer.type()==5&&layer.identity["archive_reference_index"]==c["archive_reference_index"],"retained current Layer identity/reference mismatch");if(primary)require(primary->settings().CurrentLayerId()==uuid(c["id"])&&(primary->settings().CurrentLayerIndex()==ON_UNSET_INT_INDEX||primary->settings().CurrentLayerIndex()==retained_index(c["archive_reference_index"])),"Unsupported: changed primary current Layer requires selected settings codec");}
  }
};

inline void retained_entity_attributes(ON_3dmObjectAttributes& a,const J& c) {
  const auto& v=c["attributes"];require(flag(v["selectable"]),"Unsupported: independent Entity selectable=false");
  const bool visible=flag(v["visible"]),locked=flag(v["locked"]),member=flag(c["definition_member"]);
  require(!(locked&&!visible)&&!(member&&(locked||!visible)),"Unsupported: native object mode cannot retain hidden/locked/definition-member combination");
  a.m_layer_index=static_cast<int>(count(v["layer_index"],INT_MAX));a.SetMode(member?ON::idef_object:locked?ON::locked_object:visible?ON::normal_object:ON::hidden_object);a.SetVisible(visible);
  if(v["color"].is_null()){a.SetColorSource(ON::color_from_layer);a.m_color=ON_Color::UnsetColor;}
  else {a.SetColorSource(ON::color_from_object);a.m_color=color(v["color"]);}
}
inline void retained_layer_fields(ON_Layer& layer,const J& c,int index) {
  require(flag(c["selectable"]),"Unsupported: independent Layer selectable=false");require(layer.SetId(uuid(c["id"]))&&layer.SetIndex(index)&&layer.SetName(ON_wString(text(c["name"]).c_str()))&&native_text(layer.Name())==text(c["name"]),"retained native Layer ID/index/name normalization");
  layer.SetParentLayerId(c["parent"].is_null()?ON_nil_uuid:uuid(c["parent"]));layer.SetVisible(flag(c["visible"]));layer.SetLocked(flag(c["locked"]));layer.SetColor(color(c["color"]));
}
inline void retained_definition_fields(ON_InstanceDefinition& def,const J& c,bool fresh) {
  require(!def.IsLinkedType(),"Unsupported: linked definition current patch requires selected linked-definition codec");
  require(def.SetId(uuid(c["id"]))&&def.SetName(ON_wString(text(c["label"]).c_str()))&&native_text(def.Name())==text(c["label"]),"retained native Definition identity/name normalization");
  ON_SimpleArray<ON_UUID> members;std::set<std::string> ids;for(const auto& id:c["members"]){require(ids.insert(text(id)).second,"retained duplicate Definition member");members.Append(uuid(id));}def.SetInstanceGeometryIdList(members);
  if(fresh){require(real(c["mm_per_unit"])==1,"Unsupported: new/regenerated definitions require explicit 1-mm declaration");def.SetUnitSystem(ON::LengthUnitSystem::Millimeters);require(def.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static),"retained static definition type");}
}
inline bool retained_same_geometry(const ON_Geometry& a,const ON_Geometry& b) {
  return native_class(a)==native_class(b)&&record_fingerprint(a,50,ON::Version())==record_fingerprint(b,50,ON::Version());
}
inline bool retained_curve_userdata(const ON_Curve& curve,size_t depth=0) {
  require(depth<=max_depth,"retained child userdata depth budget");if(curve.FirstUserData())return true;
  if(const auto* poly=ON_PolyCurve::Cast(&curve)){require(poly->Count()>=0&&static_cast<size_t>(poly->Count())<=max_records,"retained child userdata record budget");for(int i=0;i<poly->Count();++i){require(poly->SegmentCurve(i),"retained child userdata missing curve");if(retained_curve_userdata(*poly->SegmentCurve(i),depth+1))return true;}}return false;
}
inline void retained_build_current(RetainedRequest& request) {
  const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();
  for(auto& row:request.live) {
    operation_record="retained current native record "+row.id();const auto type=row.type();
    if(row.original){row.master=row.original->object.get();row.attributes=row.original->attributes.get();}
    if(row.action=="retain")continue;
    const bool fresh=row.action=="regenerate";const auto& c=row.current.begin().value();
    if(fresh&&row.original)require(row.original->object->FirstUserData()==nullptr&&(!row.original->attributes||row.original->attributes->FirstUserData()==nullptr)&&row.original->unrepresented_record_chunks.empty(),"Unsupported: regeneration with opaque master/attribute/record userdata");
    if(type==5) {
      if(fresh){row.generated=std::make_unique<ON_Layer>();row.master=row.generated.get();}
      require(row.master,"retained missing Layer master");auto* layer=ON_Layer::Cast(row.master);require(layer,"retained Layer master type");
      require(!row.original||row.original->object->FirstUserData()==nullptr,"Unsupported: Layer field patch with opaque userdata");
      retained_layer_fields(*layer,c,retained_index(row.identity["serialized_index"]));
    } else if(type==11) {
      if(fresh){row.generated=std::make_unique<ON_InstanceDefinition>();row.master=row.generated.get();}
      auto* def=ON_InstanceDefinition::Cast(row.master);require(def,"retained Definition master type");
      if(!fresh&&def->FirstUserData()){require(c["members"]==J(row.original_members)&&text(c["label"])==native_text(def->Name()),"Unsupported: Definition semantic patch with opaque userdata");}
      retained_definition_fields(*def,c,fresh);
    } else if(type==12) {
      const bool replace=fresh||flag(row.changed["geometry"])||flag(row.changed["placement"]);
      if(replace) {
        require(!row.original||(row.original->object->FirstUserData()==nullptr&&row.original->unrepresented_record_chunks.empty()),"Unsupported: geometry/placement patch requires admitted opaque master codec");
        if(row.original)if(const auto* old_curve=ON_Curve::Cast(row.original->object.get()))require(!retained_curve_userdata(*old_curve),"Unsupported: replacement of compound curve with opaque child userdata");
        if(c["geometry"].contains("Instance")) {auto instance=std::make_unique<ON_InstanceRef>();instance->m_instance_definition_uuid=uuid(c["geometry"]["Instance"]);instance->m_xform=matrix(c["placement"]);row.generated=std::move(instance);}
        else row.generated=geometry(c["geometry"],matrix(c["placement"]),{});
        row.master=row.generated.get();
      }
      if(fresh) {row.generated_attributes=std::make_unique<ON_3dmObjectAttributes>();row.attributes=row.generated_attributes.get();row.attributes->m_uuid=uuid(c["id"]);row.attributes->m_name=ON_wString(text(c["label"]).c_str());retained_entity_attributes(*row.attributes,c);}
      else {
        require(row.attributes&&row.attributes->m_uuid==uuid(c["id"]),"retained original Entity attrs/identity");
        if(flag(row.changed["attributes"])||flag(row.changed["label"]))require(row.attributes->FirstUserData()==nullptr,"Unsupported: attribute/label patch with opaque attribute userdata");
        if(flag(row.changed["attributes"]))retained_entity_attributes(*row.attributes,c);
        if(flag(row.changed["label"]))row.attributes->m_name=ON_wString(text(c["label"]).c_str());
      }
      require(row.attributes&&row.attributes->IsValid(),"retained actual Entity attributes invalid");
    } else throw std::runtime_error("Unsupported: current serializer family");
    if(row.generated) {require(row.generated->SizeOf()<=max_archive_bytes-request.total_usage,"retained generated native owner budget");request.total_usage+=row.generated->SizeOf();}
  }
  // Validate actual current member/instance/layer closure without flattened
  // substitutes. Every native member belongs to exactly one definition.
  std::map<std::string,std::string> member_owner;std::set<int> layer_refs;std::map<std::string,std::string> parents;
  for(const auto i:request.order.at(5)){const auto& row=request.live[i];const auto* l=ON_Layer::Cast(row.master);require(l&&l->IsValid(),"retained live Layer invalid");layer_refs.insert(retained_index(row.identity["archive_reference_index"]));parents.emplace(row.id(),ON_UuidIsNil(l->ParentLayerId())?"":id_string(l->ParentLayerId()));}
  for(const auto& [id,parent]:parents){(void)parent;std::string cursor=id;std::set<std::string> visiting;while(!cursor.empty()){require(parents.contains(cursor)&&visiting.insert(cursor).second&&visiting.size()<=max_depth,"retained Layer missing parent/cycle/depth");cursor=parents.at(cursor);}}
  for(auto& row:request.live)if(row.type()==11){const auto* d=ON_InstanceDefinition::Cast(row.master);require(d,"retained Definition missing actual owner");const auto& members=d->InstanceGeometryIdList();require(static_cast<size_t>(members.Count())<=max_records,"retained Definition member budget");for(int i=0;i<members.Count();++i){const auto id=id_string(members[i]);require(request.by_id.contains(id)&&request.live[request.by_id.at(id)].type()==12&&member_owner.emplace(id,row.id()).second,"retained missing/shared Definition member");}}
  std::map<std::string,std::vector<std::string>> dependencies;
  for(auto& row:request.live)if(row.type()==12){require(row.attributes&&layer_refs.contains(row.attributes->m_layer_index),"retained Entity missing Layer reference");require(row.attributes->IsInstanceDefinitionObject()==member_owner.contains(row.id()),"retained Definition-member mode/closure mismatch");
    if(const auto* ref=ON_InstanceRef::Cast(row.master)){const auto child=id_string(ref->m_instance_definition_uuid);require(request.by_id.contains(child)&&request.live[request.by_id.at(child)].type()==11,"retained Instance missing Definition");if(member_owner.contains(row.id()))dependencies[member_owner.at(row.id())].push_back(child);}
  }
  std::map<std::string,size_t> heights;std::set<std::string> active;
  std::function<size_t(const std::string&,size_t)> visit=[&](const std::string& id,size_t depth)->size_t {require(depth<=max_depth,"retained Definition depth budget");if(heights.contains(id)){require(heights.at(id)<=max_depth-depth,"retained cached Definition depth budget");return heights.at(id);}require(active.insert(id).second,"retained Definition cycle");size_t h=0;for(const auto& c:dependencies[id])h=std::max(h,visit(c,depth+1)+1);active.erase(id);heights[id]=h;require(h<=max_depth-depth,"retained Definition longest path budget");return h;};
  for(const auto& row:request.live)if(row.type()==11)(void)visit(row.id(),0);
  for(const auto& row:request.live)if(row.refresh)require(row.type()==11||ON_InstanceRef::Cast(row.master),"retained cache refresh requires Definition/Instance family");
  require(ON_GetErrorCount()==errors&&ON_GetWarningCount()==warnings,"retained current construction native warning/error");
}

// Bounded conservative leaf hull, independent of all serialized Iref/definition
// cache fields. Transforming actual poles/vertices (not cached child boxes)
// maintains the declared policy through nested mirror/shear/nonuniform frames.
struct RetainedHull {
  size_t visits=0,points=0;
  ON_BoundingBox box;
  void add(const ON_3dPoint& p,const ON_Xform& x) {require(++points<=max_array,"retained aggregate cold hull point budget");const auto q=x*p;require(q.IsValid(),"retained cold hull transformed point overflow");if(!box.IsValid()){box.m_min=q;box.m_max=q;}else for(int i=0;i<3;++i){box.m_min[i]=std::min(box.m_min[i],q[i]);box.m_max[i]=std::max(box.m_max[i],q[i]);}}
  void curve_hull(const ON_Curve& c,const ON_Xform& x,size_t depth) {
    require(depth<=max_depth&&++visits<=max_nodes,"retained cold curve recursion budget");
    if(const auto* line=ON_LineCurve::Cast(&c)){add(line->PointAtStart(),x);add(line->PointAtEnd(),x);return;}
    if(const auto* arc=ON_ArcCurve::Cast(&c)) {
      const auto& a=arc->m_arc;const auto interval=a.DomainRadians();const auto center=x*a.plane.origin;const auto u=x*(a.radius*a.plane.xaxis),v=x*(a.radius*a.plane.yaxis);require(center.IsValid()&&u.IsValid()&&v.IsValid(),"retained cold arc transform overflow");
      add(a.PointAt(interval[0]),x);add(a.PointAt(interval[1]),x);
      for(int k=0;k<3;++k){const double extremum=std::atan2(v[k],u[k]);for(int sign=0;sign<2;++sign){double angle=extremum+sign*ON_PI;angle+=std::ceil((interval[0]-angle)/(2*ON_PI))*2*ON_PI;if(angle>=interval[0]&&angle<=interval[1])add(a.PointAt(angle),x);}}
      return;
    }
    if(const auto* n=ON_NurbsCurve::Cast(&c)){require(n->Dimension()==3&&n->CVCount()>0&&static_cast<size_t>(n->CVCount())<=max_array-points,"retained cold NURBS curve pole budget");for(int i=0;i<n->CVCount();++i){const auto weight=n->Weight(i);require(std::isfinite(weight)&&weight>0,"Unsupported: cold hull requires positive rational weights");ON_3dPoint p;require(n->GetCV(i,p),"retained cold curve rational pole");add(p,x);}return;}
    if(const auto* poly=ON_PolyCurve::Cast(&c)){require(poly->Count()>0&&static_cast<size_t>(poly->Count())<=max_records,"retained cold Polycurve segment budget");for(int i=0;i<poly->Count();++i){require(poly->SegmentCurve(i),"retained cold null segment");curve_hull(*poly->SegmentCurve(i),x,depth+1);}return;}
    throw std::runtime_error("Unsupported: cold leaf curve hull family");
  }
  void surface_hull(const ON_Surface& surface,const ON_Xform& x) {
    if(const auto* n=ON_NurbsSurface::Cast(&surface)){const auto u=n->CVCount(0),v=n->CVCount(1);require(u>0&&v>0&&n->Dimension()==3&&static_cast<size_t>(u)<=((max_array-points)/static_cast<size_t>(v)),"retained cold surface pole budget");for(int i=0;i<u;++i)for(int j=0;j<v;++j){const auto w=n->Weight(i,j);require(std::isfinite(w)&&w>0,"Unsupported: cold surface hull requires positive weights");ON_3dPoint p;require(n->GetCV(i,j,p),"retained cold surface rational pole");add(p,x);}return;}
    if(const auto* p=ON_PlaneSurface::Cast(&surface)){const auto u=p->Domain(0),v=p->Domain(1);require(u.IsIncreasing()&&v.IsIncreasing(),"Unsupported: unbounded plane surface hull");for(int i=0;i<2;++i)for(int j=0;j<2;++j)add(p->PointAt(u[i],v[j]),x);return;}
    throw std::runtime_error("Unsupported: cold native surface hull family");
  }
  void leaf(const ON_Geometry& g,const ON_Xform& x) {
    if(const auto* p=ON_Point::Cast(&g)){add(p->point,x);return;}
    if(const auto* c=ON_Curve::Cast(&g)){curve_hull(*c,x,0);return;}
    if(const auto* s=ON_Surface::Cast(&g)){surface_hull(*s,x);return;}
    if(const auto* m=ON_Mesh::Cast(&g)){require(m->VertexCount()>0&&static_cast<size_t>(m->VertexCount())<=max_array-points,"retained cold mesh vertex budget");for(int i=0;i<m->VertexCount();++i)add(m->Vertex(i),x);return;}
    if(const auto* c=ON_PointCloud::Cast(&g)){require(c->m_P.Count()>0&&static_cast<size_t>(c->m_P.Count())<=max_array-points,"retained cold cloud point budget");for(int i=0;i<c->m_P.Count();++i)add(c->m_P[i],x);return;}
    if(const auto* b=ON_Brep::Cast(&g)){require(static_cast<size_t>(b->m_V.Count())<=max_array-points&&static_cast<size_t>(b->m_S.Count())<=max_records,"retained cold Brep topology budget");for(int i=0;i<b->m_V.Count();++i)add(b->m_V[i].point,x);for(int i=0;i<b->m_S.Count();++i){require(b->m_S[i],"retained cold Brep null surface");surface_hull(*b->m_S[i],x);}return;}
    throw std::runtime_error("Unsupported: cold native leaf hull family");
  }
};
inline J retained_box(const ON_BoundingBox& b) {require(b.IsValid(),"retained invalid bounds");return J::array({point_json(b.m_min),point_json(b.m_max)});}
inline bool retained_box_equal(const ON_BoundingBox& a,const ON_BoundingBox& b) {
  if(!a.IsValid()||!b.IsValid())return false;
  for(int i=0;i<3;++i){
    const double extent=std::max(a.m_max[i]-a.m_min[i],b.m_max[i]-b.m_min[i]);
    if(!std::isfinite(extent)||extent<0)return false;
    for(const bool max:{false,true}){
      const double x=max?a.m_max[i]:a.m_min[i],y=max?b.m_max[i]:b.m_min[i];
      if(!std::isfinite(x)||!std::isfinite(y))return false;
      if(x==y||std::fabs(x-y)<=extent*1e-9)continue;
      // Coordinate magnitude and a fixed world-unit floor would erase tiny
      // masters or thin translated hulls. Allow only bounded roundoff here.
      double low=x,high=x;
      for(int ulp=0;ulp<32;++ulp){low=std::nextafter(low,-INFINITY);high=std::nextafter(high,INFINITY);}
      if(y<low||y>high)return false;
    }
  }
  return true;
}
inline ON_BoundingBox retained_compute_hull(const std::map<std::string,ON_Object*>& masters,const std::string& id,size_t& work) {
  RetainedHull hull;std::set<std::string> active;
  std::function<void(const std::string&,const ON_Xform&,size_t)> visit=[&](const std::string& key,const ON_Xform& x,size_t depth){require(depth<=max_depth&&++hull.visits<=max_nodes&&masters.contains(key),"retained cold graph traversal budget/dependency");require(active.insert(key).second,"retained cold graph cycle");const auto* master=masters.at(key);
    if(const auto* d=ON_InstanceDefinition::Cast(master)){require(!d->IsLinkedType(),"Unsupported: unresolved linked definition cold hull");const auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)visit(id_string(ids[i]),x,depth+1);}
    else if(const auto* ref=ON_InstanceRef::Cast(master)){require(ref->m_xform.IsValid(),"retained cold Instance transform");visit(id_string(ref->m_instance_definition_uuid),x*ref->m_xform,depth+1);}
    else {const auto* g=ON_Geometry::Cast(master);require(g,"retained cold graph nongeometry leaf");hull.leaf(*g,x);}active.erase(key);};
  visit(id,ON_Xform::IdentityTransformation,0);require(hull.points<=max_nodes-work,"retained aggregate nested hull work budget");work+=hull.points;require(hull.visits<=max_nodes-work,"retained aggregate cold graph work budget");work+=hull.visits;require(hull.box.IsValid(),"Unsupported: empty definition has no cold leaf hull");return hull.box;
}
inline std::map<std::string,ON_Object*> retained_masters(const RetainedRequest& request) {std::map<std::string,ON_Object*> masters;for(const auto& row:request.live)if(row.type()==11||row.type()==12)masters.emplace(row.id(),row.master);return masters;}
inline void retained_refresh_caches(RetainedRequest& request) {
  const auto masters=retained_masters(request);std::set<std::string> dirty;size_t work=0;
  for(auto& row:request.live)if(row.type()==12&&(row.action=="regenerate"||flag(row.changed["geometry"])||flag(row.changed["placement"])))dirty.insert(row.id());
  // Original membership, including removed members, participates in dirtiness.
  for(auto& row:request.live)if(row.type()==11) {
    const auto* d=ON_InstanceDefinition::Cast(row.master);bool changed=row.action=="regenerate"||flag(row.changed["geometry"]);
    J members_now=J::array();const auto& current_members=d->InstanceGeometryIdList();for(int i=0;i<current_members.Count();++i)members_now.push_back(id_string(current_members[i]));if(row.original)changed=changed||members_now!=J(row.original_members);
    const auto& members=d->InstanceGeometryIdList();for(int i=0;i<members.Count();++i)changed=changed||dirty.contains(id_string(members[i]));if(changed)dirty.insert(row.id());
  }
  for(size_t pass=0;pass<=request.live.size();++pass){bool change=false;for(const auto& row:request.live){bool affected=false;if(const auto* ref=ON_InstanceRef::Cast(row.master))affected=dirty.contains(id_string(ref->m_instance_definition_uuid));else if(const auto* d=ON_InstanceDefinition::Cast(row.master)){const auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)affected=affected||dirty.contains(id_string(ids[i]));}if(affected&&dirty.insert(row.id()).second)change=true;}if(!change)break;require(pass<request.live.size(),"retained dirty cache closure convergence");}
  for(auto& row:request.live)if(row.type()==11||ON_InstanceRef::Cast(row.master)) {
    row.dirty=dirty.contains(row.id());require(!row.dirty||row.refresh,"retained action plan omitted required ancestor/Instance cache refresh");
    if(row.refresh){operation_record="retained cache "+row.id();const auto box=retained_compute_hull(masters,row.id(),work);if(auto* d=ON_InstanceDefinition::Cast(row.master))d->SetBoundingBox(box);else ON_InstanceRef::Cast(row.master)->m_bbox=box;}
  }
  for(const auto& row:request.live)if(row.type()==5||row.type()==11||row.type()==12)require(row.master&&row.master->IsValid(),"retained current native master invalid after cache update");
}

inline void retained_replay_child(ON_BinaryArchive& output,const NativeSourceArchive& source,NativeArchiveSpan span) {
  const bool previous=output.EnableCRCCalculation(false);try{source.replay(output,span);}catch(...){output.EnableCRCCalculation(previous);throw;}output.EnableCRCCalculation(previous);
}
inline void retained_class_data(ON_BinaryArchive& output,const NativeSourceArchive& source,NativeArchiveSpan original,const ON_Object& current) {
  const auto children=source.class_children(original);require(children.size()>=3&&children.front().type==TCODE_OPENNURBS_CLASS_UUID&&children.back().type==TCODE_OPENNURBS_CLASS_END,"retained public class DATA codec framing");
  require(output.BeginWrite3dmChunk(TCODE_OPENNURBS_CLASS,0),"retained class DATA codec begin");bool data=false;
  for(const auto child:children)if(child.type==TCODE_OPENNURBS_CLASS_DATA){require(!data&&output.BeginWrite3dmChunk(TCODE_OPENNURBS_CLASS_DATA,0)&&current.Write(output)&&output.EndWrite3dmChunk(),"retained selected public class DATA serializer");data=true;}else retained_replay_child(output,source,child);
  require(data&&output.EndWrite3dmChunk(),"retained class DATA codec end");
}
inline ON__UINT32 retained_table_code(unsigned type) {
  if(type==5)return TCODE_LAYER_TABLE;if(type==11)return TCODE_INSTANCE_DEFINITION_TABLE;if(type==12)return TCODE_OBJECT_TABLE;throw std::runtime_error("Unsupported: retained selected table codec");
}
inline void retained_write_row(ON_BinaryArchive& archive,const RetainedLive& row) {
  operation_record="serialize retained native record "+row.id();
  if(row.original&&row.action=="retain"&&!row.refresh){retained_replay_child(archive,*row.source,row.original->span);return;}
  if(row.original&&row.action!="regenerate") {
    const auto& original=*row.original;const auto& source=*row.source;
    require(row.type()==5||row.type()==11||row.type()==12,"retained selected original record context");
    require(archive.BeginWrite3dmChunk(original.span.type,0),"retained selected record begin");
    if(row.type()==12) {
      const auto* geometry=ON_Geometry::Cast(row.master);require(geometry&&row.attributes&&original.children.size()>=3,"retained Entity actual geometry/attrs context");
      require(archive.BeginWrite3dmChunk(TCODE_OBJECT_RECORD_TYPE,geometry->ObjectType())&&archive.EndWrite3dmChunk(),"retained Entity kind serializer");
      // Replaced typed masters use the complete public object serializer. In
      // V5 Mesh::Write generates double-vertex userdata; DATA alone would omit
      // that channel and silently reduce coordinates to float precision. The
      // original opaque master/compound userdata gate was checked before build.
      // Cache-only Irefs retain the original outside-DATA chunks exactly.
      if(row.generated)require(archive.WriteObject(*row.master),"retained replaced full native master serializer");
      else {require(native_class(*row.master)==native_class(*original.object),"retained cache-only class identity");retained_class_data(archive,source,original.children[1],*row.master);}
      bool attributes=false;
      for(size_t i=2;i<original.children.size();++i) {
        const auto child=original.children[i];
        if(child.type==TCODE_OBJECT_RECORD_ATTRIBUTES&&(flag(row.changed["attributes"])||flag(row.changed["label"]))) {require(!attributes&&archive.BeginWrite3dmChunk(child.type,0)&&row.attributes->Write(archive)&&archive.EndWrite3dmChunk(),"retained selected attributes DATA codec");attributes=true;}
        else retained_replay_child(archive,source,child);
      }
    } else {require(original.children.size()==1&&native_class(*row.master)==native_class(*original.object),"retained Layer/Definition class context");retained_class_data(archive,source,original.children[0],*row.master);}
    require(archive.EndWrite3dmChunk(),"retained selected record end");return;
  }
  if(row.type()==5)require(archive.Write3dmLayer(*ON_Layer::Cast(row.master)),"retained new Layer serializer");
  else if(row.type()==11)require(archive.Write3dmInstanceDefinition(*ON_InstanceDefinition::Cast(row.master)),"retained new Definition serializer");
  else {require(row.type()==12&&row.attributes&&archive.Write3dmObject(*row.master,row.attributes),"retained new Entity serializer");}
}
inline void retained_selected_table(ON_BinaryArchive& archive,const RetainedRequest& request,unsigned type) {
  bool(ON_BinaryArchive::*begin)()=type==5?&ON_BinaryArchive::BeginWrite3dmLayerTable:type==11?&ON_BinaryArchive::BeginWrite3dmInstanceDefinitionTable:&ON_BinaryArchive::BeginWrite3dmObjectTable;
  bool(ON_BinaryArchive::*end)()=type==5?&ON_BinaryArchive::EndWrite3dmLayerTable:type==11?&ON_BinaryArchive::EndWrite3dmInstanceDefinitionTable:&ON_BinaryArchive::EndWrite3dmObjectTable;
  require((archive.*begin)(),"retained selected table begin");if(request.order.contains(type))for(const auto i:request.order.at(type))retained_write_row(archive,request.live[i]);require((archive.*end)(),"retained selected table end");
}
inline void retained_new_mm_archive(const RetainedRequest& request,const std::filesystem::path& destination) {
  ON_Write3dmBufferArchive archive(0,max_archive_bytes,50,ON::Version());ON_3dmProperties properties;ON_3dmSettings settings;
  properties.m_Application.m_application_name=L"OpenMatrix9 selected retained adapter";properties.m_Notes.m_notes=ON_wString(text(request.value["document"]["name"]).c_str());
  settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem::Millimeters;settings.m_PageUnitsAndTolerances.m_unit_system=ON_UnitSystem(unit(request.value["document"]["page_unit"]));
  const auto& current=request.value["document"]["current_layer"];settings.SetCurrentLayerId(uuid(current["id"]));settings.SetV5CurrentLayerIndex(retained_index(current["archive_reference_index"]));
  require(archive.Write3dmStartSection(50,"OM9 current-state selected native serializer")&&archive.Write3dmProperties(properties)&&archive.Write3dmSettings(settings),"retained new_mm metadata serializer");
  archive.SetReferencedComponentIndexMapping(false);archive.SetShouldSerializeUserDataDefault(true);
  auto empty=[&](bool(ON_BinaryArchive::*begin)(),bool(ON_BinaryArchive::*end)()){require((archive.*begin)()&&(archive.*end)(),"retained new_mm empty public table serializer");};
  empty(&ON_BinaryArchive::BeginWrite3dmBitmapTable,&ON_BinaryArchive::EndWrite3dmBitmapTable);
  empty(&ON_BinaryArchive::BeginWrite3dmTextureMappingTable,&ON_BinaryArchive::EndWrite3dmTextureMappingTable);
  empty(&ON_BinaryArchive::BeginWrite3dmMaterialTable,&ON_BinaryArchive::EndWrite3dmMaterialTable);
  empty(&ON_BinaryArchive::BeginWrite3dmLinetypeTable,&ON_BinaryArchive::EndWrite3dmLinetypeTable);
  retained_selected_table(archive,request,5);empty(&ON_BinaryArchive::BeginWrite3dmGroupTable,&ON_BinaryArchive::EndWrite3dmGroupTable);
  empty(&ON_BinaryArchive::BeginWrite3dmDimStyleTable,&ON_BinaryArchive::EndWrite3dmDimStyleTable);
  empty(&ON_BinaryArchive::BeginWrite3dmLightTable,&ON_BinaryArchive::EndWrite3dmLightTable);
  empty(&ON_BinaryArchive::BeginWrite3dmHatchPatternTable,&ON_BinaryArchive::EndWrite3dmHatchPatternTable);
  retained_selected_table(archive,request,11);retained_selected_table(archive,request,12);
  empty(&ON_BinaryArchive::BeginWrite3dmHistoryRecordTable,&ON_BinaryArchive::EndWrite3dmHistoryRecordTable);
  require(archive.Write3dmEndMark()&&archive.CurrentPosition()==archive.SizeOfArchive()&&archive.CurrentPosition()<=max_archive_bytes,"retained new_mm exact EOF/budget");
  request.revalidate();PrivateFile file(destination);const auto bytes=static_cast<size_t>(archive.CurrentPosition());require(std::fwrite(archive.Buffer(),1,bytes,file.value)==bytes&&!std::ferror(file.value)&&std::fflush(file.value)==0&&_commit(_fileno(file.value))==0,"retained new_mm private write/sync");const int close=std::fclose(file.value);file.value=nullptr;require(close==0,"retained new_mm private close");file.complete=true;
}
inline J retained_ack(const RetainedRequest& request,const char* status) {
  auto n=[&](unsigned type){return request.order.contains(type)?request.order.at(type).size():0;};
  return J{{"schema_version",4},{"writer_policy","om9-retained-current-v1"},{"bounds_policy","cold_leaf_hull_v1"},{"status",status},{"archive_version",50},{"objects",n(12)},{"definitions",n(11)},{"layers",n(5)},{"primary_source_sha256",request.primary?J(request.primary->sha256()):J(nullptr)},{"cold_validation_required",true},{"admission_required",true},{"host_ready",false}};
}
inline J write_retained(const std::filesystem::path& packet_path,const std::filesystem::path& destination) {
  RetainedRequest request(packet_path);retained_build_current(request);retained_refresh_caches(request);
  const int errors=ON_GetErrorCount(),warnings=ON_GetWarningCount();
  if(request.primary){std::map<ON__UINT32,std::vector<unsigned char>> replacements;for(const auto type:{5u,11u,12u})replacements.emplace(retained_table_code(type),retained_table_fragment(retained_table_code(type),[&](ON_BinaryArchive& a){if(request.order.contains(type))for(const auto i:request.order.at(type))retained_write_row(a,request.live[i]);}));retained_write_assembled(*request.primary,replacements,destination);}
  else retained_new_mm_archive(request,destination);
  require(ON_GetErrorCount()==errors&&ON_GetWarningCount()==warnings,"retained current writer native warning/error");request.revalidate();
  File output(destination,true);const auto bytes=_filelengthi64(_fileno(output.value));require(bytes>0&&static_cast<uintmax_t>(bytes)<=max_archive_bytes,"retained written file byte budget");
  auto report=retained_ack(request,"written_retained_current");report["request_sha256"]=request.request_sha256;report["output_bytes"]=bytes;report["output_sha256"]=source_sha256(output.value,static_cast<uintmax_t>(bytes));return report;
}

inline J retained_current_layer(const NativeSourceArchive& output) {
  const auto id=output.settings().CurrentLayerId();const int index=output.settings().CurrentLayerIndex();int reference=index,sdk=ON_UNSET_INT_INDEX;
  for(const auto& record:output.records())if(record.native_component_type==5&&((!ON_UuidIsNil(id)&&record.id==id)||(ON_UuidIsNil(id)&&record.reference_index==index))) {reference=record.reference_index;sdk=record.sdk_index;break;}
  return J{{"id",nullable_id(id)},{"archive_reference_index",retained_index(reference)},{"sdk_destination_index",retained_index(sdk)}};
}
inline J retained_actual(const NativeSourceArchive& output) {
  J actual{{"model_units",native_units(output.settings().m_ModelUnitsAndTolerances)},{"page_units",native_units(output.settings().m_PageUnitsAndTolerances)},
    {"current_layer",retained_current_layer(output)},{"objects",J::array()},{"layers",J::array()},{"definitions",J::array()},{"bounds",J::array()}};
  std::map<int,std::string> layer_ids;for(const auto& r:output.records())if(r.native_component_type==5)layer_ids.emplace(r.reference_index,id_string(r.id));source_record_count=0;
  for(const auto& r:output.records()) {
    operation_record="cold actual retained record "+id_string(r.id);const auto type=r.native_component_type;
    if(type!=5&&type!=11&&type!=12)continue;J record{{"id",id_string(r.id)},{"native_component_type",type},{"native_kind",native_kind(*r.object)},{"native_class_uuid",native_class(*r.object)},{"wire",retained_wire(r)}};
    if(type==12) {
      const auto* g=ON_Geometry::Cast(r.object.get());require(g&&r.attributes,"retained cold actual geometry/attrs");const auto& a=*r.attributes;
      record["label"]=bounded_native_text(a.m_name);record["is_definition_member"]=a.IsInstanceDefinitionObject();record["placement"]=matrix_json(ON_Xform::IdentityTransformation);
      if(const auto* ref=ON_InstanceRef::Cast(g))record["placement"]=matrix_json(ref->m_xform);
      try {
        if(const auto* curve=ON_Curve::Cast(g))require(exact_curve_parameters(*curve),"retained actual curve requires an unrepresented segment parameter map");
        if(const auto* mesh=ON_Mesh::Cast(g))require(mesh->m_FN.Count()==0&&mesh->m_T.Count()==0&&mesh->m_S.Count()==0&&mesh->m_K.Count()==0&&mesh->NgonCount()==0,"retained actual Mesh has unrepresented native channels");
        if(const auto* cloud=ON_PointCloud::Cast(g))require(cloud->m_V.Count()==0&&cloud->m_flags==0,"retained actual Cloud has unrepresented scalar/flag channels");
        record["geometry"]=geometry_inventory(*g);
      }catch(const std::runtime_error&){record["geometry"]=nullptr;}
      J refs=J::array();record["attributes"]=attribute_inventory(a,refs);require(layer_ids.contains(a.m_layer_index),"retained cold actual Layer reference");record["attributes"]["layer"]=layer_ids.at(a.m_layer_index);
      record["preservation"]=attr_values(a,record["attributes"]["userdata"]);actual["objects"].push_back(std::move(record));
    } else if(type==5) {const auto* l=ON_Layer::Cast(r.object.get());require(l,"retained cold actual Layer");record["name"]=bounded_native_text(l->Name());record["parent"]=nullable_id(l->ParentLayerId());record["visible"]=l->IsVisible();record["locked"]=l->IsLocked();record["color"]=color_json(l->Color());record["preservation"]=layer_values(*l,native_userdata(*l));actual["layers"].push_back(std::move(record));}
    else {const auto* d=ON_InstanceDefinition::Cast(r.object.get());require(d,"retained cold actual Definition");record["label"]=bounded_native_text(d->Name());record["members"]=J::array();const auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)record["members"].push_back(id_string(ids[i]));record["units"]=definition_units(d->UnitSystem());record["linked"]=d->IsLinkedType();record["preservation"]=definition_values(*d,native_userdata(*d));actual["definitions"].push_back(std::move(record));}
  }
  return actual;
}
inline size_t retained_preserved_class_check(const NativeSourceArchive& source,NativeArchiveSpan original,const NativeSourceArchive& output,NativeArchiveSpan current) {
  const auto a=source.class_children(original),b=output.class_children(current);require(a.size()==b.size(),"retained cold class chunk coverage changed");size_t checked=0;
  for(size_t i=0;i<a.size();++i)if(a[i].type!=TCODE_OPENNURBS_CLASS_DATA){require(a[i].type==b[i].type&&source.span_sha256(a[i])==output.span_sha256(b[i]),"retained cold original class UUID/userdata/framing bytes changed");++checked;}return checked;
}
inline J inspect_retained_current(const std::filesystem::path& packet_path,const std::filesystem::path& output_path) {
  RetainedRequest request(packet_path);retained_build_current(request);retained_refresh_caches(request);auto output=NativeSourceArchive::open(output_path);
  require(output->retained_usage()<=max_archive_bytes-request.total_usage,"retained cold aggregate source/output owner budget");
  require(output->records().size()==request.live.size(),"retained cold live record count mismatch");
  J report=retained_ack(request,"validated_retained_current");report["request_sha256"]=request.request_sha256;report["output_sha256"]=output->sha256();report["output_bytes"]=output->byte_length();
  report["raw_spans"]=J::array();
  for(const auto span:output->tables())report["raw_spans"].push_back(J{{"kind","top_level"},{"native_component_type",nullptr},{"id",nullptr},{"table_type",span.type},{"record_type",nullptr},{"offset",span.begin},{"byte_length",span.size()},{"sha256",output->span_sha256(span)}});
  for(const auto& record:output->records())report["raw_spans"].push_back(J{{"kind","record"},{"native_component_type",record.native_component_type},{"id",id_string(record.id)},{"table_type",record.table_type},{"record_type",record.span.type},{"offset",record.span.begin},{"byte_length",record.span.size()},{"sha256",output->span_sha256(record.span)}});
  J checks{{"live_records",0},{"raw_records",0},{"raw_tables",0},{"opaque_chunks",0},{"bounds",0}};std::map<unsigned,size_t> ordinals;std::map<std::string,ON_Object*> masters;
  for(const auto& record:output->records()) {
    const auto id=id_string(record.id);require(request.by_id.contains(id),"retained cold unexpected output UUID");const auto& row=request.live[request.by_id.at(id)];operation_record="cold retained correspondence "+id;
    require(row.identity==retained_output(record),"retained cold actual wire/reference/SDK identity mismatch");require(request.order.contains(row.type())&&ordinals[row.type()]<request.order.at(row.type()).size()&&request.live[request.order.at(row.type())[ordinals[row.type()]++]].id()==id,"retained cold actual table order mismatch");
    checks["live_records"]=checks["live_records"].get<size_t>()+1;
    if(row.type()==11||row.type()==12)masters.emplace(id,record.object.get());
    if(row.original&&row.action=="retain"&&!row.refresh) {require(row.source->span_sha256(row.original->span)==output->span_sha256(record.span),"retained cold unchanged raw record changed");checks["raw_records"]=checks["raw_records"].get<size_t>()+1;continue;}
    require(native_class(*row.master)==native_class(*record.object),"retained cold selected master family mismatch");
    if(row.type()==12) {
      const auto* a=ON_Geometry::Cast(row.master);const auto* b=ON_Geometry::Cast(record.object.get());require(a&&b&&row.attributes&&record.attributes,"retained cold selected Entity geometry/attrs");
      if(const auto* i=ON_InstanceRef::Cast(a)){const auto* j=ON_InstanceRef::Cast(b);require(j&&i->m_instance_definition_uuid==j->m_instance_definition_uuid&&matrix_json(i->m_xform)==matrix_json(j->m_xform),"retained cold actual Instance UUID/frame mismatch");}
      else require(geometry_inventory(*a)==geometry_inventory(*b),"retained cold selected actual geometry mismatch");
      require(row.attributes->m_uuid==record.attributes->m_uuid&&native_text(row.attributes->m_name)==native_text(record.attributes->m_name)&&row.attributes->m_layer_index==record.attributes->m_layer_index&&row.attributes->IsVisible()==record.attributes->IsVisible()&&row.attributes->Mode()==record.attributes->Mode(),"retained cold selected Entity identity/label/mode/Layer mismatch");
      require(attr_values(*row.attributes,native_userdata(*row.attributes))==attr_values(*record.attributes,native_userdata(*record.attributes)),"retained cold selected actual richer attributes mismatch");
      if(row.original&&row.action!="regenerate") {
        const auto& old=*row.original;require(old.children.size()==record.children.size(),"retained cold selected Entity child coverage");
        if(!row.generated)checks["opaque_chunks"]=checks["opaque_chunks"].get<size_t>()+retained_preserved_class_check(*row.source,old.children[1],*output,record.children[1]);
        for(size_t i=2;i<old.children.size();++i)if(old.children[i].type!=TCODE_OBJECT_RECORD_ATTRIBUTES||(!flag(row.changed["attributes"])&&!flag(row.changed["label"]))) {require(old.children[i].type==record.children[i].type&&row.source->span_sha256(old.children[i])==output->span_sha256(record.children[i]),"retained cold original attribute/userdata/chunk bytes changed");checks["opaque_chunks"]=checks["opaque_chunks"].get<size_t>()+1;}
      }
    } else if(row.type()==5) {const auto* a=ON_Layer::Cast(row.master);const auto* b=ON_Layer::Cast(record.object.get());require(a&&b&&native_text(a->Name())==native_text(b->Name())&&a->ParentLayerId()==b->ParentLayerId()&&a->Color()==b->Color()&&a->IsVisible()==b->IsVisible()&&a->IsLocked()==b->IsLocked()&&layer_values(*a,native_userdata(*a))==layer_values(*b,native_userdata(*b)),"retained cold selected actual Layer fields mismatch");}
    else if(row.type()==11) {const auto* a=ON_InstanceDefinition::Cast(row.master);const auto* b=ON_InstanceDefinition::Cast(record.object.get());require(a&&b&&native_text(a->Name())==native_text(b->Name())&&definition_units(a->UnitSystem())==definition_units(b->UnitSystem())&&definition_values(*a,native_userdata(*a))==definition_values(*b,native_userdata(*b)),"retained cold selected actual Definition fields/units mismatch");const auto& ids=a->InstanceGeometryIdList();const auto& actual_ids=b->InstanceGeometryIdList();require(ids.Count()==actual_ids.Count(),"retained cold Definition member count");for(int i=0;i<ids.Count();++i)require(ids[i]==actual_ids[i],"retained cold Definition member UUID/order mismatch");}
    if((row.type()==5||row.type()==11)&&row.original&&row.action!="regenerate")checks["opaque_chunks"]=checks["opaque_chunks"].get<size_t>()+retained_preserved_class_check(*row.source,row.original->children[0],*output,record.children[0]);
  }
  if(request.primary) {
    std::map<ON__UINT32,size_t> positions;
    for(const auto span:request.primary->tables())if(span.type!=TCODE_ENDOFFILE&&span.type!=TCODE_LAYER_TABLE&&span.type!=TCODE_INSTANCE_DEFINITION_TABLE&&span.type!=TCODE_OBJECT_TABLE) {
      const auto ordinal=positions[span.type]++;size_t seen=0;const NativeArchiveSpan* found=nullptr;
      for(const auto& candidate:output->tables())if(candidate.type==span.type&&seen++==ordinal){found=&candidate;break;}
      require(found&&request.primary->span_sha256(span)==output->span_sha256(*found),"retained cold untouched primary table/header/settings bytes changed");checks["raw_tables"]=checks["raw_tables"].get<size_t>()+1;
    }
    require(native_units(request.primary->settings().m_ModelUnitsAndTolerances)==native_units(output->settings().m_ModelUnitsAndTolerances)&&native_units(request.primary->settings().m_PageUnitsAndTolerances)==native_units(output->settings().m_PageUnitsAndTolerances),"retained cold primary original units/tolerances changed");
  } else require(output->settings().m_ModelUnitsAndTolerances.m_unit_system.UnitSystem()==ON::LengthUnitSystem::Millimeters&&output->settings().m_PageUnitsAndTolerances.m_unit_system.UnitSystem()==unit(request.value["document"]["page_unit"]),"retained cold new_mm declared units mismatch");
  report["actual"]=retained_actual(*output);
  if(request.value["document"]["current_layer"]["policy"]=="explicit_output"){const auto& expected=request.value["document"]["current_layer"];require(report["actual"]["current_layer"]["id"]==expected["id"]&&report["actual"]["current_layer"]["archive_reference_index"]==expected["archive_reference_index"],"retained cold actual current Layer mismatch");}
  size_t work=0;for(const auto& row:request.live)if(row.refresh) {
    const auto* master=masters.at(row.id());const auto* d=ON_InstanceDefinition::Cast(master);const auto* ref=ON_InstanceRef::Cast(master);require(d||ref,"retained cache refresh on noncache family");operation_record="cold independently traversed bounds "+row.id();const auto computed=retained_compute_hull(masters,row.id(),work);const auto stored=d?d->BoundingBox():ref->m_bbox;const bool matched=retained_box_equal(stored,computed);require(matched,"retained cold stored cache differs from actual current leaf hull");
    report["actual"]["bounds"].push_back(J{{"id",row.id()},{"native_component_type",row.type()},{"stored",retained_box(stored)},{"computed",retained_box(computed)},{"matched",matched}});checks["bounds"]=checks["bounds"].get<size_t>()+1;
  }
  report["checks"]=checks;size_t nodes=0;source_output_tree(report,nodes);request.revalidate();output->revalidate_bytes();return report;
}
