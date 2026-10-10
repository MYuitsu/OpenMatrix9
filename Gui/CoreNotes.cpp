#include "CoreNotes.h"
#include "RustBridge.h"
#include "CoreViewControls.h"
#include "CurveController.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyPythonObject.h>
#include <App/PropertyStandard.h>
#include <Base/Console.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <Gui/Control.h>
#include <Gui/MainWindow.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <fastsignals/signal.h>
#include <QDialog>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QVBoxLayout>
#include <QCloseEvent>
#include <QMessageBox>
#include <string>
#include <set>

namespace OpenMatrix9Gui {
// Native API exception: persisted helper identity/view provider requires a
// registered DocumentObject type. This class owns no geometry or layer policy.
class ProjectNotesObject final:public App::DocumentObject {
    PROPERTY_HEADER_WITH_OVERRIDE(OpenMatrix9Gui::ProjectNotesObject);
public:
    App::PropertyString OM9NotesSchema,Notes;
    ProjectNotesObject() {
        ADD_PROPERTY_TYPE(OM9NotesSchema,("1"),"OpenMatrix9",static_cast<App::PropertyType>(App::Prop_ReadOnly|App::Prop_Hidden|App::Prop_NoRecompute),"Native project notes identity");
        ADD_PROPERTY_TYPE(Notes,(""),"OpenMatrix9",static_cast<App::PropertyType>(App::Prop_Hidden|App::Prop_NoRecompute),"Project notes text");
    }
    const char* getViewProviderName()const override {return "Gui::ViewProviderDocumentObject";}
};
PROPERTY_SOURCE(OpenMatrix9Gui::ProjectNotesObject,App::DocumentObject)
void CoreNotes::registerTypes(){ProjectNotesObject::init();}
}

namespace {
bool enabled=false;
bool notesStorageIdentity(const App::DocumentObject* object,bool requireTextWitness);
App::DocumentObject* storage(const App::Document& doc) {
    for(auto* object:doc.getObjects())if(OpenMatrix9Gui::CoreNotes::isStorageObject(object))return object;
    return nullptr;
}
void hideVerifiedHelpers() {
    // Legacy FeaturePython view providers can restore their default visible
    // state after GuiDocument data. Run after the complete native open lifecycle
    // and touch only helpers whose persisted identity/text context is verified.
    for(auto* doc:App::GetApplication().getDocuments()) {
        auto* gui=Gui::Application::Instance->getDocument(doc);if(!gui)continue;
        for(auto* object:doc->getObjects())if(OpenMatrix9Gui::CoreNotes::isStorageObject(object))
            if(auto* view=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(object))) {
                if(view->ShowInTree.getValue())view->ShowInTree.setValue(false);
                if(view->Visibility.getValue())view->Visibility.setValue(false);
            }
    }
}
void synchronize(const App::Document& document) {
    auto& doc=const_cast<App::Document&>(document);auto metadata=doc.Meta.getValues();
    if(metadata.find("OpenMatrix9.ProjectNotesStorage")==metadata.end())return;
    // Native Undo/Redo restores Notes but Document.Meta is not an object Undo
    // property. Verify the same complete factory recipe here, then repair the
    // text witness. Public model inventory still requires that witness to match.
    App::DocumentObject* object=nullptr;
    for(auto* candidate:doc.getObjects())if(notesStorageIdentity(candidate,false)){object=candidate;break;}
    auto* text=object?dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes")):nullptr;
    if(object)if(auto* gui=Gui::Application::Instance->getDocument(&doc))if(auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(object))){provider->ShowInTree.setValue(false);provider->Visibility.setValue(false);}
    if(text&&*text->getValue())metadata[om9_notes_metadata_key()]=text->getValue();else metadata.erase(om9_notes_metadata_key());
    if(metadata!=doc.Meta.getValues())doc.Meta.setValues(std::move(metadata));
}
std::string id(std::size_t index){const char* value=om9_command_id(index);return value?value:"";}
class NotesDialog final:public QDialog {
public:
    explicit NotesDialog(bool fileNotes):QDialog(Gui::getMainWindow()),saveOnClose(fileNotes) {
        setObjectName("OM9ProjectNotes");setWindowTitle(fileNotes?"Notes":"Project Notes");resize(600,400);
        auto* layout=new QVBoxLayout(this);editor=new QPlainTextEdit(this);editor->setObjectName("OM9NotesText");layout->addWidget(editor);
        if(!fileNotes){auto* done=new QPushButton("Done",this);done->setObjectName("OM9NotesDone");layout->addWidget(done);connect(done,&QPushButton::clicked,this,&QDialog::accept);}
    }
    QPlainTextEdit* editor;
protected:
    void closeEvent(QCloseEvent* event)override {if(saveOnClose)accept();else reject();event->accept();}
private:
    bool saveOnClose;
};
}
namespace OpenMatrix9Gui {
void CoreNotes::activate(){
    static fastsignals::scoped_connection undo=App::GetApplication().signalUndoDocument.connect(synchronize);
    static fastsignals::scoped_connection redo=App::GetApplication().signalRedoDocument.connect(synchronize);
    static fastsignals::scoped_connection opened=App::GetApplication().signalFinishOpenDocument.connect(hideVerifiedHelpers);
    hideVerifiedHelpers();
    enabled=true;
}
void CoreNotes::deactivate(){enabled=false;}
bool CoreNotes::handles(std::size_t index){const auto name=id(index);return name=="OM9_FileNotes"||name=="OM9_ProjectNotes";}
const char* CoreNotes::alias(std::size_t index){return id(index)=="OM9_FileNotes"?"Notes":id(index)=="OM9_ProjectNotes"?"ProjectNotes":nullptr;}
}
namespace {
bool notesStorageIdentity(const App::DocumentObject* object,bool requireTextWitness){
    if(!object)return false;
    // Verify the actual native factory recipe. Any extra dynamic property,
    // including CAD/mesh/cloud/retained payload or container links, means this
    // object must be inventoried and explicitly preflighted as model content.
    static const std::set<std::string> fields{"OM9NotesSchema","Notes","OM9LayerObjectId","OM9LayerHostName","OM9LayerId","OM9Locked"};
    for(const auto& name:object->getDynamicPropertyNames())if(!fields.count(name))return false;
    auto* tag=dynamic_cast<App::PropertyString*>(object->getPropertyByName("OM9NotesSchema"));
    auto* notes=dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes"));
    if(!tag || std::string(tag->getValue())!="1" || !notes)return false;
    if(object->getTypeId()==OpenMatrix9Gui::ProjectNotesObject::getClassTypeId())return true;
    // Old accepted FCStd used FeaturePython with an empty Proxy, immutable
    // factory name and read-only schema. Markers alone never establish identity.
    // FeaturePython's template getClassTypeId is not exported by this SDK DLL;
    // compare the actual registered native type through Base's public registry.
    if(object->getTypeId()!=Base::Type::fromName("App::FeaturePython") ||
       std::string(object->getNameInDocument())!="OM9ProjectNotesData" ||
       (tag->getType()&(App::Prop_ReadOnly|App::Prop_Hidden))!=(App::Prop_ReadOnly|App::Prop_Hidden) ||
       !(notes->getType()&App::Prop_Hidden))return false;
    const auto* doc=object->getDocument();if(!doc)return false;
    const auto& meta=doc->Meta.getValues();auto version=meta.find("OpenMatrix9.ProjectNotesStorage");
    if(version==meta.end() || version->second!="v1")return false;
    auto text=meta.find(om9_notes_metadata_key());
    if(requireTextWitness && (text==meta.end()?std::string():text->second)!=notes->getValue())return false;
    auto* proxy=dynamic_cast<App::PropertyPythonObject*>(object->getPropertyByName("Proxy"));
    if(!proxy)return false;
    Base::PyGILStateLocker lock;
    return proxy->getValue().ptr()==Py_None;
}
}
namespace OpenMatrix9Gui {
bool CoreNotes::isStorageObject(const App::DocumentObject* object){return notesStorageIdentity(object,true);}
bool CoreNotes::available(std::size_t index){auto* doc=App::GetApplication().getActiveDocument();return enabled&&handles(index)&&doc&&Gui::Control().isAllowedAlterDocument(doc);}
bool CoreNotes::execute(std::size_t index) {
    if(!available(index))return false;
    CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();
    auto* original=App::GetApplication().getActiveDocument();const std::string name=original->getName();
    const std::string key=om9_notes_metadata_key();const auto& values=original->Meta.getValues();auto found=values.find(key);const std::string previous=found==values.end()?"":found->second;
    NotesDialog dialog(id(index)=="OM9_FileNotes");dialog.editor->setPlainText(QString::fromUtf8(previous.data(),qsizetype(previous.size())));dialog.editor->setFocus();
    if(dialog.exec()!=QDialog::Accepted)return false;
    auto* doc=App::GetApplication().getDocument(name.c_str());
    if(doc!=original||!enabled||!Gui::Control().isAllowedAlterDocument(doc))return false;
    const auto proposed=dialog.editor->toPlainText().toUtf8();
    const auto decision=om9_notes_decide(reinterpret_cast<const unsigned char*>(previous.data()),previous.size(),reinterpret_cast<const unsigned char*>(proposed.constData()),std::size_t(proposed.size()));
    if(decision==2){QMessageBox::warning(Gui::getMainWindow(),"Notes","Notes must be valid text without NUL and at most 1 MiB.");return false;}
    if(decision==0)return false;
    try {
        auto* object=storage(*doc);
        if(!object){object=doc->addObject("OpenMatrix9Gui::ProjectNotesObject","OM9ProjectNotesData");object->Label.setValue("Project Notes");auto* baseline=dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes"));baseline->setValue(previous.c_str());}
        auto* text=dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes"));if(!text)throw Base::RuntimeError("Invalid project notes storage");
        if(auto* gui=Gui::Application::Instance->getDocument(doc))if(auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(object))){provider->ShowInTree.setValue(false);provider->Visibility.setValue(false);}
        auto metadata=doc->Meta.getValues();metadata["OpenMatrix9.ProjectNotesStorage"]="v1";doc->Meta.setValues(std::move(metadata));
        doc->openTransaction("Project Notes");text->setValue(proposed.constData());
        // Keep the legacy identity's text witness in the same owned Undo record
        // as Notes. Observers must see a consistent factory context at commit.
        auto committedMetadata=doc->Meta.getValues();
        if(proposed.isEmpty())committedMetadata.erase(key);else committedMetadata[key]=proposed.constData();
        doc->Meta.setValues(std::move(committedMetadata));
        doc->commitTransaction();synchronize(*doc);return true;
    }catch(const Base::Exception& error){doc->abortTransaction();Base::Console().error("OpenMatrix9 Notes: %s\n",error.what());return false;}
}
}
