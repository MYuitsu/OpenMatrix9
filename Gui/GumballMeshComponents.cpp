// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballMeshComponents.h"
#include "CoreLayers.h"
#include <App/Document.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Mod/Mesh/App/Mesh.h>
#include <Mod/Mesh/App/MeshFeature.h>
#include <Inventor/SoPickedPoint.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/actions/SoRayPickAction.h>
#include <Inventor/details/SoFaceDetail.h>
#include <Inventor/nodes/SoAnnotation.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <Inventor/nodes/SoLightModel.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoPointSet.h>
#include <Inventor/nodes/SoSeparator.h>
#include <QApplication>
#include <QThread>
#include <QWidget>
#include <algorithm>
#include <cmath>
#include <limits>

namespace OpenMatrix9Gui {
namespace {
constexpr std::size_t pointBudget=131072, facetBudget=131072;
bool finite(const Base::Vector3d& p) {
    return std::isfinite(p.x)&&std::isfinite(p.y)&&std::isfinite(p.z)
        &&std::abs(p.x)<=1e12&&std::abs(p.y)<=1e12&&std::abs(p.z)<=1e12;
}
struct ScreenPoint { double x,y; };
double distanceSquared(ScreenPoint a,ScreenPoint b) {
    const double x=a.x-b.x,y=a.y-b.y;return x*x+y*y;
}
double edgeDistanceSquared(ScreenPoint p,ScreenPoint a,ScreenPoint b) {
    const double x=b.x-a.x,y=b.y-a.y,denominator=x*x+y*y;
    if(denominator<=1e-12)return std::numeric_limits<double>::infinity();
    const double t=std::clamp(((p.x-a.x)*x+(p.y-a.y)*y)/denominator,0.0,1.0);
    return distanceSquared(p,{a.x+t*x,a.y+t*y});
}
}

std::optional<GumballMeshComponentPick> pickGumballMeshComponent(
    Gui::View3DInventor* view,const QPoint& position,double radius) {
    if(!qApp||QThread::currentThread()!=qApp->thread()||!view
       ||!std::isfinite(radius)||radius<=0||radius>1024)return {};
    auto* viewer=view->getViewer();if(!viewer||!viewer->viewport())return {};
    auto* port=viewer->viewport();if(!port->rect().contains(position))return {};
    auto* manager=viewer->getSoRenderManager();
    auto* camera=manager?manager->getCamera():nullptr;
    auto* scene=manager?manager->getSceneGraph():nullptr;
    const int width=port->width(),height=port->height();
    const double dpr=port->devicePixelRatioF();
    if(!camera||!scene||width<=0||height<=0||!std::isfinite(dpr)||dpr<=0
       ||width*dpr>32767||height*dpr>32767)return {};

    // The real scene preserves visibility, clipping, parent transforms and
    // occlusion. A nearer non-Mesh object must not permit picking through it.
    SoRayPickAction action{SbViewportRegion(short(width*dpr),short(height*dpr))};
    action.setNormalizedPoint(SbVec2f(float(position.x())/width,float(height-1-position.y())/height));
    action.setRadius(float(radius*dpr));action.setPickAll(false);action.apply(scene);
    const auto* picked=action.getPickedPoint();
    if(!picked||!picked->isOnGeometry()||!picked->getPath())return {};
    auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(
        viewer->getViewProviderByPathFromTail(picked->getPath()));
    auto* owner=provider?dynamic_cast<Mesh::Feature*>(provider->getObject()):nullptr;
    if(!owner||!owner->getDocument()||!CoreLayers::objectEditable(owner))return {};
    const auto* detail=dynamic_cast<const SoFaceDetail*>(picked->getDetail());
    if(!detail||detail->getNumPoints()!=3||detail->getFaceIndex()<0)return {};
    const auto& mesh=owner->Mesh.getValue();
    if(mesh.countPoints()>pointBudget||mesh.countFacets()>facetBudget)return {};
    const auto& kernel=mesh.getKernel();
    const auto& points=kernel.GetPoints();const auto& facets=kernel.GetFacets();
    const auto facet=std::size_t(detail->getFaceIndex());
    if(points.empty()||facet>=facets.size())return {};
    // Bounded native observations are validated before constructing any
    // owned identity/geometry. Portable frame mathematics remains in Rust.
    for(const auto& p:points)if(!finite(Base::Vector3d(p.x,p.y,p.z)))return {};
    for(const auto& f:facets){
        if(f._aulPoints[0]>=points.size()||f._aulPoints[1]>=points.size()||f._aulPoints[2]>=points.size()
           ||f._aulPoints[0]==f._aulPoints[1]||f._aulPoints[1]==f._aulPoints[2]
           ||f._aulPoints[2]==f._aulPoints[0])return {};
    }
    std::array<std::uint32_t,3> ids{};
    std::array<Base::Vector3d,3> world{Base::Vector3d(),Base::Vector3d(),Base::Vector3d()};
    std::array<ScreenPoint,3> screen{};
    const auto& transform=picked->getObjectToWorld();
    const auto volume=camera->getViewVolume(float(width)/height);
    for(unsigned i=0;i<3;++i){
        const auto* pointDetail=detail->getPoint(int(i));
        if(!pointDetail||pointDetail->getCoordinateIndex()<0)return {};
        const auto id=std::size_t(pointDetail->getCoordinateIndex());
        // Require the exact native facet winding, not guessed display indices.
        if(id!=facets[facet]._aulPoints[i])return {};
        ids[i]=std::uint32_t(id);const auto& p=points[id];
        SbVec3f w;transform.multVecMatrix(SbVec3f(p.x,p.y,p.z),w);
        world[i]=Base::Vector3d(w[0],w[1],w[2]);if(!finite(world[i]))return {};
        SbVec3f projected;volume.projectToScreen(w,projected);
        if(!std::isfinite(projected[0])||!std::isfinite(projected[1])
           ||!std::isfinite(projected[2])||projected[2]<0||projected[2]>1)return {};
        screen[i]={double(projected[0])*width,(1.0-double(projected[1]))*height-1.0};
    }
    GumballMeshComponentPick result;
    result.documentName=owner->getDocument()->getName();result.objectName=owner->getNameInDocument();
    result.facet=std::uint32_t(facet);
    const ScreenPoint query{double(position.x()),double(position.y())};
    const double threshold=radius*radius;
    double best=std::numeric_limits<double>::infinity();unsigned closest=0;
    for(unsigned i=0;i<3;++i){
        const double distance=distanceSquared(query,screen[i]);
        if(distance<best-1e-9||(std::abs(distance-best)<=1e-9&&ids[i]<ids[closest])){best=distance;closest=i;}
    }
    if(best<=threshold){
        result.kind=GumballMeshComponentKind::Vertex;result.first=ids[closest];
        result.subelement="OM9MeshVertex:"+std::to_string(result.first);
        result.worldHighlightPoints[0]=world[closest];result.highlightPointCount=1;return result;
    }
    best=std::numeric_limits<double>::infinity();unsigned first=0,second=1;
    for(unsigned i=0;i<3;++i){
        unsigned a=i,b=(i+1)%3;if(ids[a]>ids[b])std::swap(a,b);
        const double distance=edgeDistanceSquared(query,screen[a],screen[b]);
        const bool earlier=ids[a]<ids[first]||(ids[a]==ids[first]&&ids[b]<ids[second]);
        if(distance<best-1e-9||(std::abs(distance-best)<=1e-9&&earlier)){best=distance;first=a;second=b;}
    }
    if(best<=threshold){
        result.kind=GumballMeshComponentKind::Edge;result.first=ids[first];result.second=ids[second];
        result.subelement="OM9MeshEdge:"+std::to_string(result.first)+":"+std::to_string(result.second);
        result.worldHighlightPoints={world[first],world[second]};result.highlightPointCount=2;
    }
    return result;
}

void updateGumballMeshComponentHighlight(SoSeparator* root,
    const GumballMeshComponentPick* pick,double dpr) {
    if(!root)return;root->removeAllChildren();root->setName("OM9MeshComponentHighlight");
    if(!pick||pick->kind==GumballMeshComponentKind::WholeMesh
       ||pick->highlightPointCount!=(pick->kind==GumballMeshComponentKind::Vertex?1u:2u)
       ||!std::isfinite(dpr)||dpr<=0)return;
    for(unsigned i=0;i<pick->highlightPointCount;++i)if(!finite(pick->worldHighlightPoints[i]))return;
    auto* skip=new Gui::SoSkipBoundingGroup;skip->mode=Gui::SoSkipBoundingGroup::EXCLUDE_BBOX;root->addChild(skip);
    auto* annotation=new SoAnnotation;skip->addChild(annotation);
    auto* noPick=new SoPickStyle;noPick->style=SoPickStyle::UNPICKABLE;annotation->addChild(noPick);
    auto* light=new SoLightModel;light->model=SoLightModel::BASE_COLOR;annotation->addChild(light);
    auto* color=new SoBaseColor;color->rgb.setValue(1.f,.85f,.15f);annotation->addChild(color);
    auto* style=new SoDrawStyle;style->lineWidth=float(3*std::min(dpr,8.0));
    style->pointSize=float(7*std::min(dpr,8.0));annotation->addChild(style);
    auto* coordinates=new SoCoordinate3;
    for(unsigned i=0;i<pick->highlightPointCount;++i){
        const auto& p=pick->worldHighlightPoints[i];coordinates->point.set1Value(int(i),float(p.x),float(p.y),float(p.z));
    }
    annotation->addChild(coordinates);
    if(pick->kind==GumballMeshComponentKind::Vertex){
        auto* marker=new SoPointSet;marker->numPoints=1;annotation->addChild(marker);
    }else{
        auto* edge=new SoLineSet;edge->numVertices=2;annotation->addChild(edge);
    }
}
}
