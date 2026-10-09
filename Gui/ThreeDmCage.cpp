// SPDX-License-Identifier: LGPL-2.1-or-later
#include "ThreeDmCage.h"
#include <algorithm>
#include <cmath>
#include <map>
#include <set>
#include <fstream>

namespace OpenMatrix9Gui::ThreeDm {
namespace {
struct UuidLess { bool operator()(const ON_UUID& a,const ON_UUID& b) const { return ON_UuidCompare(a,b)<0; } };
std::string uuid(const ON_UUID& id) { char s[37]{}; ON_UuidToString(id,s); return s; }
std::array<double,16> matrix(const ON_Xform& x) {
    std::array<double,16> result{};
    for(int r=0;r<4;++r)for(int c=0;c<4;++c)result[r*4+c]=x[r][c];
    return result;
}
ON_Xform invertibleAffine(ON_Xform x) {
    if(x.IsValid() && std::abs(x[3][0])<=1e-12 && std::abs(x[3][1])<=1e-12
       && std::abs(x[3][2])<=1e-12 && std::abs(x[3][3]-1)<=1e-12) {
        x[3][0]=x[3][1]=x[3][2]=0; x[3][3]=1;
    }
    const double det=x.Determinant();
    if(!x.IsValid()||!x.IsAffine()||!std::isfinite(det)||det==0)
        throw ExchangeError("Invalid or singular original cage reference transform");
    return x;
}
CageDescriptor extract(const ON_NurbsCage& c,double scale) {
    // Native bounds precede SDK validation/iteration. Editing/basis policy belongs
    // to Rust; these limits protect adapter traversal and detached allocations.
    CageDescriptor result; std::size_t total=1;
    if(c.Dimension()!=3)throw ExchangeError("Cage dimension must be three");
    for(int d=0;d<3;++d) {
        const int count=c.CVCount(d), degree=c.Order(d)-1;
        if(degree<1||degree>32||count<degree+1||count>1000000)
            throw ExchangeError("Cage count/degree exceeds decoder bounds");
        if(total>1000000/static_cast<std::size_t>(count))
            throw ExchangeError("Cage control-point limit exceeded");
        total*=static_cast<std::size_t>(count);
        result.counts[d]=count; result.degrees[d]=degree;
    }
    if(!c.IsValid())throw ExchangeError("Invalid source NURBS cage");
    result.rational=c.IsRational(); result.points.reserve(total);result.weights.reserve(total);
    for(int d=0;d<3;++d) {
        const int n=c.KnotCount(d);
        if(n!=result.counts[d]+result.degrees[d]-1||n<2)
            throw ExchangeError("Invalid cage compact knot count");
        auto& knots=result.fullKnots[d];knots.reserve(static_cast<std::size_t>(n)+2);
        knots.push_back(c.Knot(d,0));
        for(int k=0;k<n;++k) {
            const double t=c.Knot(d,k);
            if(!std::isfinite(t)||(k>0&&t<c.Knot(d,k-1)))throw ExchangeError("Invalid cage knot value");
            knots.push_back(t);
        }
        knots.push_back(c.Knot(d,n-1));
        const auto domain=c.Domain(d);
        if(!domain.IsIncreasing())throw ExchangeError("Invalid cage parameter domain");
    }
    for(int u=0;u<result.counts[0];++u)for(int v=0;v<result.counts[1];++v)for(int w=0;w<result.counts[2];++w) {
        ON_3dPoint p;const double weight=c.Weight(u,v,w);
        if(!c.GetCV(u,v,w,p)||!p.IsValid()||!std::isfinite(weight)||weight<=0)
            throw ExchangeError("Invalid cage control point or nonpositive weight");
        p*=scale;
        if(!p.IsValid())throw ExchangeError("Cage control point overflows millimeter conversion");
        result.points.push_back({p.x,p.y,p.z});result.weights.push_back(weight);
    }
    return result;
}
}
CageArchiveRecord cageArchiveRecord(std::span<const unsigned char> bytes,const std::string& sourceUuid,double custom) {
    if(bytes.size()>512ULL*1024*1024)
        throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    ON::Begin();ONX_Model model;
    // ON_Read3dmBufferArchive presets the version for embedded chunks; complete
    // files require the unset version consumed by Read3dmStartSection instead.
    ON_Buffer buffer;if(bytes.empty()||buffer.Write(bytes.size(),bytes.data())!=bytes.size())throw ExchangeError("Cannot buffer retained 3DM source archive");
    ON_BinaryArchiveBuffer archive(ON::archive_mode::read3dm,&buffer);
    if(!model.Read(archive,nullptr))throw ExchangeError("Cannot read retained 3DM source archive");
    const ON_UUID id=ON_UuidFromString(sourceUuid.c_str());
    if(id==ON_nil_uuid||uuid(id)!=sourceUuid)throw ExchangeError("Invalid source UUID");
    CageArchiveRecord result;result.sourceUuid=sourceUuid;
    const auto units=model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem();
    result.scaleMm=(units==ON::LengthUnitSystem::None||units==ON::LengthUnitSystem::CustomUnits)
        ?custom:ON::UnitScale(units,ON::LengthUnitSystem::Millimeters);
    if(!std::isfinite(result.scaleMm)||result.scaleMm<=0)
        throw ExchangeError("3DM custom/unitless file requires millimeters per file unit");
    const double sourceTolerance=model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance;
    if(!std::isfinite(sourceTolerance)||sourceTolerance<=0)
        throw ExchangeError("Invalid source tolerance");
    result.toleranceMm=std::max(1e-7,sourceTolerance*result.scaleMm);
    if(!std::isfinite(result.toleranceMm))throw ExchangeError("Source tolerance overflows millimeter conversion");
    std::map<ON_UUID,const ON_ModelGeometryComponent*,UuidLess> objects;
    ONX_ModelComponentIterator iterator(model,ON_ModelComponent::Type::ModelGeometry);
    for(auto c=iterator.FirstComponent();c;c=iterator.NextComponent())if(auto g=ON_ModelGeometryComponent::Cast(c)) {
        if(objects.size()>=1000000)throw ExchangeError("3DM object limit exceeded");
        if(!objects.emplace(g->Id(),g).second)throw ExchangeError("Duplicate archive object UUID");
    }
    const auto found=objects.find(id);
    if(found==objects.end())throw ExchangeError("Source UUID is absent from retained archive");
    const auto geometry=found->second->Geometry(nullptr);
    const auto attributes=found->second->Attributes(nullptr);
    if(!geometry||!attributes)throw ExchangeError("Invalid source geometry component");
    if(attributes->m_space==ON::page_space||attributes->IsInstanceDefinitionObject())
        throw ExchangeError("Cage editing requires a world model-space record");
    result.sourceClass=geometry->ClassId()->ClassName();
    if(auto control=ON_MorphControl::Cast(geometry)) {
        if(control->m_varient==1)
            throw ExchangeError("Curve morph-control variant requires original/current curve correspondence and is not supported for volume cage editing");
        if(control->m_varient==2)
            throw ExchangeError("Surface morph-control variant requires original/current surface correspondence and is not supported for volume cage editing");
        if(control->m_varient!=3)throw ExchangeError("Unsupported morph-control variant");
        if(control->m_localizers.Count()!=0)
            throw ExchangeError("Localized morph control is not supported; localizer attenuation cannot be discarded");
        result.isMorphControl=true;result.currentCage=extract(control->m_nurbs_cage,result.scaleMm);
        const auto original=invertibleAffine(control->m_nurbs_cage0*ON_Xform::DiagonalTransformation(1/result.scaleMm));
        ON_Xform inverse=original;
        if(!inverse.Invert())throw ExchangeError("Cannot invert original cage reference transform");
        inverse=invertibleAffine(inverse);
        result.originalReference=CageOriginalReference{matrix(original),matrix(inverse)};
        result.morphToleranceMm=control->m_sporh_tolerance*result.scaleMm;
        if(!std::isfinite(result.morphToleranceMm)||result.morphToleranceMm<0)
            throw ExchangeError("Invalid morph-control tolerance");
        result.preserveStructure=control->m_sporh_bPreserveStructure;
        result.quickPreview=control->m_sporh_bQuickPreview;
        const int count=control->m_captive_id.Count();
        if(count<0||count>1000000)throw ExchangeError("Captive relationship limit exceeded");
        const auto ids=control->m_captive_id.Array();
        if(count>0&&!ids)throw ExchangeError("Invalid captive UUID table");
        std::set<ON_UUID,UuidLess> seen;
        for(int i=0;i<count;++i) {
            if(ids[i]==ON_nil_uuid)throw ExchangeError("Captive relationship has nil UUID");
            if(ids[i]==id)throw ExchangeError("Cage control cannot capture itself");
            if(!seen.insert(ids[i]).second)throw ExchangeError("Duplicate captive relationship UUID");
            const auto captive=objects.find(ids[i]);
            if(captive==objects.end())throw ExchangeError("Missing captive relationship geometry: "+uuid(ids[i]));
            if(!captive->second->Geometry(nullptr)||!captive->second->Attributes(nullptr))
                throw ExchangeError("Invalid captive relationship geometry: "+uuid(ids[i]));
            result.captiveSourceUuids.push_back(uuid(ids[i]));
        }
    }else if(auto cage=ON_NurbsCage::Cast(geometry))result.currentCage=extract(*cage,result.scaleMm);
    else throw ExchangeError("Source record is not a NURBS cage or morph control");
    return result;
}
CageArchiveRecord cageArchiveRecord(const std::filesystem::path& path,const std::string& sourceUuid,double custom) {
    const auto size=std::filesystem::file_size(path);if(size>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    std::ifstream file(path,std::ios::binary);std::vector<unsigned char> bytes(static_cast<std::size_t>(size));
    if(!file||!file.read(reinterpret_cast<char*>(bytes.data()),static_cast<std::streamsize>(size))||file.peek()!=std::char_traits<char>::eof())throw ExchangeError("Cannot read retained 3DM source archive");
    return cageArchiveRecord(std::span<const unsigned char>(bytes),sourceUuid,custom);
}
}
