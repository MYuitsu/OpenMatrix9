#pragma once
#include <Gui/View3DInventorViewer.h>
#include <Base/Exception.h>
#include <Inventor/SoDB.h>
#include <Inventor/SoInput.h>
#include <Inventor/SoOutput.h>
#include <Inventor/actions/SoWriteAction.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoCamera.h>
#include <memory>
#include <string>
#include <cstdlib>
namespace OpenMatrix9Gui {
inline std::string cameraState(Gui::View3DInventorViewer* viewer) {
    auto* camera=viewer->getSoRenderManager()->getCamera();
    if(!camera)return {};
    SoOutput output;void* initial=std::malloc(256);
    if(!initial)throw std::bad_alloc();
    output.setBuffer(initial,256,[](void* p,size_t n)->void*{return std::realloc(p,n);});
    SoWriteAction action(&output);action.apply(camera);
    void* data=nullptr;size_t size=0;output.getBuffer(data,size);
    std::string result(static_cast<const char*>(data),size);std::free(data);return result;
}
// Native setCamera omits Perspective heightAngle. Restore all camera fields.
inline void restoreCameraState(Gui::View3DInventorViewer* viewer,const std::string& state) {
    SoInput input;input.setBuffer(state.data(),state.size());SoNode* node=nullptr;
    if(!SoDB::read(&input,node)||!node)throw Base::RuntimeError("Invalid camera state");
    node->ref();auto release=[](SoNode* value){value->unref();};std::unique_ptr<SoNode,decltype(release)> guard(node,release);
    auto* source=dynamic_cast<SoCamera*>(node);if(!source)throw Base::TypeError("Expected camera state");
    viewer->setCameraType(source->getTypeId());auto* target=viewer->getSoRenderManager()->getCamera();
    if(target&&target->getTypeId()==source->getTypeId())target->copyFieldValues(source,false);
}
}
