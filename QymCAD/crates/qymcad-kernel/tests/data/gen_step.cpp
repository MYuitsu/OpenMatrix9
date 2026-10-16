// Writes the small assembly the STEP structure checks read (`tests/data/assembly.step`): names in Cyrillic, written
// here as UTF-8 byte escapes; one part placed twice, a subassembly; the plate red with its top face green, a colour of
// the face's own. Build against the same OCCT and run with the path.
#include <XCAFApp_Application.hxx>
#include <TDocStd_Document.hxx>
#include <XCAFDoc_DocumentTool.hxx>
#include <XCAFDoc_ShapeTool.hxx>
#include <XCAFDoc_ColorTool.hxx>
#include <TDataStd_Name.hxx>
#include <STEPCAFControl_Writer.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <Quantity_Color.hxx>
#include <gp_Trsf.hxx>
#include <TopExp_Explorer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <cmath>
#include <TopLoc_Location.hxx>
#include <cstdio>
static TopLoc_Location at(double x, double y, double z) { gp_Trsf t; t.SetTranslation(gp_Vec(x, y, z)); return TopLoc_Location(t); }
int main(int argc, char** argv) {
    if (argc < 2) return 2;
    Handle(TDocStd_Document) doc;
    XCAFApp_Application::GetApplication()->NewDocument("MDTV-XCAF", doc);
    Handle(XCAFDoc_ShapeTool) st = XCAFDoc_DocumentTool::ShapeTool(doc->Main());
    Handle(XCAFDoc_ColorTool) ct = XCAFDoc_DocumentTool::ColorTool(doc->Main());
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(10, 20, 5).Shape();
    TDF_Label plate = st->AddShape(box, false);
    TDataStd_Name::Set(plate, TCollection_ExtendedString("\xD0\x9F\xD0\xBB\xD0\xB0\xD1\x81\xD1\x82\xD0\xB8\xD0\xBD\xD0\xB0", Standard_True));
    ct->SetColor(plate, Quantity_Color(0.8, 0.1, 0.1, Quantity_TOC_sRGB), XCAFDoc_ColorSurf);
    for (TopExp_Explorer ex(box, TopAbs_FACE); ex.More(); ex.Next()) {
        GProp_GProps g;
        BRepGProp::SurfaceProperties(ex.Current(), g);
        if (std::abs(g.CentreOfMass().Z() - 5.0) < 1e-9) ct->SetColor(st->AddSubShape(plate, ex.Current()), Quantity_Color(0.1, 0.8, 0.1, Quantity_TOC_sRGB), XCAFDoc_ColorSurf);
    }
    TDF_Label pin = st->AddShape(BRepPrimAPI_MakeCylinder(4, 12).Shape(), false);
    TDataStd_Name::Set(pin, TCollection_ExtendedString("\xD0\xA8\xD1\x82\xD0\xB8\xD1\x84\xD1\x82", Standard_True));
    ct->SetColor(pin, Quantity_Color(0.1, 0.2, 0.9, Quantity_TOC_sRGB), XCAFDoc_ColorSurf);
    TDF_Label unit = st->NewShape();
    TDataStd_Name::Set(unit, TCollection_ExtendedString("\xD0\xA3\xD0\xB7\xD0\xB5\xD0\xBB", Standard_True));
    st->AddComponent(unit, pin, at(0, 0, 5));
    TDF_Label top = st->NewShape();
    TDataStd_Name::Set(top, TCollection_ExtendedString("\xD0\xA1\xD0\xB1\xD0\xBE\xD1\x80\xD0\xBA\xD0\xB0", Standard_True));
    st->AddComponent(top, plate, at(0, 0, 0));
    st->AddComponent(top, plate, at(30, 0, 0));
    st->AddComponent(top, unit, at(5, 10, 0));
    st->UpdateAssemblies();
    STEPCAFControl_Writer w;
    w.SetNameMode(true);
    w.SetColorMode(true);
    if (!w.Transfer(doc, STEPControl_AsIs)) return 3;
    return w.Write(argv[1]) == IFSelect_RetDone ? 0 : 4;
}
