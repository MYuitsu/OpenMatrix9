#include "CoreNotes.h"
#include "RustBridge.h"
#include "CoreViewControls.h"
#include "CurveController.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <Base/Console.h>
#include <Base/Exception.h>
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

namespace {
bool enabled=false;
App::DocumentObject* storage(const App::Document& doc) {
    for(auto* object:doc.getObjects())if(OpenMatrix9Gui::CoreNotes::isStorageObject(object))return object;
    return nullptr;
}
void synchronize(const App::Document& document) {
    auto& doc=const_cast<App::Document&>(document);auto metadata=doc.Meta.getValues();
    if(metadata.find("OpenMatrix9.ProjectNotesStorage")==metadata.end())return;
    auto* object=storage(doc);auto* text=object?dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes")):nullptr;
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
    enabled=true;
}
void CoreNotes::deactivate(){enabled=false;}
bool CoreNotes::handles(std::size_t index){const auto name=id(index);return name=="OM9_FileNotes"||name=="OM9_ProjectNotes";}
const char* CoreNotes::alias(std::size_t index){return id(index)=="OM9_FileNotes"?"Notes":id(index)=="OM9_ProjectNotes"?"ProjectNotes":nullptr;}
bool CoreNotes::isStorageObject(const App::DocumentObject* object){
    if(!object)return false;auto* tag=dynamic_cast<App::PropertyString*>(object->getPropertyByName("OM9NotesSchema"));return tag&&std::string(tag->getValue())=="1"&&dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes"));
}
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
        if(!object){object=doc->addObject("App::FeaturePython","OM9ProjectNotesData");object->Label.setValue("Project Notes");auto* tag=dynamic_cast<App::PropertyString*>(object->addDynamicProperty("App::PropertyString","OM9NotesSchema","OpenMatrix9",nullptr,0,true,true));tag->setValue("1");auto* baseline=dynamic_cast<App::PropertyString*>(object->addDynamicProperty("App::PropertyString","Notes","OpenMatrix9",nullptr,0,false,true));baseline->setValue(previous.c_str());}
        auto* text=dynamic_cast<App::PropertyString*>(object->getPropertyByName("Notes"));if(!text)throw Base::RuntimeError("Invalid project notes storage");
        if(auto* gui=Gui::Application::Instance->getDocument(doc))if(auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(object))){provider->ShowInTree.setValue(false);provider->Visibility.setValue(false);}
        auto metadata=doc->Meta.getValues();metadata["OpenMatrix9.ProjectNotesStorage"]="v1";doc->Meta.setValues(std::move(metadata));
        doc->openTransaction("Project Notes");text->setValue(proposed.constData());doc->commitTransaction();synchronize(*doc);return true;
    }catch(const Base::Exception& error){doc->abortTransaction();Base::Console().error("OpenMatrix9 Notes: %s\n",error.what());return false;}
}
}
