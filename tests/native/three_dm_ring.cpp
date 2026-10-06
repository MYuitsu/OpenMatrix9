#include "ThreeDmArchive.h"
#include <BRepCheck_Analyzer.hxx>
#include <BRepBndLib.hxx>
#include <BRepGProp.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <iostream>
#include <cmath>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool condition,const char* message){if(!condition)throw ExchangeError(message);}
int main(){try{
    const auto source=std::filesystem::path(OM9_RING_FIXTURE);
    const auto original=readArchive(source);
    require(original.items.size()==101,"User ring fixture object count changed");
    const auto output=std::filesystem::temp_directory_path()/"om9-ring-regression-rhino5.3dm";
    writeArchive5(original,output);
    const auto restored=readArchive(output);
    require(restored.items.size()==original.items.size(),"Round-trip object count");
    for(std::size_t i=0;i<original.items.size();++i){auto& a=original.items[i];auto& b=restored.items[i];
        require(a.name==b.name&&a.layer==b.layer&&a.color==b.color&&a.visible==b.visible&&a.locked==b.locked,"Round-trip metadata");
        require(a.geometry.index()==b.geometry.index(),"Geometry type changed");
        if(auto shape=std::get_if<TopoDS_Shape>(&a.geometry)){const auto& copy=std::get<TopoDS_Shape>(b.geometry);require(BRepCheck_Analyzer(copy).IsValid(),"Round-trip invalid BRep");
            Bnd_Box aa,bb;BRepBndLib::AddOptimal(*shape,aa,false,false);BRepBndLib::AddOptimal(copy,bb,false,false);double av[6],bv[6];aa.Get(av[0],av[1],av[2],av[3],av[4],av[5]);bb.Get(bv[0],bv[1],bv[2],bv[3],bv[4],bv[5]);for(int k=0;k<6;++k)require(std::abs(av[k]-bv[k])<1e-3,"Round-trip bounds");
            GProp_GProps pa,pb;BRepGProp::SurfaceProperties(*shape,pa);BRepGProp::SurfaceProperties(copy,pb);require(std::abs(pa.Mass()-pb.Mass())<std::max(1e-3,std::abs(pa.Mass())*1e-4),"Round-trip area");
        }else{auto& ma=std::get<MeshData>(a.geometry);auto& mb=std::get<MeshData>(b.geometry);require(ma.vertices.size()==mb.vertices.size()&&ma.faces==mb.faces,"Round-trip mesh topology");}
    }
    std::cout<<"OM9-FILE-012 user ring PASS: "<<original.items.size()<<" objects\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
