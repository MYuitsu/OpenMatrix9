// SPDX-License-Identifier: LGPL-2.1-or-later
#include <Inventor/SbVec3f.h>
#include "ExplodedText.h"
#include <Inventor/nodes/SoMatrixTransform.h>
#include <Inventor/nodes/SoSeparator.h>
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::ExplodedText,App::Annotation)
PROPERTY_SOURCE(OpenMatrix9Gui::ViewProviderExplodedText,Gui::ViewProviderAnnotation)
ExplodedText::ExplodedText(){ADD_PROPERTY(TextFrame,(Base::Matrix4D()));}
void ViewProviderExplodedText::attach(App::DocumentObject* object){Gui::ViewProviderAnnotation::attach(object);frame=new SoMatrixTransform;getRoot()->insertChild(frame,0);updateData(&static_cast<ExplodedText*>(object)->TextFrame);}
void ViewProviderExplodedText::updateData(const App::Property* property){if(frame&&property==&static_cast<ExplodedText*>(getObject())->TextFrame){const auto& m=static_cast<ExplodedText*>(getObject())->TextFrame.getValue();SbMatrix matrix;for(unsigned i=0;i<4;++i)for(unsigned j=0;j<4;++j)matrix[i][j]=float(m[j][i]);frame->matrix=matrix;}Gui::ViewProviderAnnotation::updateData(property);}
void initializeEditSpecialTypes(){ExplodedText::init();ViewProviderExplodedText::init();}
}
