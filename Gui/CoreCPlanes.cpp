#include "CoreCPlanes.h"
#include "RustBridge.h"
#include <App/Application.h>
#include <App/Document.h>
#include <Base/Console.h>
#include <Base/Exception.h>
#include <Base/Matrix.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/MainWindow.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Inventor/actions/SoSearchAction.h>
#include <Inventor/nodes/SoTransform.h>
#include <Inventor/SoRenderManager.h>
#include <QApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QThread>
#include <QMenu>
#include <QAction>
#include <QPointer>
#include <QInputDialog>
#include <QMessageBox>
#include <fastsignals/signal.h>
#include <array>
#include <cstdint>
#include <limits>
#include <map>
#include <stdexcept>
#include <vector>

extern "C" {
bool om9_cplane_ensure(std::uint64_t,std::uint64_t,const double*);
bool om9_cplane_set(std::uint64_t,std::uint64_t,const double*);
bool om9_cplane_read(std::uint64_t,std::uint64_t,double*,std::uint64_t*);
bool om9_cplane_previous(std::uint64_t,std::uint64_t);
bool om9_cplane_next(std::uint64_t,std::uint64_t);
bool om9_cplane_set_world(std::uint64_t,std::uint64_t,unsigned);
bool om9_cplane_history_available(std::uint64_t,std::uint64_t,bool);
void om9_cplane_drop_document(std::uint64_t);
std::uint64_t om9_cplane_named_save(std::uint64_t,std::uint64_t,const char*);
bool om9_cplane_named_rename(std::uint64_t,std::uint64_t,const char*);
bool om9_cplane_named_restore(std::uint64_t,std::uint64_t,std::uint64_t);
std::size_t om9_cplane_named_count(std::uint64_t);
std::size_t om9_cplane_named_read(std::uint64_t,std::size_t,std::uint64_t*,double*,char*,std::size_t);
bool om9_cplane_named_import(std::uint64_t,const std::uint64_t*,const char* const*,const double*,std::size_t);
bool om9_cplane_three_points(const double*,double*);
}

namespace OpenMatrix9Gui {
namespace {
using Frame=std::array<double,12>;
constexpr const char* metadataKey="OpenMatrix9.CPlanes.v1";

void guiThread() {
    if(!qApp||QThread::currentThread()!=qApp->thread())throw std::runtime_error("CPlane operations require the GUI thread");
}
Frame values(const Base::Placement& plane) {
    Frame result{};const auto origin=plane.getPosition();const auto& rotation=plane.getRotation();
    const std::array<Base::Vector3d,4> rows={origin,rotation.multVec(Base::Vector3d(1,0,0)),rotation.multVec(Base::Vector3d(0,1,0)),rotation.multVec(Base::Vector3d(0,0,1))};
    for(std::size_t i=0;i<rows.size();++i){result[3*i]=rows[i].x;result[3*i+1]=rows[i].y;result[3*i+2]=rows[i].z;}
    return result;
}
Base::Placement placement(const Frame& frame) {
    Base::Matrix4D matrix;
    for(unsigned axis=0;axis<3;++axis)for(unsigned component=0;component<3;++component)matrix[component][axis]=frame[3+3*axis+component];
    return Base::Placement(Base::Vector3d(frame[0],frame[1],frame[2]),Base::Rotation(matrix));
}
Frame defaultFrame(unsigned slot) {
    return values(Base::Placement(Base::Vector3d(),Base::Rotation(om9_core_view_rotation(slot,0,true),om9_core_view_rotation(slot,1,true),om9_core_view_rotation(slot,2,true),om9_core_view_rotation(slot,3,true))));
}
QJsonArray encodeFrame(const Frame& frame){QJsonArray array;for(double value:frame)array.append(value);return array;}
Frame decodeFrame(const QJsonValue& value) {
    if(!value.isArray()||value.toArray().size()!=12)throw std::runtime_error("Invalid saved CPlane frame");
    Frame frame{};const auto array=value.toArray();
    for(unsigned i=0;i<12;++i){if(!array[i].isDouble())throw std::runtime_error("Invalid saved CPlane component");frame[i]=array[i].toDouble();}
    return frame;
}

class Store {
public:
    Store() {
        deleted=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){
            if(auto found=identities.find(&doc);found!=identities.end()){om9_cplane_drop_document(found->second);identities.erase(found);}
        });
        saving=App::GetApplication().signalStartSaveDocument.connect([this](const App::Document& doc,const std::string&){
            if(auto found=identities.find(&doc);found!=identities.end()){
                try{save(const_cast<App::Document&>(doc),found->second);}catch(const std::exception& error){Base::Console().error("OpenMatrix9 CPlane save: %s\n",error.what());}
            }
        });
    }
    std::uint64_t identity(const App::Document& doc) {
        guiThread();if(auto found=identities.find(&doc);found!=identities.end())return found->second;
        if(next==std::numeric_limits<std::uint64_t>::max())throw std::runtime_error("CPlane document identity exhausted");
        const auto id=++next;
        try { load(doc,id);identities.emplace(&doc,id);return id; }
        catch(...){om9_cplane_drop_document(id);throw;}
    }
    Frame ensure(const App::Document& doc,unsigned slot,const Frame& fallback) {
        if(slot>=4)throw std::runtime_error("CPlane slot must be 0 through 3");
        const auto id=identity(doc);Frame frame{};
        if(!om9_cplane_ensure(id,slot+1,fallback.data())||!om9_cplane_read(id,slot+1,frame.data(),nullptr))throw std::runtime_error("Invalid construction plane");
        return frame;
    }
private:
    void load(const App::Document& doc,std::uint64_t id) {
        const auto& metadata=doc.Meta.getValues();const auto found=metadata.find(metadataKey);if(found==metadata.end())return;
        if(found->second.size()>4*1024*1024)throw std::runtime_error("Saved CPlane table exceeds the import budget");
        QJsonParseError error;const auto data=QJsonDocument::fromJson(QByteArray::fromStdString(found->second),&error);
        if(error.error!=QJsonParseError::NoError||!data.isObject()||data.object().value("version").toInt()!=1)throw std::runtime_error("Invalid saved CPlane schema");
        const auto object=data.object();
        if(!object.value("views").isArray()||!object.value("named").isArray())throw std::runtime_error("Invalid saved CPlane tables");
        const auto views=object.value("views").toArray();std::array<bool,4> seen{};
        for(const auto& value:views){
            const auto entry=value.toObject();const int slot=entry.value("slot").toInt(-1);
            if(slot<0||slot>=4||seen[slot])throw std::runtime_error("Invalid or duplicate saved CPlane slot");seen[slot]=true;
            const auto frame=decodeFrame(entry.value("frame"));
            if(!om9_cplane_ensure(id,std::uint64_t(slot)+1,frame.data()))throw std::runtime_error("Invalid saved construction plane");
        }
        const auto named=object.value("named").toArray();std::vector<std::uint64_t> ids;std::vector<std::string> names;std::vector<double> frames;
        if(named.size()>4096)throw std::runtime_error("Too many saved named construction planes");
        for(const auto& value:named){
            const auto entry=value.toObject();bool ok=false;const auto identity=entry.value("id").toString().toULongLong(&ok);
            if(!ok||!entry.value("name").isString())throw std::runtime_error("Invalid saved named construction plane");
            const auto name=entry.value("name").toString();
            // The ABI carries NUL-terminated UTF-8. Reject embedded NUL here,
            // before that boundary can hide the suffix from Rust validation.
            if(name.contains(QChar(u'\0')))throw std::runtime_error("Saved named CPlane contains an embedded NUL");
            ids.push_back(identity);names.push_back(name.toStdString());
            const auto frame=decodeFrame(entry.value("frame"));frames.insert(frames.end(),frame.begin(),frame.end());
        }
        std::vector<const char*> pointers;for(const auto& name:names)pointers.push_back(name.c_str());
        if(!om9_cplane_named_import(id,ids.data(),pointers.data(),frames.data(),ids.size()))throw std::runtime_error("Invalid saved named construction plane table");
    }
    void save(App::Document& doc,std::uint64_t id) {
        QJsonArray views,named;
        for(unsigned slot=0;slot<4;++slot){Frame frame{};if(om9_cplane_read(id,slot+1,frame.data(),nullptr))views.append(QJsonObject{{"slot",int(slot)},{"frame",encodeFrame(frame)}});}
        const auto count=om9_cplane_named_count(id);
        for(std::size_t i=0;i<count;++i){
            Frame frame{};std::uint64_t identity=0;const auto size=om9_cplane_named_read(id,i,&identity,frame.data(),nullptr,0);
            if(!size||size>1024*1024)throw std::runtime_error("Invalid named CPlane label size");std::vector<char> name(size);
            if(om9_cplane_named_read(id,i,&identity,frame.data(),name.data(),name.size())!=size)throw std::runtime_error("Named CPlane changed while saving");
            named.append(QJsonObject{{"id",QString::number(qulonglong(identity))},{"name",QString::fromUtf8(name.data())},{"frame",encodeFrame(frame)}});
        }
        const auto raw=QJsonDocument(QJsonObject{{"version",1},{"views",views},{"named",named}}).toJson(QJsonDocument::Compact).toStdString();
        auto metadata=doc.Meta.getValues();metadata[metadataKey]=raw;if(metadata!=doc.Meta.getValues())doc.Meta.setValues(std::move(metadata));
    }
    std::map<const App::Document*,std::uint64_t> identities;
    std::uint64_t next=0;
    fastsignals::scoped_connection deleted,saving;
};
Store& store(){static auto* instance=new Store;return *instance;}

App::Document& document(const char* name){
    guiThread();auto* doc=App::GetApplication().getDocument(name);
    if(!doc||doc->testStatus(App::Document::Restoring))throw std::runtime_error("CPlane document is missing or restoring");return *doc;
}
void refreshGrid(App::Document& doc,unsigned slot,const Frame& frame) {
    auto* gui=Gui::Application::Instance->getDocument(&doc);if(!gui)return;
    // Plane edits are document settings, not geometry transactions. Mark the
    // GUI document dirty so closing it offers Save before the deferred metadata
    // serialization runs in signalStartSaveDocument.
    gui->setModified(true);
    const auto plane=placement(frame);const auto origin=plane.getPosition();const auto& q=plane.getRotation();
    for(auto* mdi:gui->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&view->property("om9ViewSlot").isValid()&&view->property("om9ViewSlot").toUInt()==slot){
        SoSearchAction search;search.setSearchingAll(true);search.setName("OM9ConstructionGridTransform");search.setInterest(SoSearchAction::FIRST);search.apply(view->getViewer()->getSceneGraph());
        if(auto* path=search.getPath())if(auto* transform=dynamic_cast<SoTransform*>(path->getTail())){
            transform->translation.setValue(float(origin.x),float(origin.y),float(origin.z));
            transform->rotation.setValue(float(q[0]),float(q[1]),float(q[2]),float(q[3]));
            view->getViewer()->getSoRenderManager()->scheduleRedraw();
        }
    }
}
Frame sequence(PyObject* value,std::size_t count=12) {
    PyObject* fast=PySequence_Fast(value,"CPlane coordinates must be a sequence");if(!fast)throw std::runtime_error("CPlane coordinates must be a sequence");
    Frame frame{};
    if(std::size_t(PySequence_Fast_GET_SIZE(fast))!=count){Py_DECREF(fast);throw std::runtime_error("Unexpected CPlane coordinate count");}
    for(std::size_t i=0;i<count;++i){frame[i]=PyFloat_AsDouble(PySequence_Fast_GET_ITEM(fast,Py_ssize_t(i)));if(PyErr_Occurred()){Py_DECREF(fast);throw std::runtime_error("CPlane coordinates must be numeric");}}
    Py_DECREF(fast);return frame;
}
PyObject* tuple(const Frame& frame){auto* result=PyTuple_New(12);if(!result)return nullptr;for(Py_ssize_t i=0;i<12;++i)PyTuple_SET_ITEM(result,i,PyFloat_FromDouble(frame[std::size_t(i)]));return result;}
template<class F>PyObject* api(F&& action){
    try{return action();}catch(const Base::Exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());}catch(const std::exception& error){PyErr_SetString(PyExc_ValueError,error.what());}catch(...){PyErr_SetString(PyExc_RuntimeError,"CPlane native adapter failed");}return nullptr;
}
PyObject* getAPI(PyObject*,PyObject* args){const char* name=nullptr;unsigned slot=0;if(!PyArg_ParseTuple(args,"sI",&name,&slot))return nullptr;return api([&]{return tuple(store().ensure(document(name),slot,defaultFrame(slot)));});}
PyObject* setAPI(PyObject*,PyObject* args){const char* name=nullptr;unsigned slot=0;PyObject* values=nullptr;if(!PyArg_ParseTuple(args,"sIO",&name,&slot,&values))return nullptr;return api([&]() -> PyObject* {
    auto& doc=document(name);const auto proposed=sequence(values);const auto previous=store().ensure(doc,slot,defaultFrame(slot));
    if(!om9_cplane_set(store().identity(doc),slot+1,proposed.data()))throw std::runtime_error("CPlane must be finite, right-handed and orthonormal");if(previous!=proposed)refreshGrid(doc,slot,proposed);return tuple(proposed);
});}
PyObject* threePointAPI(PyObject*,PyObject* args){const char* name=nullptr;unsigned slot=0;PyObject* values=nullptr;if(!PyArg_ParseTuple(args,"sIO",&name,&slot,&values))return nullptr;return api([&]() -> PyObject* {
    auto& doc=document(name);const auto points=sequence(values,9);Frame proposed{};
    if(!om9_cplane_three_points(points.data(),proposed.data()))throw std::runtime_error("CPlane needs three distinct noncollinear points");
    const auto previous=store().ensure(doc,slot,defaultFrame(slot));if(!om9_cplane_set(store().identity(doc),slot+1,proposed.data()))throw std::runtime_error("Cannot set three-point CPlane");if(previous!=proposed)refreshGrid(doc,slot,proposed);return tuple(proposed);
});}
PyObject* navigate(PyObject* args,bool forward){const char* name=nullptr;unsigned slot=0;if(!PyArg_ParseTuple(args,"sI",&name,&slot))return nullptr;return api([&]() -> PyObject* {
    auto& doc=document(name);store().ensure(doc,slot,defaultFrame(slot));const auto id=store().identity(doc);
    if(!(forward?om9_cplane_next(id,slot+1):om9_cplane_previous(id,slot+1)))throw std::runtime_error("No construction plane in that history direction");Frame frame{};
    if(!om9_cplane_read(id,slot+1,frame.data(),nullptr))throw std::runtime_error("Construction plane disappeared");refreshGrid(doc,slot,frame);return tuple(frame);
});}
PyObject* previousAPI(PyObject*,PyObject* args){return navigate(args,false);}
PyObject* nextAPI(PyObject*,PyObject* args){return navigate(args,true);}
PyObject* saveNamedAPI(PyObject*,PyObject* args){const char* name=nullptr;const char* label=nullptr;unsigned slot=0;if(!PyArg_ParseTuple(args,"sIs",&name,&slot,&label))return nullptr;return api([&]() -> PyObject* {
    auto& doc=document(name);store().ensure(doc,slot,defaultFrame(slot));const auto id=om9_cplane_named_save(store().identity(doc),slot+1,label);if(!id)throw std::runtime_error("Named CPlane label is empty or already exists");if(auto* gui=Gui::Application::Instance->getDocument(&doc))gui->setModified(true);return PyLong_FromUnsignedLongLong(id);
});}
PyObject* restoreNamedAPI(PyObject*,PyObject* args){const char* name=nullptr;unsigned slot=0;unsigned long long id=0;if(!PyArg_ParseTuple(args,"sIK",&name,&slot,&id))return nullptr;return api([&]() -> PyObject* {
    auto& doc=document(name);const auto previous=store().ensure(doc,slot,defaultFrame(slot));const auto owner=store().identity(doc);if(!om9_cplane_named_restore(owner,slot+1,id))throw std::runtime_error("Unknown named CPlane");Frame frame{};if(!om9_cplane_read(owner,slot+1,frame.data(),nullptr))throw std::runtime_error("Construction plane disappeared");if(previous!=frame)refreshGrid(doc,slot,frame);return tuple(frame);
});}
PyObject* renameNamedAPI(PyObject*,PyObject* args){const char* name=nullptr;const char* label=nullptr;unsigned long long id=0;if(!PyArg_ParseTuple(args,"sKs",&name,&id,&label))return nullptr;return api([&]() -> PyObject* {
    auto& doc=document(name);const auto owner=store().identity(doc);bool unchanged=false;
    for(std::size_t i=0;i<om9_cplane_named_count(owner);++i){std::uint64_t found=0;const auto size=om9_cplane_named_read(owner,i,&found,nullptr,nullptr,0);if(found==id&&size&&size<=1024*1024){std::vector<char> old(size);om9_cplane_named_read(owner,i,&found,nullptr,old.data(),size);unchanged=std::string(old.data())==label;break;}}
    if(!om9_cplane_named_rename(owner,id,label))throw std::runtime_error("Unknown named CPlane or conflicting/invalid name");if(!unchanged)if(auto* gui=Gui::Application::Instance->getDocument(&doc))gui->setModified(true);Py_RETURN_NONE;
});}
PyObject* namedAPI(PyObject*,PyObject* args){const char* name=nullptr;if(!PyArg_ParseTuple(args,"s",&name))return nullptr;return api([&]() -> PyObject* {
    const auto owner=store().identity(document(name));const auto count=om9_cplane_named_count(owner);auto* result=PyList_New(0);if(!result)return nullptr;
    for(std::size_t i=0;i<count;++i){Frame frame{};std::uint64_t id=0;const auto size=om9_cplane_named_read(owner,i,&id,frame.data(),nullptr,0);if(!size||size>1024*1024){Py_DECREF(result);throw std::runtime_error("Invalid named CPlane label");}std::vector<char> label(size);om9_cplane_named_read(owner,i,&id,frame.data(),label.data(),size);auto* item=Py_BuildValue("{s:K,s:s,s:N}","id",static_cast<unsigned long long>(id),"name",label.data(),"frame",tuple(frame));if(!item||PyList_Append(result,item)<0){Py_XDECREF(item);Py_DECREF(result);return nullptr;}Py_DECREF(item);}
    return result;
});}

struct MenuContext { App::Document* document; unsigned slot; std::uint64_t owner; };
MenuContext menuContext(QPointer<Gui::View3DInventor> view) {
    guiThread();
    auto* gui=view?view->getGuiDocument():nullptr;
    if(!gui||gui->isAboutToClose()||gui!=Gui::Application::Instance->activeDocument()||!view->property("om9ViewSlot").isValid())throw std::runtime_error("The CPlane viewport is no longer active");
    auto* doc=gui->getDocument();if(!doc||doc->testStatus(App::Document::Restoring)||!Gui::Control().isAllowedAlterView(doc))throw std::runtime_error("The viewport does not allow construction-plane changes");
    const auto slot=view->property("om9ViewSlot").toUInt();store().ensure(*doc,slot,defaultFrame(slot));
    return {doc,slot,store().identity(*doc)};
}
void menuError(const char* text) {
    Base::Console().error("OpenMatrix9 Set CPlane: %s\n",text);
    auto* message=new QMessageBox(QMessageBox::Warning,"Set CPlane",QString::fromUtf8(text),QMessageBox::Ok,Gui::getMainWindow());
    message->setAttribute(Qt::WA_DeleteOnClose);message->open();
}
template<class F>void menuAction(F&& action) {
    try{action();}catch(const Base::Exception& error){menuError(error.what());}catch(const std::exception& error){menuError(error.what());}catch(...){menuError("Construction-plane operation failed");}
}
void refreshAfterMenu(const MenuContext& context,const Frame& before) {
    Frame after{};if(!om9_cplane_read(context.owner,context.slot+1,after.data(),nullptr))throw std::runtime_error("Construction plane disappeared");
    if(before!=after)refreshGrid(*context.document,context.slot,after);
}
void populateNamedMenu(QMenu* menu,QPointer<Gui::View3DInventor> target) {
    menu->clear();
    try {
        const auto context=menuContext(target);const auto count=om9_cplane_named_count(context.owner);
        for(std::size_t i=0;i<count;++i){
            std::uint64_t id=0;const auto size=om9_cplane_named_read(context.owner,i,&id,nullptr,nullptr,0);
            if(!size||size>1024*1024)throw std::runtime_error("Invalid named CPlane label");std::vector<char> label(size);
            if(om9_cplane_named_read(context.owner,i,&id,nullptr,label.data(),size)!=size)throw std::runtime_error("Named CPlane changed while opening the menu");
            auto* action=menu->addAction(QString::fromUtf8(label.data()));action->setObjectName("OM9CPlaneRestoreNamed_"+QString::number(qulonglong(id)));
            const auto owner=context.owner;
            QObject::connect(action,&QAction::triggered,menu,[target,id,owner]{menuAction([&]{
                const auto current=menuContext(target);if(current.owner!=owner)throw std::runtime_error("The named CPlane document changed");
                const auto before=store().ensure(*current.document,current.slot,defaultFrame(current.slot));
                if(!om9_cplane_named_restore(current.owner,current.slot+1,id))throw std::runtime_error("The named CPlane no longer exists");refreshAfterMenu(current,before);
            });});
        }
        if(!count)menu->addAction("No named CPlanes")->setEnabled(false);
    }catch(const std::exception& error){menu->addAction("CPlane unavailable")->setEnabled(false);menu->setToolTip(QString::fromUtf8(error.what()));}
    catch(...){menu->addAction("CPlane unavailable")->setEnabled(false);}
}
}

Base::Placement constructionPlane(Gui::View3DInventor* view,const Base::Placement& fallback) {
    if(!view||!view->property("om9ViewSlot").isValid()||!view->getGuiDocument())return fallback;
    return placement(store().ensure(*view->getGuiDocument()->getDocument(),view->property("om9ViewSlot").toUInt(),values(fallback)));
}
QMenu* addCPlaneMenu(QMenu* parent,Gui::View3DInventor* view) {
    auto* menu=parent->addMenu("Set CPlane");menu->setObjectName("OM9CPlaneMenu");QPointer<Gui::View3DInventor> target(view);
    const std::array<const char*,3> titles={"World XY","World XZ","World YZ"};
    const std::array<const char*,3> names={"OM9CPlaneWorldXY","OM9CPlaneWorldXZ","OM9CPlaneWorldYZ"};
    for(unsigned plane=0;plane<3;++plane){
        auto* action=menu->addAction(titles[plane]);action->setObjectName(names[plane]);
        QObject::connect(action,&QAction::triggered,menu,[target,plane]{menuAction([&]{
            const auto context=menuContext(target);const auto before=store().ensure(*context.document,context.slot,defaultFrame(context.slot));
            if(!om9_cplane_set_world(context.owner,context.slot+1,plane))throw std::runtime_error("Cannot set world construction plane");refreshAfterMenu(context,before);
        });});
    }
    menu->addSeparator();
    auto* previous=menu->addAction("Previous");previous->setObjectName("OM9CPlanePrevious");
    auto* next=menu->addAction("Next");next->setObjectName("OM9CPlaneNext");
    for(const auto item:{std::pair(previous,false),std::pair(next,true)})QObject::connect(item.first,&QAction::triggered,menu,[target,forward=item.second]{menuAction([&]{
        const auto context=menuContext(target);const auto before=store().ensure(*context.document,context.slot,defaultFrame(context.slot));
        if(!(forward?om9_cplane_next(context.owner,context.slot+1):om9_cplane_previous(context.owner,context.slot+1)))throw std::runtime_error("No construction plane in that history direction");refreshAfterMenu(context,before);
    });});
    menu->addSeparator();auto* save=menu->addAction("Save Named CPlane…");save->setObjectName("OM9CPlaneSaveNamed");
    QObject::connect(save,&QAction::triggered,menu,[target]{menuAction([&]{
        const auto before=menuContext(target);
        QInputDialog dialog(Gui::getMainWindow());dialog.setObjectName("OM9SaveNamedCPlaneDialog");dialog.setWindowTitle("Save Named CPlane");dialog.setLabelText("Name:");dialog.setInputMode(QInputDialog::TextInput);
        if(dialog.exec()!=QDialog::Accepted)return;
        const auto current=menuContext(target);if(current.owner!=before.owner||current.slot!=before.slot)throw std::runtime_error("The CPlane viewport changed while naming the plane");
        const auto label=dialog.textValue();if(label.contains(QChar(u'\0')))throw std::runtime_error("A named CPlane label cannot contain an embedded NUL");
        const auto name=label.toUtf8();if(!om9_cplane_named_save(current.owner,current.slot+1,name.constData()))throw std::runtime_error("Use a nonempty name that is not already in this document");
        if(auto* gui=Gui::Application::Instance->getDocument(current.document))gui->setModified(true);
    });});
    auto* named=menu->addMenu("Restore Named CPlane");named->setObjectName("OM9CPlaneRestoreNamed");
    QObject::connect(named,&QMenu::aboutToShow,menu,[named,target]{populateNamedMenu(named,target);});
    QObject::connect(menu,&QMenu::aboutToShow,menu,[menu,previous,next,named,target]{
        for(auto* action:menu->actions())action->setEnabled(true);
        try{const auto context=menuContext(target);previous->setEnabled(om9_cplane_history_available(context.owner,context.slot+1,false));next->setEnabled(om9_cplane_history_available(context.owner,context.slot+1,true));named->menuAction()->setEnabled(om9_cplane_named_count(context.owner)!=0);menu->setToolTip({});}
        catch(const std::exception& error){for(auto* action:menu->actions())action->setEnabled(false);menu->setToolTip(QString::fromUtf8(error.what()));}
        catch(...){for(auto* action:menu->actions())action->setEnabled(false);menu->setToolTip("CPlane unavailable");}
    });
    return menu;
}
void addCPlaneMethods(PyObject* module) {
    static PyMethodDef methods[]={
        {"cplane",getAPI,METH_VARARGS,"Read document/slot CPlane as origin,X,Y,Z (12 world-mm values)."},
        {"setCPlane",setAPI,METH_VARARGS,"Set document/slot CPlane from validated origin,X,Y,Z without changing camera or geometry."},
        {"setCPlane3Point",threePointAPI,METH_VARARGS,"Set document/slot CPlane from origin, X point, Y-side point (9 world-mm values)."},
        {"previousCPlane",previousAPI,METH_VARARGS,"Restore previous plane in this document/slot without document geometry Undo."},
        {"nextCPlane",nextAPI,METH_VARARGS,"Restore next plane in this document/slot."},
        {"saveNamedCPlane",saveNamedAPI,METH_VARARGS,"Save document/slot with a unique nonempty label; return persistent numeric identity."},
        {"restoreNamedCPlane",restoreNamedAPI,METH_VARARGS,"Restore document-scoped named plane identity into a slot."},
        {"renameNamedCPlane",renameNamedAPI,METH_VARARGS,"Rename document-scoped identity; reject duplicate labels."},
        {"namedCPlanes",namedAPI,METH_VARARGS,"Read document-scoped named plane records."},
        {nullptr,nullptr,0,nullptr}};
    if(PyModule_AddFunctions(module,methods)<0)throw Base::RuntimeError("Cannot register construction-plane API");
}
}
