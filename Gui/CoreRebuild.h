#pragma once
#include <QObject>
#include <memory>
#include <cstddef>
#include <QString>
#include <Python.h>
namespace OpenMatrix9Gui {
// OM9-CURVE-009: native selection/dialog/transaction host for the Rust fitter.
class CoreRebuild final:public QObject {
public:
    static CoreRebuild& instance();
    ~CoreRebuild() override;
    static bool handles(std::size_t);
    bool active()const;
    bool start(std::size_t);
    void cancel();
    void submit(const QString&);
    bool optionEvent(unsigned,const QString& text={},double number=0.);
    // Registered only for the native fixture environment, never a user command.
    static void addDeliveryTestMethods(PyObject*);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    CoreRebuild();
    struct State;
    struct Result;
    std::unique_ptr<State> state;
    void options();
    void bindOptions();void renderOptions();
    bool valid()const;
    void calculate(bool commit);
    void publish(const std::shared_ptr<Result>&);
    PyObject* deliveryTest(const char* action);
    void clearPreview();
    void shutdownDelivery();
};
}
