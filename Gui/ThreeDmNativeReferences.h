#pragma once
#include "ThreeDmGeometry.h"
#include <QJsonObject>
#include <QJsonArray>
#include <map>
#include <memory>
namespace OpenMatrix9Gui::ThreeDm {
// Graph ownership is part of the API: proxy targets must outlive linked curves.
// Source records remain immutable; these objects are derived native geometry.
struct NativeReferenceGraph {
    std::shared_ptr<const ONX_Model> source;
    std::map<QString,std::shared_ptr<ON_Geometry>> geometry;
    QString rootUuid;
};
struct NativeReferenceResolution {
    QJsonObject diagnostics;
    QJsonObject statistics;
    std::map<QString,std::shared_ptr<NativeReferenceGraph>> graphs;
};
struct NativeReferenceLimits {
    size_t cloneBytes=512ULL*1024*1024;
    size_t owners=16384;
    size_t nodes=16384;
    unsigned depth=64;
};
struct NativeTrimParameter {double parameter=0;double deviation=0;};
// Detached archive trees only. Preflight every known owner slot before changing
// any UUID; live proxy pointers and opaque identity dependencies are rejected.
void remapDeferredNativeReferences(ON_Curve&,const std::map<QString,ON_UUID>&,NativeReferenceLimits limits={});
// Physical inverse mapping; native trim evaluation validates every returned result.
// Ambiguous or out-of-tolerance correspondence is rejected, never guessed affine.
NativeTrimParameter nativeTrimParameterAt(const ON_BrepTrim&,const ON_BrepEdge&,const ON_Surface&,ON_Interval trimDomain,double edgeParameter,double tolerance);
std::shared_ptr<NativeReferenceResolution> resolveNativeReferences(std::shared_ptr<const ONX_Model> source,const QJsonArray& records,NativeReferenceLimits limits={});
}
