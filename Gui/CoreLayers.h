// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QString>
#include <array>
#include <string>
#include <vector>
#include <utility>
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
    std::string groupName;
    long objectId=0;
    int index=0;
    bool active=false;
    bool locked=false;
    bool visible=true;
    std::array<double,3> color;
};
class CoreLayers {
public:
    static bool available();
    // 1..32 model slots, 33 Lights; 0 clears current to root. Own transaction.
    static bool select(int index);
    // Returns true for a handled Layer command, including invalid values; reports
    // rejection via error (no transaction). False means another command owns text.
    static bool submit(const QString& text, QString& error);
    static SidebarLayerState state(int index);
    static bool toggleLocked(int index);
    static bool toggleVisible(int index);
    static bool setColor(int index,const std::array<double,3>& color);
    static bool setName(int index,const QString& name);
    static bool assignSelected(int index);
    static bool selectMembers(int index);
    static bool toggleGroupVisible(int first,int count=4);
    static bool hideSelected(bool reverse=false);
    static bool showHidden(const std::vector<std::string>& names={},bool only=false);
    static std::vector<std::pair<std::string,QString>> hiddenObjects();
    static bool objectEditable(App::DocumentObject* object,bool requireVisible=true);
    static bool objectLocked(App::DocumentObject* object);
    // Plain, root OM9 group with a unique validated slot, containing this member.
    static bool isLayerOwner(App::DocumentObject* owner,App::DocumentObject& member);
    static bool selectionEditable();
    // Explode retains source ownership instead of relocating placed components.
    // Inside the caller's transaction, after its existing group assignment.
    static void inheritLayer(App::DocumentObject& output,App::DocumentObject& source);
    // Must be called BEFORE Circle's openTransaction. Throws on lock/corrupt state.
    static CircleLayerSnapshot captureCircle(App::Document& document);
    // Must be inside caller's existing Circle transaction, after native output
    // creation and before recompute/commit; caller aborts on any thrown error.
    // Accepts analytic Part::Feature, persistent History and deformable outputs.
    static void applyCircle(App::Document& document,App::DocumentObject& output,
                            const CircleLayerSnapshot& captured);
};
}
