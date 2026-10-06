#pragma once
#include <Base/Vector3D.h>
#include <vector>
namespace App {class DocumentObject;}
namespace OpenMatrix9Gui {
// Native topology vertices with the object's Placement applied by getSubObject.
// Visibility, parent/link transforms and cursor projection belong to the caller.
std::vector<Base::Vector3d> endCandidates(const App::DocumentObject* object);
std::vector<Base::Vector3d> midCandidates(const App::DocumentObject* object);
std::vector<Base::Vector3d> pointCandidates(const App::DocumentObject* object);
}
