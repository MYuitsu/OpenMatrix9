#pragma once
#include <QObject>
#include <QPointer>
#include <cstddef>
#include <string>
#include <QStringList>
#include <vector>
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
    void acceptInput();
    bool pendingInput()const;
    void logMessage(const QString&);
    void setPrompt(const QString&);
protected:
    bool eventFilter(QObject*,QEvent*) override;
private:
    CurveController();
    void submit(const QString&);void result(unsigned int);void refresh();bool validDocument() const;
    QPointer<QDockWidget> dock;QPointer<CommandConsole> console;
    void clearPreview();void updatePreview(const double* hover=nullptr);
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previewOwners;
    QStringList inputHistory;QString historyDraft;int historyPosition=-1;
    QPointer<QObject> releaseTarget;
    std::string document;std::size_t command=0;bool enabled=false;
    const void* documentIdentity=nullptr;
    fastsignals::scoped_connection deleteConnection,activeConnection;
};
bool isCurveCommand(std::size_t command);
}
