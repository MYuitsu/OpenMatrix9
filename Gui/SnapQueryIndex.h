#pragma once
#include <Python.h>
#include "CoreSnapGeometry.h"
#include <QPoint>
#include <string>
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
struct SnapQuery {
    QPoint cursor_px;
    double radius_px=8;
    unsigned modes=2;
    std::size_t max_objects=64,max_candidates_per_object=2048,max_candidates_total=8192;
};
struct SnapQueryResult {
    std::vector<Base::Vector3d> candidates;
    bool complete=true,picked=false,index_rebuilt=false;
    Base::Vector3d point;
    std::size_t visited_objects=0,visited_topology=0,max_per_object=0,index_objects=0;
    std::size_t generated_candidates=0;
    unsigned read_mesh_vertices=0,read_cloud_points=0;
    int viewport_width=0,viewport_height=0;
    long long elapsed_us=0;
    std::string reason;
};
SnapQueryResult querySnapCandidates(Gui::View3DInventor*,const SnapQuery&);
}
void AddSnapQueryMethods(PyObject*);
