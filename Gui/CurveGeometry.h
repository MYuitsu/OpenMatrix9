#pragma once
#include <array>
#include <vector>
#include <stdexcept>
#include <utility>
#include <TopoDS_Shape.hxx>
namespace App {class Document;class DocumentObject;}
class TopoDS_Wire;
namespace OpenMatrix9Gui {
TopoDS_Shape publishedSplineShape();
App::DocumentObject* createCurveFeature(App::Document&,const TopoDS_Shape&,const char* name);
std::vector<std::array<double,3>> sampleCurve(const TopoDS_Shape&,std::size_t count);
void validateCurveWire(const TopoDS_Wire&,std::size_t expectedEdges);
}
