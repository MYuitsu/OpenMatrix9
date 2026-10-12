#include "ThreeDmGeometry.h"
#include <QJsonArray>
#include <QJsonObject>
#include <QJsonDocument>
#include <fstream>
#include <iostream>
#include <filesystem>
#include <Standard_Failure.hxx>
#include <algorithm>
using namespace OpenMatrix9Gui::ThreeDm;
static QJsonArray point(const ON_3dPoint& p){return {p.x,p.y,p.z};}
int main(int argc,char**argv){try{
    if(argc!=3)throw std::runtime_error("source.3dm output.json required");
    ON::Begin();ONX_Model model;if(!model.Read(std::filesystem::u8path(argv[1]).c_str()))throw std::runtime_error("source read failed");
    QJsonArray rows;ONX_ModelComponentIterator it(model,ON_ModelComponent::Type::ModelGeometry);
    for(auto ref=it.FirstComponentReference();!ref.IsEmpty();ref=it.NextComponentReference()){
        auto* component=ON_ModelGeometryComponent::Cast(ref.ModelComponent());auto* geometry=component?component->Geometry(nullptr):nullptr;if(!geometry)continue;
        char uuid[37];ON_UuidToString(component->Id(),uuid);
        QJsonObject row{{"uuid",uuid},{"class",geometry->ClassId()->ClassName()}};
        if(auto* c=ON_Curve::Cast(geometry)){
            auto imported=curve3d(*c);QJsonArray source,host;double max=0;ON_NurbsCurve n;int form=c->GetNurbForm(n);
            for(int i=0;i<=256;++i){double t=c->Domain().ParameterAt(i/256.0);auto p=c->PointAt(t);auto q=imported->Value(t);source.append(point(p));host.append(QJsonArray{q.X(),q.Y(),q.Z()});max=std::max(max,p.DistanceTo(ON_3dPoint(q.X(),q.Y(),q.Z())));}
            row["form"]=form;row["degree"]=n.Degree();row["cv_count"]=n.CVCount();row["periodic"]=n.IsPeriodic();row["rational"]=n.IsRational();row["max_same_parameter_mm"]=max;row["native_samples"]=source;row["host_samples"]=host;
        }else if(auto* b=ON_Brep::Cast(geometry)){
            QJsonArray surfaces;
            for(int si=0;si<b->m_S.Count();++si){auto* s=b->m_S[si];auto imported=importSurface(*s);double max=0;
                for(int i=0;i<=12;++i)for(int j=0;j<=12;++j){auto u=s->Domain(0).ParameterAt(i/12.0),v=s->Domain(1).ParameterAt(j/12.0);auto uv=importedSurfaceParameters(*s,u,v);auto p=s->PointAt(u,v);auto q=imported->Value(uv[0],uv[1]);max=std::max(max,p.DistanceTo(ON_3dPoint(q.X(),q.Y(),q.Z())));}
                surfaces.append(QJsonObject{{"index",si},{"class",s->ClassId()->ClassName()},{"max_same_parameter_mm",max}});
            }row["surfaces"]=surfaces;
        }
        rows.append(row);
    }
    std::ofstream(argv[2])<<QJsonDocument(rows).toJson().constData();std::cout<<"Read-only native/OCCT display geometry diagnostic rows: "<<rows.size()<<'\n';return 0;
}catch(const Standard_Failure&e){std::cerr<<e.GetMessageString()<<'\n';return 2;}catch(const std::exception&e){std::cerr<<e.what()<<'\n';return 1;}}
