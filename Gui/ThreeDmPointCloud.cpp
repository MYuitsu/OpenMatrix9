#include "ThreeDmPointCloud.h"
#include <QJsonArray>
#include <QJsonDocument>
#include <QCryptographicHash>
#include <QFile>
#include <QDataStream>
#include <cmath>
namespace OpenMatrix9Gui::ThreeDm {
void validatePointCloud(const ON_PointCloud& cloud){
    auto count=cloud.m_P.Count();if(count<=0)throw ExchangeError("Native PointCloud requires nonempty points");
    if((cloud.m_N.Count()!=0&&cloud.m_N.Count()!=count)||(cloud.m_C.Count()!=0&&cloud.m_C.Count()!=count)||(cloud.m_V.Count()!=0&&cloud.m_V.Count()!=count))throw ExchangeError("Native PointCloud optional array count differs from points");
    for(int i=0;i<count;++i)if(!cloud.m_P[i].IsValid())throw ExchangeError("Invalid native PointCloud point");
    for(int i=0;i<cloud.m_N.Count();++i)if(!cloud.m_N[i].IsValid())throw ExchangeError("Invalid native PointCloud normal");
    for(int i=0;i<cloud.m_V.Count();++i)if(!std::isfinite(cloud.m_V[i]))throw ExchangeError("Invalid native PointCloud intensity");
    if(cloud.HasPlane()&&!cloud.m_plane.IsValid())throw ExchangeError("Invalid active native PointCloud plane");
}
static QJsonArray planeFields(const ON_Plane& plane){QJsonArray values;for(auto v:{plane.origin,ON_3dPoint(plane.xaxis),ON_3dPoint(plane.yaxis),ON_3dPoint(plane.zaxis)})for(int i=0;i<3;++i)values.append(v[i]);for(int i=0;i<4;++i)values.append(plane.plane_equation[i]);return values;}
void validateRhino5PointCloud(const ON_PointCloud& cloud){
    validatePointCloud(cloud);
    if(cloud.m_V.Count()>0)throw ExchangeError("Rhino5-era openNURBS20130711 reader loses PointCloud intensity; retain current values in FCStd or explicitly clear them before Rhino5 export");
}
QJsonObject pointCloudFields(const ON_PointCloud& cloud){
    validatePointCloud(cloud);QJsonArray points,normals,colors,values;
    for(int i=0;i<cloud.m_P.Count();++i){auto p=cloud.m_P[i];points.append(QJsonArray{p.x,p.y,p.z});}
    for(int i=0;i<cloud.m_N.Count();++i){auto n=cloud.m_N[i];normals.append(QJsonArray{n.x,n.y,n.z});}
    for(int i=0;i<cloud.m_C.Count();++i){auto c=cloud.m_C[i];for(auto v:{c.Red(),c.Green(),c.Blue(),c.Alpha()})colors.append(v);}
    for(int i=0;i<cloud.m_V.Count();++i)values.append(cloud.m_V[i]);
    return QJsonObject{{"schema_version",1},{"points",points},{"normals",normals},{"colors",colors},{"values",values},{"ordered",cloud.IsOrdered()},{"has_plane",cloud.HasPlane()},{"plane",planeFields(cloud.m_plane)}};
}
QJsonObject pointCloudSummary(const ON_PointCloud& cloud){
    validatePointCloud(cloud);ON_BoundingBox box;for(int i=0;i<cloud.m_P.Count();++i)box.Set(cloud.m_P[i],true);
    QJsonObject summary{{"point_count",cloud.m_P.Count()},{"normal_count",cloud.m_N.Count()},{"color_count",cloud.m_C.Count()},{"value_count",cloud.m_V.Count()},{"ordered",cloud.IsOrdered()},{"has_plane",cloud.HasPlane()},{"plane",planeFields(cloud.m_plane)},{"flags",static_cast<double>(cloud.m_flags)}};
    if(box.IsValid())summary["bounds"]=QJsonArray{box.m_min.x,box.m_min.y,box.m_min.z,box.m_max.x,box.m_max.y,box.m_max.z};
    // Bounded binary array hashes keep the inventory small and independent of
    // locale/JSON floating-point formatting. Inspecting never fills m_bbox.
    auto hash=[&](auto append){QByteArray bytes;QDataStream stream(&bytes,QIODevice::WriteOnly);stream.setByteOrder(QDataStream::LittleEndian);stream.setFloatingPointPrecision(QDataStream::DoublePrecision);append(stream);return QString::fromLatin1(QCryptographicHash::hash(bytes,QCryptographicHash::Sha256).toHex());};
    summary["points_sha256"]=hash([&](QDataStream& s){s<<quint32(cloud.m_P.Count());for(int i=0;i<cloud.m_P.Count();++i)for(int j=0;j<3;++j)s<<cloud.m_P[i][j];});
    summary["normals_sha256"]=hash([&](QDataStream& s){s<<quint32(cloud.m_N.Count());for(int i=0;i<cloud.m_N.Count();++i)for(int j=0;j<3;++j)s<<cloud.m_N[i][j];});
    summary["colors_sha256"]=hash([&](QDataStream& s){s<<quint32(cloud.m_C.Count());for(int i=0;i<cloud.m_C.Count();++i){auto c=cloud.m_C[i];s<<quint8(c.Red())<<quint8(c.Green())<<quint8(c.Blue())<<quint8(c.Alpha());}});
    summary["values_sha256"]=hash([&](QDataStream& s){s<<quint32(cloud.m_V.Count());for(int i=0;i<cloud.m_V.Count();++i)s<<cloud.m_V[i];});return summary;
}
QString writePointCloudFields(const ON_PointCloud& cloud,const std::filesystem::path& path){
    auto payload=QJsonDocument(pointCloudFields(cloud)).toJson(QJsonDocument::Compact);if(payload.size()>512LL*1024*1024)throw ExchangeError("PointCloud current fields exceed512MiB");QFile file(QString::fromStdWString(path.wstring()));if(!file.open(QIODevice::WriteOnly)||file.write(payload)!=payload.size())throw ExchangeError("Cannot stage PointCloud current fields");file.close();return QString::fromLatin1(QCryptographicHash::hash(payload,QCryptographicHash::Sha256).toHex());
}
void applyPointCloudFields(ON_Geometry& geometry,const std::filesystem::path& path,const QString& sha256){
    auto cloud=ON_PointCloud::Cast(&geometry);ON_PointCloud expected;if(!cloud||geometry.ClassId()!=expected.ClassId())throw ExchangeError("PointCloud fields require native ON_PointCloud geometry");
    if(sha256.size()!=64)throw ExchangeError("Invalid PointCloud current-field SHA256");for(auto c:sha256)if(!((c>='0'&&c<='9')||(c>='a'&&c<='f')))throw ExchangeError("Invalid PointCloud current-field SHA256");
    QFile file(QString::fromStdWString(path.wstring()));if(!file.open(QIODevice::ReadOnly)||file.size()<=0||file.size()>512LL*1024*1024)throw ExchangeError("Missing or oversized PointCloud current-field file");auto bytes=file.readAll();if(bytes.size()!=file.size()||QString::fromLatin1(QCryptographicHash::hash(bytes,QCryptographicHash::Sha256).toHex())!=sha256)throw ExchangeError("PointCloud current-field file hash mismatch");
    QJsonParseError error;auto document=QJsonDocument::fromJson(bytes,&error);if(error.error!=QJsonParseError::NoError||!document.isObject())throw ExchangeError("Invalid PointCloud current-field JSON");auto fields=document.object();const QStringList keys{"schema_version","points","normals","colors","values","ordered","has_plane","plane"};
    if(fields.size()!=keys.size())throw ExchangeError("PointCloud overlay requires complete known current fields");for(auto key:fields.keys())if(!keys.contains(key))throw ExchangeError("Unknown PointCloud current field");if(!fields["schema_version"].isDouble()||fields["schema_version"].toDouble()!=1)throw ExchangeError("Unsupported PointCloud current-field schema");
    for(auto key:{"points","normals","colors","values","plane"})if(!fields[key].isArray())throw ExchangeError("PointCloud current field requires an array");for(auto key:{"ordered","has_plane"})if(!fields[key].isBool())throw ExchangeError("PointCloud current flag requires a boolean");
    auto points=fields["points"].toArray(),normals=fields["normals"].toArray(),colors=fields["colors"].toArray(),values=fields["values"].toArray(),plane=fields["plane"].toArray();auto count=points.size();
    if(count<=0||count>2147483647||(normals.size()!=0&&normals.size()!=count)||(colors.size()!=0&&colors.size()!=4*count)||(values.size()!=0&&values.size()!=count)||plane.size()!=16)throw ExchangeError("Invalid PointCloud current array count");
    auto number=[](const QJsonValue& value){if(!value.isDouble()||!std::isfinite(value.toDouble()))throw ExchangeError("Invalid PointCloud current numeric field");return value.toDouble();};
    ON_PointCloud edited=*cloud;edited.m_P.Empty();edited.m_N.Empty();edited.m_C.Empty();edited.m_V.Empty();
    for(auto value:points){if(!value.isArray()||value.toArray().size()!=3)throw ExchangeError("PointCloud point requires3 doubles");auto row=value.toArray();edited.m_P.Append(ON_3dPoint(number(row[0]),number(row[1]),number(row[2])));}
    for(auto value:normals){if(!value.isArray()||value.toArray().size()!=3)throw ExchangeError("PointCloud normal requires3 doubles");auto row=value.toArray();edited.m_N.Append(ON_3dVector(number(row[0]),number(row[1]),number(row[2])));}
    for(qsizetype i=0;i<colors.size();i+=4){int rgba[4];for(int j=0;j<4;++j){auto value=number(colors[i+j]);if(value<0||value>255||value!=std::floor(value))throw ExchangeError("PointCloud color requires integer RGBA bytes");rgba[j]=static_cast<int>(value);}ON_Color color;color.SetRGBA(rgba[0],rgba[1],rgba[2],rgba[3]);edited.m_C.Append(color);}
    for(auto value:values)edited.m_V.Append(number(value));for(int i=0;i<16;++i)number(plane[i]);
    edited.m_plane.origin=ON_3dPoint(plane[0].toDouble(),plane[1].toDouble(),plane[2].toDouble());edited.m_plane.xaxis=ON_3dVector(plane[3].toDouble(),plane[4].toDouble(),plane[5].toDouble());edited.m_plane.yaxis=ON_3dVector(plane[6].toDouble(),plane[7].toDouble(),plane[8].toDouble());edited.m_plane.zaxis=ON_3dVector(plane[9].toDouble(),plane[10].toDouble(),plane[11].toDouble());for(int i=0;i<4;++i)edited.m_plane.plane_equation[i]=plane[12+i].toDouble();edited.SetOrdered(fields["ordered"].toBool());if(fields["has_plane"].toBool())edited.m_flags|=2U;else edited.m_flags&=~2U;
    validatePointCloud(edited);auto original=cloud->m_P.Count()?pointCloudFields(*cloud):QJsonObject{};if(original["points"]!=fields["points"]){if(edited.m_P.Count()!=cloud->m_P.Count()&&cloud->HiddenPointCount())throw ExchangeError("PointCloud runtime hidden indices require explicit remapping");edited.m_bbox.Destroy();}
    if(pointCloudFields(edited)!=fields)throw ExchangeError("PointCloud native field result differs from current payload");
    if(original!=fields)*cloud=edited;
}
}
