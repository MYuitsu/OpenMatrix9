#pragma once
#include <Inventor/nodes/SoDrawStyle.h>
#include <Inventor/fields/SoSFInt32.h>
#include <map>
#include <string>
namespace Gui {class Document;}
namespace OpenMatrix9Gui {
// View-local traversal state; shared providers never change mode per render.
class CadPresentation : public SoDrawStyle {
    SO_NODE_HEADER(CadPresentation);
public:
    static void initClass();
    CadPresentation();
    SoSFInt32 mode; // 0 native, 1 CAD Wireframe, 2 Shaded
    void GLRender(SoGLRenderAction*) override;
    void rayPick(SoRayPickAction*) override;
    void pick(SoPickAction*) override;
protected:
    ~CadPresentation() override=default;
};
void adaptCadProviders(Gui::Document*,const std::map<std::string,std::string>& originalModes);
void restoreCadProviders();
}
