// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballOverlay.h"
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Gui/Inventor/Draggers/SoTransformDragger.h>
#include <Gui/Inventor/Draggers/SoLinearDragger.h>
#include <Gui/Inventor/Draggers/SoRotationDragger.h>
#include <Gui/Inventor/Draggers/SoPlanarDragger.h>
#include <Inventor/actions/SoRayPickAction.h>
#include <Inventor/SoPickedPoint.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoAnnotation.h>
#include <Inventor/nodes/SoTransform.h>
#include <Inventor/nodes/SoTranslation.h>
#include <Inventor/nodes/SoScale.h>
#include <Inventor/nodes/SoCube.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoPointSet.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <QLineEdit>
#include <QJsonObject>
#include <cmath>
#include <stdexcept>
namespace OpenMatrix9Gui {
GumballOverlay::~GumballOverlay(){detach();}
void GumballOverlay::attach(Gui::View3DInventor* native){
    if(view==native&&root&&owner&&owner->findChild(root)>=0&&owner==native->getViewer()->getSceneGraph())return;detach();if(!native)return;
    auto* scene=dynamic_cast<SoSeparator*>(native->getViewer()->getSceneGraph());if(!scene)return;
    view=native;view->getViewer()->viewport()->setProperty("om9GumballViewport",true);
    owner=scene;owner->ref();root=new SoSeparator;root->ref();root->setName("OM9GumballOverlay");
    // This overlay deliberately has no model bounds. Coin must not use the
    // cached empty box from fitAll() to cull its rendering or native picking.
    root->pickCulling=SoSeparator::OFF;root->renderCulling=SoSeparator::OFF;owner->addChild(root);
    auto* skip=new Gui::SoSkipBoundingGroup;skip->mode=Gui::SoSkipBoundingGroup::EXCLUDE_BBOX;root->addChild(skip);auto* annotation=new SoAnnotation;skip->addChild(annotation);
    pose=new SoTransform;annotation->addChild(pose);dragger=new Gui::SoTransformDragger;annotation->addChild(dragger);
    dragger->setAxisColors(0xff3030ff,0x30cf50ff,0xffdd20ff);
    dragger->showPlanarTranslationXY();dragger->showPlanarTranslationYZ();dragger->showPlanarTranslationZX();
    dragger->showRotationX();dragger->showRotationY();dragger->showRotationZ();
    const char* parts[]={"xTranslatorDragger","yTranslatorDragger","zTranslatorDragger","xyPlanarTranslatorDragger","yzPlanarTranslatorDragger","zxPlanarTranslatorDragger","","xRotatorDragger","yRotatorDragger","zRotatorDragger"};
    for(unsigned i=0;i<10;++i)if(i!=6)handles[i]=dragger->getPart(parts[i],true);
    // FreeCAD's native geometry is authored with 10-unit arrows and 8-unit
    // arcs. Normalize those visual units before applying the camera size.
    for(unsigned i=0;i<3;++i)dynamic_cast<Gui::SoLinearDraggerContainer*>(handles[i])->getDragger()->geometryScale.setValue(.1f,.1f,.1f);
    for(unsigned i=7;i<10;++i)dynamic_cast<Gui::SoRotationDraggerContainer*>(handles[i])->getDragger()->geometryScale.setValue(.1f,.1f,.1f);
    // Native planar squares are two visual units wide and offset by 2.15.
    // Wrap geometry parts on our own instances; do not depend on the
    // transform dragger's internal catalog layout. Defaults stay read-only.
    for(unsigned i=3;i<6;++i)for(const char* name:{"planarTranslator","planarTranslatorActive"}){
        auto* planar=dynamic_cast<Gui::SoPlanarDragger*>(handles[i]);
        auto* geometry=planar?planar->getPart(name,true):nullptr;
        if(!geometry)throw std::runtime_error("Native planar gumball geometry is unavailable");
        auto* separator=new SoSeparator;auto* scale=new SoScale;
        scale->scaleFactor.setValue(.15f,.15f,.15f);separator->addChild(scale);separator->addChild(geometry);
        if(!planar->setPart(name,separator))throw std::runtime_error("Cannot attach native planar gumball geometry");
    }
    auto* extras=new SoSeparator;annotation->addChild(extras);extraScale=new SoScale;extras->addChild(extraScale);
    for(unsigned i=0;i<4;++i){auto* item=new SoSeparator;extras->addChild(item);auto* color=new SoBaseColor;color->rgb.setValue(i==1?1.f:i==2?0.2f:1.f,i==1?0.2f:i==2?0.8f:0.85f,i?0.1f:1.f);item->addChild(color);auto* translation=new SoTranslation;if(i)translation->translation.setValue(i==1?1.35f:0.f,i==2?1.35f:0.f,i==3?1.35f:0.f);item->addChild(translation);auto* cube=new SoCube;cube->width=cube->height=cube->depth=i?0.12f:0.10f;item->addChild(cube);handles[i?9+i:6]=item;}
    previewNode=new SoSeparator;skip->addChild(previewNode);
}
void GumballOverlay::detach(){closeEditor();if(view)view->getViewer()->viewport()->setProperty("om9GumballViewport",false);if(owner){if(root&&owner->findChild(root)>=0)owner->removeChild(root);owner->unref();}if(root)root->unref();view=nullptr;owner=root=previewNode=nullptr;pose=nullptr;dragger=nullptr;extraScale=nullptr;handles.fill(nullptr);}
void GumballOverlay::render(const Om9GumballSnapshot& s){
    if(!view||!root)return;auto* viewer=view->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return;
    pose->translation.setValue(float(s.frame[0]),float(s.frame[1]),float(s.frame[2]));
    SbMatrix basis;basis.makeIdentity();for(unsigned r=0;r<3;++r)for(unsigned c=0;c<3;++c)basis[r][c]=float(s.frame[3+3*r+c]);pose->rotation=SbRotation(basis);
    // Size is measured in logical viewport pixels; Coin floats are render-only.
    const float size=80.f/std::max(1,viewer->viewport()->height());dragger->draggerSize=size;
    dragger->setUpAutoScale(camera);const auto volume=camera->getViewVolume();
    const float scale=volume.getWorldToScreenScale(pose->translation.getValue(),size/2);extraScale->scaleFactor.setValue(scale,scale,scale);viewer->redraw();
}
HandlePick GumballOverlay::ray(const QPoint& p)const{
    HandlePick pick{};pick.screen=p;if(!view)return pick;auto* viewer=view->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return pick;
    const auto size=viewer->viewport()->size();const float width=std::max(1,size.width()),height=std::max(1,size.height());
    SbVec3f a,b;camera->getViewVolume(width/height).projectPointToLine(SbVec2f(p.x()/width,(height-1-p.y())/height),a,b);
    auto d=b-a;for(unsigned i=0;i<3;++i){pick.origin[i]=a[i];pick.direction[i]=d[i];}return pick;
}
std::optional<HandlePick> GumballOverlay::hit(const QPoint& p)const{
    if(!view||!root)return {};auto* viewer=view->getViewer();
    auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return {};
    const auto size=viewer->viewport()->size();const float width=std::max(1,size.width()),height=std::max(1,size.height());
    const auto ratio=viewer->devicePixelRatioF();SbViewportRegion viewport(short(width*ratio),short(height*ratio));
    // An owned query scene uses the active camera and overlay. Other host
    // viewports, clipping and temporary navigation pick styles cannot leak in.
    auto* scene=new SoSeparator;scene->ref();scene->addChild(camera);scene->addChild(root);
    SoRayPickAction pick(viewport);pick.setNormalizedPoint(SbVec2f(p.x()/width,(height-1-p.y())/height));pick.setRadius(float(3*ratio));pick.setPickAll(true);pick.apply(scene);scene->unref();
    // Resolve actual native paths to owned handle nodes.
    const auto& list=pick.getPickedPointList();const unsigned order[]={6,10,11,12,0,1,2,3,4,5,7,8,9};for(auto i:order)for(int j=0;j<list.getLength();++j){auto* path=list[j]->getPath();if(handles[i]&&path->containsNode(handles[i])){auto result=ray(p);result.handle=i;return result;}}
    return {};
}
QJsonArray GumballOverlay::handlePositions()const{
    QJsonArray rows;if(!view||!root)return rows;auto* viewer=view->getViewer();const auto size=viewer->viewport()->size();const char* names[]={"MoveX","MoveY","MoveZ","MoveXY","MoveYZ","MoveZX","MoveFree","RotateX","RotateY","RotateZ","ScaleX","ScaleY","ScaleZ"};
    // Read native hit paths on a bounded grid around the widget; reports are
    // targets for actual QTest events, and cannot alter the session or objects.
    std::array<std::optional<QPoint>,13> found;SbVec3f center;auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return rows;camera->getViewVolume(float(size.width())/std::max(1,size.height())).projectToScreen(pose->translation.getValue(),center);const QPoint origin(int(center[0]*(size.width()-1)),int((1-center[1])*(size.height()-1)));
    for(int y=-130;y<=130;y+=3)for(int x=-130;x<=130;x+=3){const QPoint p=origin+QPoint(x,y);if(!viewer->viewport()->rect().contains(p))continue;auto picked=hit(p);if(picked&&!found[picked->handle])found[picked->handle]=p;}
    for(unsigned i=0;i<13;++i)if(found[i])rows.append(QJsonObject{{"name",names[i]},{"handle",int(i)},{"x",found[i]->x()},{"y",found[i]->y()},{"hit_verified",true}});return rows;
}
QJsonObject GumballOverlay::diagnostics()const{
    return {{"attached",bool(view&&root)},{"current_scene",bool(view&&owner&&owner==view->getViewer()->getSceneGraph())},{"in_owner",bool(owner&&root&&owner->findChild(root)>=0)}};
}
QLineEdit* GumballOverlay::numericEditor(const HandlePick& pick,bool copy){
    closeEditor();if(!view)return nullptr;field=new QLineEdit(view->getViewer()->viewport());field->setObjectName("OM9GumballNumeric");field->setPlaceholderText(pick.handle>=10?"Scale factor":pick.handle>=7?"Angle (deg)":"Distance");field->setToolTip(copy?"Copy · Enter to accept · Esc to close":"Enter to accept · Esc to close");field->resize(190,28);field->move(pick.screen+QPoint(12,12));field->show();field->setFocus();return field;
}
void GumballOverlay::closeEditor(){if(field){field->hide();field->deleteLater();field=nullptr;}}
QLineEdit* GumballOverlay::editor()const{return field;}
void GumballOverlay::clearPreview(){if(previewNode)previewNode->removeAllChildren();if(view)view->getViewer()->redraw();}
void GumballOverlay::preview(const std::vector<std::vector<Base::Vector3d>>& lines){
    clearPreview();if(!previewNode)return;
    auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;previewNode->addChild(pick);
    auto* color=new SoBaseColor;color->rgb.setValue(0.2f,1.f,0.8f);previewNode->addChild(color);
    auto* style=new SoDrawStyle;style->lineWidth=2;style->pointSize=4;previewNode->addChild(style);
    auto* coords=new SoCoordinate3;auto* segments=new SoLineSet;
    std::vector<SbVec3f> points,singletons;unsigned line=0;
    for(const auto& segment:lines){
        if(segment.size()==1){auto p=segment.front();singletons.emplace_back(float(p.x),float(p.y),float(p.z));continue;}
        if(segment.empty())continue;
        for(auto p:segment)points.emplace_back(float(p.x),float(p.y),float(p.z));
        segments->numVertices.set1Value(line++,int(segment.size()));
    }
    if(!points.empty())coords->point.setValues(0,int(points.size()),points.data());
    previewNode->addChild(coords);previewNode->addChild(segments);
    if(!singletons.empty()){
        auto* dots=new SoSeparator;auto* positions=new SoCoordinate3;
        positions->point.setValues(0,int(singletons.size()),singletons.data());dots->addChild(positions);
        auto* shape=new SoPointSet;shape->numPoints=int(singletons.size());dots->addChild(shape);previewNode->addChild(dots);
    }
    if(view)view->getViewer()->redraw();
}
}
