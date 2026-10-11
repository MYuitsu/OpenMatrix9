// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballOverlay.h"
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Inventor/actions/SoRayPickAction.h>
#include <Inventor/SoPickedPoint.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoAnnotation.h>
#include <Inventor/nodes/SoTransform.h>
#include <Inventor/nodes/SoTranslation.h>
#include <Inventor/nodes/SoRotation.h>
#include <Inventor/nodes/SoCone.h>
#include <Inventor/nodes/SoLightModel.h>
#include <Inventor/nodes/SoFaceSet.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoPointSet.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <QLineEdit>
#include <QJsonObject>
#include <QFontMetrics>
#include <algorithm>
#include <cmath>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
// Rendering-only Coin geometry. Rust still owns every operation/frame/input.
SbVec3f along(unsigned axis,float distance){SbVec3f p(0,0,0);p[axis]=distance;return p;}
SbColor handleColor(unsigned handle){
    if(handle==6)return SbColor(.9f,.9f,.9f);
    const unsigned axis=handle<3?handle:handle<6?(handle+2)%3:handle<10?handle-7:handle-10;
    const SbColor palette[]={SbColor(1.f,.18f,.18f),SbColor(.18f,.85f,.25f),SbColor(.18f,.35f,1.f)};
    return palette[axis];
}
void polyline(SoSeparator* parent,const std::vector<SbVec3f>& points){
    auto* coordinates=new SoCoordinate3;coordinates->point.setValues(0,int(points.size()),points.data());
    auto* line=new SoLineSet;line->numVertices.setValue(int(points.size()));
    parent->addChild(coordinates);parent->addChild(line);
}
void square(SoSeparator* parent,float half){
    polyline(parent,{{-half,-half,0},{half,-half,0},{half,half,0},{-half,half,0},{-half,-half,0}});
}
}
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
    auto* widget=new SoSeparator;annotation->addChild(widget);pose=new SoTransform;widget->addChild(pose);
    auto* unlit=new SoLightModel;unlit->model=SoLightModel::BASE_COLOR;widget->addChild(unlit);
    for(unsigned i=0;i<13;++i){
        auto* item=new SoSeparator;widget->addChild(item);handles[i]=item;
        colors[i]=new SoBaseColor;colors[i]->rgb.setValue(handleColor(i));item->addChild(colors[i]);
        auto* style=new SoDrawStyle;style->lineWidth=1.6f;item->addChild(style);
        if(i<3){
            polyline(item,{along(i,0),along(i,1.f)});
            auto* tip=new SoSeparator;item->addChild(tip);
            auto* offset=new SoTranslation;offset->translation=along(i,1.055f);tip->addChild(offset);
            auto* rotation=new SoRotation;rotation->rotation=SbRotation(SbVec3f(0,1,0),along(i,1));tip->addChild(rotation);
            auto* cone=new SoCone;cone->bottomRadius=.065f;cone->height=.19f;tip->addChild(cone);
        }else if(i<6){
            const unsigned a=i==3?0:i==4?1:2,b=(a+1)%3;
            auto p=[a,b](float x,float y){return along(a,x)+along(b,y);};
            polyline(item,{p(.22f,.22f),p(.45f,.22f),p(.45f,.45f),p(.22f,.45f),p(.22f,.22f)});
        }else if(i==6){
            facing[0]=new SoRotation;item->addChild(facing[0]);square(item,.055f);
        }else if(i<10){
            const unsigned axis=i-7,a=(axis+1)%3,b=(axis+2)%3;
            std::vector<SbVec3f> arc;
            for(unsigned step=0;step<=32;++step){const float angle=float(step)*1.57079632679f/32;arc.push_back(along(a,-.82f*std::cos(angle))+along(b,-.82f*std::sin(angle)));}
            polyline(item,arc);
        }else{
            const unsigned axis=i-10;
            // Scale is on the negative side; its dashed guide is not a target.
            auto* guide=new SoSeparator;item->addChild(guide);auto* noPick=new SoPickStyle;noPick->style=SoPickStyle::UNPICKABLE;guide->addChild(noPick);
            for(float d=.12f;d<1.22f;d+=.09f)polyline(guide,{along(axis,-d),along(axis,-d-.04f)});
            auto* offset=new SoTranslation;offset->translation=along(axis,-1.35f);item->addChild(offset);
            facing[axis+1]=new SoRotation;item->addChild(facing[axis+1]);square(item,.10f);
        }
    }
    // Menu placement follows screen right/up, independent of the model frame.
    auto* menu=new SoSeparator;annotation->addChild(menu);handles[13]=menu;
    menuPose=new SoTransform;menu->addChild(menuPose);
    auto* menuLight=new SoLightModel;menuLight->model=SoLightModel::BASE_COLOR;menu->addChild(menuLight);
    std::vector<SbVec3f> circle;for(unsigned i=0;i<=32;++i){const float angle=float(i)*6.28318530718f/32;circle.emplace_back(.085f*std::cos(angle),.085f*std::sin(angle),0);}
    auto* fill=new SoBaseColor;fill->rgb.setValue(.85f,.85f,.85f);menu->addChild(fill);
    auto* coordinates=new SoCoordinate3;coordinates->point.setValues(0,int(circle.size()-1),circle.data());menu->addChild(coordinates);
    auto* disc=new SoFaceSet;disc->numVertices.setValue(int(circle.size()-1));menu->addChild(disc);
    auto* border=new SoBaseColor;border->rgb.setValue(.25f,.25f,.25f);menu->addChild(border);
    auto* menuStyle=new SoDrawStyle;menuStyle->lineWidth=1.6f;menu->addChild(menuStyle);polyline(menu,circle);
    previewNode=new SoSeparator;skip->addChild(previewNode);
}
void GumballOverlay::detach(){closeEditor();if(view)view->getViewer()->viewport()->setProperty("om9GumballViewport",false);if(owner){if(root&&owner->findChild(root)>=0)owner->removeChild(root);owner->unref();}if(root)root->unref();view=nullptr;owner=root=previewNode=nullptr;pose=menuPose=nullptr;handles.fill(nullptr);colors.fill(nullptr);facing.fill(nullptr);}
void GumballOverlay::render(const Om9GumballSnapshot& s){
    if(!view||!root)return;auto* viewer=view->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return;
    pose->translation.setValue(float(s.frame[0]),float(s.frame[1]),float(s.frame[2]));
    SbMatrix basis;basis.makeIdentity();for(unsigned r=0;r<3;++r)for(unsigned c=0;c<3;++c)basis[r][c]=float(s.frame[3+3*r+c]);pose->rotation=SbRotation(basis);
    // Size is measured in logical viewport pixels; Coin floats are render-only.
    const float size=80.f/std::max(1,viewer->viewport()->height());const auto volume=camera->getViewVolume();
    const float scale=volume.getWorldToScreenScale(pose->translation.getValue(),size/2);pose->scaleFactor.setValue(scale,scale,scale);
    // Coin quaternion multiplication applies the left operand first.
    const auto screenFacing=camera->orientation.getValue()*pose->rotation.getValue().inverse();
    for(auto* rotation:facing)rotation->rotation=screenFacing;
    SbVec3f menuOffset;camera->orientation.getValue().multVec(SbVec3f(-.55f*scale,-1.05f*scale,0),menuOffset);
    menuPose->translation=pose->translation.getValue()+menuOffset;menuPose->rotation=camera->orientation.getValue();menuPose->scaleFactor.setValue(scale,scale,scale);
    for(unsigned i=0;i<13;++i)colors[i]->rgb.setValue(s.phase>=2&&s.phase<=3&&s.handle==i?SbColor(1.f,.9f,.25f):handleColor(i));
    viewer->redraw();
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
    const auto& list=pick.getPickedPointList();const unsigned order[]={13,6,10,11,12,0,1,2,3,4,5,7,8,9};for(auto i:order)for(int j=0;j<list.getLength();++j){auto* path=list[j]->getPath();if(handles[i]&&path->containsNode(handles[i])){auto result=ray(p);result.handle=i;return result;}}
    return {};
}
QJsonArray GumballOverlay::handlePositions()const{
    QJsonArray rows;if(!view||!root)return rows;auto* viewer=view->getViewer();const auto size=viewer->viewport()->size();const char* names[]={"MoveX","MoveY","MoveZ","MoveXY","MoveYZ","MoveZX","MoveFree","RotateX","RotateY","RotateZ","ScaleX","ScaleY","ScaleZ","MenuBall"};
    // Read native hit paths on a bounded grid around the widget; reports are
    // targets for actual QTest events, and cannot alter the session or objects.
    std::array<std::optional<QPoint>,14> found;SbVec3f center;auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return rows;camera->getViewVolume(float(size.width())/std::max(1,size.height())).projectToScreen(pose->translation.getValue(),center);const QPoint origin(int(center[0]*(size.width()-1)),int((1-center[1])*(size.height()-1)));
    for(int y=-130;y<=130;y+=3)for(int x=-130;x<=130;x+=3){const QPoint p=origin+QPoint(x,y);if(!viewer->viewport()->rect().contains(p))continue;auto picked=hit(p);if(picked&&!found[picked->handle])found[picked->handle]=p;}
    for(unsigned i=0;i<14;++i)if(found[i])rows.append(QJsonObject{{"name",names[i]},{"handle",int(i)},{"x",found[i]->x()},{"y",found[i]->y()},{"hit_verified",true}});return rows;
}
QJsonObject GumballOverlay::diagnostics()const{
    return {{"attached",bool(view&&root)},{"current_scene",bool(view&&owner&&owner==view->getViewer()->getSceneGraph())},{"in_owner",bool(owner&&root&&owner->findChild(root)>=0)}};
}
QLineEdit* GumballOverlay::numericEditor(const HandlePick& pick,bool copy){
    closeEditor();if(!view)return nullptr;auto* viewport=view->getViewer()->viewport();field=new QLineEdit(viewport);field->setObjectName("OM9GumballNumeric");field->setPlaceholderText(pick.handle>=10?"Scale factor":pick.handle>=7?"Angle (deg)":"Distance");field->setToolTip(copy?"Copy · Enter to accept · Esc to close":"Enter to accept · Esc to close");
    field->setStyleSheet("QLineEdit { background: white; color: black; border: 1px solid #888; padding: 1px 4px; }");
    auto fit=[this,viewport,pick]{const auto text=field->text().isEmpty()?field->placeholderText():field->text();const auto metrics=field->fontMetrics();field->resize(std::clamp(metrics.horizontalAdvance(text)+18,80,140),metrics.height()+6);const auto p=pick.screen+QPoint(8,8);field->move(std::clamp(p.x(),0,std::max(0,viewport->width()-field->width())),std::clamp(p.y(),0,std::max(0,viewport->height()-field->height())));};
    QObject::connect(field,&QLineEdit::textChanged,field,fit);fit();field->show();field->setFocus();return field;
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
