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
#include <Inventor/SbVec3f.h>
class SoSeparator;class SoNode;class SoTransform;class SoBaseColor;class SoRotation;class QLineEdit;
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
struct GumballMeshComponentPick;
struct HandlePick {unsigned handle;std::array<double,3> origin,direction;QPoint screen;};
class GumballOverlay {
public:
    ~GumballOverlay();void attach(Gui::View3DInventor*);void detach();
    void render(const Om9GumballSnapshot&,std::uint64_t,const Om9GumballSettings&,const std::vector<std::array<double,3>>&);std::optional<HandlePick> hit(const QPoint&)const;
    HandlePick ray(const QPoint&)const;QJsonArray handlePositions()const;
    QJsonObject diagnostics()const;
    QJsonArray cameraState()const;
    QLineEdit* numericEditor(const HandlePick&,bool);void closeEditor();
    QLineEdit* editor()const;void clearPreview();
    void preview(const std::vector<std::vector<Base::Vector3d>>&);
    void componentHighlight(const GumballMeshComponentPick*);
private:
    QPointer<Gui::View3DInventor> view;QPointer<QLineEdit> field;
    SoSeparator *owner=nullptr,*root=nullptr,*previewNode=nullptr,*componentNode=nullptr;
    SoSeparator* widgetNode=nullptr;
    SoTransform *pose=nullptr,*menuPose=nullptr;
    // Thirteen Rust operation handles, plus the native view-menu target.
    std::array<SoNode*,14> handles{};
    std::array<SoBaseColor*,13> colors{};
    std::array<SoRotation*,4> facing{};
    std::array<SbVec3f,14> centers{};
    Om9GumballLayout layout{};Om9GumballSettings settings{};
};
}
