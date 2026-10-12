// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "RustBridge.h"
#include <QString>
#include <QStringList>
#include <QVector>
#include <QSpinBox>
#include <QDoubleSpinBox>
#include <functional>
#include <cstdint>
namespace OpenMatrix9Gui {
struct CommandOptionSpan {QString key,input,reason,mnemonic;int start=0,length=0;bool enabled=false,selected=false;};
struct CommandOptionSnapshot {QString text,featureId;QVector<CommandOptionSpan> spans;int questionLength=0;bool pending=false;std::uint64_t revision=0;};
struct CommandOptionChoice {QString value,child,reason;bool enabled=true;QStringList aliases;};
struct CommandOptionDefinition {
    QString key,parent,unit;QStringList aliases;QVector<CommandOptionChoice> choices;
    unsigned kind=4,scope=3,initialKind=0;double minimum=0.,maximum=0.,initialNumber=0.;bool integer=false,shown=true;QString initialChoice;
    static CommandOptionDefinition number(const QString& key,double minimum,double maximum,bool integer,double current,const QString& unit={});
    static CommandOptionDefinition toggle(const QString& key,bool current);
    static CommandOptionDefinition action(const QString& key);
    CommandOptionDefinition alias(const QString& shortcut)const{auto definition=*this;definition.aliases.push_back(shortcut);return definition;}
};
struct CommandOptionGreater {QString higher,lower,message;};
// Owns only an opaque Rust identity. Values/validation/cursor remain in Rust.
class CommandOptionsBridge {
public:
    ~CommandOptionsBridge(){close();}
    CommandOptionsBridge()=default;
    CommandOptionsBridge(const CommandOptionsBridge&)=delete;
    CommandOptionsBridge& operator=(const CommandOptionsBridge&)=delete;
    bool open(unsigned family,double mmPerUnit=1.,const QString& featureId={});
    bool openFeature(const QString& featureId,const QVector<CommandOptionDefinition>&,const QVector<CommandOptionGreater>& rules={},double mmPerUnit=1.);
    void close();bool active()const{return handle!=0;}
    unsigned event(unsigned kind,const QString& text={},double number=0.);
    bool accept(bool success=true);
    bool syncCurve();unsigned acceptCurve();
    bool syncSolid();
    bool question(const QString&);
    static QString invocationLine(const QString&);
    CommandOptionSnapshot snapshot(const QString& prefix={})const;
    QString error()const;QString action()const;QString pending()const;
    QStringList changes()const;
    QString choiceInput(const char* key,int index)const;
    Om9OptionValue value(const char* key,bool candidate=false)const;
    void gate(const char* key,bool enabled,const QString& reason={});
    void seed(const char* key,unsigned kind,double number=0.,const QString& text={});
    void draft(const QString& value);
    bool visible(const char* key)const;
    using Handler=std::function<void(unsigned,const QString&,double)>;
    void bind(QWidget* widget,const char* key,QObject* oldReceiver,Handler handler);
    void render(QWidget* widget,const char* key)const;
    static void discardDrafts(QWidget* root);
    static bool hasDrafts(QWidget* root);
private:
    std::uint64_t handle=0,token=0;
    QStringList acceptedKeys;
};
// Preserve raw text until the shared Rust parser accepts it. Qt must not clamp
// or silently restore invalid edits before the command sees them.
class CommandOptionSpinBox:public QSpinBox {
public: explicit CommandOptionSpinBox(QWidget* parent):QSpinBox(parent){setKeyboardTracking(false);}
protected:
    QValidator::State validate(QString&,int&)const override{return QValidator::Acceptable;}
    int valueFromText(const QString&)const override{return value();}
    void fixup(QString&)const override{}
};
class CommandOptionDoubleSpinBox:public QDoubleSpinBox {
public: explicit CommandOptionDoubleSpinBox(QWidget* parent):QDoubleSpinBox(parent){setKeyboardTracking(false);}
protected:
    QValidator::State validate(QString&,int&)const override{return QValidator::Acceptable;}
    double valueFromText(const QString&)const override{return value();}
    void fixup(QString&)const override{}
};
}
