#include <Base/Interpreter.h>
#include "CoreKeyboard.h"
#include "RustBridge.h"
#include "NativeCommands.h"
#include "CoreWorkspace.h"
#include "CurveController.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Base/PyObjectBase.h>
#include <Gui/Application.h>
#include <Gui/MainWindow.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/Selection/Selection.h>
#include <Gui/Command.h>
#include <Gui/Action.h>
#include <QApplication>
#include <QDockWidget>
#include <QDialog>
#include <QTextEdit>
#include <QPlainTextEdit>
#include <QLineEdit>
#include <QAbstractSpinBox>
#include <QKeyEvent>
#include <QKeySequence>
#include <QMenu>
#include <QLabel>
#include <QDir>
#include <QFile>
#include <QDomDocument>
#include <QCursor>
#include <QSignalBlocker>
#include <limits>
namespace {
std::size_t find(const QString& name){for(std::size_t i=0;i<om9_command_count();++i)if(name.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0)return i;return std::numeric_limits<std::size_t>::max();}
QString id(std::size_t command){auto* value=om9_command_id(command);return value?QString::fromUtf8(value):QString();}
auto prefs(){return App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/Keyboard");}
void message(const QString& text){OpenMatrix9Gui::CurveController::instance().setPrompt(text);OpenMatrix9Gui::CurveController::instance().logMessage(text);}
QString selectedType(){
    auto* document=App::GetApplication().getActiveDocument();if(!document)return "Empty";
    const auto objects=Gui::Selection().getSelection(document->getName());if(objects.empty())return "Empty";
    if(objects.size()!=1)return "Default";
    Base::PyGILStateLocker lock;PyObject* object=objects.front().pObject->getPyObject();
    PyObject* shape=PyObject_GetAttrString(object,"Shape");Py_DECREF(object);if(!shape){PyErr_Clear();return "Default";}
    PyObject* faces=PyObject_GetAttrString(shape,"Faces");PyObject* edges=PyObject_GetAttrString(shape,"Edges");
    const auto faceCount=faces?PyObject_Length(faces):0;const auto edgeCount=edges?PyObject_Length(edges):0;
    Py_XDECREF(faces);Py_XDECREF(edges);Py_DECREF(shape);PyErr_Clear();
    return faceCount>1?"ObjPolysurface":faceCount==1?"Surface":edgeCount>0?"Curve":"Default";
}
QString normalized(QString command){command=command.trimmed();while(command.startsWith('_')||command.startsWith('!'))command.remove(0,1);return command;}
}
namespace OpenMatrix9Gui {
CoreKeyboard& CoreKeyboard::instance(){static auto* value=new CoreKeyboard;return *value;}
CoreKeyboard::CoreKeyboard():QObject(qApp){}
void CoreKeyboard::activate(){enabled=true;om9_ortho_load(prefs()->GetBool("Ortho",false));qApp->installEventFilter(this);}
void CoreKeyboard::deactivate(){enabled=false;if(historyDock)historyDock->hide();if(menu)menu->close();}
bool CoreKeyboard::handles(std::size_t command){const auto name=id(command);return name=="CommandHistory"||name=="Properties"||name=="Ortho"||name=="F6";}
bool CoreKeyboard::checked(std::size_t command){return id(command)=="Ortho"&&om9_ortho_enabled();}
bool CoreKeyboard::inputContext(QObject* receiver){
    auto* widget=qobject_cast<QWidget*>(receiver);auto* window=Gui::getMainWindow();
    return widget&&(widget==window||window->isAncestorOf(widget))&&!qobject_cast<QDialog*>(widget->window())&&!QApplication::activeModalWidget()&&!QApplication::activePopupWidget();
}
bool CoreKeyboard::pointKeyAllowed(QObject* receiver,const QKeyEvent* key){return inputContext(receiver)&&!key->isAutoRepeat()&&(key->modifiers()&~Qt::KeypadModifier)==Qt::NoModifier;}
bool CoreKeyboard::available(std::size_t command)const{
    if(!enabled||!handles(command)||Gui::Application::Instance->isClosing())return false;
    if(id(command)=="CommandHistory")return true;
    auto* document=App::GetApplication().getActiveDocument();if(!document)return false;
    if(id(command)=="Ortho")return om9AlterView(document);
    if(id(command)=="Properties")return !Gui::Selection().getSelection(document->getName()).empty()&&om9AlterSelection(document);
    return om9AlterSelection(document);
}
void CoreKeyboard::record(const QString& text){if(enabled&&!text.trimmed().isEmpty()){inputs.append("Command: "+text);if(inputs.size()>10000)inputs.removeFirst();}}
bool CoreKeyboard::submit(const QString& text){const auto command=find(text.trimmed());if(!handles(command))return false;om9_sidebar_record_execution(command,execute(command));return true;}
bool CoreKeyboard::execute(std::size_t command){
    if(!available(command))return false;
    const auto name=id(command);
    if(name=="Ortho"){
        prefs()->SetBool("Ortho",om9_ortho_toggle());
        auto* native=Gui::Application::Instance->commandManager().getCommandByName("Ortho");if(native&&native->getAction())native->getAction()->setBlockedChecked(om9_ortho_enabled());
        return true;
    }
    if(name=="CommandHistory"){history();return true;}
    if(name=="F6"){contextMenu();return true;}
    auto* window=Gui::getMainWindow();auto* dock=window->findChild<QDockWidget*>("Std_PropertyView");if(!dock)dock=window->findChild<QDockWidget*>("Property view");if(!dock)dock=window->findChild<QDockWidget*>("Combo View");if(!dock)dock=window->findChild<QDockWidget*>("Model");
    if(!dock){message("Properties: native property inspector is unavailable");return false;}
    dock->show();dock->raise();dock->setFocus();return true;
}
void CoreKeyboard::history(){
    auto* window=Gui::getMainWindow();if(!historyDock){historyDock=new QDockWidget("Command History",window);historyDock->setObjectName("OM9CommandHistory");auto* text=new QTextEdit(historyDock);text->setReadOnly(true);historyDock->setWidget(text);window->addDockWidget(Qt::BottomDockWidgetArea,historyDock);historyDock->hide();}
    if(historyDock->isVisible()){historyDock->hide();return;}
    QString contents=inputs.join('\n');auto* report=window->findChild<QTextEdit*>("Report view");
    if(!report)if(auto* legacy=window->findChild<QWidget*>("ReportOutput"))report=legacy->findChild<QTextEdit*>();
    if(report)contents+="\n\n"+report->toPlainText();
    auto* text=historyDock->findChild<QTextEdit*>();text->setPlainText(contents);text->moveCursor(QTextCursor::End);historyDock->show();historyDock->raise();
}
void CoreKeyboard::contextMenu(){
    auto* window=Gui::getMainWindow();if(menu){menu->close();delete menu.data();}menu=new QMenu(window);menu->setObjectName("OM9F6Menu");const auto type=selectedType();menu->setProperty("om9SelectionType",type);
    const auto root=qEnvironmentVariable("OM9_PLUGIN_ROOT");
    QFile file(root.isEmpty()?QDir(QString::fromStdString(App::Application::getHomePath())).filePath("Mod/OpenMatrix9/Resources/menu/ContextMenu.xml"):QDir(root).filePath("Resources/menu/ContextMenu.xml"));QDomDocument source;if(file.open(QIODevice::ReadOnly))source.setContent(&file);
    const QStringList names={"General","Curve","Gem","Surface","T-Splines","User","Report","Materials"};const QStringList modes={"ObjectActions","CurveLayout","GemLayout","SurfaceModeling","TSplines","User","Report","Materials"};
    const auto groups=source.elementsByTagName("ObjectContextMenuGroup");
    for(int mode=0;mode<names.size();++mode){auto* submenu=menu->addMenu(names[mode]);QDomElement chosen;
        for(int pass=0;pass<2&&chosen.isNull();++pass)for(int g=0;g<groups.size();++g){const auto group=groups.at(g).toElement();const auto wanted=pass==0?type:QString("Default");if(group.firstChildElement("Mode").text()==modes[mode]&&group.firstChildElement("EnglishName").text()==wanted){chosen=group;break;}}
        const auto items=chosen.firstChildElement("MenuItems").elementsByTagName("ObjectContextMenuItem");
        for(int a=0;a<items.size();++a){auto item=items.at(a).toElement();const auto name=item.firstChildElement("ActionName").text();const auto command=normalized(item.firstChildElement("ActionCommand").text());if(command=="NewLine"){submenu->addSeparator();continue;}
            auto* action=submenu->addAction(name);action->setProperty("om9OriginalCommand",command);const auto index=find(command);const bool available=index<om9_command_count()&&om9NativeCommandAvailable(index);action->setEnabled(available);
            if(available)connect(action,&QAction::triggered,this,[this,index]{menu->close();om9_sidebar_record_execution(index,om9ExecuteNativeCommand(index));});
        }
        if(submenu->actions().isEmpty()){auto* action=submenu->addAction("Chưa có lệnh khả dụng cho lựa chọn này");action->setEnabled(false);}
    }
    menu->popup(QCursor::pos());
}
bool CoreKeyboard::eventFilter(QObject* receiver,QEvent* event){
    if(!enabled||Gui::Application::Instance->isClosing()||(event->type()!=QEvent::ShortcutOverride&&event->type()!=QEvent::KeyPress))return false;
    if(!inputContext(receiver))return false;
    const auto* key=static_cast<QKeyEvent*>(event);const auto modifiers=key->modifiers()&~Qt::KeypadModifier;const auto sequence=QKeySequence(key->key()|int(modifiers)).toString(QKeySequence::PortableText);
    QString target;for(std::size_t i=0;i<om9_keyboard_count();++i)if(sequence==QString::fromUtf8(om9_keyboard_key(i))){target=QString::fromUtf8(om9_keyboard_target(i));break;}
    // The table has no text-editing combinations. In particular Ctrl+Q/W must
    // reserve Matrix group commands even while the command input has focus.
    if(target.isEmpty())return false;
    if(event->type()==QEvent::ShortcutOverride){event->accept();return true;}
    if(key->isAutoRepeat())return true;
    // These keys are context-sensitive native point/view operations, not commands
    // that can be replaced with a global menu action.
    if(target=="@origin")return false;
    if(target=="@grid"){CoreWorkspace::instance().toggleActiveGrid();return true;}
    const auto command=find(target);
    if(command<om9_command_count()&&om9NativeCommandAvailable(command))om9_sidebar_record_execution(command,om9ExecuteNativeCommand(command));
    else message(target+": chức năng chưa khả dụng cho lựa chọn hiện tại");
    return true;
}
}
