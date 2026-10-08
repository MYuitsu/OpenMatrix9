#include "opennurbs.h"
#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <type_traits>
#include <utility>
template<class T,class=void>struct HasIntensity:std::false_type{};
template<class T>struct HasIntensity<T,std::void_t<decltype(std::declval<T>().m_V)>>:std::true_type{};
static_assert(!HasIntensity<ON_PointCloud>::value,"Legacy reader must verify the native cloud has no intensity member");
static void require(bool value,const char* error){if(!value)throw std::runtime_error(error);}
int main(int argc,char** argv){try{
    require(argc==4,"usage: Rhino5PointCloudReader input.3dm output.3dm report.json");ON::Begin();require(ON::Version()==201307115,"Unexpected legacy SDK version");ONX_Model model;ON_TextLog log(stderr);require(model.Read(argv[1],&log),"legacy SDK read failed");
    std::ofstream report(argv[3]);report<<std::setprecision(17)<<"{\"sdk_version\":"<<ON::Version()<<",\"objects\":[";bool first=true;int clouds=0;
    for(int i=0;i<model.m_object_table.Count();++i){auto& row=model.m_object_table[i];auto cloud=ON_PointCloud::Cast(row.m_object);if(!cloud)continue;++clouds;if(!first)report<<',';first=false;char uuid[37]{};ON_UuidToString(row.m_attributes.m_uuid,uuid);report<<"{\"uuid\":\""<<uuid<<"\",\"flags\":"<<cloud->m_flags<<",\"points\":[";
        for(int j=0;j<cloud->m_P.Count();++j){if(j)report<<',';report<<'['<<cloud->m_P[j].x<<','<<cloud->m_P[j].y<<','<<cloud->m_P[j].z<<']';}report<<"],\"normals\":[";
        for(int j=0;j<cloud->m_N.Count();++j){if(j)report<<',';report<<'['<<cloud->m_N[j].x<<','<<cloud->m_N[j].y<<','<<cloud->m_N[j].z<<']';}report<<"],\"colors\":[";
        for(int j=0;j<cloud->m_C.Count();++j){if(j)report<<',';auto c=cloud->m_C[j];report<<'['<<c.Red()<<','<<c.Green()<<','<<c.Blue()<<','<<c.Alpha()<<']';}report<<"]}";
    }
    report<<"],\"intensity_member_available\":false}\n";report.close();require(clouds>0,"no cloud read by legacy SDK");require(model.Write(argv[2],5,"OpenMatrix9 independent Rhino5-era reader",&log),"legacy SDK write failed");std::cout<<"SDK "<<ON::Version()<<" read/wrote "<<clouds<<" native clouds\n";return 0;
}catch(const std::exception& error){std::cerr<<error.what()<<'\n';return 1;}}
