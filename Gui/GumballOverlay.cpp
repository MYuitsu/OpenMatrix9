// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballOverlay.h"
#include "GumballMeshComponents.h"
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Inventor/actions/SoRayPickAction.h>
#include <Inventor/actions/SoSearchAction.h>
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
#include <Inventor/nodes/SoPerspectiveCamera.h>
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
void GumballOverlay::componentHighlight(const GumballMeshComponentPick* pick){if(componentNode)updateGumballMeshComponentHighlight(componentNode,pick,view?view->getViewer()->viewport()->devicePixelRatioF():1.0);}
void GumballOverlay::attach(Gui::View3DInventor* native){
    if(view==native&&root&&owner&&owner->findChild(root)>=0&&owner==native->getViewer()->getSceneGraph())return;
    detach();if(!native)return;
    auto* scene=dynamic_cast<SoSeparator*>(native->getViewer()->getSceneGraph());if(!scene)return;
    view=native;view->getViewer()->viewport()->setProperty("om9GumballViewport",true);
    owner=scene;owner->ref();root=new SoSeparator;root->ref();root->setName("OM9GumballOverlay");
    root->pickCulling=SoSeparator::OFF;root->renderCulling=SoSeparator::OFF;owner->addChild(root);
    auto* skip=new Gui::SoSkipBoundingGroup;skip->mode=Gui::SoSkipBoundingGroup::EXCLUDE_BBOX;root->addChild(skip);
    auto* annotation=new SoAnnotation;skip->addChild(annotation);widgetNode=new SoSeparator;annotation->addChild(widgetNode);
    previewNode=new SoSeparator;skip->addChild(previewNode);
    componentNode=new SoSeparator;skip->addChild(componentNode);
}
void GumballOverlay::detach(){
    closeEditor();if(view)view->getViewer()->viewport()->setProperty("om9GumballViewport",false);
    if(owner){if(root&&owner->findChild(root)>=0)owner->removeChild(root);owner->unref();}
    if(root)root->unref();view=nullptr;owner=root=previewNode=widgetNode=componentNode=nullptr;pose=menuPose=nullptr;
    handles.fill(nullptr);colors.fill(nullptr);facing.fill(nullptr);
}
void GumballOverlay::render(const Om9GumballSnapshot& s,std::uint64_t session,const Om9GumballSettings& configuration,const std::vector<std::array<double,3>>& points){
    if(!view||!root||points.empty())return;auto* viewer=view->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return;
    settings=configuration;
    SbVec3f origin(float(s.frame[0]),float(s.frame[1]),float(s.frame[2]));
    const float pixel=camera->getViewVolume().getWorldToScreenScale(origin,1.f/std::max(1,viewer->viewport()->height()));
    Om9GumballProjection projection{};projection.world_per_pixel=pixel;
    auto rotation=camera->orientation.getValue();
    for(unsigned i=0;i<3;++i){SbVec3f v;rotation.multVec(i==0?SbVec3f(1,0,0):i==1?SbVec3f(0,1,0):SbVec3f(0,0,-1),v);auto* out=i==0?projection.right:i==1?projection.up:projection.forward;for(unsigned j=0;j<3;++j)out[j]=v[j];}
    if(om9_gumball_layout(session,&projection,reinterpret_cast<const double(*)[3]>(points.data()),points.size(),&layout))return;
    widgetNode->removeAllChildren();handles.fill(nullptr);colors.fill(nullptr);facing.fill(nullptr);
    const float dpr=float(viewer->viewport()->devicePixelRatioF());
    auto* baseStyle=new SoDrawStyle;baseStyle->lineWidth=1.5f*dpr;widgetNode->addChild(baseStyle);
    auto* axes=new SoSeparator;widgetNode->addChild(axes);pose=new SoTransform;axes->addChild(pose);pose->translation=origin;
    SbMatrix basis;basis.makeIdentity();for(unsigned r=0;r<3;++r)for(unsigned c=0;c<3;++c)basis[r][c]=float(s.frame[3+3*r+c]);pose->rotation=SbRotation(basis);
    auto* light=new SoLightModel;light->model=SoLightModel::BASE_COLOR;axes->addChild(light);
    auto local=[&](const double* world){SbVec3f result;for(unsigned i=0;i<3;++i)result[i]=float((world[0]-s.frame[0])*s.frame[3+3*i]+(world[1]-s.frame[1])*s.frame[4+3*i]+(world[2]-s.frame[2])*s.frame[5+3*i]);return result;};
    auto world=[&](const SbVec3f& point){SbVec3f result;pose->rotation.getValue().multVec(point,result);return origin+result;};
    auto color=[&](unsigned axis){return SbColor(float(settings.colors[3*axis]),float(settings.colors[3*axis+1]),float(settings.colors[3*axis+2]));};
    auto dotted=[&](SoSeparator* parent,SbVec3f a,SbVec3f b){
        auto* guide=new SoSeparator;parent->addChild(guide);auto* noPick=new SoPickStyle;noPick->style=SoPickStyle::UNPICKABLE;guide->addChild(noPick);
        auto* coords=new SoCoordinate3;auto* lines=new SoLineSet;guide->addChild(coords);guide->addChild(lines);
        const unsigned count=std::clamp(unsigned((b-a).length()/std::max(pixel*5.f,1e-20f)),1u,512u);
        for(unsigned i=0;i<count;++i){coords->point.set1Value(2*i,a+(b-a)*(float(i)/count));coords->point.set1Value(2*i+1,a+(b-a)*((i+.3f)/count));lines->numVertices.set1Value(i,2);}
    };
    const auto screenFacing=rotation*pose->rotation.getValue().inverse();
    for(unsigned i=0;i<13;++i){
        if(!layout.visible[i])continue;
        auto* item=new SoSeparator;axes->addChild(item);handles[i]=item;
        colors[i]=new SoBaseColor;const unsigned axis=i<3?i:i<6?(i+2)%3:i<10?i-7:i-10;
        colors[i]->rgb.setValue(i==6?SbColor(1,1,1):s.phase>=2&&s.phase<=3&&s.handle==i?SbColor(1,.9f,.25f):color(axis));item->addChild(colors[i]);
        auto* style=new SoDrawStyle;style->lineWidth=dpr*float(i<3?settings.move_thickness:i>=7&&i<10?settings.rotate_thickness:1.5);item->addChild(style);
        if(i<3){
            const float length=float(settings.move_length)*pixel,head=float(std::min(settings.arrow_length,settings.move_length))*pixel;
            polyline(item,{along(i,0),along(i,length-head)});
            auto* tip=new SoSeparator;item->addChild(tip);auto* offset=new SoTranslation;offset->translation=along(i,length-head/2);tip->addChild(offset);
            auto* orient=new SoRotation;orient->rotation=SbRotation(SbVec3f(0,1,0),along(i,1));tip->addChild(orient);
            auto* cone=new SoCone;cone->bottomRadius=float(settings.arrow_width/2)*pixel;cone->height=head;tip->addChild(cone);centers[i]=world(along(i,length-head/2));
        }else if(i<6){
            const unsigned a=i==3?0:i==4?1:2,b=(a+1)%3;
            const float center=float(settings.plane_offset)*pixel,half=float(settings.plane_size/2)*pixel;
            auto p=[&](float x,float y){return along(a,x)+along(b,y);};
            colors[i]->rgb=color(a);polyline(item,{p(center-half,center+half),p(center-half,center-half),p(center+half,center-half)});
            auto* second=new SoBaseColor;second->rgb=color(b);item->addChild(second);polyline(item,{p(center-half,center+half),p(center+half,center+half),p(center+half,center-half)});
            polyline(item,{p(center,center-half),p(center,center+half)});auto* first=new SoBaseColor;first->rgb=color(a);item->addChild(first);polyline(item,{p(center-half,center),p(center+half,center)});centers[i]=world(p(center,center));
        }else if(i==6){
            auto* facing=new SoRotation;facing->rotation=screenFacing;item->addChild(facing);
            auto* black=new SoBaseColor;black->rgb=SbColor(0,0,0);item->addChild(black);square(item,float(settings.origin_size/2+1)*pixel);
            auto* white=new SoBaseColor;white->rgb=SbColor(1,1,1);item->addChild(white);square(item,float(settings.origin_size/2)*pixel);centers[i]=origin;
        }else if(i<10){
            const unsigned a=(i-7+1)%3,b=(i-7+2)%3;const float radius=float(settings.rotate_radius)*pixel;
            std::vector<SbVec3f> arc;for(unsigned step=0;step<=48;++step){const float angle=float(step)*1.57079632679f/48;arc.push_back(along(a,-radius*std::cos(angle))+along(b,-radius*std::sin(angle)));}
            polyline(item,arc);centers[i]=world(along(a,-radius*.70710678f)+along(b,-radius*.70710678f));
        }else{
            const auto anchor=local(layout.anchors[i-10]);dotted(item,SbVec3f(0,0,0),anchor);
            auto* offset=new SoTranslation;offset->translation=anchor;item->addChild(offset);
            auto* facing=new SoRotation;facing->rotation=screenFacing;item->addChild(facing);square(item,float(settings.scale_size/2)*pixel);centers[i]=world(anchor);
        }
    }
    if(layout.visible[13]){
        auto* menu=new SoSeparator;widgetNode->addChild(menu);handles[13]=menu;
        auto* light=new SoLightModel;light->model=SoLightModel::BASE_COLOR;menu->addChild(light);
        auto* white=new SoBaseColor;white->rgb=SbColor(1,1,1);menu->addChild(white);
        SbVec3f offset;rotation.multVec(SbVec3f(float(settings.menu_x)*pixel,-float(settings.menu_y)*pixel,0),offset);
        dotted(menu,origin,origin+offset);menuPose=new SoTransform;menuPose->translation=origin+offset;menuPose->rotation=rotation;menu->addChild(menuPose);
        auto* style=new SoDrawStyle;style->lineWidth=1.5f*dpr;menu->addChild(style);
        std::vector<SbVec3f> circle;for(unsigned i=0;i<=40;++i){float angle=float(i)*6.28318530718f/40;circle.emplace_back(float(settings.menu_size/2)*pixel*std::cos(angle),float(settings.menu_size/2)*pixel*std::sin(angle),0);}
        polyline(menu,circle);centers[13]=origin+offset;
    }else menuPose=nullptr;
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
    SoRayPickAction pick(viewport);pick.setNormalizedPoint(SbVec2f(p.x()/width,(height-1-p.y())/height));pick.setRadius(float(settings.pick_radius*ratio));pick.setPickAll(true);pick.apply(scene);scene->unref();
    // Resolve actual native paths to owned handle nodes.
    const auto& list=pick.getPickedPointList();const unsigned order[]={13,6,10,11,12,0,1,2,3,4,5,7,8,9};for(auto i:order)for(int j=0;j<list.getLength();++j){auto* path=list[j]->getPath();if(handles[i]&&path->containsNode(handles[i])){auto result=ray(p);result.handle=i;return result;}}
    return {};
}
QJsonArray GumballOverlay::handlePositions()const{
    QJsonArray rows;if(!view||!root)return rows;auto* viewer=view->getViewer();const auto size=viewer->viewport()->size();
    const char* names[]={"MoveX","MoveY","MoveZ","MoveXY","MoveYZ","MoveZX","MoveFree","RotateX","RotateY","RotateZ","ScaleX","ScaleY","ScaleZ","MenuBall"};
    auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return rows;
    for(unsigned i=0;i<14;++i){
        if(!handles[i])continue;SbVec3f screen;camera->getViewVolume(float(size.width())/std::max(1,size.height())).projectToScreen(centers[i],screen);
        if(screen[2]<0||screen[2]>1)continue;
        QPoint center(int(screen[0]*size.width()),int((1-screen[1])*size.height()-1));std::optional<QPoint> found;
        for(int radius=0;radius<=12&&!found;++radius)for(int y=-radius;y<=radius&&!found;++y)for(int x=-radius;x<=radius;++x){
            if(radius&&std::abs(x)!=radius&&std::abs(y)!=radius)continue;const QPoint p=center+QPoint(x,y);if(!viewer->viewport()->rect().contains(p))continue;
            const auto picked=hit(p);if(picked&&picked->handle==i){found=p;break;}
        }
        if(found)rows.append(QJsonObject{{"name",names[i]},{"handle",int(i)},{"x",found->x()},{"y",found->y()},{"hit_verified",true},{"center_x",center.x()},{"center_y",center.y()}});
    }
    return rows;
}
QJsonArray GumballOverlay::cameraState()const{
    QJsonArray result;if(!view)return result;auto* camera=view->getViewer()->getSoRenderManager()->getCamera();if(!camera)return result;
    auto p=camera->position.getValue();for(unsigned i=0;i<3;++i)result.append(p[i]);const float* q=camera->orientation.getValue().getValue();for(unsigned i=0;i<4;++i)result.append(q[i]);
    // Perspective view-volume height is measured at the near clip plane.
    // Host auto-clipping can change it during a repaint without navigation.
    result.append(QString::fromLatin1(camera->getTypeId().getName().getString()));
    if(auto* perspective=dynamic_cast<SoPerspectiveCamera*>(camera))result.append(perspective->heightAngle.getValue());
    else result.append(camera->getViewVolume().getHeight());
    result.append(camera->focalDistance.getValue());return result;
}
QJsonObject GumballOverlay::diagnostics()const{
    int vertices=0;
    if(previewNode){SoSearchAction search;search.setSearchingAll(true);search.setType(SoCoordinate3::getClassTypeId());search.setInterest(SoSearchAction::ALL);search.apply(previewNode);auto& paths=search.getPaths();for(int i=0;i<paths.getLength();++i)if(auto* coordinates=dynamic_cast<SoCoordinate3*>(paths[i]->getTail()))vertices+=coordinates->point.getNum();}
    QJsonArray visible,anchors;for(unsigned i=0;i<14;++i)if(layout.visible[i])visible.append(int(i));for(const auto& p:layout.anchors)anchors.append(QJsonArray{p[0],p[1],p[2]});
    return {{"attached",bool(view&&root)},{"current_scene",bool(view&&owner&&owner==view->getViewer()->getSceneGraph())},{"in_owner",bool(owner&&root&&owner->findChild(root)>=0)},{"visible_handles",visible},{"scale_anchors",anchors},{"preview_vertices",vertices}};
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
    const float dpr=view?float(view->getViewer()->viewport()->devicePixelRatioF()):1.f;
    auto* style=new SoDrawStyle;style->lineWidth=2*dpr;style->pointSize=4*dpr;previewNode->addChild(style);
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
