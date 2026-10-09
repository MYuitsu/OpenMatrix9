// SPDX-License-Identifier: LGPL-2.1-or-later
#include "PolycurveContract.h"
#include "NurbsBasisAdapter.h"
#include "ThreeDmArchive.h"
#include "ThreeDmStaging.h"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QFile>
#include <cmath>
#include <iostream>
#include <filesystem>
using namespace OpenMatrix9Gui::ThreeDm;
using namespace OpenMatrix9Gui::ThreeDm::Polycurve;
namespace {
QJsonArray checks,cases;
constexpr Options bounds{1e-7,1e-12};
void check(bool value,const char* label){checks.append(QJsonObject{{"name",label},{"passed",value}});if(!value)throw std::runtime_error(label);}
template<class F> void rejected(F&& f,const char* label){bool fail=false;try{f();}catch(const ExchangeError&){fail=true;}check(fail,label);}
void append(ON_PolyCurve& p,std::unique_ptr<ON_Curve> c){check(p.Append(c.get()),"source append succeeds");c.release();}
std::unique_ptr<ON_PolyCurve> mixed(){
 auto p=std::make_unique<ON_PolyCurve>();auto line=std::make_unique<ON_LineCurve>(ON_3dPoint(-5,0,0),ON_3dPoint(0,0,0));line->SetDomain(0,5);append(*p,std::move(line));
 auto arc=std::make_unique<ON_NurbsCurve>(3,true,3,3);double w=std::sqrt(.5);
 arc->SetCV(0,ON_4dPoint(0,0,0,1));arc->SetCV(1,ON_4dPoint(10*w,0,0,w));arc->SetCV(2,ON_4dPoint(10,10,0,1));
 arc->SetKnot(0,11);arc->SetKnot(1,11);arc->SetKnot(2,19);arc->SetKnot(3,19);append(*p,std::move(arc));
 const double t[]{2,3,7};check(p->SetParameterization(t),"independent parent parameterization");return p;
}
struct GuardedNurbs : ON_NurbsCurve {
 mutable unsigned metadataReads=0;
 bool metadataTrap=false;
 explicit GuardedNurbs(const ON_NurbsCurve& n):ON_NurbsCurve(n){}
 ON_Interval Domain() const override {++metadataReads;if(metadataTrap)throw ExchangeError("Guarded malformed child Domain trap");return ON_NurbsCurve::Domain();}
 ON__UINT32 DataCRC(ON__UINT32 seed) const override {++metadataReads;if(metadataTrap)throw ExchangeError("Guarded malformed child CRC trap");return ON_NurbsCurve::DataCRC(seed);}
};
void rawChildren(){
 auto fixture=mixed();auto guarded=std::make_unique<GuardedNurbs>(*ON_NurbsCurve::Cast(fixture->SegmentCurve(1)));auto* raw=guarded.get();ON_PolyCurve p;append(p,std::move(guarded));
 auto count=raw->m_cv_count;raw->metadataReads=0;raw->metadataTrap=true;raw->m_cv_count=1000001;
 rejected([&]{importPolyCurveObserved(p,bounds);},"over-budget raw child rejected through polycurve entry");
 raw->m_cv_count=count;raw->metadataTrap=false;check(raw->metadataReads==0,"over-budget child rejected before Domain or parent CRC traversal");
 auto cvs=raw->m_cv;raw->metadataReads=0;raw->metadataTrap=true;raw->m_cv=nullptr;
 rejected([&]{importPolyCurveObserved(p,bounds);},"null raw child CV buffer rejected through polycurve entry");
 raw->m_cv=cvs;raw->metadataTrap=false;check(raw->metadataReads==0,"null child buffer rejected before Domain or parent CRC traversal");
}
void sample(const TopoDS_Wire& wire,bool reverse){
 int ordinal=0;for(BRepTools_WireExplorer e(wire);e.More();e.Next(),++ordinal){
  double a,b;auto c=BRep_Tool::Curve(e.Current(),a,b);bool arc=ordinal==(reverse?0:1);double residual=0;
  for(int n=0;n<33;++n){auto p=c->Value(a+(b-a)*n/32.);residual=std::max(residual,arc?std::abs(std::hypot(p.X(),p.Y()-10.)-10.):std::abs(p.Y()));check(std::abs(p.Z())<=bounds.lengthTolerance,"sample remains planar");}
  check(residual<=bounds.lengthTolerance,arc?"sampled rational R10 arc bound":"sampled line bound");
  if(arc){auto basis=BasisContract::checkedBasis(Handle(Geom_BSplineCurve)::DownCast(c));check(basis.axis.degree==2 && basis.poles.size()==3,"rational quadratic partition basis retained");check(basis.weights.size()==3 && std::abs(basis.weights[1]-std::sqrt(.5))<1e-12,"nonunit middle weight retained");}
 }
}
void run(const std::filesystem::path& root){
 rawChildren();
 auto source=mixed();auto sibling=std::make_unique<ON_PolyCurve>(*source);auto crc=source->DataCRC(0),siblingCrc=sibling->DataCRC(0);
 auto output=importPolyCurveObserved(*source,bounds);
 check(output.wire.ShapeType()==TopAbs_WIRE,"polycurve remains a wire");
 check(BRepCheck_Analyzer(output.wire).IsValid(),"native wire is valid");
 check(output.report.source.size()==2 && output.report.native.size()==2,"two copied ordered segment rows");
 check(output.report.source[0].child_domain==std::array<double,2>{0,5} && output.report.source[1].child_domain==std::array<double,2>{11,19},"child domains remain distinct from parent spans");
 check(output.report.source[1].parent_domain==std::array<double,2>{3,7},"source parent span retained");
 check(output.report.native[1].shared_previous==2,"actual joint shares native vertex");
 sample(output.wire,false);
 int count=0;for(BRepTools_WireExplorer e(output.wire);e.More();e.Next())++count;check(count==2,"actual native traversal has two occurrences");
 auto exported=exportWireChecked(output.wire,bounds);check(exported->Count()==2,"export preserves two children");
 auto roundtrip=importPolyCurveObserved(*exported,bounds);check(roundtrip.report.native.size()==2,"roundtrip retains partition");
 sample(roundtrip.wire,false);
 check(source->DataCRC(0)==crc && sibling->DataCRC(0)==siblingCrc,"source and sibling unchanged");
 auto json=output.report.json();check(json["parameter_map"].isNull() && json["global_deviation"].isNull(),"uninspected global preservation unknown");cases.append(json);
 auto reversed=TopoDS::Wire(output.wire.Reversed());auto reverseExport=exportWireChecked(reversed,bounds);auto reverseImport=importPolyCurveObserved(*reverseExport,bounds);
 check(reverseImport.report.source.front().start[0]==10 && reverseImport.report.source.back().end[0]==-5,"reversed native traversal retains oriented endpoints");cases.append(reverseImport.report.json());
 sample(reverseImport.wire,true);
 auto reversedSource=std::make_unique<ON_PolyCurve>(*source);check(reversedSource->Reverse(),"source polycurve native reversal succeeds");auto nativeReverse=importPolyCurveObserved(*reversedSource,bounds);sample(nativeReverse.wire,true);
 check(nativeReverse.report.source.front().parent_domain[0]==-7 && nativeReverse.report.source.back().parent_domain[1]==-2,"reversed parent parameter intervals retained separately");
 auto shifted=std::make_unique<ON_PolyCurve>(*source);check(shifted->Transform(ON_Xform::TranslationTransformation(ON_3dVector(21,17,3))),"shifted source fixture");auto shiftedOutput=importPolyCurveObserved(*shifted,bounds);check(shiftedOutput.report.source.front().start==std::array<double,3>{16,17,3},"nonzero joint domain preflight does not inspect fabricated geometry");
 auto gapped=std::make_unique<ON_PolyCurve>(*source);check(gapped->SegmentCurve(1)->SetStartPoint(ON_3dPoint(.01,0,0)),"gap fixture authored without parent healing");const auto gapCrc=gapped->DataCRC(0);rejected([&]{importPolyCurveObserved(*gapped,bounds);},"source gap rejected without midpoint repair");check(gapped->DataCRC(0)==gapCrc,"gapped rejection leaves source unchanged");
 ON_PolyCurve empty;rejected([&]{importPolyCurveObserved(empty,bounds);},"empty source rejected before Domain");
 auto malformed=std::make_unique<ON_PolyCurve>(*source);auto& parameters=const_cast<ON_SimpleArray<double>&>(malformed->SegmentParameters());parameters.SetCount(2);rejected([&]{importPolyCurveObserved(*malformed,bounds);},"incomplete parent container rejected");
 ON_PolyCurve nested;append(nested,mixed());rejected([&]{importPolyCurveObserved(nested,bounds);},"nested source explicitly unsupported instead of flattening");
 TopoDS_Wire branched;BRep_Builder builder;builder.MakeWire(branched);for(BRepTools_WireExplorer e(output.wire);e.More();e.Next())builder.Add(branched,e.Current());builder.Add(branched,BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(0,-3,0)).Edge());rejected([&]{exportWireChecked(branched,bounds);},"branch or uncovered occurrence rejected instead of partial export");
 ExchangeModel model;ExchangeItem item;item.name="Mixed";item.geometry=reversed;model.items.push_back(item);model.tolerance=bounds.lengthTolerance;
 std::filesystem::create_directories(root);auto path=root/"mixed-reversed.3dm";writeArchive5(model,path);auto reread=readArchive(path);
 check(reread.items.size()==1,"actual V5 reread one object");auto wire=std::get<TopoDS_Shape>(reread.items[0].geometry);check(wire.ShapeType()==TopAbs_WIRE,"archive dispatch keeps actual polycurve wire");verifyWirePartition(reversed,TopoDS::Wire(wire),bounds);sample(TopoDS::Wire(wire),true);
 auto prepared=encodeExchangeItem(reread.items[0],root,0);check(prepared["conversion_report"].toObject()["segment_count"].toInt()==2,"copied segment facts staged before host commit");
 check(!prepared["conversion_report"].toObject()["length_unit"].isNull() && prepared["conversion_report"].toObject()["observed_before_unit_scale"].toBool(),"archive report explicitly declares source unit frame");
 cases.append(prepared["conversion_report"].toObject());
 ON_PolyCurve lines;append(lines,std::make_unique<ON_LineCurve>(ON_3dPoint(4,3,0),ON_3dPoint(8,3,0)));append(lines,std::make_unique<ON_LineCurve>(ON_3dPoint(8,3,0),ON_3dPoint(8,9,0)));
 auto lineWire=importPolyCurveObserved(lines,bounds);check(lineWire.report.native.size()==2,"existing all-line polycurve control retains two occurrences");auto lineExport=exportWireChecked(lineWire.wire,bounds);check(lineExport->Count()==2,"all-line export retains partition");
 ONX_Model retained;ON_Layer layer;layer.SetName(L"Fixture");retained.AddModelComponent(layer);retained.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_3dmObjectAttributes attributes;attributes.m_layer_index=0;attributes.m_name=L"NestedUnsupported";
 check(!retained.AddModelGeometryComponent(&nested,&attributes).IsEmpty(),"nested archive fixture owns copied geometry");auto nestedPath=root/"nested-unsupported.3dm";check(retained.Write(nestedPath.c_str(),5,nullptr),"nested source archive write");
 auto preserved=readArchive(nestedPath,0,true);check(preserved.items.size()==1 && preserved.items[0].retained && !preserved.items[0].representationIssue.empty(),"nested ordinary archive explicitly retained with reason");
 rejected([&]{readArchive(nestedPath,0,false);},"working nested archive rejects instead of flattening or publishing output");
 auto destination=root/"unchanged-destination.3dm";{QFile f(QString::fromStdWString(destination.wstring()));check(f.open(QIODevice::WriteOnly),"negative export destination fixture");f.write("existing destination bytes");}
 ExchangeModel invalid;ExchangeItem bad;bad.geometry=branched;invalid.items.push_back(bad);invalid.tolerance=bounds.lengthTolerance;
 rejected([&]{writeArchive5(invalid,destination);},"branch export rejects before output file write");{QFile f(QString::fromStdWString(destination.wstring()));check(f.open(QIODevice::ReadOnly) && f.readAll()=="existing destination bytes","rejected branch export leaves destination bytes unchanged");}
}
}
int main(int argc,char** argv){ON::Begin();QString error;auto root=argc>1?std::filesystem::path(argv[1]).parent_path()/"polycurve-fixtures":std::filesystem::temp_directory_path()/"om9-polycurve-test";try{run(root);}catch(const Standard_Failure& e){error=e.GetMessageString();}catch(const std::exception& e){error=e.what();}
 QJsonObject report{{"schema",1},{"fixture_length_unit","mm"},{"checks",checks},{"cases",cases},{"error",error}};
 if(argc>1){QFile f(QString::fromLocal8Bit(argv[1]));if(!f.open(QIODevice::WriteOnly))return 2;f.write(QJsonDocument(report).toJson());}
 std::cout<<checks.size()<<" checks; "<<(error.isEmpty()?"PASS":error.toStdString())<<"\n";return error.isEmpty()?0:1;
}
