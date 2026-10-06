#pragma once
#include <Gui/View3DInventorViewer.h>
#include <Base/Exception.h>
#include <Inventor/SoDB.h>
#include <Inventor/SoInput.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoCamera.h>
#include <memory>
#include <string>
namespace OpenMatrix9Gui {
// Native setCamera omits Perspective heightAngle. Restore all camera fields.
inline void restoreCameraState(Gui::View3DInventorViewer* viewer,const std::string& state) {
    SoInput input;input.setBuffer(state.data(),state.size());SoNode* node=nullptr;
    if(!SoDB::read(&input,node)||!node)throw Base::RuntimeError("Invalid camera state");
    node->ref();auto release=[](SoNode* value){value->unref();};std::unique_ptr<SoNode,decltype(release)> guard(node,release);
    auto* source=dynamic_cast<SoCamera*>(node);if(!source)throw Base::TypeError("Expected camera state");
    if(!viewer->setCamera(state.c_str()))return;auto* target=viewer->getSoRenderManager()->getCamera();
    if(target&&target->getTypeId()==source->getTypeId())target->copyFieldValues(source,false);
}
}
