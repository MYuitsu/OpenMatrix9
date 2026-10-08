#include "ThreeDmMerge.h"
#include "ThreeDmCurveOnSurface.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* message){if(!value)throw ExchangeError(message);}
static QByteArray bytes(const std::filesystem::path& path){QFile file(QString::fromStdWString(path.wstring()));require(file.open(QIODevice::ReadOnly),"Read surface-curve proof");return file.readAll();}
int main(){try{ON::Begin();QDir root(QStringLiteral(OM9_COS_V5_FIXTURES)),out(QStringLiteral(OM9_COS_V5_EVIDENCE));require(QDir().mkpath(out.path()),"Surface-curve evidence folder");QJsonArray cases;int checks=0;
    for(int kind=0;kind<4;++kind){auto source=std::filesystem::path(root.filePath(QString("coherent-reference-%1.3dm").arg(kind)).toStdWString());auto immutable=bytes(source);auto input=inspectArchive(source);QJsonObject record;for(auto value:input.document["records"].toArray())if(value.toObject()["class_name"]=="ON_CurveOnSurface")record=value.toObject();require(!record.isEmpty(),"Exact original native CurveOnSurface source");
        QJsonObject archive{{"namespace","11223344-1122-3344-5566-112233445566"},{"snapshot",QString::fromStdWString(source.wstring())},{"archive_sha256",input.document["archive_sha256"]},{"scale_mm",1}};
        for(bool moved:{false,true}){ON_Xform transform=ON_Xform::TranslationTransformation(ON_3dVector(5,6,7));QJsonArray matrix;for(int i=0;i<16;++i)matrix.append(transform[i/4][i%4]);
            QJsonObject selected{{"host_id","surface-curve"},{"namespace",archive["namespace"]},{"source_uuid",record["source_uuid"]},{"action",moved?"transform":"unchanged"}};if(moved)selected["geometry_matrix"]=matrix;
            QJsonObject request{{"schema_version",1},{"sources",QJsonArray{archive}},{"selected",QJsonArray{selected}}};auto name=QString("reference-%1-moved-%2.3dm").arg(kind).arg(moved?1:0);auto target=std::filesystem::path(out.filePath(name).toStdWString());QFile sentinel(QString::fromStdWString(target.wstring()));require(sentinel.open(QIODevice::WriteOnly),"Protected output sentinel");sentinel.write("keep destination");sentinel.close();bool rejected=false;std::string reason;
            try{writePreservedArchive(request,target);}catch(const ExchangeError& e){rejected=true;reason=e.what();}
            if(kind>0){require(rejected&&reason.find("Rhino 5")!=std::string::npos&&reason.find("m_c3")!=std::string::npos&&bytes(target)=="keep destination","Actual Rhino5 optional-C3 incompatibility is refused atomically");}
            else{if(rejected)throw ExchangeError("Verified no-C3 plane/line profile should export: "+reason);auto result=inspectArchive(target);auto component=ON_ModelGeometryComponent::Cast(result.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(record["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto native=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(native&&!native->m_c3&&result.document["records"].toArray().size()==1,"Exact source class and optional absence remain after selected V5 export");
                QJsonArray samples;for(int i=0;i<=16;++i){double s=i/16.;const auto actual=native->PointAt(31+i);ON_3dPoint expected(s,s,0);if(moved)expected=transform*expected;require(actual.DistanceTo(expected)<1e-12,"Exact physical sample after native coupled transform");samples.append(QJsonArray{actual.x,actual.y,actual.z});}
                cases.append(QJsonObject{{"fixture",name},{"uuid",record["source_uuid"]},{"domain",QJsonArray{31,47}},{"samples",samples},{"source_sha256",result.document["archive_sha256"]},{"passed",true}});
            }require(bytes(source)==immutable,"All target compatibility decisions leave immutable source untouched");++checks;
        }
        if(kind>0){auto target=std::filesystem::path(out.filePath(QString("direct-%1.3dm").arg(kind)).toStdWString());QFile sentinel(QString::fromStdWString(target.wstring()));require(sentinel.open(QIODevice::WriteOnly),"Direct writer sentinel");sentinel.write("keep destination");sentinel.close();bool rejected=false;try{writeModelRhino5(*input.nativeModel,target);}catch(const ExchangeError& e){rejected=std::string(e.what()).find("m_c3")!=std::string::npos;}require(rejected&&bytes(target)=="keep destination","Direct File/CMD native writer also checks target compatibility before opening destination");++checks;}
    }
    QDir matrixRoot(QStringLiteral(OM9_COS_V5_MATRIX));auto proof=QJsonDocument::fromJson(bytes(std::filesystem::path(matrixRoot.filePath("oracle.json").toStdWString()))).object();
    require(proof["cases"].toArray().size()==30,"Thirty actual Rhino5 UV/surface profile fixtures");
    for(auto value:proof["cases"].toArray()){auto row=value.toObject();auto source=std::filesystem::path(matrixRoot.filePath(row["fixture"].toString()).toStdWString());auto immutable=bytes(source);auto input=inspectArchive(source);require(input.document["archive_sha256"]==row["source_sha256"],"Matrix source bound to actual Rhino5 oracle");auto record=input.document["records"].toArray()[0].toObject();
        for(bool moved:{false,true}){auto transform=ON_Xform::TranslationTransformation(ON_3dVector(5,6,7));QJsonArray matrix;for(int i=0;i<16;++i)matrix.append(transform[i/4][i%4]);
            QJsonObject archive{{"namespace","11223344-1122-3344-5566-112233445566"},{"snapshot",QString::fromStdWString(source.wstring())},{"archive_sha256",input.document["archive_sha256"]},{"scale_mm",1}};
            QJsonObject selected{{"host_id","matrix-curve"},{"namespace",archive["namespace"]},{"source_uuid",record["source_uuid"]},{"action",moved?"transform":"unchanged"}};if(moved)selected["geometry_matrix"]=matrix;
            auto target=std::filesystem::path(out.filePath(QString("matrix-%1-moved-%2.3dm").arg(row["fixture"].toString()).arg(moved?1:0)).toStdWString());
            writePreservedArchive(QJsonObject{{"schema_version",1},{"sources",QJsonArray{archive}},{"selected",QJsonArray{selected}}},target);
            auto result=inspectArchive(target);require(result.document["records"].toArray().size()==1,"Matrix selected export root count");auto exported=result.document["records"].toArray()[0].toObject();auto oldFields=record["curve_on_surface_native"].toObject(),newFields=exported["curve_on_surface_native"].toObject();
            require(exported["source_uuid"]==record["source_uuid"]&&exported["class_name"]==record["class_name"]&&newFields["parameter"]==oldFields["parameter"]&&newFields["approximation"].isNull(),"Matrix native class UUID UV and optional absence");
            if(!moved)require(newFields==oldFields,"Unchanged matrix export retains every native child field");
            auto component=ON_ModelGeometryComponent::Cast(result.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(record["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto native=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(native,"Native matrix output curve remains present");
            auto expected=row["objects"].toArray()[0].toObject();auto domain=expected["domain"].toArray();auto samples=expected["samples"].toArray();
            for(int i=0;i<17;++i){auto xyz=samples[i].toArray();ON_3dPoint point(xyz[0].toDouble(),xyz[1].toDouble(),xyz[2].toDouble());if(moved)point=transform*point;auto actual=native->PointAt(domain[0].toDouble()+(domain[1].toDouble()-domain[0].toDouble())*i/16.);require(actual.DistanceTo(point)<1e-9,"Matrix interior samples after coupled selected export placement");}
            require(bytes(source)==immutable,"Frozen matrix source remains immutable");++checks;
        }
    }
    QDir nestedRoot(QStringLiteral(OM9_COS_V5_NESTED));auto oraclePath=std::filesystem::path(nestedRoot.filePath("oracle.json").toStdWString());
    auto nestedProof=QJsonDocument::fromJson(bytes(oraclePath)).object();auto actualProof=QJsonDocument::fromJson(bytes(std::filesystem::path(nestedRoot.filePath("rhino5-exchange-profile-results.json").toStdWString()))).object();
    require(actualProof["ok"].toBool()&&actualProof["active_document_untouched"].toBool()&&nestedProof["cases"].toArray().size()==6,"Actual Rhino5 nested read/write evidence prerequisite");
    QJsonArray nestedWriterCases;
    for(auto value:nestedProof["cases"].toArray()){
        auto row=value.toObject();auto source=std::filesystem::path(nestedRoot.filePath(row["fixture"].toString()).toStdWString());auto immutable=bytes(source);auto input=inspectArchive(source);auto record=input.document["records"].toArray()[0].toObject();
        require(input.document["archive_sha256"]==row["source_sha256"],"Nested source oracle binding");
        for(bool moved:{false,true}){
            auto transform=ON_Xform::TranslationTransformation(ON_3dVector(5,6,7));QJsonArray matrix;for(int i=0;i<16;++i)matrix.append(transform[i/4][i%4]);
            QJsonObject archive{{"namespace","11223344-1122-3344-5566-112233445566"},{"snapshot",QString::fromStdWString(source.wstring())},{"archive_sha256",input.document["archive_sha256"]},{"scale_mm",1}};
            QJsonObject selected{{"host_id","nested-curve"},{"namespace",archive["namespace"]},{"source_uuid",record["source_uuid"]},{"action",moved?"transform":"unchanged"}};if(moved)selected["geometry_matrix"]=matrix;
            auto target=std::filesystem::path(out.filePath(QString("nested-%1-moved-%2.3dm").arg(row["fixture"].toString()).arg(moved?1:0)).toStdWString());
            writePreservedArchive(QJsonObject{{"schema_version",1},{"sources",QJsonArray{archive}},{"selected",QJsonArray{selected}}},target);
            auto result=inspectArchive(target);auto exported=result.document["records"].toArray()[0].toObject();auto oldFields=record["curve_on_surface_native"].toObject(),newFields=exported["curve_on_surface_native"].toObject();
            require(result.document["source_version"]==50&&result.document["records"].toArray().size()==1&&exported["source_uuid"]==record["source_uuid"]&&newFields["parameter"]==oldFields["parameter"]&&newFields["approximation"].isNull(),"Nested selected writer preserves native identities and complete UV tree");
            if(!moved)require(newFields==oldFields,"Unchanged nested writer preserves all decoded native fields");
            auto component=ON_ModelGeometryComponent::Cast(result.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(record["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto curve=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(curve,"Nested output stays exact native class");
            auto expected=row["objects"].toArray()[0].toObject();auto samples=expected["samples"].toArray();
            QJsonArray physicalSamples;
            for(int i=0;i<17;++i){auto xyz=samples[i].toArray();ON_3dPoint point(xyz[0].toDouble(),xyz[1].toDouble(),xyz[2].toDouble());if(moved)point=transform*point;require(curve->PointAt(11+12*i/16.).DistanceTo(point)<=1e-9,"Nested writer interior physical samples");physicalSamples.append(QJsonArray{point.x,point.y,point.z});}
            nestedWriterCases.append(QJsonObject{{"fixture",QString::fromStdWString(target.filename().wstring())},{"source_sha256",result.document["archive_sha256"]},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"domain",QJsonArray{11,23}},{"samples",physicalSamples}}}},{"passed",true}});
            require(bytes(source)==immutable,"Nested writer leaves source immutable");++checks;
        }
        // Untested nested properties must continue to refuse before touching a target.
        auto component=ON_ModelGeometryComponent::Cast(input.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(record["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto original=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(original,"Nested rejection control");
        for(int kind=0;kind<8;++kind){std::unique_ptr<ON_CurveOnSurface> copy(static_cast<ON_CurveOnSurface*>(original->DuplicateCurve()));auto child=ON_CurveOnSurface::Cast(copy->m_c2);auto surface=ON_NurbsSurface::Cast(child->m_s);require(child&&surface,"Exact nested negative control types");
            if(kind==0)child->m_c3=child->m_c2->DuplicateCurve();
            if(kind==1){delete child->m_c2;auto arc=new ON_ArcCurve(ON_Circle(ON_Plane::World_xy,.1));arc->ChangeDimension(2);child->m_c2=arc;}
            if(kind==2){auto unsupported=new ON_NurbsSurface(2,false,3,3,3,3);for(int d=0;d<2;++d)for(int i=0;i<4;++i)unsupported->SetKnot(d,i,i<2?0:1);for(int i=0;i<3;++i)for(int j=0;j<3;++j)unsupported->SetCV(i,j,ON_3dPoint(.2+.3*i,.2+.3*j,0));delete child->m_s;child->m_s=unsupported;}
            if(kind==3){auto old=child->m_c2;child->m_c2=new ON_CurveOnSurface(old,nullptr,surface->DuplicateSurface());}
            if(kind==4)require(surface->SetCV(1,0,ON_3dPoint(copy->m_s->Domain(0).Max()+1,.3,0)),"Outside outer domain nested UV control");
            if(kind==5){require(surface->MakeRational(),"Negative UV surface weight control");for(int i=0;i<2;++i)for(int j=0;j<2;++j)require(surface->SetWeight(i,j,-1),"Negative UV weight");}
            if(kind==6){auto negative=new ON_NurbsCurve(2,true,2,2);negative->SetKnot(0,0);negative->SetKnot(1,1);negative->SetCV(0,ON_4dPoint(-.25,-.3,0,-1));negative->SetCV(1,ON_4dPoint(-.75,-.7,0,-1));negative->SetDomain(11,23);delete child->m_c2;child->m_c2=negative;}
            if(kind==7){auto unsupported=new ON_NurbsSurface(2,true,4,4,4,4);for(int d=0;d<2;++d)for(int i=0;i<6;++i)unsupported->SetKnot(d,i,i<3?0:1);for(int i=0;i<4;++i)for(int j=0;j<4;++j)unsupported->SetCV(i,j,ON_4dPoint(.2+.2*i,.2+.2*j,0,1));delete child->m_s;child->m_s=unsupported;}
            require(copy->IsValid(),"Negative nested target control is native-valid, not a malformed source");
            bool refused=false;try{validateCurveOnSurfaceRhino5(*copy);}catch(const ExchangeError&){refused=true;}require(refused,"Unverified nested property remains refused");++checks;
            ONX_Model control;require(!control.AddModelGeometryComponent(copy.get(),nullptr).IsEmpty(),"Owning native-valid negative writer control");
            auto protectedTarget=std::filesystem::path(out.filePath(QString("nested-refused-%1-%2.3dm").arg(row["fixture"].toString()).arg(kind)).toStdWString());
            QFile sentinel(QString::fromStdWString(protectedTarget.wstring()));require(sentinel.open(QIODevice::WriteOnly),"Negative nested protected target");sentinel.write("keep destination");sentinel.close();
            refused=false;try{writeModelRhino5(control,protectedTarget);}catch(const ExchangeError& error){refused=std::string(error.what()).find("CurveOnSurface")!=std::string::npos;}
            require(refused&&bytes(protectedTarget)=="keep destination","Direct native writer refuses unverified nested property atomically");++checks;
        }
    }
    QFile nestedWriterOracle(out.filePath("nested-writer-oracle.json"));require(nestedWriterOracle.open(QIODevice::WriteOnly),"Nested OM9 writer independent physical oracle");nestedWriterOracle.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",nestedWriterCases}}).toJson());
    QDir expandedRoot(QStringLiteral(OM9_COS_V5_EXPANDED));auto expandedProof=QJsonDocument::fromJson(bytes(std::filesystem::path(expandedRoot.filePath("oracle.json").toStdWString()))).object();auto expandedActual=QJsonDocument::fromJson(bytes(std::filesystem::path(expandedRoot.filePath("rhino5-exchange-profile-results.json").toStdWString()))).object();
    require(expandedActual["ok"].toBool()&&expandedActual["active_document_untouched"].toBool()&&expandedActual["cases"].toArray().size()==90&&expandedProof["cases"].toArray().size()==90,"Actual90 expanded target evidence prerequisite");QJsonArray expandedWriterCases;
    for(auto value:expandedProof["cases"].toArray()){
        auto row=value.toObject();auto source=std::filesystem::path(expandedRoot.filePath(row["fixture"].toString()).toStdWString());auto immutable=bytes(source);auto input=inspectArchive(source);auto record=input.document["records"].toArray()[0].toObject();require(input.document["archive_sha256"]==row["source_sha256"],"Expanded source binds independent oracle");
        for(bool moved:{false,true}){
            auto transform=ON_Xform::TranslationTransformation(ON_3dVector(5,6,7));QJsonArray matrix;for(int i=0;i<16;++i)matrix.append(transform[i/4][i%4]);
            QJsonObject archive{{"namespace","11223344-1122-3344-5566-112233445566"},{"snapshot",QString::fromStdWString(source.wstring())},{"archive_sha256",input.document["archive_sha256"]},{"scale_mm",1}};
            QJsonObject selected{{"host_id","expanded-curve"},{"namespace",archive["namespace"]},{"source_uuid",record["source_uuid"]},{"action",moved?"transform":"unchanged"}};if(moved)selected["geometry_matrix"]=matrix;
            auto name=QString("writer-%1-moved-%2.3dm").arg(row["fixture"].toString()).arg(moved?1:0);auto target=std::filesystem::path(out.filePath(name).toStdWString());
            writePreservedArchive(QJsonObject{{"schema_version",1},{"sources",QJsonArray{archive}},{"selected",QJsonArray{selected}}},target);auto output=inspectArchive(target);auto exported=output.document["records"].toArray()[0].toObject();auto fields=exported["curve_on_surface_native"].toObject(),original=record["curve_on_surface_native"].toObject();
            require(output.document["source_version"]==50&&output.document["records"].toArray().size()==1&&exported["source_uuid"]==record["source_uuid"]&&exported["class_name"]=="ON_CurveOnSurface"&&fields["parameter"]==original["parameter"]&&fields["approximation"].isNull(),"Expanded V5 writer preserves native UUID and entire UV subtree");if(!moved)require(fields==original,"Unchanged expanded V5 retains every decoded field");
            auto component=ON_ModelGeometryComponent::Cast(output.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(record["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto curve=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(curve,"Expanded native output class");QJsonArray physical;
            auto samples=row["objects"].toArray()[0].toObject()["samples"].toArray();require(samples.size()==17,"Expanded independent sample count");for(int i=0;i<17;++i){auto xyz=samples[i].toArray();ON_3dPoint point(xyz[0].toDouble(),xyz[1].toDouble(),xyz[2].toDouble());if(moved)point=transform*point;require(curve->PointAt(11+12*i/16.).DistanceTo(point)<=1e-9,"Expanded writer independent interior/end samples");physical.append(QJsonArray{point.x,point.y,point.z});}
            expandedWriterCases.append(QJsonObject{{"fixture",name},{"source_sha256",output.document["archive_sha256"]},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"domain",QJsonArray{11,23}},{"samples",physical}}}},{"passed",true}});require(bytes(source)==immutable,"Expanded writer source remains immutable");++checks;
        }
    }
    QFile expandedWriterOracle(out.filePath("expanded-writer-oracle.json"));require(expandedWriterOracle.open(QIODevice::WriteOnly),"Expanded writer target oracle");expandedWriterOracle.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",expandedWriterCases}}).toJson());
    QFile report(out.filePath("native-results.json"));require(report.open(QIODevice::WriteOnly),"V5 profile report");report.write(QJsonDocument(QJsonObject{{"ok",true},{"checks",checks},{"cases",cases}}).toJson());std::cout<<"CurveOnSurface scoped V5 profile PASS "<<checks<<" checks\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
