#include "CommandConsole.h"
#include "CommandOptions.h"
#include <QApplication>
#include <QImage>
#include <iostream>
using namespace OpenMatrix9Gui;
int main(int argc,char** argv){
    QApplication app(argc,argv);
    CommandOptionsBridge bridge;
    auto option=CommandOptionDefinition::action(QString::fromUtf8("Lựa chọn 🔷"));
    option.aliases={"L"};
    if(!bridge.openFeature("UNICODE-PRESENTATION",{option},{},1.)||!bridge.question(QString::fromUtf8("Điểm đầu 🔷")))return 2;
    CommandConsole console(nullptr);console.setOptionSnapshot(bridge.snapshot());console.setInputText("draft");
    bool question=false,link=false;
    for(const auto& mark:console.extraSelections()){
        if(mark.format.fontWeight()==QFont::Bold){
            question=mark.cursor.selectedText()==QString::fromUtf8("Điểm đầu 🔷");
            std::cout<<"question="<<mark.cursor.selectedText().toStdString()<<'\n';
        }
        if(mark.format.fontUnderline()){
            link=mark.cursor.selectedText()=="L";
            std::cout<<"link="<<mark.cursor.selectedText().toStdString()<<'\n';
        }
    }
    QTextCursor styled(console.document());styled.setPosition(int(console.property("om9InputStart").toInt())-bridge.snapshot().text.size());styled.movePosition(QTextCursor::NextCharacter,QTextCursor::KeepAnchor);
    const bool boldDocument=styled.charFormat().fontWeight()==QFont::Bold;
    console.logInvocation("Command: _Box");console.resize(700,70);console.show();app.processEvents();
    auto* history=console.findChild<QPlainTextEdit*>("OM9CommandHistoryPane");const auto pixels=history->viewport()->grab().toImage();int dark=0;
    for(int y=0;y<pixels.height();++y)for(int x=0;x<std::min(300,pixels.width());++x){const auto c=pixels.pixelColor(x,y);if(c.red()<100&&c.green()<100&&c.blue()<100)++dark;}
    std::cout<<"question_exact="<<question<<" link_exact="<<link<<" draft="<<console.inputText().toStdString()<<" bold_document="<<boldDocument<<" history_text_pixels="<<dark<<'\n';
    const bool first=question&&link&&console.inputText()=="draft"&&boldDocument&&dark>8;
    auto choice=CommandOptionDefinition{};choice.key="Mode";choice.kind=2;choice.initialKind=2;choice.initialChoice="Reverse";
    choice.choices={{QString::fromUtf8("Lựa chọn 🔷"),{},{},true,{"L"}},{"Reverse",{},{},true,{"R"}}};
    if(!bridge.openFeature("UNICODE-CHOICE",{choice})||bridge.event(1,"Mode")!=1)return 3;
    console.setOptionSnapshot(bridge.snapshot());bool nestedL=false,nestedR=false;
    for(const auto& mark:console.extraSelections())if(mark.format.fontUnderline()){
        nestedL|=mark.cursor.selectedText()=="L";nestedR|=mark.cursor.selectedText()=="R";
        if(mark.cursor.selectedText().size()!=1)return 4;
    }
    std::cout<<"nested_unicode_mnemonic="<<nestedL<<" nested_ascii_mnemonic="<<nestedR<<'\n';
    return first&&nestedL&&nestedR&&console.inputText()=="draft"?0:1;
}
