// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "GumballRust.h"
#include <QPointer>
#include <QPoint>
#include <QJsonArray>
#include <QJsonObject>
#include <optional>
#include <array>
#include <vector>
#include <Base/Vector3D.h>
class SoSeparator;class SoNode;class SoTransform;class SoScale;class QLineEdit;
namespace Gui {class View3DInventor;class SoTransformDragger;}
namespace OpenMatrix9Gui {
struct HandlePick {unsigned handle;std::array<double,3> origin,direction;QPoint screen;};
class GumballOverlay {
public:
    ~GumballOverlay();void attach(Gui::View3DInventor*);void detach();
    void render(const Om9GumballSnapshot&);std::optional<HandlePick> hit(const QPoint&)const;
    HandlePick ray(const QPoint&)const;QJsonArray handlePositions()const;
    QJsonObject diagnostics()const;
    QLineEdit* numericEditor(const HandlePick&,bool);void closeEditor();
    QLineEdit* editor()const;void clearPreview();
    void preview(const std::vector<std::vector<Base::Vector3d>>&);
private:
    QPointer<Gui::View3DInventor> view;QPointer<QLineEdit> field;
    SoSeparator *owner=nullptr,*root=nullptr,*previewNode=nullptr;
    SoTransform* pose=nullptr;SoScale* extraScale=nullptr;
    Gui::SoTransformDragger* dragger=nullptr;
    std::array<SoNode*,13> handles{};
};
}
