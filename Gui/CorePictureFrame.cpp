#include "CorePictureFrame.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include "CoreSnaps.h"
#include "CoreDistance.h"
#include "CoreWorkspace.h"
#include "CoreViewControls.h"
#include "CurveController.h"
#include "RustBridge.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/ImagePlane.h>
#include <Base/Exception.h>
#include <Base/Matrix.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/ViewProvider.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Gui/BitmapFactory.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Gui/Selection/SoFCUnifiedSelection.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoTexture2.h>
#include <Inventor/nodes/SoTextureCoordinate2.h>
#include <Inventor/nodes/SoFaceSet.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoLightModel.h>
#include <Inventor/nodes/SoMaterial.h>
#include <Inventor/actions/SoSearchAction.h>
#include <QApplication>
#include <QFileDialog>
#include <QFileInfo>
#include <QImageReader>
#include <QLabel>
#include <QMouseEvent>
#include <QKeyEvent>
#include <QRegularExpression>
#include <QTimer>
#include <cmath>
namespace {
Gui::View3DInventor* view(){auto* doc=Gui::Application::Instance->activeDocument();return doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;}
}
namespace OpenMatrix9Gui {
CorePictureFrame& CorePictureFrame::instance(){static auto* value=new CorePictureFrame;return *value;}
CorePictureFrame::CorePictureFrame():QObject(qApp) {
    deletedConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==&doc)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(running&&document!=&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{
        if(running&&!valid()){cancel();return;}
        if(running&&previewOwner){SoSearchAction search;search.setNode(previewNode);search.apply(view()->getViewer()->getSceneGraph());if(!search.getPath())clearPreview();}
    });timer->start();
    qApp->installEventFilter(this);
}
bool CorePictureFrame::handles(std::size_t index){const auto* name=om9_command_id(index);return name&&QString::fromUtf8(name)=="PictureFrame";}
bool CorePictureFrame::available()const {
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&doc&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&view()&&om9AlterDocument(doc);
}
bool CorePictureFrame::valid()const{return running&&document==App::GetApplication().getActiveDocument()&&available();}
void CorePictureFrame::activate(){enabled=true;}
void CorePictureFrame::deactivate(){enabled=false;cancel();}
void CorePictureFrame::clearPreview(){
    if(previewOwner){if(previewNode&&previewOwner->findChild(previewNode)>=0)previewOwner->removeChild(previewNode);previewOwner->unref();}
    previewOwner=nullptr;previewNode=nullptr;
}
void CorePictureFrame::preview(const double* p){
    clearPreview();auto* native=view();if(!native)return;
    auto* root=dynamic_cast<SoSeparator*>(native->getViewer()->getSceneGraph());if(!root)return;
    previewOwner=root;
    for(int i=0;i<root->getNumChildren();++i)if(auto* selection=dynamic_cast<Gui::SoFCUnifiedSelection*>(root->getChild(i))){previewOwner=selection;break;}
    previewOwner->ref();
    auto* wrapper=new SoSeparator;previewOwner->addChild(wrapper);previewNode=wrapper;
    auto* skip=new Gui::SoSkipBoundingGroup;skip->mode=Gui::SoSkipBoundingGroup::INCLUDE_BBOX;wrapper->addChild(skip);
    auto* content=new SoSeparator;content->setName("OM9PictureFramePreview");skip->addChild(content);
    auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;content->addChild(pick);
    auto* material=new SoMaterial;material->diffuseColor.setValue(1,1,1);content->addChild(material);
    auto* light=new SoLightModel;light->model=selfIllumination?SoLightModel::BASE_COLOR:SoLightModel::PHONG;content->addChild(light);
    auto* texture=new SoTexture2;Gui::BitmapFactory().convert(image,texture->image);content->addChild(texture);
    auto* uv=new SoTextureCoordinate2;uv->point.set1Value(0,0,0);uv->point.set1Value(1,1,0);uv->point.set1Value(2,1,1);uv->point.set1Value(3,0,1);content->addChild(uv);
    auto* coordinates=new SoCoordinate3;
    for(int i=0;i<4;++i){double v[3];for(int j=0;j<3;++j)v[j]=p[j]+((i==1||i==2)?p[6+j]*p[15]:0)+(i>=2?p[9+j]*p[16]:0);coordinates->point.set1Value(i,float(v[0]),float(v[1]),float(v[2]));}
    content->addChild(coordinates);auto* face=new SoFaceSet;face->numVertices.setValue(4);content->addChild(face);native->getViewer()->redraw();
}
void CorePictureFrame::cancel(){clearPreview();running=false;hasCorner=false;document=nullptr;source.clear();image=QImage();}
void CorePictureFrame::prompt(const QString& error) {
    CurveController::instance().setPrompt(error.isEmpty()?(hasCorner?"PictureFrame: Pick width reference or type length; Shift for Ortho; Esc cancels":"PictureFrame: Pick first corner or enter x,y,z; Esc cancels"):error);
    if(!error.isEmpty())CurveController::instance().logMessage(error);
}
bool CorePictureFrame::start(std::size_t index) {
    if(!handles(index)||!available())return false;
    cancel();CoreDistance::instance().cancel();CurveController::instance().cancel();CoreViewControls::instance().cancel();
    auto* identity=App::GetApplication().getActiveDocument();const auto construction=CoreWorkspace::instance().plane(view());
    QFileDialog dialog(Gui::getMainWindow(),"PictureFrame: Select image");dialog.setObjectName("OM9PictureFrameImageDialog");dialog.setFileMode(QFileDialog::ExistingFile);
    dialog.setNameFilter("Images (*.png *.jpg *.jpeg *.bmp *.gif *.tif *.tiff *.webp)");
    if(dialog.exec()!=QDialog::Accepted||dialog.selectedFiles().isEmpty())return false;
    if(identity!=App::GetApplication().getActiveDocument()||!available())return false;
    const auto file=dialog.selectedFiles().front();QImageReader reader(file);auto loaded=reader.read();
    if(loaded.isNull()){prompt("PictureFrame: Cannot read image: "+reader.errorString());return false;}
    document=identity;command=index;plane=construction;source=file;image=loaded;hasCorner=false;vertical=false;selfIllumination=true;autoname=true;running=true;
    qApp->installEventFilter(this);CoreMouse::instance().prioritize();prompt();return false; // successful history waits for image commit
}
bool CorePictureFrame::plan(const Base::Vector3d* reference,double width,bool ortho,double* output)const {
    double axes[9];for(int i=0;i<3;++i){Base::Vector3d axis;axis[i]=1;axis=plane.getRotation().multVec(axis);for(int j=0;j<3;++j)axes[i*3+j]=axis[j];}
    const double first[]={corner.x,corner.y,corner.z};double second[3]={};if(reference)for(int i=0;i<3;++i)second[i]=(*reference)[i];
    return om9_picture_frame_plan(axes,first,reference?second:nullptr,width,double(image.width())/image.height(),(ortho?1U:0U)|(vertical?2U:0U)|(reference?0U:4U),output);
}
void CorePictureFrame::point(const Base::Vector3d& value,bool ortho) {
    if(!valid()){cancel();return;}
    for(int i=0;i<3;++i)if(!std::isfinite(value[i])||std::abs(value[i])>1e9){prompt("PictureFrame: Point must be finite within supported coordinates");return;}
    if(!hasCorner){corner=value;hasCorner=true;prompt();return;}
    double placement[17];if(!plan(&value,0,ortho,placement)){prompt("PictureFrame: Choose a valid second reference point");return;}finish(placement);
}
void CorePictureFrame::finish(const double* p) {
    if(!valid()){cancel();return;}
    auto* doc=document;om9OpenTransaction(*doc,"PictureFrame");
    try {
        auto* object=dynamic_cast<Image::ImagePlane*>(doc->addObject("Image::ImagePlane","PictureFrame"));
        if(!object)throw Base::RuntimeError("Native ImagePlane is unavailable");
        object->ImageFile.setValue(source.toUtf8().constData());object->XSize.setValue(p[15]);object->YSize.setValue(p[16]);
        Base::Matrix4D matrix;for(int row=0;row<3;++row)for(int col=0;col<3;++col)matrix[row][col]=p[6+col*3+row];
        object->Placement.setValue(Base::Placement(Base::Vector3d(p[3],p[4],p[5]),Base::Rotation(matrix)));
        if(autoname)object->Label.setValue(QFileInfo(source).completeBaseName().toUtf8().constData());
        if(auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(Gui::Application::Instance->getDocument(doc)->getViewProvider(object)))provider->DisplayMode.setValue(selfIllumination?"No shading":"Shading");
        doc->recompute();om9CommitTransaction(*doc);om9_sidebar_record_execution(command,true);cancel();
        CurveController::instance().setPrompt("PictureFrame created");
        CurveController::instance().logMessage("PictureFrame created");
    }catch(const Base::Exception& error){om9AbortTransaction(*doc);prompt(QString::fromUtf8(error.what()));}
}
void CorePictureFrame::submit(const QString& text) {
    if(!valid()){cancel();return;}const auto input=text.trimmed();
    if(input.compare("Cancel",Qt::CaseInsensitive)==0){cancel();prompt("PictureFrame cancelled");return;}
    const auto parts=input.split('=');
    if(parts.size()==2) {
        const auto name=parts[0].trimmed();const auto value=parts[1].trimmed();
        const bool yes=value.compare("Yes",Qt::CaseInsensitive)==0||value=="1",no=value.compare("No",Qt::CaseInsensitive)==0||value=="0";
        if(yes||no) {
            if(name.compare("Vertical",Qt::CaseInsensitive)==0)vertical=yes;
            else if(name.compare("SelfIllumination",Qt::CaseInsensitive)==0)selfIllumination=yes;
            else if(name.compare("Autoname",Qt::CaseInsensitive)==0)autoname=yes;
            else {prompt("PictureFrame: This option is not implemented yet");return;}
            prompt();return;
        }
    }
    const auto fields=input.split(QRegularExpression("[,\\s]+"),Qt::SkipEmptyParts);
    if(fields.size()==3){bool ok[3];Base::Vector3d value;for(int i=0;i<3;++i)value[i]=fields[i].toDouble(&ok[i]);if(ok[0]&&ok[1]&&ok[2]){point(value);return;}}
    if(hasCorner){bool ok=false;const double width=input.toDouble(&ok);double placement[17];if(ok&&plan(nullptr,width,false,placement)){finish(placement);return;}}
    prompt("PictureFrame: Enter a valid point x,y,z or a positive width after the first corner");
}
bool CorePictureFrame::eventFilter(QObject* watched,QEvent* event) {
    if(!running||Gui::Application::Instance->isClosing())return false;
    if(event->type()==QEvent::KeyPress) {
        const auto* key=static_cast<QKeyEvent*>(event);if(key->key()==Qt::Key_Escape){cancel();prompt("PictureFrame cancelled");return true;}
        if(key->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,key)){point(plane.getPosition());return true;}
    }
    const bool moving=event->type()==QEvent::MouseMove;
    if(!moving&&event->type()!=QEvent::MouseButtonPress)return false;
    const auto* mouse=static_cast<QMouseEvent*>(event);if((!moving&&mouse->button()!=Qt::LeftButton)||!valid())return false;
    auto* native=view();auto* widget=qobject_cast<QWidget*>(watched);auto* viewer=native->getViewer();if(!widget||!viewer->isAncestorOf(widget))return false;
    try {
        const auto pos=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());const auto pixel=viewer->fromQPoint(pos);
        Base::Vector3d reference;
        if(!CoreSnaps::pick(native,pos,reference)){
            SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);const auto direction=farPoint-nearPoint;const auto normal=plane.getRotation().multVec(Base::Vector3d(0,0,1));
            if(std::abs(direction[0]*normal.x+direction[1]*normal.y+direction[2]*normal.z)<=1e-6*direction.length())throw Base::RuntimeError("Ray parallel to construction plane");
            const auto value=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);reference=Base::Vector3d(value[0],value[1],value[2]);
        }
        const bool ortho=om9_ortho_active(mouse->modifiers().testFlag(Qt::ShiftModifier));
        if(moving){if(hasCorner){double placement[17];if(plan(&reference,0,ortho,placement))preview(placement);else clearPreview();}return false;}
        point(reference,ortho);
    }catch(const Base::Exception& error){prompt(QString::fromUtf8(error.what()));}
    return true;
}
}
