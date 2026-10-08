#include "ThreeDmArchive.h"
#include "ThreeDmPointCloud.h"
#include "ThreeDmHatch.h"
#include "ThreeDmCurveOnSurface.h"
#include <cmath>
namespace OpenMatrix9Gui::ThreeDm {
void transformNativeGeometry(ON_Geometry& geometry,const ON_Xform& transform,ONX_Model* model){
    if(!transform.IsValid()||!transform.IsAffine()||!std::isfinite(transform.Determinant())||transform.Determinant()==0)throw ExchangeError("Invalid or singular native geometry transform");
    if(auto curve=ON_CurveOnSurface::Cast(&geometry)){transformCurveOnSurfaceNative(*curve,transform);return;}
    // Native 2D curve transforms discard out-of-plane coordinates unless the
    // representation is explicitly promoted before applying the affine map.
    if(auto curve=ON_Curve::Cast(&geometry);curve&&curve->Dimension()==2&&(transform[2][0]!=0||transform[2][1]!=0||transform[2][3]!=0)){
        if(!curve->ChangeDimension(3)||curve->Dimension()!=3)throw ExchangeError("Cannot promote native 2D curve for an out-of-plane transform");
    }
    // The pinned ON_PointCloud::Transform transforms points/plane but not m_N.
    // Preserve optional per-point fields and rotate normals with inverse transpose.
    ON_SimpleArray<ON_3dVector> normals;
    auto cloud=ON_PointCloud::Cast(&geometry);
    if(cloud)validatePointCloud(*cloud);
    if(transform.IsIdentity(0.0)){
        if(!ON_InstanceRef::Cast(&geometry)&&!geometry.IsValid())throw ExchangeError("Cannot retain invalid native geometry");
        return;
    }
    // The serialized dormant plane still uses model coordinates. Normalize it
    // without enabling the height-field flag; native Transform skips this case.
    const bool dormantPlane=cloud&&!cloud->HasPlane()&&cloud->m_plane.IsValid();
    if(cloud&&cloud->m_N.Count()){
        if(!cloud->HasPointNormals())throw ExchangeError("Native point cloud normal count differs from point count");
        ON_Xform normalTransform;if(!transform.GetSurfaceNormalXform(normalTransform))throw ExchangeError("Cannot invert native point cloud normal transform");
        normals=cloud->m_N;
        for(int i=0;i<normals.Count();++i){
            if(!normals[i].IsValid())throw ExchangeError("Cannot transform invalid native point cloud normal");
            double length=normals[i].Length();if(!std::isfinite(length))throw ExchangeError("Invalid native point cloud normal length");
            if(length==0)continue;
            auto transformed=normalTransform*normals[i];if(!transformed.IsValid()||!transformed.Unitize())throw ExchangeError("Cannot transform native point cloud normal direction");
            normals[i]=length*transformed;
        }
    }
    if(auto hatch=ON_Hatch::Cast(&geometry)){transformHatchNative(*hatch,transform,model);return;}
    auto brep=ON_Brep::Cast(&geometry);
    const int sourceOrientation=brep?resolvedBrepOrientation(*brep,model?model->m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance:1e-7):0;
    if(!geometry.Transform(transform)||(!ON_InstanceRef::Cast(&geometry)&&!geometry.IsValid()))throw ExchangeError("Cannot transform valid native geometry");
    // The SDK clears its direction cache for reflections/nonuniform affine
    // maps. Preserve the resolved source sign with determinant parity, including
    // solids whose SDK cache did not contain a declared direction.
    if(brep&&(sourceOrientation==1||sourceOrientation==-1))
        brep->SetSolidOrientationForExperts(sourceOrientation*(transform.Determinant()<0?-1:1));
    if(dormantPlane&&!cloud->m_plane.Transform(transform))throw ExchangeError("Cannot transform dormant native PointCloud plane");
    if(cloud&&normals.Count())cloud->m_N=normals;
}
}
