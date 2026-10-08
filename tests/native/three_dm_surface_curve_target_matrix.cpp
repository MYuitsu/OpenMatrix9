#include "ThreeDmInventory.h"
#include "ThreeDmCurveOnSurface.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* what){if(!value)throw ExchangeError(what);}
static ON_Surface* surface(int kind){
    if(kind==0||kind==5){int dimension=kind==5?2:3;auto n=new ON_NurbsSurface(dimension,true,4,4,4,4);for(int i=0;i<4;++i)for(int j=0;j<4;++j){double u=i/3.,v=j/3.,w=1+.125*u+.25*v;require(n->SetCV(i,j,ON_4dPoint(w*(10+u),w*(20+v),dimension==3?w*(30+.5*u*v):0,w)),"Rational surface grid");}for(int d=0;d<2;++d)for(int i=0;i<n->KnotCount(d);++i)n->SetKnot(d,i,i<3?0:1);return n;}
    if(kind==1){auto p=new ON_PlaneSurface(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector::ZAxis));for(int d=0;d<2;++d){p->SetDomain(d,0,1);p->SetExtents(d,ON_Interval(-.25,1.25),false);}return p;}
    if(kind==2){auto r=new ON_RevSurface;r->m_curve=new ON_LineCurve(ON_3dPoint(2,0,0),ON_3dPoint(2,0,1));r->m_curve->SetDomain(0,1);r->m_axis=ON_Line(ON_3dPoint::Origin,ON_3dPoint(0,0,1));r->m_angle=ON_Interval(0,2*ON_PI);r->m_t=ON_Interval(0,1);return r;}
    if(kind==3){auto s=new ON_SumSurface;ON_LineCurve a(ON_3dPoint(10,20,30),ON_3dPoint(11,20,30)),b(ON_3dPoint(0,0,0),ON_3dPoint(0,1,.5));a.SetDomain(0,1);b.SetDomain(0,1);require(s->Create(a,b),"Native sum surface");return s;}
    return ON_Extrusion::Cylinder(ON_Cylinder(ON_Circle(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector::ZAxis),1),1),false,false);
}
static ON_Curve* parameter(int kind){
    ON_Curve* result=nullptr;
    if(kind==0)result=new ON_LineCurve(ON_3dPoint(.25,.3,0),ON_3dPoint(.75,.7,0));
    if(kind==1||kind==2){auto arc=new ON_ArcCurve(ON_Circle(ON_Plane(ON_3dPoint(.5,.5,0),ON_3dVector::ZAxis),.25),11,23);arc->ChangeDimension(2);if(kind==1)result=arc;else{auto nurbs=new ON_NurbsCurve;require(arc->GetNurbForm(*nurbs)==2,"UV angular/rational curve");delete arc;result=nurbs;}}
    if(kind==3){ON_3dPointArray points;points.Append(ON_3dPoint(.25,.25,0));points.Append(ON_3dPoint(.5,.75,0));points.Append(ON_3dPoint(.75,.25,0));result=new ON_PolylineCurve(points);}
    if(kind==4){auto poly=new ON_PolyCurve;auto a=new ON_LineCurve(ON_3dPoint(.25,.25,0),ON_3dPoint(.5,.75,0));auto b=new ON_LineCurve(ON_3dPoint(.5,.75,0),ON_3dPoint(.75,.25,0));a->ChangeDimension(2);b->ChangeDimension(2);require(poly->Append(a)&&poly->Append(b),"Owning UV polycurve segments");result=poly;}
    require(result&&result->ChangeDimension(2)&&result->SetDomain(11,23)&&result->IsValid(),"Valid native UV parameter profile");return result;
}
static ON_NurbsSurface* expandedUVSurface(int kind){
    const int count=kind==2?4:2;auto s=new ON_NurbsSurface(2,kind==1,count,count,count,count);
    for(int d=0;d<2;++d)for(int i=0;i<s->KnotCount(d);++i)require(s->SetKnot(d,i,i<count-1?0:1),"Expanded UV unit knots");
    for(int i=0;i<count;++i)for(int j=0;j<count;++j){
        double u=double(i)/(count-1),v=double(j)/(count-1),x=.2+.6*u,y=.2+.6*v;
        if(kind==0)x=.2+.5*u+.1*v;
        if(kind==2)y+=.025*u*v;
        if(kind==1){double w=1+.25*u+.125*v;require(s->SetCV(i,j,ON_4dPoint(w*x,w*y,0,w)),"Expanded rational 2D UV grid");}
        else require(s->SetCV(i,j,ON_3dPoint(x,y,0)),"Expanded nonrational 2D UV grid");
    }
    require(s->IsValid()&&s->Dimension()==2,"Valid expanded 2D UV surface");return s;
}
int main(){try{ON::Begin();QDir out(QStringLiteral(OM9_COS_MATRIX_DIR));require(QDir().mkpath(out.path()),"Surface-curve matrix output");QJsonArray cases;
    for(int sk=0;sk<6;++sk)for(int ck=0;ck<5;++ck){auto s=surface(sk);require(s&&s->IsValid(),"Valid independent surface");s->SetUserString(L"Surface",L"Native target profile");auto uv=parameter(ck);uv->SetUserString(L"UV",L"Native target profile");auto root=new ON_CurveOnSurface(uv,nullptr,s);root->SetUserString(L"Root",L"Native target profile");require(root->IsValid(),"Coherent no-approximation native surface curve");QJsonArray samples;
        for(int i=0;i<=16;++i){auto q=uv->PointAt(11+12*i/16.);auto expected=s->PointAt(q.x,q.y);auto actual=root->PointAt(11+12*i/16.);require(actual.IsValid()&&actual.DistanceTo(expected)<=1e-12,"Independent UV/surface physical composition");samples.append(QJsonArray{expected.x,expected.y,expected.z});}
        auto sourceFields=curveOnSurfaceNativeFields(*root);ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);auto attributes=new ON_3dmObjectAttributes;attributes->m_name=L"Native no-C3 profile";auto owned=model.AddManagedModelGeometryComponent(root,attributes);require(!owned.IsEmpty(),"Own root and heap attributes");auto name=QString("surface-%1-uv-%2.3dm").arg(sk).arg(ck);auto path=std::filesystem::path(out.filePath(name).toStdWString());require(model.Write(path.c_str(),5,nullptr),"V5 matrix source");auto inventory=inspectArchive(path);require(inventory.document["issues"].toArray().isEmpty()&&inventory.document["records"].toArray().size()==1,"Complete native matrix source inventory");auto record=inventory.document["records"].toArray()[0].toObject();require(record["curve_on_surface_native"].toObject()==sourceFields,"All decoded UV/surface fields exact after native V5 serialization");
        cases.append(QJsonObject{{"fixture",name},{"source_sha256",inventory.document["archive_sha256"]},{"surface_class",s->ClassId()->ClassName()},{"parameter_class",uv->ClassId()->ClassName()},{"dimension",s->Dimension()},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"domain",QJsonArray{11,23}},{"samples",samples}}}},{"passed",true}});
    }
    QFile report(out.filePath("native-results.json"));require(report.open(QIODevice::WriteOnly),"Native matrix oracle");report.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",cases}}).toJson());
    QJsonArray nested;
    for(int sk=0;sk<6;++sk){
        auto uvSurface=new ON_NurbsSurface(2,false,2,2,2,2);
        for(int d=0;d<2;++d){uvSurface->SetKnot(d,0,0);uvSurface->SetKnot(d,1,1);}
        for(int i=0;i<2;++i)for(int j=0;j<2;++j)require(uvSurface->SetCV(i,j,ON_3dPoint(.2+.6*i,.2+.6*j,0)),"Independent nested UV surface grid");
        auto child=new ON_CurveOnSurface(parameter(0),nullptr,uvSurface);
        auto outerSurface=surface(sk);auto root=new ON_CurveOnSurface(child,nullptr,outerSurface);
        require(child->IsValid()&&child->Dimension()==2&&root->IsValid(),"Valid owning nested UV profile");
        child->SetUserString(L"Child",L"Nested native UV source");root->SetUserString(L"Root",L"Nested target control");
        QJsonArray samples;
        for(int i=0;i<=16;++i){double t=11+12*i/16.;auto raw=child->m_c2->PointAt(t);auto uv=uvSurface->PointAt(raw.x,raw.y);auto expected=outerSurface->PointAt(uv.x,uv.y);require(root->PointAt(t).DistanceTo(expected)<=1e-12,"Independent nested UV interior composition");samples.append(QJsonArray{expected.x,expected.y,expected.z});}
        auto fields=curveOnSurfaceNativeFields(*root);ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
        auto attributes=new ON_3dmObjectAttributes;attributes->m_name=L"Nested UV target control";require(!model.AddManagedModelGeometryComponent(root,attributes).IsEmpty(),"Independent nested root ownership");
        auto name=QString("nested-surface-%1.3dm").arg(sk);auto path=std::filesystem::path(out.filePath(name).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Native nested V5 fixture write");
        auto inventory=inspectArchive(path);require(inventory.document["issues"].toArray().isEmpty()&&inventory.document["records"].toArray().size()==1,"Nested source graph inventory");auto record=inventory.document["records"].toArray()[0].toObject();require(record["curve_on_surface_native"].toObject()==fields,"Decoded owning nested children retain exact native fields");
        nested.append(QJsonObject{{"fixture",name},{"source_sha256",inventory.document["archive_sha256"]},{"surface_class",outerSurface->ClassId()->ClassName()},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"domain",QJsonArray{11,23}},{"samples",samples}}}},{"passed",true}});
    }
    QFile nestedReport(out.filePath("nested-native-results.json"));require(nestedReport.open(QIODevice::WriteOnly),"Nested native target oracle");nestedReport.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",nested}}).toJson());
    QJsonArray expanded;
    for(int sk=0;sk<6;++sk)for(int mapping=0;mapping<3;++mapping)for(int ck=0;ck<5;++ck){
        auto uv=expandedUVSurface(mapping);auto raw=parameter(ck);auto child=new ON_CurveOnSurface(raw,nullptr,uv);auto outer=surface(sk);auto root=new ON_CurveOnSurface(child,nullptr,outer);
        require(child->IsValid()&&child->Dimension()==2&&root->IsValid(),"Valid expanded nested UV composition");
        child->SetUserString(L"Child",L"Expanded native UV control");root->SetUserString(L"Root",L"Target evidence still pending");
        QJsonArray samples;
        for(int i=0;i<=16;++i){double t=11+12*i/16.;auto p=raw->PointAt(t);auto q=uv->PointAt(p.x,p.y);require(q.x>=0&&q.x<=1&&q.y>=0&&q.y<=1,"Expanded UV remains in outer domains");auto expected=outer->PointAt(q.x,q.y);require(root->PointAt(t).DistanceTo(expected)<=1e-12,"Independent expanded UV interior composition");samples.append(QJsonArray{expected.x,expected.y,expected.z});}
        auto fields=curveOnSurfaceNativeFields(*root);ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);auto attributes=new ON_3dmObjectAttributes;attributes->m_name=L"Expanded nested UV target control";require(!model.AddManagedModelGeometryComponent(root,attributes).IsEmpty(),"Owning expanded root");
        auto name=QString("expanded-surface-%1-map-%2-uv-%3.3dm").arg(sk).arg(mapping).arg(ck);auto path=std::filesystem::path(out.filePath(name).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Independent expanded V5 fixture write");auto inventory=inspectArchive(path);require(inventory.document["issues"].toArray().isEmpty()&&inventory.document["records"].toArray().size()==1,"Expanded source inventory");auto record=inventory.document["records"].toArray()[0].toObject();require(record["curve_on_surface_native"].toObject()==fields,"All expanded child fields exact after native serialization");
        expanded.append(QJsonObject{{"fixture",name},{"source_sha256",inventory.document["archive_sha256"]},{"surface_class",outer->ClassId()->ClassName()},{"parameter_class",raw->ClassId()->ClassName()},{"uv_mapping",mapping==0?"sheared-bilinear":mapping==1?"rational-bilinear":"nonrational-bicubic"},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"domain",QJsonArray{11,23}},{"samples",samples}}}},{"passed",true}});
    }
    QFile expandedReport(out.filePath("expanded-nested-native-results.json"));require(expandedReport.open(QIODevice::WriteOnly),"Expanded nested oracle");expandedReport.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",expanded}}).toJson());
    std::cout<<"Surface-curve target matrix PASS30 coherent,6 original nested and90 expanded native profiles\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
