#pragma once
#include <QObject>
#include <QPointer>
#include <cstddef>
#include <functional>
#include <string>
#include <QStringList>
#include <vector>
#include "CommandOptions.h"
class SoSeparator;
#include <fastsignals/signal.h>
class QDockWidget;class QLineEdit;class QLabel;class QEvent;class QPlainTextEdit;
namespace OpenMatrix9Gui {
class CommandConsole;
class CurveController final:public QObject {
public:
    static CurveController& instance();
    void activate();void deactivate();
    bool available(std::size_t command) const;
    bool start(std::size_t command);
    void cancel();
    void cancelInput();
    void acceptInput();
    bool pendingInput()const;
    void logMessage(const QString&);
    void logInvocation(const QString& commandName);
    void setPrompt(const QString&);
    bool optionEvent(unsigned,const QString& text={},double number=0.);
    void renderOptions(const CommandOptionsBridge&,const QString& prefix);
    QString draftInput()const;
    QString promptInput()const;
    void setDraftInput(const QString&);
    void focusInput(std::function<bool()> current={});
    void consumeOptionInput(unsigned);
protected:
    bool eventFilter(QObject*,QEvent*) override;
private:
    void rememberInput(const QString&);
    CurveController();
    void submit(const QString&);void result(unsigned int);void refresh();bool validDocument() const;
    QPointer<QDockWidget> dock;QPointer<CommandConsole> console;
    void clearPreview();void updatePreview(const double* hover=nullptr,bool close=false,bool square=false);
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previewOwners;
    QStringList inputHistory;QString historyDraft;int historyPosition=-1;
    QPointer<QObject> releaseTarget;
    std::string document;std::size_t command=0;bool enabled=false;
    std::string unitsSnapshot;
    CommandOptionsBridge commandOptions;
    const void* documentIdentity=nullptr;
    fastsignals::scoped_connection deleteConnection,activeConnection;
};
bool isCurveCommand(std::size_t command);
}
