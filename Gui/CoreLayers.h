// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QString>
#include <array>
#include <string>
namespace App { class Document; class DocumentObject; }
namespace OpenMatrix9Gui {
// GUI-thread snapshots; only owned values, never retained document pointers.
struct CircleLayerSnapshot {
    std::string documentName;
    std::string documentUid;
    std::string groupName;
    int index=0;
    bool locked=false;
    bool visible=true;
    std::array<double,3> color={};
};
struct SidebarLayerState {
    QString name;
    int index=0;
    bool active=false;
    bool locked=false;
    bool visible=true;
    std::array<double,3> color;
};
class CoreLayers {
public:
    static bool available();
    // 1..32 stable slots; 0 clears selection to legacy root. Opens own transaction.
    static bool select(int index);
    // Returns true for a handled Layer command, including invalid values; reports
    // rejection via error (no transaction). False means another command owns text.
    static bool submit(const QString& text, QString& error);
    static SidebarLayerState state(int index);
    static bool toggleLocked(int index);
    static bool toggleVisible(int index);
    static bool setColor(int index,const std::array<double,3>& color);
    // Must be called BEFORE Circle's openTransaction. Throws on lock/corrupt state.
    static CircleLayerSnapshot captureCircle(App::Document& document);
    // Must be inside caller's existing Circle transaction, after native output
    // creation and before recompute/commit; caller aborts on any thrown error.
    // Accepts analytic Part::Feature, persistent History and deformable outputs.
    static void applyCircle(App::Document& document,App::DocumentObject& output,
                            const CircleLayerSnapshot& captured);
};
}
