#include "ThreeDmArchive.h"
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <sstream>
#include <iostream>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
using namespace OpenMatrix9Gui::ThreeDm;
int main(){try{auto model=readArchive(std::filesystem::path(OM9_UNMESHED_FIXTURE));int invalid=0,index=0;std::cout<<"count "<<model.items.size()<<" tolerance "<<model.tolerance<<" scale "<<model.scaleMm<<'\n';
for(auto& item:model.items){if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)){bool before=BRepCheck_Analyzer(*shape).IsValid();std::stringstream buffer;BRepTools::Write(*shape,buffer);TopoDS_Shape restored;BRep_Builder builder;BRepTools::Read(restored,buffer,builder);if(!BRepCheck_Analyzer(restored).IsValid()){std::cout<<"Invalid serialized object "<<index<<" beforeValid="<<before<<" name "<<item.name<<'\n';invalid++;}}index++;}
if(invalid)throw ExchangeError("Serialized geometry invalid");auto path=std::filesystem::temp_directory_path()/"om9-unmeshed-roundtrip.3dm";writeArchive5(model,path);auto restored=readArchive(path);if(restored.items.size()!=model.items.size())throw ExchangeError("Roundtrip count");
for(int i=5;i<=6;++i){GProp_GProps a,b;BRepGProp::VolumeProperties(std::get<TopoDS_Shape>(model.items[i].geometry),a,1e-10);BRepGProp::VolumeProperties(std::get<TopoDS_Shape>(restored.items[i].geometry),b,1e-10);std::cout.precision(16);std::cout<<"Accurate volume object "<<i<<": "<<a.Mass()<<" -> "<<b.Mass()<<" difference "<<b.Mass()-a.Mass()<<'\n';}
std::cout<<"unmeshed fixture PASS\n";return 0;}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
