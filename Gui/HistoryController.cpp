// SPDX-License-Identifier: LGPL-2.1-or-later
#include "HistoryController.h"
#include "HistoryFeature.h"
#include "RustBridge.h"
#include "CurveController.h"
#include "CoreKeyboard.h"
#include <App/Application.h>
#include <App/Document.h>
#include <Base/Interpreter.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Selection/Selection.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <QApplication>
#include <QDialog>
#include <QVBoxLayout>
#include <QDialogButtonBox>
#include <QCheckBox>
#include <QDoubleSpinBox>
#include <QLabel>
#include <QKeyEvent>
#include <QTimer>
#include <QSignalBlocker>
#include <cmath>
extern "C" unsigned om9_history_command_kind(const char*);
extern "C" unsigned om9_history_option(const char*);
extern "C" int om9_history_parse_boolean(const char*);
namespace OpenMatrix9Gui {
HistoryController& HistoryController::instance(){static auto* c=new HistoryController;return *c;}
HistoryController::HistoryController():QObject(qApp){
    qApp->installEventFilter(this);
    deleted=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& d){if(document==&d)cancel();});
    changed=App::GetApplication().signalActiveDocument.connect([this](const App::Document& d){if(active()&&document!=&d)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool HistoryController::handles(std::size_t i){return om9_history_command_kind(om9_command_id(i))!=0;}
bool HistoryController::matches(std::size_t i,const QString& text){return handles(i)&&om9_history_command_kind(text.toUtf8().constData())==om9_history_command_kind(om9_command_id(i));}
bool HistoryController::active()const{return document&&kind!=0;}
bool HistoryController::valid()const{auto* g=Gui::Application::Instance->activeDocument();return document==App::GetApplication().getActiveDocument()&&document&&g&&!g->isAboutToClose()&&!g->getInEdit()&&Gui::Control().isAllowedAlterDocument(document);}
bool HistoryController::available(std::size_t i)const{auto* d=App::GetApplication().getActiveDocument();auto* g=Gui::Application::Instance->activeDocument();return handles(i)&&d&&g&&!g->getInEdit()&&!g->isAboutToClose()&&Gui::Control().isAllowedAlterDocument(d);}
void HistoryController::prompt(const QString& s){CurveController::instance().setPrompt(s);CurveController::instance().logMessage(s);}
void HistoryController::cancel(){document=nullptr;kind=0;{Base::PyGILStateLocker lock;inputs.clear();}if(dialog){auto* d=dialog.data();dialog=nullptr;status=nullptr;d->hide();d->deleteLater();}}
bool HistoryController::start(std::size_t i){
    if(!available(i))return false;cancel();CurveController::instance().cancel();document=App::GetApplication().getActiveDocument();command=i;kind=om9_history_command_kind(om9_command_id(i));tolerance=1e-7;
    try{
        if(kind==1){settingsOptions();return false;}
        if(kind==5){prompt("JoinHistory: Select separate curves; Enter / Undo / Cancel. Tolerance=mm");Base::PyGILStateLocker lock;
            for(const auto& s:Gui::Selection().getSelection(document->getName())){if(s.SubName&&*s.SubName)throw std::runtime_error("Join History requires whole curves");add(s.FeatName);}return false;}
        document->openTransaction(kind==2?"Clear Object History":kind==3?"History Record":"History Update");
        try{
            if(kind==2){auto selection=Gui::Selection().getSelection(document->getName());if(selection.empty())throw std::runtime_error("Select objects whose History should be cleared");for(const auto& s:selection){if(s.SubName&&*s.SubName)throw std::runtime_error("Clear Object History requires whole objects");detachObjectHistory(document->getObject(s.FeatName));}}
            else{auto* s=historySettings(*document);auto& p=kind==3?s->Record:s->Update;p.setValue(!p.getValue());document->recompute();}
            document->commitTransaction();
        }catch(...){document->abortTransaction();throw;}
        auto* s=historySettings(*document,false);prompt(QString("History: Record=%1 Update=%2. Command:").arg(!s||s->Record.getValue()?"Yes":"No").arg(!s||s->Update.getValue()?"Yes":"No"));cancel();return true;
    }catch(const std::exception& e){prompt(QString("History: ")+e.what());cancel();return false;}
}
void HistoryController::add(const std::string& name){for(const auto& i:inputs)if(i.name==name)return;inputs.push_back(editInput(*document,name,1));}
void HistoryController::setOption(unsigned option,bool value){
    document->openTransaction("History policy");try{auto* s=historySettings(*document);App::PropertyBool* p=option==1?&s->Record:option==2?&s->Update:option==3?&s->Lock:option==4?&s->BrokenHistoryWarning:nullptr;
        if(!p)throw std::runtime_error("Unknown History option");p->setValue(value);document->recompute();document->commitTransaction();
        if(dialog)if(auto* box=dialog->findChild<QCheckBox*>(QString("OM9HistoryOption%1").arg(option))){QSignalBlocker block(box);box->setChecked(value);}
        prompt(QString("History: %1=%2. Record / Update / Lock / BrokenHistoryWarning = Yes/No; Enter closes").arg(option==1?"Record":option==2?"Update":option==3?"Lock":"BrokenHistoryWarning").arg(value?"Yes":"No"));
    }catch(...){document->abortTransaction();throw;}
}
void HistoryController::settingsOptions(){
    auto* s=historySettings(*document,false);dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9HistoryOptions");dialog->setWindowTitle("History");auto* layout=new QVBoxLayout(dialog);
    const bool values[]={!s||s->Record.getValue(),!s||s->Update.getValue(),s&&s->Lock.getValue(),!s||s->BrokenHistoryWarning.getValue()};const char* names[]={"Record","Update","Lock child geometry","Broken History Warning"};
    for(unsigned i=0;i<4;++i){auto* box=new QCheckBox(names[i],dialog);box->setObjectName(QString("OM9HistoryOption%1").arg(i+1));box->setChecked(values[i]);layout->addWidget(box);connect(box,&QCheckBox::toggled,this,[this,i](bool b){try{setOption(i+1,b);}catch(const std::exception& e){prompt(QString("History: ")+e.what());}});}
    auto* done=new QDialogButtonBox(QDialogButtonBox::Close,dialog);layout->addWidget(done);connect(done,&QDialogButtonBox::rejected,this,[this]{cancel();prompt("Command:");});connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Command:");});dialog->show();prompt("History: Record / Update / Lock / BrokenHistoryWarning = Yes/No; Enter closes");
}
void HistoryController::joinOptions(){
    verifyEditInputs(*document,inputs);if(inputs.size()<2)throw std::runtime_error("Select at least two curves");if(dialog)return;
    dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9JoinHistoryOptions");dialog->setWindowTitle("Join History");auto* layout=new QVBoxLayout(dialog);
    auto* spin=new QDoubleSpinBox(dialog);spin->setObjectName("OM9JoinHistoryTolerance");spin->setDecimals(9);spin->setRange(1e-9,1e6);spin->setValue(tolerance);spin->setSuffix(" mm — Join tolerance");layout->addWidget(spin);connect(spin,&QDoubleSpinBox::valueChanged,this,[this](double t){tolerance=t;});
    status=new QLabel("OK creates the joined child and preserves the separate parent curves.",dialog);status->setObjectName("OM9JoinHistoryStatus");status->setWordWrap(true);layout->addWidget(status);
    auto* buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);layout->addWidget(buttons);connect(buttons,&QDialogButtonBox::accepted,this,[this]{commit();});connect(buttons,&QDialogButtonBox::rejected,this,[this]{cancel();prompt("Command:");});connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Command:");});dialog->show();prompt("JoinHistory: OK / Cancel; Tolerance=mm");
}
void HistoryController::commit(){
    if(!valid()){cancel();return;}try{Base::PyGILStateLocker lock;verifyEditInputs(*document,inputs);std::vector<std::string> names;for(const auto& i:inputs)names.push_back(i.name);createHistoryJoin(*document,names,tolerance);om9_sidebar_record_execution(command,true);cancel();prompt("Join History completed. Command:");}
    catch(const std::exception& e){if(status)status->setText(e.what());prompt(QString("JoinHistory: ")+e.what());}
}
void HistoryController::submit(const QString& text){
    if(!active())return;if(!valid()){cancel();return;}auto s=text.trimmed();if(s.compare("Cancel",Qt::CaseInsensitive)==0||s.compare("Esc",Qt::CaseInsensitive)==0){cancel();prompt("Command:");return;}
    try{Base::PyGILStateLocker lock;
        const auto equal=s.indexOf('=');if(equal>0){auto key=s.left(equal).trimmed();key.remove(' ');auto value=s.mid(equal+1).trimmed();
            if(kind==1){auto option=om9_history_option(key.toUtf8().constData());auto b=om9_history_parse_boolean(value.toUtf8().constData());if(!option||b<0)throw std::runtime_error("History options require Yes or No");setOption(option,b==1);return;}
            if(kind==5&&key.compare("Tolerance",Qt::CaseInsensitive)==0){bool ok;double next=value.toDouble(&ok);if(!ok||!std::isfinite(next)||next<=0)throw std::runtime_error("Tolerance must be finite and positive");tolerance=next;if(dialog)dialog->findChild<QDoubleSpinBox*>("OM9JoinHistoryTolerance")->setValue(tolerance);return;}
            throw std::runtime_error("Unknown History option");}
        if(s.isEmpty()||s.compare("OK",Qt::CaseInsensitive)==0){if(kind==1){cancel();prompt("Command:");}else if(dialog)commit();else{for(const auto& selection:Gui::Selection().getSelection(document->getName())){if(selection.SubName&&*selection.SubName)throw std::runtime_error("Select whole curves");add(selection.FeatName);}joinOptions();}return;}
        if(kind==5&&!dialog){if(s.compare("Undo",Qt::CaseInsensitive)==0){if(!inputs.empty()){Gui::Selection().rmvSelection(document->getName(),inputs.back().name.c_str());inputs.pop_back();}}else add(s.toStdString());return;}
        throw std::runtime_error("Use History options, OK or Cancel");
    }catch(const std::exception& e){prompt(QString("History: ")+e.what());}
}
bool HistoryController::eventFilter(QObject* watched,QEvent* e){
    if(!active()||e->type()!=QEvent::KeyPress||!CoreKeyboard::inputContext(watched))return false;auto* key=static_cast<QKeyEvent*>(e);
    if(key->key()==Qt::Key_Escape){cancel();prompt("Command:");return true;}
    if(kind==5&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter)){
        auto* g=Gui::Application::Instance->activeDocument();auto* v=g?dynamic_cast<Gui::View3DInventor*>(g->getActiveView()):nullptr;auto* w=qobject_cast<QWidget*>(watched);
        if(v&&w&&(w==v->getViewer()||v->getViewer()->isAncestorOf(w))){submit({});return true;}
    }return false;
}
}
