// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CommandOptions.h"
#include <QStringList>
#include <cstring>
#include <vector>
#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QKeyEvent>
#include <QLineEdit>
#include <QPointer>
#include <QSignalBlocker>
namespace OpenMatrix9Gui {
namespace {
QString readMessage(std::uint64_t h,unsigned field){unsigned char b[2048]={};om9_options_message(h,field,b,sizeof(b));return QString::fromUtf8(reinterpret_cast<char*>(b));}
}
bool CommandOptionsBridge::open(unsigned family,double scale,const QString& featureId){close();const auto id=featureId.toUtf8();handle=id.isEmpty()?om9_options_open(family,scale):om9_options_open_named(family,scale,reinterpret_cast<const unsigned char*>(id.constData()),std::size_t(id.size()));return active();}
CommandOptionDefinition CommandOptionDefinition::number(const QString& key,double minimum,double maximum,bool integer,double current,const QString& unit){CommandOptionDefinition d;d.key=key;d.kind=3;d.minimum=minimum;d.maximum=maximum;d.integer=integer;d.unit=unit;d.initialKind=3;d.initialNumber=current;return d;}
CommandOptionDefinition CommandOptionDefinition::toggle(const QString& key,bool current){CommandOptionDefinition d;d.key=key;d.kind=1;d.initialKind=1;d.initialNumber=current?1.:0.;return d;}
CommandOptionDefinition CommandOptionDefinition::action(const QString& key){CommandOptionDefinition d;d.key=key;return d;}
bool CommandOptionsBridge::openFeature(const QString& id,const QVector<CommandOptionDefinition>& definitions,const QVector<CommandOptionGreater>& rules,double scale){
    close();const auto feature=id.toUtf8();handle=om9_options_begin(reinterpret_cast<const unsigned char*>(feature.constData()),std::size_t(feature.size()),scale);if(!handle)return false;
    auto bytes=[](const QByteArray& b){return reinterpret_cast<const unsigned char*>(b.constData());};
    for(const auto& d:definitions){const auto key=d.key.toUtf8(),unit=d.unit.toUtf8(),parent=d.parent.toUtf8();
        if(!om9_options_define(handle,bytes(key),std::size_t(key.size()),d.kind,d.minimum,d.maximum,d.integer,bytes(unit),std::size_t(unit.size()),bytes(parent),std::size_t(parent.size()),d.scope)){close();return false;}
        if(!d.shown&&!om9_options_shown(handle,bytes(key),std::size_t(key.size()),false)){close();return false;}
        for(const auto& c:d.choices){const auto value=c.value.toUtf8(),child=c.child.toUtf8(),reason=c.reason.toUtf8();if(!om9_options_choice(handle,bytes(key),std::size_t(key.size()),bytes(value),std::size_t(value.size()),bytes(child),std::size_t(child.size()),c.enabled,bytes(reason),std::size_t(reason.size()))){close();return false;}
            const auto identity=(d.key+"="+c.value).toUtf8();for(const auto& a:c.aliases){const auto alias=a.toUtf8();if(!om9_options_alias(handle,bytes(identity),std::size_t(identity.size()),bytes(alias),std::size_t(alias.size()))){close();return false;}}}
        for(const auto& a:d.aliases){const auto alias=a.toUtf8();if(!om9_options_alias(handle,bytes(key),std::size_t(key.size()),bytes(alias),std::size_t(alias.size()))){close();return false;}}
        if(d.initialKind){const auto choice=d.initialChoice.toUtf8();if(!om9_options_initial(handle,bytes(key),std::size_t(key.size()),d.initialKind,d.initialNumber,bytes(choice),std::size_t(choice.size()))){close();return false;}}
    }
    for(const auto& r:rules){const auto high=r.higher.toUtf8(),low=r.lower.toUtf8(),message=r.message.toUtf8();if(!om9_options_greater(handle,bytes(high),std::size_t(high.size()),bytes(low),std::size_t(low.size()),bytes(message),std::size_t(message.size()))){close();return false;}}
    if(!om9_options_seal(handle)){close();return false;}return true;
}
void CommandOptionsBridge::close(){if(handle)om9_options_close(handle);handle=0;token=0;}
unsigned CommandOptionsBridge::event(unsigned kind,const QString& text,double number){acceptedKeys.clear();const auto bytes=text.toUtf8();return om9_options_event(handle,kind,reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size()),number,&token);}
bool CommandOptionsBridge::accept(bool success){const auto proposed=changes();const auto current=token;token=0;const bool ok=om9_options_accept(handle,current,success);acceptedKeys.clear();if(ok&&success)for(const auto& assignment:proposed)acceptedKeys.push_back(assignment.section('=',0,0));return ok;}
QString CommandOptionsBridge::error()const{return readMessage(handle,0);}
QString CommandOptionsBridge::action()const{return readMessage(handle,1);}
QString CommandOptionsBridge::pending()const{return readMessage(handle,2);}
void CommandOptionsBridge::draft(const QString& value){std::uint64_t ignored=0;const auto bytes=value.toUtf8();om9_options_event(handle,8,reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size()),0,&ignored);}
bool CommandOptionsBridge::syncSolid(){return handle&&om9_options_sync_solid(handle);}
bool CommandOptionsBridge::question(const QString& value){const auto bytes=value.toUtf8();return om9_options_question(handle,reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size()));}
QString CommandOptionsBridge::invocationLine(const QString& name){const auto bytes=name.toUtf8();unsigned char line[512]={};if(!om9_options_invocation_line(reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size()),line,sizeof(line)))return {};return QString::fromUtf8(reinterpret_cast<char*>(line));}
CommandOptionSnapshot CommandOptionsBridge::snapshot(const QString& prefix)const{
    CommandOptionSnapshot result;if(!handle)return result;
    const auto needed=om9_options_snapshot(handle,nullptr,0);if(!needed||needed>65536)return result;
    std::vector<unsigned char> bytes(needed);om9_options_snapshot(handle,bytes.data(),bytes.size());
    result.text=prefix+QString::fromUtf8(reinterpret_cast<const char*>(bytes.data()));result.featureId=readMessage(handle,3);result.pending=!pending().isEmpty();result.revision=om9_options_revision(handle);
    result.questionLength=prefix.trimmed().size()+int(om9_options_question_length(handle));
    if(om9_options_question_length(handle))result.questionLength=prefix.size()+int(om9_options_question_length(handle));
    for(std::size_t i=0;i<om9_options_item_count(handle);++i){Om9OptionItem item{};if(!om9_options_item(handle,i,&item))continue;
        auto read=[&](unsigned field){unsigned char b[2048]={};om9_options_item_text(handle,i,field,b,sizeof(b));return QString::fromUtf8(reinterpret_cast<char*>(b));};
        result.spans.push_back({read(0),read(1),read(2),read(3),int(item.start)+int(prefix.size()),int(item.length),bool(item.enabled),bool(item.selected)});
    }return result;
}
QStringList CommandOptionsBridge::changes()const{
    QStringList result;for(std::size_t i=0;i<om9_options_change_count(handle,token);++i){unsigned char b[2048]={};om9_options_change_text(handle,token,i,b,sizeof(b));result.push_back(QString::fromUtf8(reinterpret_cast<char*>(b)));}return result;
}
Om9OptionValue CommandOptionsBridge::value(const char* key,bool candidate)const{Om9OptionValue v{};om9_options_value(handle,reinterpret_cast<const unsigned char*>(key),std::strlen(key),candidate?token:0,&v);return v;}
QString CommandOptionsBridge::choiceInput(const char* key,int index)const{unsigned char b[2048]={};if(index<0||!om9_options_choice_name(handle,reinterpret_cast<const unsigned char*>(key),std::strlen(key),std::size_t(index),b,sizeof(b)))return {};return QString::fromUtf8(key)+"="+QString::fromUtf8(reinterpret_cast<char*>(b));}
void CommandOptionsBridge::gate(const char* key,bool enabled,const QString& reason){const auto b=reason.toUtf8();om9_options_gate(handle,reinterpret_cast<const unsigned char*>(key),std::strlen(key),enabled,reinterpret_cast<const unsigned char*>(b.constData()),std::size_t(b.size()));}
void CommandOptionsBridge::seed(const char* key,unsigned kind,double number,const QString& text){const auto input=kind==4?QString::fromUtf8(key)+"="+text:QString::fromUtf8(key);if(event(kind,input,number)==2)accept();}
bool CommandOptionsBridge::visible(const char* key)const{return om9_options_visible(handle,reinterpret_cast<const unsigned char*>(key),std::strlen(key));}
bool CommandOptionsBridge::syncCurve(){return om9_options_sync_curve(handle);}
unsigned CommandOptionsBridge::acceptCurve(){const auto current=token;token=0;return om9_options_accept_curve(handle,current);}
namespace {
class WidgetBinding:public QObject {
public:
    WidgetBinding(QWidget* widget,const QString& key,CommandOptionsBridge& bridge,CommandOptionsBridge::Handler handler):QObject(widget),widget(widget),key(key),bridge(bridge),handler(std::move(handler)){
        editor=widget->findChild<QLineEdit*>();if(editor)qApp->installEventFilter(this);
    }
    ~WidgetBinding()override{if(qApp)qApp->removeEventFilter(this);}
protected:
    bool eventFilter(QObject* target,QEvent* event)override{
        if(!widget||!editor||(target!=editor&&target!=widget)||!bridge.active())return false;
        if(event->type()==QEvent::KeyPress){
            auto* keyEvent=static_cast<QKeyEvent*>(event);
            if(keyEvent->key()==Qt::Key_Escape){
                if(widget->property("om9OptionRawDraft").isValid()||editor->isModified()){
                    widget->setProperty("om9OptionRawDraft",QVariant());editor->setModified(false);handler(6,{},0.);bridge.render(widget,key.toUtf8().constData());return true;
                }
            }
            if(keyEvent->key()==Qt::Key_Return||keyEvent->key()==Qt::Key_Enter){submit();return true;}
        }
        if(event->type()==QEvent::FocusOut&&editor->isModified()){submit();return true;}
        return false;
    }
private:
    void submit(){const auto raw=editor->text();QString input=raw,prefix,suffix;
        if(auto* count=qobject_cast<QSpinBox*>(widget.data())){prefix=count->prefix();suffix=count->suffix();}
        else if(auto* number=qobject_cast<QDoubleSpinBox*>(widget.data())){prefix=number->prefix();suffix=number->suffix();}
        if(!prefix.isEmpty()&&input.startsWith(prefix))input.remove(0,prefix.size());
        if(!suffix.isEmpty()&&input.endsWith(suffix))input.chop(suffix.size());
        widget->setProperty("om9OptionRawDraft",raw);handler(9,key+"="+input,0.);bridge.render(widget,key.toUtf8().constData());}
    QPointer<QWidget> widget;QPointer<QLineEdit> editor;QString key;CommandOptionsBridge& bridge;CommandOptionsBridge::Handler handler;
};
}
void CommandOptionsBridge::bind(QWidget* widget,const char* key,QObject* oldReceiver,Handler handler){
    if(!widget)return;QObject::disconnect(widget,nullptr,oldReceiver,nullptr);const QString identity=QString::fromUtf8(key);widget->setProperty("om9OptionKey",identity);
    auto* binding=new WidgetBinding(widget,identity,*this,handler);
    if(auto* check=qobject_cast<QCheckBox*>(widget))QObject::connect(check,&QCheckBox::toggled,binding,[handler,identity](bool value){handler(3,identity,value?1.:0.);});
    else if(auto* combo=qobject_cast<QComboBox*>(widget))QObject::connect(combo,&QComboBox::currentIndexChanged,binding,[this,handler,identity](int index){handler(4,choiceInput(identity.toUtf8().constData(),index),0.);});
    else if(auto* count=qobject_cast<QSpinBox*>(widget))QObject::connect(count,&QSpinBox::valueChanged,binding,[handler,identity](int value){handler(5,identity,value);});
    else if(auto* number=qobject_cast<QDoubleSpinBox*>(widget))QObject::connect(number,&QDoubleSpinBox::valueChanged,binding,[handler,identity](double value){handler(5,identity,value);});
}
void CommandOptionsBridge::render(QWidget* widget,const char* key)const{
    if(!widget)return;const auto current=value(key);if(!current.kind)return;QSignalBlocker block(widget);
    if(acceptedKeys.contains(QString::fromUtf8(key)))widget->setProperty("om9OptionRawDraft",QVariant());
    if(auto* check=qobject_cast<QCheckBox*>(widget))check->setChecked(current.number!=0.);
    else if(auto* combo=qobject_cast<QComboBox*>(widget))combo->setCurrentIndex(int(current.choice));
    else if(auto* count=qobject_cast<QSpinBox*>(widget))count->setValue(int(current.number));
    else if(auto* number=qobject_cast<QDoubleSpinBox*>(widget))number->setValue(current.number);
    widget->setEnabled(visible(key));
    if(auto* editor=widget->findChild<QLineEdit*>()){
        const auto raw=widget->property("om9OptionRawDraft");if(raw.isValid())editor->setText(raw.toString());editor->setModified(raw.isValid());
    }
}
void CommandOptionsBridge::discardDrafts(QWidget* root){
    if(!root)return;for(auto* widget:root->findChildren<QWidget*>())widget->setProperty("om9OptionRawDraft",QVariant());
}
bool CommandOptionsBridge::hasDrafts(QWidget* root){
    if(!root)return false;for(auto* widget:root->findChildren<QWidget*>())if(widget->property("om9OptionRawDraft").isValid())return true;return false;
}
}
