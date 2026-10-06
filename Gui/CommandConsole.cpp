#include "CommandConsole.h"
#include "RustBridge.h"
#include <QKeyEvent>
#include <QInputMethodEvent>
#include <QMimeData>
#include <QMenu>
#include <QTextBlock>
#include <QApplication>
#include <QClipboard>
#include <QMouseEvent>
#include <QRegularExpression>
#include <QTextDocument>
#include <QScrollBar>
#include <QPainter>
#include <QPaintEvent>
#include <QResizeEvent>
#include <QFocusEvent>
#include <QScopedValueRollback>
#include <QListWidget>
#include <QTimer>
#include <QScreen>
#include <algorithm>
#include <vector>
namespace {
// Read-only history viewport over the same document; its final block is live input.
class HistoryPane final:public QPlainTextEdit {
public:
    explicit HistoryPane(QWidget* parent):QPlainTextEdit(parent){
        setObjectName("OM9CommandHistoryPane");setReadOnly(true);setFrameShape(QFrame::NoFrame);
        setLineWrapMode(QPlainTextEdit::NoWrap);setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
        connect(verticalScrollBar(),&QScrollBar::rangeChanged,this,[this]{limitScroll();});
        connect(this,&QPlainTextEdit::textChanged,this,[this]{limitScroll();});
    }
protected:
    void resizeEvent(QResizeEvent* event)override{QPlainTextEdit::resizeEvent(event);limitScroll();}
    void paintEvent(QPaintEvent* event)override{
        QPlainTextEdit::paintEvent(event);
        QTextCursor last(document()->lastBlock());const int top=cursorRect(last).top();
        if(top<viewport()->height()){QPainter painter(viewport());painter.fillRect(QRect(0,std::max(0,top),viewport()->width(),viewport()->height()),palette().brush(QPalette::Base));}
    }
private:
    void limitScroll(){
        if(limiting)return;QScopedValueRollback<bool> guard(limiting,true);
        const int rows=std::max(1,(viewport()->height()-2*int(document()->documentMargin()))/std::max(1,fontMetrics().lineSpacing()));
        verticalScrollBar()->setMaximum(std::max(0,document()->blockCount()-1-rows));
    }
    bool limiting=false;
};
QString singleLine(QString value){
    value.replace(QLatin1Char('\r'),QLatin1Char(' '));value.replace(QLatin1Char('\n'),QLatin1Char(' '));
    value.replace(QChar(0x2028),QLatin1Char(' '));value.replace(QChar(0x2029),QLatin1Char(' '));return value;
}
QString inputPrefix(const QString& value,int room){
    auto prefix=value.left(std::max(0,room));
    if(prefix.size()<value.size()&&!prefix.isEmpty()&&prefix.back().isHighSurrogate()&&value[prefix.size()].isLowSurrogate())prefix.chop(1);
    return prefix;
}
}
namespace OpenMatrix9Gui {
CommandConsole::CommandConsole(QWidget* parent):QPlainTextEdit(parent){
    setObjectName("OM9CommandTranscript");setMaximumBlockCount(10000);setUndoRedoEnabled(false);
    setFrameShape(QFrame::NoFrame);setAcceptDrops(false);
    setLineWrapMode(QPlainTextEdit::NoWrap);setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);setCenterOnScroll(true);
    history=new HistoryPane(this);history->setDocument(document());history->installEventFilter(this);
    separator=new QWidget(this);separator->setObjectName("OM9CommandSeparator");separator->setStyleSheet("background:#526e59;");
    suggestions=new QListWidget(this);suggestions->setObjectName("OM9CommandSuggestions");
    suggestions->setWindowFlags(Qt::ToolTip|Qt::FramelessWindowHint|Qt::WindowDoesNotAcceptFocus);suggestions->setAttribute(Qt::WA_ShowWithoutActivating);suggestions->setFocusPolicy(Qt::NoFocus);
    suggestions->setStyleSheet("QListWidget { background:white; color:black; border:1px solid #777; } QListWidget::item:selected { background:#0078d7; color:white; } QListWidget::item:disabled { color:#777; }");
    connect(suggestions,&QListWidget::itemClicked,this,[this](QListWidgetItem*){chooseCompletion(true);});
    connect(this,&QPlainTextEdit::textChanged,this,[this]{queueCompletions();});
    connect(verticalScrollBar(),&QScrollBar::valueChanged,this,[this]{keepInputVisible();});
    connect(verticalScrollBar(),&QScrollBar::rangeChanged,this,[this]{keepInputVisible();});
    connect(history,&QPlainTextEdit::selectionChanged,this,[this]{if(syncingSelection)return;QScopedValueRollback<bool> guard(syncingSelection,true);setTextCursor(history->textCursor());keepInputVisible();});
    connect(this,&QPlainTextEdit::selectionChanged,this,[this]{if(syncingSelection)return;QScopedValueRollback<bool> guard(syncingSelection,true);const int scroll=history->verticalScrollBar()->value();history->setTextCursor(textCursor());history->verticalScrollBar()->setValue(scroll);keepInputVisible();});
    setPlainText("\n\n"+currentPrompt);promptStart=2;inputStart=promptStart+currentPrompt.size();updateBoundary();
    updatePanes();
}
QSize CommandConsole::sizeHint()const{return QSize(600,3*fontMetrics().lineSpacing()+4*int(document()->documentMargin())+1);}
void CommandConsole::keepInputVisible(){const int last=std::min(verticalScrollBar()->maximum(),document()->lastBlock().firstLineNumber());if(verticalScrollBar()->value()!=last)verticalScrollBar()->setValue(last);}
void CommandConsole::updatePanes(){
    if(!history||layouting)return;QScopedValueRollback<bool> guard(layouting,true);
    const int line=fontMetrics().lineSpacing(),margin=int(document()->documentMargin()),liveHeight=line+2*margin;
    setMinimumHeight(liveHeight+1);const int historyHeight=std::max(0,height()-liveHeight-1);
    setViewportMargins(0,historyHeight+1,0,0);history->setGeometry(0,0,width(),historyHeight);history->setVisible(historyHeight>0);
    separator->setGeometry(0,historyHeight,width(),1);separator->show();
    setProperty("om9HistoryRows",std::max(0,(historyHeight-2*margin)/std::max(1,line)));setProperty("om9LiveRowHeight",liveHeight);
    keepInputVisible();if(suggestions&&suggestions->isVisible())queueCompletions();
}
void CommandConsole::resizeEvent(QResizeEvent* event){QPlainTextEdit::resizeEvent(event);updatePanes();}
void CommandConsole::changeEvent(QEvent* event){QPlainTextEdit::changeEvent(event);if(event->type()==QEvent::FontChange)updatePanes();}
void CommandConsole::hideEvent(QHideEvent* event){if(suggestions)suggestions->hide();QPlainTextEdit::hideEvent(event);}
void CommandConsole::focusOutEvent(QFocusEvent* event){if(suggestions)suggestions->hide();QPlainTextEdit::focusOutEvent(event);}
bool CommandConsole::focusNextPrevChild(bool next){
    if(suggestions&&suggestions->isVisible())return false;
    return QPlainTextEdit::focusNextPrevChild(next);
}
bool CommandConsole::eventFilter(QObject* watched,QEvent* event){
    if(watched==history&&event->type()==QEvent::KeyPress){
        auto* key=static_cast<QKeyEvent*>(event);
        const bool navigation=key->key()==Qt::Key_Left||key->key()==Qt::Key_Right||key->key()==Qt::Key_Home||key->key()==Qt::Key_End||((key->key()==Qt::Key_Up||key->key()==Qt::Key_Down)&&key->modifiers().testFlag(Qt::ShiftModifier));
        if(!navigation&&!key->matches(QKeySequence::Copy)&&!key->matches(QKeySequence::SelectAll)){setFocus();keyPressEvent(key);return true;}
    }
    return QPlainTextEdit::eventFilter(watched,event);
}
void CommandConsole::queueCompletions(){
    if(completionQueued)return;completionQueued=true;QTimer::singleShot(0,this,[this]{completionQueued=false;updateCompletions();});
}
void CommandConsole::updateCompletions(){
    if(!suggestions)return;
    if(!isVisible()||!hasFocus()||!completionEnabled||!completionEnabled()){suggestions->hide();return;}
    const auto input=inputText();if(!input.isEmpty()&&input==dismissedCompletion){suggestions->hide();return;}dismissedCompletion.clear();const auto bytes=input.toUtf8();
    const auto selected=suggestions->currentItem()?suggestions->currentItem()->data(Qt::UserRole).toString():QString();
    struct Match{QString name;unsigned int rank;bool available;};std::vector<Match> matches;
    for(std::size_t i=0;i<om9_console_completion_count();++i){
        const auto rank=om9_console_completion_rank(i,bytes.constData());if(!rank)continue;
        const auto name=QString::fromUtf8(om9_console_completion_name(i));matches.push_back({name,rank,completionAvailable&&completionAvailable(name)});
    }
    std::sort(matches.begin(),matches.end(),[](const Match& a,const Match& b){return a.rank!=b.rank?a.rank<b.rank:a.name.compare(b.name,Qt::CaseInsensitive)<0;});
    suggestions->clear();int row=-1,exactRow=-1,width=240;
    for(const auto& match:matches){
        auto* item=new QListWidgetItem(match.available?match.name:match.name+tr(" (chưa khả dụng)"),suggestions);item->setData(Qt::UserRole,match.name);
        if(!match.available)item->setFlags(item->flags()&~(Qt::ItemIsEnabled|Qt::ItemIsSelectable));
        else if(row<0||match.name==selected)row=suggestions->count()-1;
        if(match.available&&match.rank==1)exactRow=suggestions->count()-1;
        width=std::max(width,suggestions->fontMetrics().horizontalAdvance(item->text())+32);
    }
    if(matches.empty()){suggestions->hide();return;}
    suggestions->setCurrentRow(exactRow>=0?exactRow:row);
    QTextCursor start(document());start.setPosition(inputStart);auto position=viewport()->mapToGlobal(cursorRect(start).bottomLeft());
    const auto bounds=screen()->availableGeometry();const int popupHeight=std::min(10,suggestions->count())*(suggestions->fontMetrics().height()+4)+4;
    width=std::min(width,bounds.width());position.setX(std::clamp(position.x(),bounds.left(),bounds.right()-width+1));
    if(position.y()+popupHeight>bounds.bottom())position.setY(viewport()->mapToGlobal(cursorRect(start).topLeft()).y()-popupHeight);
    suggestions->setGeometry(position.x(),std::max(bounds.top(),position.y()),width,popupHeight);suggestions->show();
}
void CommandConsole::moveCompletion(int direction){
    int row=suggestions->currentRow();
    for(int i=0;i<suggestions->count();++i){row=(row+direction+suggestions->count())%suggestions->count();if(suggestions->item(row)->flags().testFlag(Qt::ItemIsEnabled)){suggestions->setCurrentRow(row);return;}}
}
void CommandConsole::chooseCompletion(bool submit){
    auto* item=suggestions->currentItem();if(!item||!item->flags().testFlag(Qt::ItemIsEnabled))return;
    const auto value=item->data(Qt::UserRole).toString();dismissedCompletion=value;suggestions->hide();setInputText(value);if(submit&&accepted)accepted();
}
void CommandConsole::updateBoundary(){
    setProperty("om9InputStart",inputStart);setProperty("om9Prompt",currentPrompt);
    stylePrompt();
}
void CommandConsole::stylePrompt(){
    QList<QTextEdit::ExtraSelection> marks;
    const int open=currentPrompt.indexOf('('),close=currentPrompt.lastIndexOf(')');
    if(open>=0&&close>open){
        static const QRegularExpression options("\\b(BothSides|PersistentClose|Close|Mode|Length|Undo)\\b");
        auto matches=options.globalMatch(currentPrompt,open+1);
        while(matches.hasNext()){
            const auto match=matches.next();if(match.capturedStart()>=close)break;
            QTextEdit::ExtraSelection mark;mark.cursor=QTextCursor(document());mark.cursor.setPosition(promptStart+int(match.capturedStart()));mark.cursor.movePosition(QTextCursor::NextCharacter,QTextCursor::KeepAnchor);mark.format.setFontUnderline(true);marks.append(mark);
        }
    }
    setExtraSelections(marks);
}
QString CommandConsole::optionAt(const QPoint& point)const{
    const auto cursor=cursorForPosition(point);
    if(!cursorRect(cursor).adjusted(-4,-2,4,2).contains(point))return {};
    const int offset=cursor.position()-promptStart;
    const int open=currentPrompt.indexOf('('),close=currentPrompt.lastIndexOf(')');
    if(open<0||offset<=open||offset>=close)return {};
    static const QRegularExpression options("\\b(BothSides|PersistentClose|Close|Mode|Length|Undo)\\b");
    auto matches=options.globalMatch(currentPrompt,open+1);
    while(matches.hasNext()){
        const auto match=matches.next();
        if(offset>=match.capturedStart()&&offset<match.capturedEnd())return match.captured()=="Mode"?"Mode=Line":match.captured();
    }
    return {};
}
void CommandConsole::mousePressEvent(QMouseEvent* event){
    pressedOption=event->button()==Qt::LeftButton&&event->modifiers()==Qt::NoModifier?optionAt(event->position().toPoint()):QString();
    QPlainTextEdit::mousePressEvent(event);
}
void CommandConsole::mouseReleaseEvent(QMouseEvent* event){
    QPlainTextEdit::mouseReleaseEvent(event);
    const auto option=pressedOption;pressedOption.clear();
    if(!option.isEmpty()&&event->button()==Qt::LeftButton&&event->modifiers()==Qt::NoModifier&&!textCursor().hasSelection()&&inputText().isEmpty()&&optionAt(event->position().toPoint())==option){setInputText(option);if(accepted)accepted();}
}
QString CommandConsole::inputText()const{
    QTextCursor c(document());c.setPosition(inputStart);c.movePosition(QTextCursor::End,QTextCursor::KeepAnchor);return c.selectedText();
}
bool CommandConsole::editableSelection()const{return textCursor().selectionStart()>=inputStart;}
void CommandConsole::editPosition(){if(!editableSelection()){auto c=textCursor();c.movePosition(QTextCursor::End);setTextCursor(c);}}
void CommandConsole::setInputText(const QString& value){
    QTextCursor c(document());c.setPosition(inputStart);c.movePosition(QTextCursor::End,QTextCursor::KeepAnchor);c.insertText(inputPrefix(singleLine(value),1024));setTextCursor(c);ensureCursorVisible();
}
void CommandConsole::insertInput(const QString& value){
    editPosition();const auto room=1024-inputText().size()+textCursor().selectedText().size();if(room>0)insertPlainText(inputPrefix(singleLine(value),room));ensureCursorVisible();
}
void CommandConsole::setPrompt(const QString& value){
    const bool follow=history->verticalScrollBar()->value()==history->verticalScrollBar()->maximum();
    const auto draft=inputText();const auto previous=textCursor();const bool atInput=previous.position()>=inputStart;
    QTextCursor c(document());c.setPosition(promptStart);c.setPosition(inputStart,QTextCursor::KeepAnchor);c.insertText(value);
    currentPrompt=value;promptStart=document()->characterCount()-1-value.size()-draft.size();inputStart=promptStart+value.size();updateBoundary();
    if(follow)history->verticalScrollBar()->setValue(history->verticalScrollBar()->maximum());
    if(atInput){c.movePosition(QTextCursor::End);setTextCursor(c);ensureCursorVisible();}
}
void CommandConsole::logMessage(const QString& message){
    if(message.isEmpty()||message==lastMessage)return;lastMessage=message;
    const bool follow=history->verticalScrollBar()->value()==history->verticalScrollBar()->maximum();
    const auto draft=inputText();const bool atInput=textCursor().position()>=inputStart;
    QTextCursor c(document());c.setPosition(promptStart);c.insertText(message+"\n");
    promptStart=document()->characterCount()-1-currentPrompt.size()-draft.size();inputStart=promptStart+currentPrompt.size();updateBoundary();
    if(follow)history->verticalScrollBar()->setValue(history->verticalScrollBar()->maximum());
    if(atInput){c.movePosition(QTextCursor::End);setTextCursor(c);ensureCursorVisible();}
}
QString CommandConsole::takeInput(){
    const bool follow=history->verticalScrollBar()->value()==history->verticalScrollBar()->maximum();
    const auto value=inputText();QTextCursor c(document());c.movePosition(QTextCursor::End);c.insertText("\n"+currentPrompt);
    promptStart=document()->characterCount()-1-currentPrompt.size();inputStart=promptStart+currentPrompt.size();lastMessage.clear();updateBoundary();if(follow)history->verticalScrollBar()->setValue(history->verticalScrollBar()->maximum());setTextCursor(c);ensureCursorVisible();return value;
}
void CommandConsole::keyPressEvent(QKeyEvent* event){
    if(event->matches(QKeySequence::Copy)||event->matches(QKeySequence::SelectAll)){QPlainTextEdit::keyPressEvent(event);return;}
    if(event->matches(QKeySequence::Undo)||event->matches(QKeySequence::Redo))return;
    if(event->matches(QKeySequence::Paste)){insertFromMimeData(QApplication::clipboard()->mimeData());return;}
    if(event->matches(QKeySequence::Cut)){if(editableSelection())QPlainTextEdit::keyPressEvent(event);return;}
    const auto plain=(event->modifiers()&~Qt::KeypadModifier)==Qt::NoModifier;
    if(plain&&suggestions->isVisible()){
        if(event->key()==Qt::Key_Up||event->key()==Qt::Key_Down){moveCompletion(event->key()==Qt::Key_Up?-1:1);return;}
        if(event->key()==Qt::Key_Escape){dismissedCompletion=inputText();suggestions->hide();return;}
        if(event->key()==Qt::Key_Tab){chooseCompletion(false);return;}
        if((event->key()==Qt::Key_Return||event->key()==Qt::Key_Enter)&&suggestions->currentItem()){chooseCompletion(true);return;}
    }
    if(plain&&(event->key()==Qt::Key_Return||event->key()==Qt::Key_Enter)){if(accepted)accepted();return;}
    if(plain&&event->key()==Qt::Key_Escape){if(cancelled)cancelled();return;}
    if(plain&&(event->key()==Qt::Key_Up||event->key()==Qt::Key_Down)){if(recalled)recalled(event->key()==Qt::Key_Up?-1:1);return;}
    if(plain&&event->key()==Qt::Key_Home){auto c=textCursor();c.setPosition(inputStart);setTextCursor(c);return;}
    if(event->key()==Qt::Key_Backspace||event->key()==Qt::Key_Delete){
        if(!editableSelection()||(!textCursor().hasSelection()&&textCursor().position()<=inputStart&&event->key()==Qt::Key_Backspace))return;
        if(event->key()==Qt::Key_Backspace&&event->modifiers().testFlag(Qt::ControlModifier)&&!textCursor().hasSelection()){
            auto c=textCursor();const int end=c.position();c.movePosition(QTextCursor::PreviousWord);c.setPosition(std::max(inputStart,c.position()));c.setPosition(end,QTextCursor::KeepAnchor);c.removeSelectedText();setTextCursor(c);return;
        }
    }
    if(!event->text().isEmpty()){
        const auto text=event->text();const auto first=text.front();
        const bool format=first.category()==QChar::Other_Format;
        const bool control=event->modifiers()==Qt::ControlModifier||event->modifiers()==(Qt::ControlModifier|Qt::ShiftModifier);
        const bool pair=first.isHighSurrogate()&&text.size()>1&&text[1].isLowSurrogate();
        if(format||(!control&&(first.isPrint()||first.category()==QChar::Other_PrivateUse||pair))){insertInput(text);return;}
    }
    if(event->key()==Qt::Key_Return||event->key()==Qt::Key_Enter||event->key()==Qt::Key_Tab)return;
    QPlainTextEdit::keyPressEvent(event);
}
void CommandConsole::insertFromMimeData(const QMimeData* data){if(data&&data->hasText()){auto value=data->text();value.replace('\r',' ');value.replace('\n',' ');insertInput(value);}}
void CommandConsole::inputMethodEvent(QInputMethodEvent* event){
    editPosition();auto committed=singleLine(event->commitString());
    // Qt removes the selection before applying replacementStart/Length.
    const int selected=textCursor().selectionEnd()-textCursor().selectionStart();
    const int replacement=textCursor().selectionStart()+event->replacementStart();
    if(event->replacementLength()<0||replacement<inputStart||replacement+event->replacementLength()>document()->characterCount()-1-selected||inputText().size()-selected+committed.size()-event->replacementLength()>1024){event->ignore();return;}
    QInputMethodEvent normalized(event->preeditString(),event->attributes());normalized.setCommitString(committed,event->replacementStart(),event->replacementLength());QPlainTextEdit::inputMethodEvent(&normalized);event->setAccepted(normalized.isAccepted());
}
void CommandConsole::contextMenuEvent(QContextMenuEvent* event){
    QMenu menu(this);auto* copyAction=menu.addAction(tr("Copy"),this,&QPlainTextEdit::copy);copyAction->setEnabled(textCursor().hasSelection());
    menu.addAction(tr("Select All"),this,&QPlainTextEdit::selectAll);
    auto* cutAction=menu.addAction(tr("Cut"),this,&QPlainTextEdit::cut);cutAction->setEnabled(editableSelection()&&textCursor().hasSelection());
    menu.addAction(tr("Paste"),this,[this]{insertFromMimeData(QApplication::clipboard()->mimeData());});menu.exec(event->globalPos());
}
}
