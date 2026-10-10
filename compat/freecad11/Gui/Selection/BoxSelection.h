// SPDX-License-Identifier: LGPL-2.1-or-later
// Selection traversal from FreeCAD 1.1.4 CommandView.cpp (FreeCAD contributors).
// Adapted only to receive the OM9 drag rectangle instead of native event callback.
#pragma once
#include <Base/Interpreter.h>
#include <App/ComplexGeoDataPy.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Gui/Utilities.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoCamera.h>
namespace Gui {
enum SelectionMode { CENTER, INTERSECT };
static std::vector<std::string> getBoxSelection(
    ViewProviderDocumentObject* vp,
    SelectionMode mode,
    bool selectElement,
    const Base::ViewProjMethod& proj,
    const Base::Polygon2d& polygon,
    const Base::Matrix4D& mat,
    bool transform = true,
    int depth = 0
)
{
    std::vector<std::string> ret;
    auto obj = vp->getObject();
    if (!obj || !obj->isAttachedToDocument()) {
        return ret;
    }

    // DO NOT check this view object Visibility, let the caller do this. Because
    // we may be called by upper object hierarchy that manages our visibility.

    auto bbox3 = vp->getBoundingBox(nullptr, transform);
    if (!bbox3.IsValid()) {
        return ret;
    }

    auto bbox = bbox3.Transformed(mat).ProjectBox(&proj);

    // check if both two boundary points are inside polygon, only
    // valid since we know the given polygon is a box.
    if (polygon.Contains(Base::Vector2d(bbox.MinX, bbox.MinY))
        && polygon.Contains(Base::Vector2d(bbox.MaxX, bbox.MaxY))) {
        ret.emplace_back("");
        return ret;
    }

    if (!bbox.Intersect(polygon)) {
        return ret;
    }

    const auto& subs = obj->getSubObjects(App::DocumentObject::GS_SELECT);
    if (subs.empty()) {
        if (!selectElement) {
            if (mode == INTERSECT || polygon.Contains(bbox.GetCenter())) {
                ret.emplace_back("");
            }
            return ret;
        }
        Base::PyGILStateLocker lock;
        PyObject* pyobj = nullptr;
        Base::Matrix4D matCopy(mat);
        obj->getSubObject(nullptr, &pyobj, &matCopy, transform, depth);
        if (!pyobj) {
            return ret;
        }
        Py::Object pyobject(pyobj, true);
        if (!PyObject_TypeCheck(pyobj, &Data::ComplexGeoDataPy::Type)) {
            return ret;
        }
        auto data = static_cast<Data::ComplexGeoDataPy*>(pyobj)->getComplexGeoDataPtr();
        const bool trace=!qgetenv("OM9_STOCK_SELECTION_TRACE").isEmpty();
        for (auto type : data->getElementTypes()) {
            size_t count = data->countSubElements(type);
            if(trace)Base::Console().message("Stock selection %s %s count=%zu\n",obj->getNameInDocument(),type,count);
            if (!count) {
                continue;
            }
            for (size_t i = 1; i <= count; ++i) {
                std::string element(type);
                element += std::to_string(i);
                std::unique_ptr<Data::Segment> segment(data->getSubElementByName(element.c_str()));
                if (!segment) {
                    continue;
                }
                std::vector<Base::Vector3d> points;
                std::vector<Data::ComplexGeoData::Line> lines;
                data->getLinesFromSubElement(segment.get(), points, lines);
                if (lines.empty()) {
                    if (points.empty()) {
                        continue;
                    }
                    auto v = proj(points[0]);
                    if (polygon.Contains(Base::Vector2d(v.x, v.y))) {
                        ret.push_back(element);
                    }
                    continue;
                }
                Base::Polygon2d loop;
                // TODO: can we assume the line returned above are in proper
                // order if the element is a face?
                auto v = proj(points[lines.front().I1]);
                loop.Add(Base::Vector2d(v.x, v.y));
                for (auto& line : lines) {
                    for (auto i = line.I1; i < line.I2; ++i) {
                        auto v = proj(points[i + 1]);
                        loop.Add(Base::Vector2d(v.x, v.y));
                    }
                }
                if (!polygon.Intersect(loop)) {
                    if(trace)Base::Console().message("Stock selection %s loop missed\n",element.c_str());
                    continue;
                }
                if (mode == CENTER && !polygon.Contains(loop.CalcBoundBox().GetCenter())) {
                    continue;
                }
                ret.push_back(element);
                if(trace)Base::Console().message("Stock selection accepted %s\n",element.c_str());
            }
            break;
        }
        return ret;
    }

    size_t count = 0;
    for (auto& sub : subs) {
        App::DocumentObject* parent = nullptr;
        std::string childName;
        Base::Matrix4D smat(mat);
        auto sobj
            = obj->resolve(sub.c_str(), &parent, &childName, nullptr, nullptr, &smat, transform, depth + 1);
        if (!sobj) {
            continue;
        }
        int vis;
        if (!parent || (vis = parent->isElementVisible(childName.c_str())) < 0) {
            vis = sobj->Visibility.getValue() ? 1 : 0;
        }

        if (!vis) {
            continue;
        }

        auto svp = freecad_cast<ViewProviderDocumentObject*>(
            Application::Instance->getViewProvider(sobj)
        );
        if (!svp) {
            continue;
        }

        const auto& sels
            = getBoxSelection(svp, mode, selectElement, proj, polygon, smat, false, depth + 1);
        if (sels.size() == 1 && sels[0].empty()) {
            ++count;
        }
        for (auto& sel : sels) {
            ret.emplace_back(sub + sel);
        }
    }
    if (count == subs.size()) {
        ret.resize(1);
        ret[0].clear();
    }
    return ret;
}


inline void applyBoxSelection(View3DInventorViewer* viewer,const std::vector<SbVec2s>& pixels,bool selectElement,bool additive) {
    if(pixels.size()!=2)return;
    auto* doc=App::GetApplication().getActiveDocument();if(!doc)return;
    auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return;
    const auto region=viewer->getSoRenderManager()->getViewportRegion();
    const auto points=viewer->getGLPolygon(pixels);
    const Base::Vector2d a(points[0][0],points[0][1]),b(points[1][0],points[1][1]);
    Base::Polygon2d polygon;polygon.Add(a);polygon.Add(Base::Vector2d(a.x,b.y));polygon.Add(b);polygon.Add(Base::Vector2d(b.x,a.y));
    ViewVolumeProjection projection(camera->getViewVolume());
    if(!additive)Selection().clearSelection(doc->getName());
    for(auto* object:doc->getObjects()) {
        if(App::GeoFeatureGroupExtension::getGroupOfObject(object))continue;
        auto* provider=dynamic_cast<ViewProviderDocumentObject*>(Application::Instance->getViewProvider(object));
        if(!provider||!provider->isVisible()||!provider->isSelectable())continue;
        Base::Matrix4D matrix;
        for(const auto& sub:getBoxSelection(provider,a.x>b.x?INTERSECT:CENTER,selectElement,projection,polygon,matrix))
            Selection().addSelection(doc->getName(),object->getNameInDocument(),sub.c_str());
    }
}
}
