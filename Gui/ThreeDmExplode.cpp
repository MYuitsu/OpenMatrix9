#include "ThreeDmExplode.h"
#include <fstream>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <algorithm>
#include <cmath>
#include <functional>
#include <map>

namespace OpenMatrix9Gui::ThreeDm {
namespace {
std::string utf8(const ON_wString& s) { ON_String v(s); return v.IsEmpty()?std::string{}:v.Array(); }
std::string uuid(const ON_UUID& id) { char s[37]{}; ON_UuidToString(id,s); return s; }
ON_Xform affine(ON_Xform t) {
    if(t.IsValid() && std::abs(t[3][0])<=1e-12 && std::abs(t[3][1])<=1e-12
       && std::abs(t[3][2])<=1e-12 && std::abs(t[3][3]-1)<=1e-12) {
        t[3][0]=t[3][1]=t[3][2]=0; t[3][3]=1;
    }
    if(!t.IsValid() || !t.IsAffine() || t.Determinant()==0)
        throw ExchangeError("Invalid or singular block transform");
    return t;
}
std::array<double,3> xyz(const ON_3dPoint& p) { return {p.x,p.y,p.z}; }
using Contours = ON_ClassArray<ON_ClassArray<ON_SimpleArray<ON_Curve*>>>;
struct UuidLess { bool operator()(const ON_UUID& a,const ON_UUID& b) const { return ON_UuidCompare(a,b)<0; } };
struct OwnedContours {
    Contours values;
    ~OwnedContours() { for(int i=0;i<values.Count();++i)for(int j=0;j<values[i].Count();++j)
        for(int k=0;k<values[i][j].Count();++k)delete values[i][j][k]; }
};
class Decoder {
    ONX_Model model;
    ExchangeExplode result;
    std::map<ON_UUID,const ON_ModelGeometryComponent*,UuidLess> objects;
    std::map<ON_UUID,const ON_InstanceDefinition*,UuidLess> definitions;
    std::map<int,const ON_Layer*> layers;
    std::vector<ON_UUID> ancestry;
    ON::LengthUnitSystem units;
    double fileTolerance = 1e-6;

    ExplodeComponent metadata(const ON_ModelGeometryComponent& c,const ExplodeComponent* parent) {
        const auto a=c.Attributes(nullptr); const auto g=c.Geometry(nullptr);
        if(!a||!g)throw ExchangeError("Invalid source geometry component");
        if(a->m_space==ON::page_space)throw ExchangeError("Layout-space records require a paper viewport and are retained");
        ExplodeComponent item; item.sourceUuid=uuid(c.Id()); item.sourceClass=g->ClassId()->ClassName();
        item.name=utf8(a->m_name); if(item.name.empty())item.name=item.sourceClass;
        const auto found=layers.find(a->m_layer_index); auto l=found==layers.end()?nullptr:found->second;
        auto color=a->ColorSource()==ON::color_from_layer&&l?l->Color():a->m_color;
        item.color={color.Red(),color.Green(),color.Blue()};
        item.visible=a->IsVisible(); item.locked=a->Mode()==ON::locked_object;
        std::vector<std::string> names; std::vector<ON_UUID> visited;
        while(l) {
            if(std::find(visited.begin(),visited.end(),l->Id())!=visited.end())throw ExchangeError("Cyclic source layer ancestry");
            visited.push_back(l->Id()); names.push_back(utf8(l->Name()));
            item.visible=item.visible&&l->IsVisible(); item.locked=item.locked||l->IsLocked();
            auto id=l->ParentId(); l=nullptr;
            if(id!=ON_nil_uuid)for(const auto& entry:layers)if(entry.second->Id()==id){l=entry.second;break;}
        }
        for(auto it=names.rbegin();it!=names.rend();++it){if(!item.layer.empty())item.layer+="::";item.layer+=*it;}
        if(parent){item.name=parent->name+"/"+item.name;item.visible=item.visible&&parent->visible;
            item.locked=item.locked||parent->locked;if(a->ColorSource()==ON::color_from_parent)item.color=parent->color;}
        return item;
    }
    void append(ExplodeComponent item) {
        if(result.components.size()>=1000000)throw ExchangeError("Explode component limit exceeded");
        result.components.push_back(std::move(item));
    }
    void native(const ON_Geometry& source,const ON_Xform& world,ExplodeComponent item,const char* role) {
        std::unique_ptr<ON_Geometry> copy(ON_Geometry::Cast(source.Duplicate()));
        if(!copy||(!world.IsIdentity()&&!copy->Transform(world)))throw ExchangeError("Cannot transform exploded geometry");
        const auto g=copy.get();
        if(auto p=ON_Point::Cast(g))item.geometry=BRepBuilderAPI_MakeVertex(gp_Pnt(p->point.x,p->point.y,p->point.z)).Shape();
        else if(auto curve=ON_Curve::Cast(g))item.geometry=importCurve(*curve,fileTolerance);
        else if(auto brep=ON_Brep::Cast(g))item.geometry=importBrep(*brep,fileTolerance);
        else if(auto mesh=ON_Mesh::Cast(g))item.geometry=importMesh(*mesh);
        else if(auto extrusion=ON_Extrusion::Cast(g)) {
            std::unique_ptr<ON_Brep> brep(extrusion->BrepForm());
            if(!brep)throw ExchangeError("Cannot recover block extrusion BRep");
            item.geometry=importBrep(*brep,fileTolerance);
        }
        else if(auto surface=ON_Surface::Cast(g)) {
            // A bounded face retains the exact NURBS surface, including cage iso-surfaces.
            const auto u=surface->Domain(0),v=surface->Domain(1);
            BRepBuilderAPI_MakeFace face(importSurface(*surface),u[0],u[1],v[0],v[1],fileTolerance);
            if(!face.IsDone())throw ExchangeError("Cannot make exploded surface face");
            item.geometry=face.Shape();
        } else throw ExchangeError("Unsupported exploded block member: "+item.sourceClass);
        if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)) {
            if(shape->IsNull())throw ExchangeError("Empty exploded shape");
            if(result.scaleMm!=1){gp_Trsf scale;scale.SetScale(gp_Pnt(0,0,0),result.scaleMm);*shape=BRepBuilderAPI_Transform(*shape,scale,true).Shape();}
        }else for(auto& p:std::get<MeshData>(item.geometry).vertices)for(auto& v:p)v*=result.scaleMm;
        item.role=role; append(std::move(item));
    }
    void curve(const ON_Curve& c,const ON_Xform& world,ExplodeComponent item,const char* role) { native(c,world,std::move(item),role); }
    void cage(const ON_NurbsCage& c,const ON_Xform& world,ExplodeComponent item) {
        if(!c.IsValid())throw ExchangeError("Invalid source NURBS cage");
        auto descriptor=std::make_shared<ExplodeCage>();
        std::size_t count=1;
        for(int d=0;d<3;++d){descriptor->counts[d]=c.CVCount(d);descriptor->degrees[d]=c.Order(d)-1;
            count*=c.CVCount(d);if(count>1000000)throw ExchangeError("Cage control-point limit exceeded");
            for(int k=0;k<c.KnotCount(d);++k)descriptor->knots[d].push_back(c.Knot(d,k));}
        for(int u=0;u<c.CVCount(0);++u)for(int v=0;v<c.CVCount(1);++v)for(int w=0;w<c.CVCount(2);++w){
            ON_3dPoint p;if(!c.GetCV(u,v,w,p))throw ExchangeError("Invalid cage control point");
            auto q=world*p;q*=result.scaleMm;descriptor->points.push_back(xyz(q));descriptor->weights.push_back(c.Weight(u,v,w));}
        item.cage=descriptor;
        for(int d=0;d<3;++d)for(int side=0;side<2;++side){
            std::unique_ptr<ON_NurbsSurface> surface(c.IsoSurface(d,c.Domain(d)[side]));
            if(!surface||!surface->IsValid())throw ExchangeError("Cannot evaluate exact cage boundary");
            auto part=item;part.name+="/Boundary"+std::to_string(d*2+side+1);native(*surface,world,std::move(part),"boundary-surface");}
        // The lattice uses actual current Euclidean CVs, including interior/deformed
        // CVs. It is not a bounding-box replacement for the NURBS volume.
        for(int u=0;u<c.CVCount(0);++u)for(int v=0;v<c.CVCount(1);++v)for(int w=0;w<c.CVCount(2);++w)
            for(int d=0;d<3;++d){int next[3]={u,v,w};if(++next[d]>=c.CVCount(d))continue;
                ON_3dPoint p,q;c.GetCV(u,v,w,p);c.GetCV(next[0],next[1],next[2],q);
                if(p.DistanceTo(q)<=ON_ZERO_TOLERANCE)continue;
                ON_LineCurve line(p,q);auto part=item;part.name+="/ControlEdge";curve(line,world,std::move(part),"control-edge");}
    }
    const ON_DimStyle& style(const ON_Annotation& a) {
        auto id=a.DimensionStyleId();
        auto ref=model.ComponentFromId(ON_ModelComponent::Type::DimStyle,id);
        auto parent=ON_DimStyle::Cast(ref.ModelComponent());
        if(id!=ON_nil_uuid&&!parent)throw ExchangeError("Missing annotation dimension style");
        return a.DimensionStyle(parent?*parent:ON_DimStyle::Default);
    }
    double dimscale(const ON_DimStyle& s) {
        const auto scale=s.DimScale();if(!std::isfinite(scale)||scale<=0)throw ExchangeError("Invalid annotation display scale");return scale;
    }
    void textContours(const ON_Annotation& a,const ON_Xform& world,ExplodeComponent item) {
        const auto& s=style(a);auto content=a.Text();
        if(s.TextOrientation()==ON::TextOrientation::InView)throw ExchangeError("View-dependent text requires a captured viewport and is retained");
        if(!content||a.PlainText().IsEmpty())throw ExchangeError("Text has no visible glyphs to explode");
        ON_Xform placement;
        if(!a.GetTextXform(nullptr,&s,dimscale(s),placement))throw ExchangeError("Cannot recover source text placement");
        // Refuse substituted fonts/glyphs before openNURBS' rendering fallback can
        // convert only some glyphs and silently omit the rest.
        const auto runs=content->TextRuns(false);bool hasGlyph=false;
        if(!runs)throw ExchangeError("Source text has no glyph runs");
        for(int i=0;i<runs->Count();++i)if(auto run=(*runs)[i]){
            auto font=run->Font();if(!font)font=&s.Font();
            if(font->IsManagedSubstitutedFont())throw ExchangeError("Source text font is unavailable: "+utf8(font->FamilyName()));
            auto points=run->UnicodeString();if(!points)continue;
            for(std::size_t k=0;points[k];++k){auto cp=points[k];if(cp==32||cp==9||cp==10||cp==13)continue;
                hasGlyph=true;const auto glyph=font->CodePointGlyph(cp);
                if(!glyph||!glyph->FontGlyphIndexIsSet())throw ExchangeError("Source font glyph is unavailable; text was retained");
                OwnedContours probe;auto& glyphContours=probe.values.AppendNew();
                glyph->GetGlyphContours(font->IsSingleStrokeFont(),s.TextHeight(),glyphContours,nullptr,nullptr);
                bool visibleContour=false;
                for(int ci=0;ci<glyphContours.Count();++ci)for(int cj=0;cj<glyphContours[ci].Count();++cj)
                    visibleContour=visibleContour||glyphContours[ci][cj]!=nullptr;
                if(!visibleContour)throw ExchangeError("Source font contours are unavailable; text was retained");}
        }
        if(!hasGlyph)throw ExchangeError("Text has no visible glyphs to explode");
        OwnedContours contours;
        // The SDK populates contours but returns false unconditionally; inspect
        // actual outputs, not that status (opennurbs_textglyph.cpp).
        a.GetTextGlyphContours(nullptr,&s,true,s.Font().IsSingleStrokeFont(),contours.values);
        std::size_t count=0;
        for(int i=0;i<contours.values.Count();++i)for(int j=0;j<contours.values[i].Count();++j)
            for(int k=0;k<contours.values[i][j].Count();++k)if(auto c=contours.values[i][j][k]){
                auto part=item;part.name+="/Contour"+std::to_string(++count);curve(*c,world,std::move(part),"text-contour");}
        if(!count)throw ExchangeError("Source font contours are unavailable; text was retained");
    }
    void arrow(ON_Arrowhead::arrow_type type,const ON_Xform& placement,const ON_Xform& world,ExplodeComponent item) {
        if(type==ON_Arrowhead::arrow_type::None)return;
        if(type==ON_Arrowhead::arrow_type::UserBlock)throw ExchangeError("Custom block dimension arrowheads are not supported");
        ON_2dPointArray points;if(ON_Arrowhead::GetPoints(type,points)<2)throw ExchangeError("Unsupported dimension arrowhead");
        ON_Polyline line;for(int i=0;i<points.Count();++i)line.Append(ON_3dPoint(points[i].x,points[i].y,0));line.Append(line[0]);
        ON_PolylineCurve outline(line);item.name+="/Arrow";curve(outline,world*placement,std::move(item),"dimension-arrow");
    }
    void dimension(const ON_Dimension& a,const ON_Xform& world,ExplodeComponent item) {
        const auto& s=style(a);const auto scale=dimscale(s);
        const bool center=ON_Centermark::Cast(&a)!=nullptr;
        if(!center&&(ON_DimRadial::Cast(&a)?s.DimRadialTextOrientation():s.DimTextOrientation())==ON::TextOrientation::InView)
            throw ExchangeError("View-dependent dimension text requires a captured viewport and is retained");
        ON_Xform textPlacement=ON_Xform::Nan;ON_3dPoint rectangle[4]={ON_3dPoint::UnsetPoint,ON_3dPoint::UnsetPoint,ON_3dPoint::UnsetPoint,ON_3dPoint::UnsetPoint};
        if(!center){
            // Angular dimensions intentionally reject the distance-unit overload.
            const auto angular=ON_DimAngular::Cast(&a);
            const bool updated=angular?angular->UpdateDimensionText(&s):a.UpdateDimensionText(units,&s);
            if(!updated||a.PlainText().IsEmpty())throw ExchangeError("Cannot recover dimension label");
            // This SDK's ON_DimLinear writes the complete transform but returns
            // an unchanged false rc. NaN initialization distinguishes that valid
            // output from an early-return failure; never substitute identity.
            a.GetTextXform(nullptr,&s,scale,textPlacement);
            if(!textPlacement.IsValid()||!textPlacement.IsAffine()||textPlacement.Determinant()==0)
                throw ExchangeError("Cannot recover dimension label placement");
            if(!a.Text()||!a.Text()->Get3dCorners(rectangle))throw ExchangeError("Cannot recover dimension text rectangle");
            for(auto& p:rectangle)p=textPlacement*p;
        }
        ON_Line lines[9];bool isline[9]{};ON_Arc arcs[2];bool isarc[2]{};int count=0;bool ok=false;
        if(auto linear=ON_DimLinear::Cast(&a)){count=4;ok=linear->GetDisplayLines(nullptr,&s,scale,rectangle,lines,isline,count);}
        else if(auto angular=ON_DimAngular::Cast(&a)){count=2;ok=angular->GetDisplayLines(nullptr,&s,scale,rectangle,lines,isline,arcs,isarc,count,2);}
        else if(auto radial=ON_DimRadial::Cast(&a)){count=9;ok=radial->GetDisplayLines(&s,scale,rectangle,lines,isline,count);}
        else if(auto ordinate=ON_DimOrdinate::Cast(&a)){count=3;ok=ordinate->GetDisplayLines(&s,scale,rectangle,lines,isline,count);}
        else if(auto mark=ON_Centermark::Cast(&a)){count=6;ok=mark->GetDisplayLines(&s,scale,lines,isline,count);}
        if(!ok)throw ExchangeError("Cannot decode source dimension display geometry");
        const auto before=result.components.size();
        for(int i=0;i<count;++i)if(isline[i]&&lines[i].Length()>ON_ZERO_TOLERANCE){ON_LineCurve c(lines[i]);auto part=item;part.name+="/Line"+std::to_string(i+1);curve(c,world,std::move(part),"dimension-line");}
        for(int i=0;i<2;++i)if(isarc[i]){ON_ArcCurve c(arcs[i]);auto part=item;part.name+="/Arc"+std::to_string(i+1);curve(c,world,std::move(part),"dimension-arc");}
        if(auto linear=ON_DimLinear::Cast(&a))for(int end=0;end<2;++end){if(end?s.SuppressArrow2():s.SuppressArrow1())continue;
            ON_Xform placement;linear->GetArrowXform(end,s.ArrowSize()*scale,a.ArrowIsFlipped(end),false,placement);arrow(end?s.ArrowType2():s.ArrowType1(),placement,world,item);}
        if(auto angular=ON_DimAngular::Cast(&a))for(int end=0;end<2;++end){if(end?s.SuppressArrow2():s.SuppressArrow1())continue;
            ON_Xform placement;angular->GetArrowXform(end,s.ArrowSize()*scale,a.ArrowIsFlipped(end),false,placement);arrow(end?s.ArrowType2():s.ArrowType1(),placement,world,item);}
        if(auto radial=ON_DimRadial::Cast(&a))if(!s.SuppressArrow1()){
            ON_Xform placement;radial->GetArrowXform(s.ArrowSize()*scale,placement);arrow(s.ArrowType1(),placement,world,item);}
        if(result.components.size()==before)throw ExchangeError("Dimension has no display curves to explode");
        if(!center){
            ExplodeText text;text.text=utf8(a.PlainText());text.richText=utf8(a.RichText());text.fontFamily=utf8(s.Font().FamilyName());text.heightMm=s.TextHeight()*result.scaleMm;
            auto matrix=ON_Xform::DiagonalTransformation(result.scaleMm)*world*textPlacement;
            if(!matrix.IsValid()||!matrix.IsAffine())throw ExchangeError("Invalid dimension text transform");
            for(int r=0;r<4;++r)for(int c=0;c<4;++c)text.worldTransform[r*4+c]=matrix[r][c];
            text.origin=xyz(matrix*ON_3dPoint::Origin);
            const auto x=matrix*ON_3dVector::XAxis,y=matrix*ON_3dVector::YAxis;
            text.xAxis={x.x,x.y,x.z};text.yAxis={y.x,y.y,y.z};
            item.name+="/Text";item.role="dimension-text";item.geometry=std::move(text);append(std::move(item));
        }
    }
    void decode(const ON_ModelGeometryComponent& c,const ON_Xform& world,const ExplodeComponent* parent) {
        auto item=metadata(c,parent);const auto g=c.Geometry(nullptr);
        if(auto ref=ON_InstanceRef::Cast(g)){
            if(ancestry.size()>=64)throw ExchangeError("Block nesting exceeds 64 levels");
            auto id=ref->m_instance_definition_uuid;
            if(std::find(ancestry.begin(),ancestry.end(),id)!=ancestry.end())throw ExchangeError("Cyclic block definition");
            auto it=definitions.find(id);if(it==definitions.end())throw ExchangeError("Missing block definition (embedded members required)");
            auto transform=affine(world*affine(ref->m_xform));
            const auto& ids=it->second->InstanceGeometryIdList();if(!ids.Count())throw ExchangeError("Block definition has no embedded geometry");
            if(!ancestry.empty())result.flattenedNestedBlocks=true;
            ancestry.push_back(id);
            for(int i=0;i<ids.Count();++i){auto object=objects.find(ids[i]);if(object==objects.end())throw ExchangeError("Missing block member geometry");decode(*object->second,transform,&item);}
            ancestry.pop_back();
        }else if(auto d=ON_Dimension::Cast(g))dimension(*d,world,std::move(item));
        else if(auto t=ON_Text::Cast(g))textContours(*t,world,std::move(item));
        else if(auto control=ON_MorphControl::Cast(g)){
            if(control->m_varient==1)native(control->m_nurbs_curve,world,std::move(item),"control-curve");
            else if(control->m_varient==2)native(control->m_nurbs_surface,world,std::move(item),"control-surface");
            else if(control->m_varient==3)cage(control->m_nurbs_cage,world,std::move(item));
            else throw ExchangeError("Unsupported cage control variant");
        }else if(auto volume=ON_NurbsCage::Cast(g))cage(*volume,world,std::move(item));
        else if(parent)native(*g,world,std::move(item),"block-member");
        else throw ExchangeError("Source record is not a block, text, dimension or cage control");
    }
public:
    ExchangeExplode read(std::span<const unsigned char> bytes,const std::string& sourceUuid,double custom) {
        if(bytes.size()>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
        // A complete file must leave archive version unset until its start section
        // is read. ON_Read3dmBufferArchive instead initializes embedded chunks.
        ON::Begin();ON_Buffer buffer;if(bytes.empty()||buffer.Write(bytes.size(),bytes.data())!=bytes.size())throw ExchangeError("Cannot buffer retained 3DM source archive");
        ON_BinaryArchiveBuffer archive(ON::archive_mode::read3dm,&buffer);
        if(!model.Read(archive,nullptr))throw ExchangeError("Cannot read retained 3DM source archive");
        ON_UUID id=ON_UuidFromString(sourceUuid.c_str());if(id==ON_nil_uuid||uuid(id)!=sourceUuid)throw ExchangeError("Invalid source UUID");
        units=model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem();
        result.scaleMm=(units==ON::LengthUnitSystem::None||units==ON::LengthUnitSystem::CustomUnits)?custom:ON::UnitScale(units,ON::LengthUnitSystem::Millimeters);
        if(!std::isfinite(result.scaleMm)||result.scaleMm<=0)throw ExchangeError("3DM custom/unitless file requires millimeters per file unit");
        fileTolerance=std::max(1e-7/result.scaleMm,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);
        if(!std::isfinite(fileTolerance)||fileTolerance<=0)throw ExchangeError("Invalid source tolerance");
        result.tolerance=fileTolerance*result.scaleMm;
        ONX_ModelComponentIterator geometry(model,ON_ModelComponent::Type::ModelGeometry);
        for(auto c=geometry.FirstComponent();c;c=geometry.NextComponent())if(auto g=ON_ModelGeometryComponent::Cast(c))objects.emplace(g->Id(),g);
        ONX_ModelComponentIterator definition(model,ON_ModelComponent::Type::InstanceDefinition);
        for(auto c=definition.FirstComponent();c;c=definition.NextComponent())if(auto d=ON_InstanceDefinition::Cast(c))definitions.emplace(d->Id(),d);
        ONX_ModelComponentIterator layer(model,ON_ModelComponent::Type::Layer);
        for(auto c=layer.FirstComponent();c;c=layer.NextComponent())if(auto l=ON_Layer::Cast(c))layers.emplace(l->Index(),l);
        auto record=objects.find(id);if(record==objects.end())throw ExchangeError("Source UUID is absent from retained archive");
        const auto g=record->second->Geometry(nullptr);if(!g)throw ExchangeError("Source UUID has no geometry");
        result.sourceUuid=sourceUuid;result.sourceClass=g->ClassId()->ClassName();
        decode(*record->second,ON_Xform::IdentityTransformation,nullptr);
        if(result.components.empty())throw ExchangeError("Source record produced no editable components");
        return std::move(result);
    }
};
}
ExchangeExplode explodeArchiveRecord(const std::filesystem::path& path,const std::string& id,double custom) {
    const auto size=std::filesystem::file_size(path);if(size>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    std::ifstream file(path,std::ios::binary);std::vector<unsigned char> bytes(static_cast<std::size_t>(size));
    if(!file||!file.read(reinterpret_cast<char*>(bytes.data()),static_cast<std::streamsize>(size))||file.peek()!=std::char_traits<char>::eof())throw ExchangeError("Cannot read retained 3DM source archive");
    return Decoder{}.read(bytes,id,custom);
}
ExchangeExplode explodeArchiveRecord(std::span<const unsigned char> bytes,const std::string& id,double custom) {
    return Decoder{}.read(bytes,id,custom);
}
}
