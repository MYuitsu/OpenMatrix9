#include "ThreeDmGeometry.h"
#include <cmath>
namespace OpenMatrix9Gui::ThreeDm {
MeshData importMesh(const ON_Mesh& m){if(!m.IsValid())throw ExchangeError("Invalid polygon mesh");MeshData out;for(int i=0;i<m.VertexCount();++i){auto p=m.Vertex(i);out.vertices.push_back({p.x,p.y,p.z});}for(int i=0;i<m.FaceCount();++i){auto& f=m.m_F[i];out.faces.push_back({f.vi[0],f.vi[1],f.vi[2],f.vi[3]});}return out;}
std::unique_ptr<ON_Mesh> exportMesh(const MeshData& data){
    auto m=std::make_unique<ON_Mesh>();for(const auto& p:data.vertices){for(double v:p)if(!std::isfinite(v))throw ExchangeError("Nonfinite mesh vertex");m->m_dV.Append(ON_3dPoint(p[0],p[1],p[2]));m->m_V.Append(ON_3fPoint(float(p[0]),float(p[1]),float(p[2])));}
    for(auto f:data.faces){for(int v:f)if(v<0||v>=int(data.vertices.size()))throw ExchangeError("Mesh face index out of bounds");ON_MeshFace face;for(int i=0;i<4;++i)face.vi[i]=f[i];m->m_F.Append(face);}m->ComputeVertexNormals();if(!m->IsValid())throw ExchangeError("Exported polygon mesh invalid");return m;
}
}
