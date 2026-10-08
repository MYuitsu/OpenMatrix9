// OM9-FILE-012: real coupled native transform, never a changed approximation alone.
#include "ThreeDmArchive.h"
#include "opennurbs_polyedgecurve.h"
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QFile>
#include <QDir>
#include <memory>
#include <iostream>
#include <stdexcept>
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
static QByteArray nativeBytes(const ON_Object& value){ON_Write3dmBufferArchive file(0,32*1024*1024,50,ON::Version());require(file.WriteObject(value),"Serialize exact child");return QByteArray(static_cast<const char*>(file.Buffer()),static_cast<qsizetype>(file.SizeOfArchive()));}
static std::unique_ptr<ON_CurveOnSurface> fixture(int dimension,bool optional,bool arc,int surfaceKind=0,bool rational=true,int degree=1){
    ON_Surface* surface=nullptr;
    if(surfaceKind==0){int count=degree+1;auto n=new ON_NurbsSurface(dimension,rational,count,count,count,count);
        for(int i=0;i<count;++i)for(int j=0;j<count;++j){double u=double(i)/degree,v=double(j)/degree,w=rational?1+.125*u+.25*v:1;ON_4dPoint cv(w*(10+u),w*(20+v),dimension==3?w*(30+.5*u*v):0,w);require(n->SetCV(i,j,cv),"Set native surface CV");}
        for(int d=0;d<2;++d)for(int i=0;i<n->KnotCount(d);++i)n->SetKnot(d,i,i<degree?0:1);surface=n;
    }else if(surfaceKind==1){auto p=new ON_PlaneSurface(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector::ZAxis));for(int d=0;d<2;++d){p->SetDomain(d,0,1);p->SetExtents(d,ON_Interval(-.25,1.25),false);}surface=p;
    }else if(surfaceKind==2){auto r=new ON_RevSurface;r->m_curve=new ON_LineCurve(ON_3dPoint(2,0,0),ON_3dPoint(2,0,1));r->m_curve->SetDomain(0,1);r->m_axis=ON_Line(ON_3dPoint::Origin,ON_3dPoint(0,0,1));r->m_angle=ON_Interval(0,2*ON_PI);r->m_t=ON_Interval(0,1);r->m_bTransposed=false;surface=r;
    }else if(surfaceKind==3){auto s=new ON_SumSurface;ON_LineCurve a(ON_3dPoint(10,20,30),ON_3dPoint(11,20,30)),b(ON_3dPoint(0,0,0),ON_3dPoint(0,1,.5));a.SetDomain(0,1);b.SetDomain(0,1);require(s->Create(a,b),"Create native SumSurface");surface=s;
    }else{surface=ON_Extrusion::Cylinder(ON_Cylinder(ON_Circle(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector::ZAxis),1),1),false,false);require(surface,"Create native Extrusion surface");}
    require(surface->IsValid(),"Valid named native surface fixture");surface->SetUserString(L"Child",L"Surface");
    auto uv=new ON_ArcCurve(ON_Circle(ON_Plane(ON_3dPoint(.5,.5,0),ON_3dVector::ZAxis),.25),11,23);uv->ChangeDimension(2);uv->SetUserString(L"Child",L"Parameter");
    ON_Curve* approximation=nullptr;
    if(optional){auto a=std::make_unique<ON_ArcCurve>(ON_Circle(ON_Plane(ON_3dPoint(10.5,20.5,dimension==3?30:0),ON_3dVector::ZAxis),.25),11,23);a->ChangeDimension(dimension);
        if(arc)approximation=a.release();else{auto n=new ON_NurbsCurve;require(a->GetNurbForm(*n)>0,"Native rational approximation fixture");approximation=n;}
        approximation->SetUserString(L"Child",L"Approximation");}
    auto result=std::make_unique<ON_CurveOnSurface>(uv,approximation,surface);result->SetUserString(L"Root",L"Coupled source");require(result->IsValid()&&result->IsClosed(),"Valid rational coupled fixture");return result;
}
static bool close(ON_3dPoint a,ON_3dPoint b){return a.DistanceTo(b)<=1e-10;}
class UnsafeChildData:public ON_UserData{public:UnsafeChildData(){m_userdata_uuid=ON_UuidFromString("00000000-0000-4000-8000-000000000158");m_application_uuid=ON_UuidFromString("00000000-0000-4000-8000-000000000159");m_userdata_copycount=1;}};
static void atomicRefusal(ON_CurveOnSurface& curve,const ON_Xform& transform,const char* message){
    auto before=nativeBytes(curve);auto c2=curve.m_c2,c3=curve.m_c3;auto surface=curve.m_s;auto data=curve.FirstUserData();bool rejected=false;
    try{OpenMatrix9Gui::ThreeDm::transformNativeGeometry(curve,transform);}catch(const OpenMatrix9Gui::ThreeDm::ExchangeError&){rejected=true;}
    require(rejected&&curve.m_c2==c2&&curve.m_c3==c3&&curve.m_s==surface&&curve.FirstUserData()==data&&nativeBytes(curve)==before,message);
}
int main(int argc,char** argv){try{ON::Begin();require(argc==2,"Native surface kind required");int surfaceKind=std::stoi(argv[1]);QDir evidence(QStringLiteral(OM9_SURFACE_TRANSFORM_EVIDENCE_DIR));require(QDir().mkpath(evidence.path()),"Evidence directory");QJsonArray cases;int count=0;
    // First RED must catch the real SDK dispatcher leaving m_c3 stale.
    for(int variant=0;variant<(surfaceKind==0?4:1);++variant)for(int dimension:{3,2})for(bool optional:{true,false})for(bool arc:{true,false}){
        if(dimension==2&&surfaceKind!=0)continue;
        if(!optional&&!arc)continue;
        for(int kind=0;kind<5;++kind){if(arc&&optional&&kind==4)continue;
            if((surfaceKind==1||surfaceKind==2)&&kind==4)continue;
            if(surfaceKind==4&&kind>=3)continue;
            bool rational=variant%2==0;int degree=variant<2?1:3;
            auto original=fixture(dimension,optional,arc,surfaceKind,rational,degree);auto current=fixture(dimension,optional,arc,surfaceKind,rational,degree);auto parameter=current->m_c2;auto parameterBytes=nativeBytes(*parameter);
            ON_Xform transform=ON_Xform::IdentityTransformation;
            if(kind==0)transform=ON_Xform::TranslationTransformation(ON_3dVector(2,-3,4));
            if(kind==1)transform.Rotation(.37,ON_3dVector::XAxis,ON_3dPoint(4,-2,8));
            if(kind==2)transform=ON_Xform::DiagonalTransformation(10);
            if(kind==3){transform[0][0]=-1;transform[2][3]=6;}
            if(kind==4){transform[0][0]=2;transform[1][1]=.5;transform[0][1]=.3;transform[2][0]=.2;transform[2][3]=7;}
            std::cout<<"surface="<<surfaceKind<<" variant="<<variant<<" dimension="<<dimension<<" optional="<<optional<<" arc="<<arc<<" transform="<<kind<<std::endl;
            OpenMatrix9Gui::ThreeDm::transformNativeGeometry(*current,transform);
            require(current->IsValid()&&current->IsClosed()&&current->m_c2==parameter&&nativeBytes(*current->m_c2)==parameterBytes,"Surface transform must preserve original UV child and exact archive fields");
            require(current->m_s->ClassId()==original->m_s->ClassId()&&current->Domain()==original->Domain(),"Coupled transform must retain native surface class and domain");
            for(double t:{11.,12.125,17.,22.75,23.}){
                require(close(current->PointAt(t),transform*original->PointAt(t)),"Composed surface curve must follow native affine transform");
                if(optional)require(close(current->m_c3->PointAt(t),transform*original->m_c3->PointAt(t)),"Optional approximation must follow the same surface transform");
            }
            const int targetDimension=dimension==2&&(transform[2][0]!=0||transform[2][1]!=0||transform[2][3]!=0)?3:dimension;
            require(current->m_s->Dimension()==targetDimension,"Native surface dimension promotion only when required");
            if(surfaceKind==0){auto n=ON_NurbsSurface::Cast(current->m_s),source=ON_NurbsSurface::Cast(original->m_s);require(n&&n->Order(0)==source->Order(0)&&n->Order(1)==source->Order(1),"Native rational surface shape preserved");
                for(int d=0;d<2;++d){require(n->Domain(d)==source->Domain(d)&&n->KnotCount(d)==source->KnotCount(d),"Native surface domains/knot counts exact");for(int i=0;i<n->KnotCount(d);++i)require(n->Knot(d,i)==source->Knot(d,i),"Native surface raw knots exact");}
                require(n->CVCount(0)==source->CVCount(0)&&n->CVCount(1)==source->CVCount(1)&&n->IsRational()==source->IsRational(),"Native surface CV counts/rationality exact");
                for(int i=0;i<n->CVCount(0);++i)for(int j=0;j<n->CVCount(1);++j){require(n->Weight(i,j)==source->Weight(i,j),"Native surface rational weights exact");auto raw=source->CV(i,j);auto expected=transform*ON_4dPoint(raw[0],raw[1],dimension==3?raw[2]:0,source->Weight(i,j));auto actual=n->CV(i,j);for(int k=0;k<targetDimension;++k)require(std::abs(actual[k]-expected[k])<=1e-12,"Native transformed homogeneous CV fields exact within declared arithmetic tolerance");}}
            if(optional)require(current->m_c3->ClassId()==original->m_c3->ClassId()&&current->m_c3->Dimension()==targetDimension&&current->m_c3->Domain()==original->m_c3->Domain(),"Approximation native class/domain retained with explicit dimension promotion");
            for(auto pair:{std::pair<const ON_Object*,const ON_Object*>(original.get(),current.get()),{original->m_s,current->m_s},{original->m_c2,current->m_c2},{original->m_c3,current->m_c3}}){if(!pair.first)continue;ON_ClassArray<ON_UserString> a,b;pair.first->GetUserStrings(a);pair.second->GetUserStrings(b);require(a.Count()==b.Count(),"Native child/root user string count exact");for(int i=0;i<a.Count();++i)require(a[i].m_key==b[i].m_key&&a[i].m_string_value==b[i].m_string_value,"Native child/root user string values exact");auto before=pair.first->GetUserData(ON_CLASS_ID(ON_UserStringList)),after=pair.second->GetUserData(ON_CLASS_ID(ON_UserStringList));require(before&&after&&after->m_userdata_xform==(pair.first==original->m_c2?before->m_userdata_xform:transform*before->m_userdata_xform),"UV metadata stays unchanged; model child/root userdata transforms exactly once");}
            ON_Write3dmBufferArchive raw(0,32*1024*1024,50,ON::Version());require(raw.WriteObject(*current),"Write coupled native class");ON_Read3dmBufferArchive reader(raw.SizeOfArchive(),raw.Buffer(),false,50,ON::Version());ON_Object* value=nullptr;require(reader.ReadObject(&value)==1,"Read transformed native class");std::unique_ptr<ON_Object> owned(value);auto decoded=ON_CurveOnSurface::Cast(value);require(decoded&&decoded->IsValid(),"Decoded transformed native surface curve");
            for(double t:{11.,12.125,17.,22.75,23.})require(close(decoded->PointAt(t),current->PointAt(t))&&(!optional||close(decoded->m_c3->PointAt(t),current->m_c3->PointAt(t))),"Transformed children survive native Rhino5 archive");
            cases.append(QJsonObject{{"dimension",dimension},{"variant",variant},{"optional",optional},{"approximation",optional?(arc?"ON_ArcCurve":"ON_NurbsCurve"):"absent"},{"transform",kind},{"exact_uv",true},{"coupled_samples",true},{"native_rational_weights",true},{"native_archive",true}});++count;
        }
    }
    // Unsupported native Arc shear must refuse without partially moving surface.
    auto denied=fixture(3,true,true,surfaceKind);ON_Xform shear=ON_Xform::IdentityTransformation;shear[0][1]=.5;
    atomicRefusal(*denied,shear,"Unrepresentable approximation transform must reject atomically");int refusals=1;
    for(int location=0;location<4;++location){auto source=fixture(3,true,false,surfaceKind);ON_Object* target=location==0?static_cast<ON_Object*>(source.get()):location==1?static_cast<ON_Object*>(source->m_c2):location==2?static_cast<ON_Object*>(source->m_c3):static_cast<ON_Object*>(source->m_s);auto unsafe=new UnsafeChildData;require(target->AttachUserData(unsafe),"Attach actual unsafe userdata");atomicRefusal(*source,ON_Xform::TranslationTransformation(ON_3dVector(2,3,4)),"Opaque native child data refuses before source mutation");require(target->GetUserData(unsafe->m_userdata_uuid)==unsafe,"Unsafe child data preserved");++refusals;}
    if(surfaceKind==0){auto source=fixture(2,true,true);delete source->m_c3;source->m_c3=source->m_c2;atomicRefusal(*source,shear,"Aliased owning child pointers must be refused before cloning");source->m_c3=nullptr;++refusals;
        auto deep=fixture(3,true,true);ON_Curve* uv=deep->m_c2;for(int i=0;i<65;++i){auto poly=new ON_PolyCurve;require(poly->Append(uv),"Create deep real native UV tree");uv=poly;}deep->m_c2=uv;atomicRefusal(*deep,ON_Xform::TranslationTransformation(ON_3dVector(1,2,3)),"Native tree depth limit refuses before mutation");++refusals;
        auto referenced=fixture(3,true,false);std::unique_ptr<ON_Curve> target(referenced->m_c2);auto reference=new ON_PolyEdgeCurve;require(reference->Create(target.get(),ON_UuidFromString("00000000-0000-4000-8000-000000000160")),"Create actual PolyEdge child reference");referenced->m_c2=reference;atomicRefusal(*referenced,ON_Xform::TranslationTransformation(ON_3dVector(1,2,3)),"Known PolyEdge child requires model resolution before transform");++refusals;
        auto huge=fixture(3,true,false);std::wstring text(17*1024*1024,L'a');require(huge->SetUserString(L"Large",text.c_str()),"Actual large native user string");auto metadata=huge->FirstUserData();auto size=metadata->SizeOf();auto uvBefore=huge->m_c2,approximationBefore=huge->m_c3;auto surfaceBefore=huge->m_s;auto surfaceBytes=nativeBytes(*surfaceBefore);bool rejected=false;
        try{OpenMatrix9Gui::ThreeDm::transformNativeGeometry(*huge,ON_Xform::TranslationTransformation(ON_3dVector(1,2,3)));}catch(const OpenMatrix9Gui::ThreeDm::ExchangeError&){rejected=true;}
        require(rejected&&huge->FirstUserData()==metadata&&metadata->SizeOf()==size&&huge->m_c2==uvBefore&&huge->m_c3==approximationBefore&&huge->m_s==surfaceBefore&&nativeBytes(*surfaceBefore)==surfaceBytes,"Native metadata budget must refuse before detached cloning or source mutation");++refusals;
    }
    if(surfaceKind==1||surfaceKind==2){auto source=fixture(3,false,true,surfaceKind);atomicRefusal(*source,shear,"Native surface parameter limitations refuse without mutation");++refusals;}
    if(surfaceKind==4){auto source=fixture(3,false,true,surfaceKind);auto mirror=ON_Xform::IdentityTransformation;mirror[0][0]=-1;atomicRefusal(*source,mirror,"Unverified extrusion reflected UV adapter refuses before mutation");++refusals;}
    QFile report(evidence.filePath(QString("surface-%1.json").arg(surfaceKind)));require(report.open(QIODevice::WriteOnly),"Report file");auto data=QJsonDocument(QJsonObject{{"ok",true},{"valid_cases",count},{"surface_kind",surfaceKind},{"atomic_arc_refusal",true},{"atomic_refusals",refusals},{"cases",cases}}).toJson();require(report.write(data)==data.size(),"Write report");std::cout<<"Coupled CurveOnSurface "<<count<<" native transform paths and "<<refusals<<" atomic refusals PASS\n";return 0;
}catch(const std::exception& error){std::cerr<<error.what()<<'\n';return 1;}}
