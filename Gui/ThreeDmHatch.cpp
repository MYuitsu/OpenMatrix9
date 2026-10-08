#include "ThreeDmHatch.h"
#include <QJsonArray>
#include <QCryptographicHash>
#include <QJsonDocument>
#include <QUuid>
#include <cmath>
#include <memory>
#include <vector>
namespace OpenMatrix9Gui::ThreeDm {
void validateHatch(const ON_Hatch& hatch){
    if(!hatch.IsValid()||!hatch.Plane().IsValid()||hatch.LoopCount()<=0||!hatch.BasePoint2d().IsValid()||!std::isfinite(hatch.PatternRotation())||!std::isfinite(hatch.PatternScale())||hatch.PatternScale()<=0)throw ExchangeError("Invalid native hatch fields");
    for(int i=0;i<hatch.LoopCount();++i){auto loop=hatch.Loop(i);if(!loop||!loop->IsValid()||!loop->Curve()||loop->Curve()->Dimension()!=2||!loop->Curve()->IsClosed())throw ExchangeError("Invalid native closed 2D hatch loop");}
}
QJsonObject hatchFacts(const ON_Hatch& hatch){
    validateHatch(hatch);QJsonArray loops,plane;auto frame=hatch.Plane();for(auto vector:{frame.origin,ON_3dPoint(frame.xaxis),ON_3dPoint(frame.yaxis),ON_3dPoint(frame.zaxis)})for(int i=0;i<3;++i)plane.append(vector[i]);for(int i=0;i<4;++i)plane.append(frame.plane_equation[i]);
    for(int i=0;i<hatch.LoopCount();++i){auto loop=hatch.Loop(i);ON_Write3dmBufferArchive buffer(0,512ULL*1024*1024,50,ON::Version());if(!buffer.WriteObject(*loop->Curve()))throw ExchangeError("Cannot serialize native hatch loop");auto hash=QCryptographicHash::hash(QByteArrayView(static_cast<const char*>(buffer.Buffer()),static_cast<qsizetype>(buffer.SizeOfArchive())),QCryptographicHash::Sha256).toHex();loops.append(QJsonObject{{"type",static_cast<int>(loop->Type())},{"class_name",loop->Curve()->ClassId()->ClassName()},{"curve_sha256",QString::fromLatin1(hash)}});}
    ON_BoundingBox box;if(!hatch.GetTightBoundingBox(box)||!box.IsValid())throw ExchangeError("Cannot compute native hatch bounds");
    auto base=hatch.BasePoint2d();return QJsonObject{{"plane",plane},{"base_point",QJsonArray{base.x,base.y}},{"rotation",hatch.PatternRotation()},{"scale",hatch.PatternScale()},{"gradient_type",static_cast<int>(hatch.GetGradientType())},{"loops",loops},{"bounds",QJsonArray{box.m_min.x,box.m_min.y,box.m_min.z,box.m_max.x,box.m_max.y,box.m_max.z}}};
}
QJsonObject hatchPatternFacts(const ON_HatchPattern& pattern){
    if(!pattern.IsValid())throw ExchangeError("Invalid native hatch pattern");QJsonArray lines;
    for(int i=0;i<pattern.HatchLineCount();++i){auto line=pattern.HatchLine(i);if(!line||!line->IsValid())throw ExchangeError("Invalid native hatch pattern line");QJsonArray dashes;for(int j=0;j<line->DashCount();++j){if(!std::isfinite(line->Dash(j)))throw ExchangeError("Invalid nonfinite native hatch dash");dashes.append(line->Dash(j));}auto base=line->Base();auto offset=line->Offset();lines.append(QJsonObject{{"angle",line->AngleRadians()},{"base",QJsonArray{base.x,base.y}},{"offset",QJsonArray{offset.x,offset.y}},{"dashes",dashes}});}
    ON_String description(pattern.Description());return QJsonObject{{"fill_type",static_cast<int>(pattern.FillType())},{"description",QString::fromUtf8(description.Array())},{"lines",lines}};
}
static QString uuid(ON_UUID value){char text[37]{};ON_UuidToString(value,text);return QString::fromLatin1(text);}
static QString utf8(const ON_wString& value){ON_String text(value);return QString::fromUtf8(text.Array());}
QJsonObject hatchPatternRecord(const ON_HatchPattern& pattern){
    QJsonArray strings,data;ON_ClassArray<ON_UserString> values;pattern.GetUserStrings(values);
    for(int i=0;i<values.Count();++i)strings.append(QJsonObject{{"key",utf8(values[i].m_key)},{"value",utf8(values[i].m_string_value)}});
    for(auto value=pattern.FirstUserData();value;value=value->Next())data.append(QJsonObject{{"class_uuid",uuid(value->ClassId()->Uuid())},{"class_name",value->ClassId()->ClassName()},{"capability","retained"}});
    return QJsonObject{{"source_uuid",uuid(pattern.Id())},{"class_uuid",uuid(pattern.ClassId()->Uuid())},{"class_name",pattern.ClassId()->ClassName()},{"component_type",utf8(ON_ModelComponent::ComponentTypeToString(ON_ModelComponent::Type::HatchPattern))},{"name",utf8(pattern.Name())},{"role","top-level"},{"dependencies",QJsonArray{}},{"capability","retained"},{"user_strings",strings},{"userdata",data},{"attribute_user_strings",QJsonArray{}},{"attribute_userdata",QJsonArray{}},{"hatch_pattern",hatchPatternFacts(pattern)}};
}
static const ON_HatchPattern* systemPattern(int index){
    switch(index){case -1:return &ON_HatchPattern::Solid;case -2:return &ON_HatchPattern::Hatch1;case -3:return &ON_HatchPattern::Hatch2;case -4:return &ON_HatchPattern::Hatch3;case -5:return &ON_HatchPattern::HatchDash;case -6:return &ON_HatchPattern::Grid;case -7:return &ON_HatchPattern::Grid60;case -8:return &ON_HatchPattern::Plus;case -9:return &ON_HatchPattern::Squares;default:return nullptr;}
}
static void exactScale(ON_Hatch& hatch,double scale){
    if(!std::isfinite(scale)||scale<=0)throw ExchangeError("Invalid positive native hatch scale");
    // SetPatternScale silently ignores values<=0.001. ScalePattern has no such
    // floor. Use a unit x axis so its length is the requested scalar exactly.
    const auto plane=hatch.Plane();hatch.SetPlane(ON_xy_plane);hatch.SetPatternScale(1);
    const bool ok=hatch.ScalePattern(ON_Xform::DiagonalTransformation(scale));hatch.SetPlane(plane);
    if(!ok||hatch.PatternScale()!=scale)throw ExchangeError("Cannot represent exact native hatch scale");
}
static const ON_HatchPattern* referencedPattern(const ON_Hatch& hatch,const ONX_Model& model){
    auto pattern=hatch.PatternIndex()<0?systemPattern(hatch.PatternIndex()):ON_HatchPattern::Cast(model.ComponentFromIndex(ON_ModelComponent::Type::HatchPattern,hatch.PatternIndex()).ModelComponent());
    if(!pattern)throw ExchangeError("Missing native hatch pattern identity");return pattern;
}
QJsonObject hatchCurrentFields(const ON_Hatch& hatch,const ONX_Model& model){
    validateHatchRhino5Data(hatch);
    if(hatch.GetGradientType()!=ON_GradientType::None)throw ExchangeError("Rhino5 cannot retain native Hatch gradient data (requires archive version6 or newer)");
    auto facts=hatchFacts(hatch);return QJsonObject{{"plane",facts["plane"]},{"base_point",facts["base_point"]},{"rotation",facts["rotation"]},{"scale",facts["scale"]},{"pattern_uuid",uuid(referencedPattern(hatch,model)->Id())}};
}
QJsonArray hatchPatternChoices(const ONX_Model& model){
    QJsonArray choices;ONX_ModelComponentIterator patterns(model,ON_ModelComponent::Type::HatchPattern);
    for(auto c=patterns.FirstComponent();c;c=patterns.NextComponent())choices.append(QJsonObject{{"uuid",uuid(c->Id())},{"label",utf8(c->Name())+" [3DM]"}});
    for(int i=-1;i>=-9;--i){auto pattern=systemPattern(i);bool found=false;for(auto value:choices)if(value.toObject()["uuid"]==uuid(pattern->Id()))found=true;if(!found)choices.append(QJsonObject{{"uuid",uuid(pattern->Id())},{"label",utf8(pattern->Name())+" [Rhino]"}});}
    return choices;
}
void applyHatchFields(ON_Geometry& geometry,const QJsonValue& value,ONX_Model& model){
    if(auto h=ON_Hatch::Cast(&geometry))validateHatchRhino5Data(*h);
    auto hatch=ON_Hatch::Cast(&geometry);if(!hatch||!value.isObject())throw ExchangeError("Hatch current fields require native ON_Hatch geometry");if(hatch->GetGradientType()!=ON_GradientType::None)throw ExchangeError("Rhino5 cannot retain native Hatch gradient data (requires archive version6 or newer)");const auto fields=value.toObject();
    const QStringList keys{"plane","base_point","rotation","scale","pattern_uuid"};auto scalar=fields;scalar.remove("loops");if(scalar.size()!=keys.size())throw ExchangeError("Hatch overlay requires complete current fields");for(auto key:scalar.keys())if(!keys.contains(key))throw ExchangeError("Unknown Hatch current field");
    auto values=fields["plane"].toArray(),base=fields["base_point"].toArray();if(!fields["plane"].isArray()||values.size()!=16||!fields["base_point"].isArray()||base.size()!=2)throw ExchangeError("Hatch plane/base requires16/2 doubles");
    auto number=[](const QJsonValue& v){if(!v.isDouble()||!std::isfinite(v.toDouble()))throw ExchangeError("Invalid native Hatch numeric field");return v.toDouble();};
    ON_Plane plane;for(int i=0;i<3;++i){plane.origin[i]=number(values[i]);plane.xaxis[i]=number(values[i+3]);plane.yaxis[i]=number(values[i+6]);plane.zaxis[i]=number(values[i+9]);}for(int i=0;i<4;++i)plane.plane_equation[i]=number(values[i+12]);if(!plane.IsValid())throw ExchangeError("Invalid native Hatch plane frame/equation");ON_2dPoint point(number(base[0]),number(base[1]));if(!point.IsValid())throw ExchangeError("Invalid native Hatch base point");
    const double rotation=number(fields["rotation"]),scale=number(fields["scale"]);const auto identity=fields["pattern_uuid"].toString();QUuid parsed(identity);if(!fields["pattern_uuid"].isString()||parsed.isNull()||parsed.toString(QUuid::WithoutBraces)!=identity)throw ExchangeError("Invalid native Hatch pattern UUID");auto target=ON_UuidFromString(identity.toLatin1().constData());
    auto pattern=ON_HatchPattern::Cast(model.ComponentFromId(ON_ModelComponent::Type::HatchPattern,target).ModelComponent());const ON_HatchPattern* builtin=nullptr;if(!pattern)for(int i=-1;i>=-9;--i)if(systemPattern(i)->Id()==target)builtin=systemPattern(i);if(!pattern&&!builtin)throw ExchangeError("Missing chosen native Hatch pattern");hatchPatternFacts(pattern?*pattern:*builtin);
    ON_Hatch staged(*hatch);if(fields.contains("loops"))applyHatchLoopFields(staged,fields["loops"]);staged.SetPlane(plane);staged.SetBasePoint(point);staged.SetPatternRotation(rotation);exactScale(staged,scale);validateHatch(staged);
    if(builtin){ON_HatchPattern materialized(*builtin);if(!materialized.ClearIndex())throw ExchangeError("Cannot materialize chosen system hatch pattern");auto added=model.AddModelComponent(materialized);pattern=ON_HatchPattern::Cast(added.ModelComponent());if(!pattern||pattern->Id()!=target)throw ExchangeError("Cannot register chosen system Hatch pattern");}
    staged.SetPatternIndex(pattern->Index());if(hatchCurrentFields(staged,model)!=scalar)throw ExchangeError("Native Hatch setters changed requested current fields");*hatch=staged;
}
void transformHatchNative(ON_Hatch& hatch,const ON_Xform& transform,ONX_Model* model){
    validateHatchRhino5Data(hatch);
    validateHatch(hatch);
    if(hatch.GetGradientType()!=ON_GradientType::None)throw ExchangeError("Rhino5 cannot retain native Hatch gradient data (requires archive version6 or newer)");
    // Work on a clone: all loops, fields and a derived pattern must validate
    // before changing the caller's native payload or shared pattern table.
    ON_Hatch staged(hatch);auto plane=hatch.Plane();
    if(!plane.Transform(transform)||!plane.IsValid())throw ExchangeError("Cannot transform native hatch plane");
    const auto x=transform*hatch.Plane().xaxis,y=transform*hatch.Plane().yaxis;
    const double a=plane.xaxis*x,b=plane.xaxis*y,c=plane.yaxis*x,d=plane.yaxis*y;
    const double determinant=a*d-b*c;
    if(!std::isfinite(a)||!std::isfinite(b)||!std::isfinite(c)||!std::isfinite(d)||!std::isfinite(determinant)||determinant<=0)throw ExchangeError("Invalid native hatch plane-coordinate transform");
    ON_Xform uv=ON_Xform::IdentityTransformation;uv[0][0]=a;uv[0][1]=b;uv[1][0]=c;uv[1][1]=d;
    std::vector<std::unique_ptr<ON_HatchLoop>> loops;
    for(int i=0;i<hatch.LoopCount();++i){auto original=hatch.Loop(i);std::unique_ptr<ON_Curve> curve(original->Curve()->DuplicateCurve());if(!curve||!curve->Transform(uv)||!curve->IsValid()||!curve->IsClosed())throw ExchangeError("Cannot rebase native hatch loop");auto loop=std::make_unique<ON_HatchLoop>(*original);if(!loop->SetCurve(*curve))throw ExchangeError("Cannot retain transformed native hatch loop");loops.push_back(std::move(loop));}
    while(staged.LoopCount())if(!staged.RemoveLoop(staged.LoopCount()-1))throw ExchangeError("Cannot replace native hatch loop");
    for(auto& loop:loops)staged.AddLoop(loop.release());
    staged.SetPlane(plane);auto base=hatch.BasePoint2d();staged.SetBasePoint(ON_2dPoint(a*base.x+b*base.y,c*base.x+d*base.y));
    const double length=std::hypot(a,c),other=std::hypot(b,d);
    // Test the induced 2D map, not the world determinant. Reflection flips the
    // plane normal, while in-plane orientation remains encoded by its frame.
    const bool conformal=std::abs(length-other)<=1e-12*std::max(length,other)&&std::abs(a*b+c*d)<=1e-12*length*other;
    std::unique_ptr<ON_HatchPattern> derived;
    if(conformal){exactScale(staged,hatch.PatternScale()*length);staged.SetPatternRotation(hatch.PatternRotation()+std::atan2(c,a));}
    else{
        if(!model)throw ExchangeError("General affine hatch pattern transformation requires native model context");
        auto source=hatch.PatternIndex()<0?systemPattern(hatch.PatternIndex()):ON_HatchPattern::Cast(model->ComponentFromIndex(ON_ModelComponent::Type::HatchPattern,hatch.PatternIndex()).ModelComponent());
        if(!source)throw ExchangeError("Missing native hatch pattern for affine transform");hatchPatternFacts(*source);
        if(source->FillType()==ON_HatchPattern::HatchFillType::Solid){/* No directional pattern to deform. */}
        else{
            derived=std::make_unique<ON_HatchPattern>();derived->SetDescription(source->Description());derived->SetFillType(source->FillType());derived->CopyUserData(*source);
            // Stable for repeated exports of the same ordered native graph.
            // Salt occupied identities instead of reusing a source component.
            const auto seed=QByteArrayLiteral("OpenMatrix9.affine-hatch-pattern:")+uuid(source->Id()).toLatin1()+QJsonDocument(hatchPatternFacts(*source)).toJson(QJsonDocument::Compact)+QJsonDocument(QJsonArray{a,b,c,d,hatch.PatternScale(),hatch.PatternRotation()}).toJson(QJsonDocument::Compact);
            ON_UUID newId=ON_nil_uuid;unsigned salt=0;
            do{if(salt>=4096)throw ExchangeError("Affine hatch pattern identity collision limit exceeded");auto candidate=QUuid::createUuidV5(QUuid("f20d70bd-5c25-498a-8cae-e9181f7925bd"),seed+':'+QByteArray::number(salt++));newId=ON_UuidFromString(candidate.toString(QUuid::WithoutBraces).toLatin1().constData());}while(!model->ComponentFromId(ON_ModelComponent::Type::HatchPattern,newId).IsEmpty());
            if(!derived->SetId(newId)||!derived->SetName((L"OM9 affine "+uuid(newId).toStdWString()).c_str()))throw ExchangeError("Cannot identify affine hatch pattern");
            for(int i=0;i<source->HatchLineCount();++i){auto line=source->HatchLine(i);const double angle=hatch.PatternRotation()+line->AngleRadians(),cs=std::cos(angle),sn=std::sin(angle),scale=hatch.PatternScale();
                const double tx=a*cs+b*sn,ty=c*cs+d*sn,lineScale=scale*std::hypot(tx,ty);double nextAngle=std::atan2(ty,tx);if(nextAngle<0)nextAngle+=2*ON_PI;
                const double co=std::cos(nextAngle),si=std::sin(nextAngle);
                // Base and repetition offset are expressed in the line frame.
                auto local=[&](double px,double py){double u=scale*(cs*px-sn*py),v=scale*(sn*px+cs*py),xx=a*u+b*v,yy=c*u+d*v;return ON_2dVector(co*xx+si*yy,-si*xx+co*yy);};
                auto origin=local(line->Base().x,line->Base().y),offset=local(line->Offset().x,line->Offset().y);ON_SimpleArray<double> dashes;for(int j=0;j<line->DashCount();++j)dashes.Append(line->Dash(j)*lineScale);
                derived->AddHatchLine(ON_HatchLine(nextAngle,ON_2dPoint(origin.x,origin.y),offset,dashes));
            }
            hatchPatternFacts(*derived);staged.SetPatternRotation(0);staged.SetPatternScale(1);
        }
    }
    staged.TransformUserData(transform);validateHatch(staged);
    if(derived){auto added=model->AddModelComponent(*derived);auto pattern=ON_HatchPattern::Cast(added.ModelComponent());if(!pattern||pattern->Id()!=derived->Id())throw ExchangeError("Cannot register affine hatch pattern");staged.SetPatternIndex(pattern->Index());}
    hatch=staged;
}
}
