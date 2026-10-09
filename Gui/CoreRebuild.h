#pragma once
#include <QObject>
#include <memory>
#include <cstddef>
#include <QString>
namespace OpenMatrix9Gui {
// OM9-CURVE-009: native selection/dialog/transaction host for the Rust fitter.
class CoreRebuild final:public QObject {
public:
    static CoreRebuild& instance();
    static bool handles(std::size_t);
    bool active()const;
    bool start(std::size_t);
    void cancel();
    void submit(const QString&);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    CoreRebuild();
    struct State;
    std::unique_ptr<State> state;
    void options();
    bool valid()const;
    void calculate(bool commit);
    void clearPreview();
};
}
