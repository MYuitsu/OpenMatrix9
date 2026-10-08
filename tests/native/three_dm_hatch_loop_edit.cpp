#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include "ThreeDmMerge.h"
#include "ThreeDmHatch.h"
#include <QJsonArray>
#include <QFile>
#include <QTemporaryDir>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool v,const char* s){if(!v)throw ExchangeError(s);}
static QString id(ON_UUID v){char s[37]{};ON_UuidToString(v,s);return QString::fromLatin1(s);}
static QByteArray bytes(const std::filesystem::path& p){QFile f(QString::fromStdWString(p.wstring()));require(f.open(QIODevice::ReadOnly),"loop edit file read");return f.readAll();}
static QJsonObject numeric(QJsonObject n,int sourceIndex){for(auto k:{"class_uuid","user_strings","userdata"})n.remove(k);n["source_index"]=sourceIndex;if(n.contains("segments")){auto children=n["segments"].toArray();for(int i=0;i<children.size();++i)children[i]=numeric(children[i].toObject(),i);n["segments"]=children;}return n;}
static QJsonObject payload(const ON_Hatch& h){auto loops=hatchLoopInventory(h);for(int i=0;i<loops.size();++i){auto row=loops[i].toObject();row["source_index"]=i;row["curve"]=numeric(row["curve"].toObject(),i);loops[i]=row;}return {{"schema_version",1},{"loops",loops}};}
static const ON_Hatch* native(const ArchiveInventory& a,ON_UUID v){auto c=ON_ModelGeometryComponent::Cast(a.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,v).ModelComponent());return c?ON_Hatch::Cast(c->Geometry(nullptr)):nullptr;}
static QJsonObject deform(QJsonObject n){
    const auto name=n["class_name"].toString();
    if(name=="ON_NurbsCurve"){auto cvs=n["cvs"].toArray();for(int i=0;i<cvs.size();++i){auto cv=cvs[i].toArray();cv[0]=cv[0].toDouble()*1.5;cv[1]=cv[1].toDouble()*0.75;if(cv.size()==3&&i==1)for(int j=0;j<3;++j)cv[j]=cv[j].toDouble()*2;cvs[i]=cv;}n["cvs"]=cvs;auto knots=n["knots"].toArray();for(int i=0;i<knots.size();++i)knots[i]=11+knots[i].toDouble()*3;n["knots"]=knots;auto d=n["domain"].toArray();n["domain"]=QJsonArray{11+d[0].toDouble()*3,11+d[1].toDouble()*3};
    }else if(name=="ON_ArcCurve"){n["radius"]=n["radius"].toDouble()*1.25;n["domain"]=QJsonArray{17,39};
    }else if(name=="ON_LineCurve"||name=="ON_PolylineCurve"){auto pts=n["points"].toArray();for(int i=0;i<pts.size();++i){auto pt=pts[i].toArray();pt[0]=pt[0].toDouble()*1.5;pt[1]=pt[1].toDouble()*0.75;pts[i]=pt;}n["points"]=pts;
    }else if(name=="ON_PolyCurve"){auto segments=n["segments"].toArray();for(int i=0;i<segments.size();++i)segments[i]=deform(segments[i].toObject());n["segments"]=segments;}
    return n;
}
static QJsonObject arcLoop(int type,double x,double radius){return {{"type",type},{"source_index",QJsonValue::Null},{"curve",QJsonObject{{"source_index",QJsonValue::Null},{"class_name","ON_ArcCurve"},{"dimension",2},{"domain",QJsonArray{0,2*ON_PI}},{"plane",QJsonArray{x,0,0,1,0,0,0,1,0,0,0,1,0,0,1,0}},{"radius",radius},{"angle_domain",QJsonArray{0,2*ON_PI}}}}};}
int main(){try{ON::Begin();QTemporaryDir temp;require(temp.isValid(),"loop edit temp dir");auto dir=std::filesystem::path(temp.path().toStdWString());int cases=0;
    for(auto unit:{"mm","cm"}){auto path=std::filesystem::temp_directory_path()/(std::string("om9-hatch-loops-")+unit+".3dm");auto immutable=bytes(path);auto inv=inspectArchive(path);QString ns="00000000-0000-4000-8000-000000000144";
        for(auto v:inv.document["records"].toArray()){auto row=v.toObject();auto uuid=ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData());auto original=native(inv,uuid);require(original,"native source Hatch");auto current=*original;auto mm=inv.document["scale_mm"].toDouble();if(mm!=1)transformHatchNative(current,ON_Xform::DiagonalTransformation(mm),inv.nativeModel.get());auto fields=hatchCurrentFields(current,*inv.nativeModel);auto edit=payload(current);auto loops=edit["loops"].toArray();auto loop=loops[0].toObject();loop["curve"]=deform(loop["curve"].toObject());loops[0]=loop;edit["loops"]=loops;fields["loops"]=edit;
            QJsonObject selected{{"namespace",ns},{"source_uuid",id(uuid)},{"host_id","native-loop-edit"},{"action","transform"},{"geometry_matrix",QJsonArray{1,0,0,5,0,1,0,6,0,0,1,7,0,0,0,1}},{"hatch_fields",fields}};
            QJsonObject request{{"schema_version",1},{"sources",QJsonArray{QJsonObject{{"namespace",ns},{"snapshot",QString::fromStdWString(path.wstring())},{"archive_sha256",inv.document["archive_sha256"]},{"scale_mm",mm}}}},{"selected",QJsonArray{selected}}};
            auto output=dir/"edited.3dm";writePreservedArchive(request,output);auto checked=inspectArchive(output);auto after=native(checked,uuid);require(after,"edited Hatch identity retained");require(payload(*after)==edit,"native typed current CV/weight/knot/radius/point/segment/domain fields applied exactly before placement");require(after->Plane().origin==ON_3dPoint(5,6,7),"current loop placement applied once");require(hatchLoopInventory(*after)[0].toObject()["curve"].toObject()["userdata"]==hatchLoopInventory(current)[0].toObject()["curve"].toObject()["userdata"],"native child metadata identities retained");++cases;
            auto twice=edit;auto twiceLoops=twice["loops"].toArray();auto twiceLoop=twiceLoops[0].toObject();twiceLoop["curve"]=deform(twiceLoop["curve"].toObject());twiceLoops[0]=twiceLoop;twice["loops"]=twiceLoops;auto twiceNative=current;applyHatchLoopFields(twiceNative,twice);require(payload(twiceNative)==twice,"successive affine loop edits remain valid");
            auto malformed=edit;malformed["schema_version"]=2;auto badFields=fields;badFields["loops"]=malformed;auto bad=selected;bad["hatch_fields"]=badFields;request["selected"]=QJsonArray{bad};auto sentinel=bytes(output);bool rejected=false;try{writePreservedArchive(request,output);}catch(const ExchangeError&){rejected=true;}require(rejected&&bytes(output)==sentinel,"unsupported loop schema rejects atomically");
        }require(bytes(path)==immutable,"typed loop source immutable");
    }
    auto inv=inspectArchive(std::filesystem::temp_directory_path()/"om9-hatch-loops-mm.3dm");
    for(auto v:inv.document["records"].toArray()){auto row=v.toObject();auto uuid=ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData());auto original=native(inv,uuid);auto shape=payload(*original);auto loop=shape["loops"].toArray()[0].toObject();if(loop["curve"].toObject()["class_name"]!="ON_PolylineCurve")continue;
        auto bowtie=loop["curve"].toObject();bowtie["points"]=QJsonArray{QJsonArray{-5,-5,0},QJsonArray{5,5,0},QJsonArray{-5,5,0},QJsonArray{5,-5,0},QJsonArray{-5,-5,0}};bowtie["parameters"]=QJsonArray{0,1,2,3,4};bowtie["domain"]=QJsonArray{0,4};loop["curve"]=bowtie;shape["loops"]=QJsonArray{loop};auto staged=*original;auto before=hatchLoopInventory(staged);bool rejected=false;try{applyHatchLoopFields(staged,shape);}catch(const ExchangeError&){rejected=true;}require(rejected&&hatchLoopInventory(staged)==before,"self-intersecting closed loop must reject without native mutation");
        auto overlap=bowtie;overlap["points"]=QJsonArray{QJsonArray{-5,-5,0},QJsonArray{5,-5,0},QJsonArray{0,-5,0},QJsonArray{5,5,0},QJsonArray{-5,5,0},QJsonArray{-5,-5,0}};overlap["parameters"]=QJsonArray{0,1,2,3,4,5};overlap["domain"]=QJsonArray{0,5};loop["curve"]=overlap;shape["loops"]=QJsonArray{loop};rejected=false;try{applyHatchLoopFields(staged,shape);}catch(const ExchangeError&){rejected=true;}require(rejected&&hatchLoopInventory(staged)==before,"distinct backtracking branches still reject atomically");
    }
    for(auto v:inv.document["records"].toArray()){auto row=v.toObject();auto uuid=ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData());auto original=native(inv,uuid);auto base=payload(*original);auto outer=base["loops"].toArray()[0];if(outer.toObject()["curve"].toObject()["class_name"]!="ON_NurbsCurve")continue;
        auto expectReject=[&](QJsonObject bad,const char* message){auto staged=*original;auto before=hatchLoopInventory(staged);bool failed=false;try{applyHatchLoopFields(staged,bad);}catch(const ExchangeError&){failed=true;}require(failed&&hatchLoopInventory(staged)==before,message);};
        auto hole=base;hole["loops"]=QJsonArray{outer,arcLoop(1,0,1)};auto withHole=*original;applyHatchLoopFields(withHole,hole);require(withHole.LoopCount()==2&&withHole.Loop(1)->Type()==ON_HatchLoop::ltInner&&withHole.Loop(1)->Curve()->ClassId()==&ON_CLASS_RTTI(ON_ArcCurve),"add native Arc inner boundary without converting existing NURBS");
        auto remove=payload(withHole);remove["loops"]=QJsonArray{remove["loops"].toArray()[0]};applyHatchLoopFields(withHole,remove);require(withHole.LoopCount()==1,"explicit native hole deletion");
        auto disjoint=base;disjoint["loops"]=QJsonArray{outer,arcLoop(0,20,1)};auto separated=*original;applyHatchLoopFields(separated,disjoint);require(separated.LoopCount()==2,"multiple disjoint native outer loops");
        auto island=base;island["loops"]=QJsonArray{outer,arcLoop(1,0,3),arcLoop(0,0,1)};auto islands=*original;applyHatchLoopFields(islands,island);require(islands.LoopCount()==3,"outer island inside native inner hole");
        auto outside=base;outside["loops"]=QJsonArray{outer,arcLoop(1,20,1)};expectReject(outside,"inner loop outside outer boundary rejects atomically");
        auto crossing=base;crossing["loops"]=QJsonArray{outer,arcLoop(1,4,3)};expectReject(crossing,"crossing native loop boundaries reject atomically");
        auto roles=base;roles["loops"]=QJsonArray{outer,arcLoop(0,0,1)};expectReject(roles,"nested duplicate outer role rejects atomically");
        auto empty=base;empty["loops"]=QJsonArray{};expectReject(empty,"removing every native loop rejects atomically");
        auto invalid=base;auto loops=invalid["loops"].toArray();auto loop=loops[0].toObject();auto curve=loop["curve"].toObject();auto cvs=curve["cvs"].toArray();auto cv=cvs[1].toArray();cv[2]=0;cvs[1]=cv;curve["cvs"]=cvs;loop["curve"]=curve;loops[0]=loop;invalid["loops"]=loops;expectReject(invalid,"zero native rational weight rejects atomically");
        invalid=base;loops=invalid["loops"].toArray();loop=loops[0].toObject();curve=loop["curve"].toObject();curve["class_name"]="ON_ArcCurve";loop["curve"]=curve;loops[0]=loop;invalid["loops"]=loops;expectReject(invalid,"source class cannot silently change representation");
    }
    require(cases==8,"eight typed native loop edits mm/cm");std::cout<<"native typed loop edit8 cases/topology PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
