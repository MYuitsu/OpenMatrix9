#pragma once
#include <QPlainTextEdit>
#include <functional>
#include "CommandOptions.h"
class QListWidget;
namespace OpenMatrix9Gui {
// One selectable document, with only its current input suffix editable.
class CommandConsole final:public QPlainTextEdit {
public:
    explicit CommandConsole(QWidget* parent);
    QSize sizeHint()const override;
    QString inputText()const;
    void setInputText(const QString&);
    void insertInput(const QString&);
    void setPrompt(const QString&);
    void setOptionSnapshot(const CommandOptionSnapshot&);
    void logMessage(const QString&);
    void logInvocation(const QString&);
    QString takeInput();
    std::function<void()> accepted,cancelled;
    std::function<void(int)> recalled;
    std::function<bool()> completionEnabled;
    std::function<bool(const QString&)> completionAvailable;
    std::function<bool(const QString&)> optionActivated;
    std::function<bool(int)> optionNavigate;
protected:
    void keyPressEvent(QKeyEvent*)override;
    void insertFromMimeData(const QMimeData*)override;
    void inputMethodEvent(QInputMethodEvent*)override;
    void contextMenuEvent(QContextMenuEvent*)override;
    void mousePressEvent(QMouseEvent*)override;
    void mouseReleaseEvent(QMouseEvent*)override;
    void resizeEvent(QResizeEvent*)override;
    void changeEvent(QEvent*)override;
    void hideEvent(QHideEvent*)override;
    void focusOutEvent(QFocusEvent*)override;
    bool eventFilter(QObject*,QEvent*)override;
    bool focusNextPrevChild(bool)override;
private:
    bool editableSelection()const;
    void editPosition();
    void updateBoundary();
    void stylePrompt();
    bool followHistory()const;
    QString optionAt(const QPoint&)const;
    void updatePanes();
    void keepInputVisible();
    void queueCompletions();
    void updateCompletions();
    void chooseCompletion(bool submit);
    void moveCompletion(int direction);
    QPlainTextEdit* history=nullptr;
    QWidget* separator=nullptr;
    QListWidget* suggestions=nullptr;
    bool syncingSelection=false,layouting=false,completionQueued=false;
    QString pressedOption;
    QString dismissedCompletion;
    QString currentPrompt="Command: ",lastMessage;
    CommandOptionSnapshot currentOptions;
    int promptStart=0,inputStart=0;
};
}
