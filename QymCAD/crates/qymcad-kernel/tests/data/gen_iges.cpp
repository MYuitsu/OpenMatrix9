// Writes the small assembly the IGES structure checks read, laid out the way a CAD that writes subfigures lays it
// out: a part is a subfigure definition (308) holding its solid (186), an occurrence a singular subfigure instance
// (408) with its translation; names in Windows-1251 with the occurrence number, colours on the faces (510 -> 314),
// the plate's top face - its sixth - green where the rest of it is red.
// As in that file, the name property (406) of an instance names the assembly it stands in, not its own product: the
// product's name is on its definition.
// With `--flat` it writes the same parts the way a CAD that writes no subfigures does: solids (186) standing on their
// own where they stand in the assembly, each named by a name property (406, form 15), the plate coloured on its faces
// (the top one green), the pin as a whole, and a group (402, form 7) holding the pin - the subassembly.
// With `--surfaces` it writes them the way a CAD that exports surfaces does: no solid (186) at all, every face a
// surface of its own (a trimmed surface, 144) in the colour of its face - the plate's top green, the rest of it red,
// the pin blue - and the parts as groups (402, form 7): "Plate" of the plate's surfaces, "Pin" of the pin's, and "Unit"
// holding "Pin".
#include <IGESControl_Writer.hxx>
#include <BRepToIGES_BREntity.hxx>
#include <BRepToIGES_BRShell.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <IGESData_IGESModel.hxx>
#include <BRepToIGESBRep_Entity.hxx>
#include <IGESBasic_SubfigureDef.hxx>
#include <IGESBasic_SingularSubfigure.hxx>
#include <IGESData_HArray1OfIGESEntity.hxx>
#include <IGESSolid_ManifoldSolid.hxx>
#include <IGESSolid_Shell.hxx>
#include <IGESSolid_Face.hxx>
#include <IGESGraph_Color.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <TCollection_HAsciiString.hxx>
#include <IGESBasic_Name.hxx>
#include <IGESBasic_GroupWithoutBackP.hxx>
#include <gp_Ax2.hxx>
#include <string>
#include <vector>

static Handle(IGESData_IGESEntity) solid(IGESControl_Writer& w, const TopoDS_Shape& s, const Handle(IGESGraph_Color)& colour, int odd = 0, const Handle(IGESGraph_Color)& odd_colour = nullptr) {
    BRepToIGESBRep_Entity be;
    be.Init();
    be.SetModel(w.Model());
    Handle(IGESData_IGESEntity) e = be.TransferShape(s);
    Handle(IGESSolid_ManifoldSolid) m = Handle(IGESSolid_ManifoldSolid)::DownCast(e);
    if (!m.IsNull()) {
        Handle(IGESSolid_Shell) sh = m->Shell();
        for (Standard_Integer i = 1; i <= sh->NbFaces(); ++i) {
            const Handle(IGESGraph_Color)& c = i == odd && !odd_colour.IsNull() ? odd_colour : colour;
            if (!c.IsNull()) sh->Face(i)->InitColor(c);
        }
    }
    return e;
}

static Handle(IGESBasic_SubfigureDef) def(int depth, const char* name, const std::vector<Handle(IGESData_IGESEntity)>& ents) {
    Handle(IGESData_HArray1OfIGESEntity) a = new IGESData_HArray1OfIGESEntity(1, (Standard_Integer)ents.size());
    for (size_t i = 0; i < ents.size(); ++i) a->SetValue((Standard_Integer)i + 1, ents[i]);
    Handle(IGESBasic_SubfigureDef) d = new IGESBasic_SubfigureDef;
    d->Init(depth, new TCollection_HAsciiString(name), a);
    return d;
}

static Handle(IGESBasic_SingularSubfigure) at(const Handle(IGESBasic_SubfigureDef)& d, double x, double y, double z, const char* within) {
    Handle(IGESBasic_SingularSubfigure) s = new IGESBasic_SingularSubfigure;
    s->Init(d, gp_XYZ(x, y, z), Standard_False, 1.0);
    Handle(IGESBasic_Name) n = new IGESBasic_Name;
    n->Init(1, new TCollection_HAsciiString(within));
    s->AddProperty(n);
    return s;
}

static void named(const Handle(IGESData_IGESEntity)& e, const char* name) {
    Handle(IGESBasic_Name) n = new IGESBasic_Name;
    n->Init(1, new TCollection_HAsciiString(name));
    e->AddProperty(n);
}

// The faces of `s`, each a surface of its own in the model, in `colour` - the `odd`-th in `odd_colour` - as a group
// named `name`.
static Handle(IGESBasic_GroupWithoutBackP) surfaces(IGESControl_Writer& w, const TopoDS_Shape& s, const char* name, const Handle(IGESGraph_Color)& colour, int odd = 0, const Handle(IGESGraph_Color)& odd_colour = nullptr) {
    BRepToIGES_BREntity be;
    be.Init();
    be.SetModel(w.Model());
    BRepToIGES_BRShell shell(be);
    std::vector<Handle(IGESData_IGESEntity)> faces;
    int k = 0;
    for (TopExp_Explorer ex(s, TopAbs_FACE); ex.More(); ex.Next()) {
        ++k;
        Handle(IGESData_IGESEntity) e = shell.TransferFace(TopoDS::Face(ex.Current()));
        if (e.IsNull()) continue;
        e->InitColor(k == odd && !odd_colour.IsNull() ? odd_colour : colour);
        w.AddEntity(e);
        faces.push_back(e);
    }
    Handle(IGESData_HArray1OfIGESEntity) a = new IGESData_HArray1OfIGESEntity(1, (Standard_Integer)faces.size());
    for (size_t i = 0; i < faces.size(); ++i) a->SetValue((Standard_Integer)i + 1, faces[i]);
    Handle(IGESBasic_GroupWithoutBackP) g = new IGESBasic_GroupWithoutBackP;
    g->Init(a);
    named(g, name);
    return g;
}

static int loose(const char* path) {
    IGESControl_Writer w("MM", 0);
    Handle(IGESGraph_Color) red = new IGESGraph_Color;
    red->Init(80.0, 10.0, 10.0, new TCollection_HAsciiString("red"));
    Handle(IGESGraph_Color) blue = new IGESGraph_Color;
    blue->Init(10.0, 20.0, 90.0, new TCollection_HAsciiString("blue"));
    Handle(IGESGraph_Color) green = new IGESGraph_Color;
    green->Init(10.0, 80.0, 10.0, new TCollection_HAsciiString("green"));
    Handle(IGESBasic_GroupWithoutBackP) plate = surfaces(w, BRepPrimAPI_MakeBox(10, 20, 5).Shape(), "\xCF\xEB\xE0\xF1\xF2\xE8\xED\xE0", red, 6, green);
    Handle(IGESBasic_GroupWithoutBackP) pin = surfaces(w, BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 10, 5), gp_Dir(0, 0, 1)), 4, 12).Shape(), "\xD8\xF2\xE8\xF4\xF2", blue);
    Handle(IGESData_HArray1OfIGESEntity) members = new IGESData_HArray1OfIGESEntity(1, 1);
    members->SetValue(1, pin);
    Handle(IGESBasic_GroupWithoutBackP) unit = new IGESBasic_GroupWithoutBackP;
    unit->Init(members);
    named(unit, "\xD3\xE7\xE5\xEB");
    w.AddEntity(plate);
    w.AddEntity(pin);
    w.AddEntity(unit);
    w.ComputeModel();
    return w.Write(path) ? 0 : 3;
}

// A solid with surfaces beside it: the plate a solid (186), named and coloured as in `--flat`, the pin surfaces of its
// own in the group "Pin" under the group "Unit".
static int mixed(const char* path) {
    IGESControl_Writer w("MM", 1);
    Handle(IGESGraph_Color) red = new IGESGraph_Color;
    red->Init(80.0, 10.0, 10.0, new TCollection_HAsciiString("red"));
    Handle(IGESGraph_Color) blue = new IGESGraph_Color;
    blue->Init(10.0, 20.0, 90.0, new TCollection_HAsciiString("blue"));
    Handle(IGESGraph_Color) green = new IGESGraph_Color;
    green->Init(10.0, 80.0, 10.0, new TCollection_HAsciiString("green"));
    Handle(IGESData_IGESEntity) plate = solid(w, BRepPrimAPI_MakeBox(10, 20, 5).Shape(), red, 6, green);
    named(plate, "\xCF\xEB\xE0\xF1\xF2\xE8\xED\xE0");
    Handle(IGESBasic_GroupWithoutBackP) pin = surfaces(w, BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 10, 5), gp_Dir(0, 0, 1)), 4, 12).Shape(), "\xD8\xF2\xE8\xF4\xF2", blue);
    Handle(IGESData_HArray1OfIGESEntity) members = new IGESData_HArray1OfIGESEntity(1, 1);
    members->SetValue(1, pin);
    Handle(IGESBasic_GroupWithoutBackP) unit = new IGESBasic_GroupWithoutBackP;
    unit->Init(members);
    named(unit, "\xD3\xE7\xE5\xEB");
    w.AddEntity(plate);
    w.AddEntity(pin);
    w.AddEntity(unit);
    w.ComputeModel();
    return w.Write(path) ? 0 : 3;
}

static int flat(const char* path) {
    IGESControl_Writer w("MM", 1);
    Handle(IGESGraph_Color) red = new IGESGraph_Color;
    red->Init(80.0, 10.0, 10.0, new TCollection_HAsciiString("red"));
    Handle(IGESGraph_Color) blue = new IGESGraph_Color;
    blue->Init(10.0, 20.0, 90.0, new TCollection_HAsciiString("blue"));
    Handle(IGESGraph_Color) green = new IGESGraph_Color;
    green->Init(10.0, 80.0, 10.0, new TCollection_HAsciiString("green"));
    Handle(IGESData_IGESEntity) plate = solid(w, BRepPrimAPI_MakeBox(10, 20, 5).Shape(), red, 6, green);
    named(plate, "\xCF\xEB\xE0\xF1\xF2\xE8\xED\xE0");
    Handle(IGESData_IGESEntity) pin = solid(w, BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 10, 5), gp_Dir(0, 0, 1)), 4, 12).Shape(), nullptr);
    pin->InitColor(blue);
    named(pin, "\xD8\xF2\xE8\xF4\xF2");
    Handle(IGESData_HArray1OfIGESEntity) members = new IGESData_HArray1OfIGESEntity(1, 1);
    members->SetValue(1, pin);
    Handle(IGESBasic_GroupWithoutBackP) unit = new IGESBasic_GroupWithoutBackP;
    unit->Init(members);
    named(unit, "\xD3\xE7\xE5\xEB");
    w.AddEntity(plate);
    w.AddEntity(pin);
    w.AddEntity(unit);
    w.ComputeModel();
    return w.Write(path) ? 0 : 3;
}

int main(int argc, char** argv) {
    if (argc >= 3 && std::string(argv[1]) == "--flat") return flat(argv[2]);
    if (argc >= 3 && std::string(argv[1]) == "--surfaces") return loose(argv[2]);
    if (argc >= 3 && std::string(argv[1]) == "--mixed") return mixed(argv[2]);
    if (argc < 2) return 2;
    IGESControl_Writer w("MM", 1);
    Handle(IGESGraph_Color) red = new IGESGraph_Color;
    red->Init(80.0, 10.0, 10.0, new TCollection_HAsciiString("red")); // IGES colour components are percent
    Handle(IGESGraph_Color) blue = new IGESGraph_Color;
    blue->Init(10.0, 20.0, 90.0, new TCollection_HAsciiString("blue"));
    Handle(IGESGraph_Color) green = new IGESGraph_Color;
    green->Init(10.0, 80.0, 10.0, new TCollection_HAsciiString("green"));
    Handle(IGESBasic_SubfigureDef) plate = def(0, "\xCF\xEB\xE0\xF1\xF2\xE8\xED\xE0:1 \xCF\xEB\xE0\xF1\xF2\xE8\xED\xE0:1", {solid(w, BRepPrimAPI_MakeBox(10, 20, 5).Shape(), red, 6, green)});
    Handle(IGESBasic_SubfigureDef) pin = def(0, "\xD8\xF2\xE8\xF4\xF2:1 \xD8\xF2\xE8\xF4\xF2:1", {solid(w, BRepPrimAPI_MakeCylinder(4, 12).Shape(), blue)});
    Handle(IGESBasic_SubfigureDef) unit = def(1, "\xD3\xE7\xE5\xEB:1 \xD3\xE7\xE5\xEB:1", {at(pin, 0, 0, 5, "\xD3\xE7\xE5\xEB")});
    Handle(IGESBasic_SubfigureDef) top = def(2, "\xD1\xE1\xEE\xF0\xEA\xE0:1 \xD1\xE1\xEE\xF0\xEA\xE0:1", {at(plate, 0, 0, 0, "\xD1\xE1\xEE\xF0\xEA\xE0"), at(plate, 30, 0, 0, "\xD1\xE1\xEE\xF0\xEA\xE0"), at(unit, 5, 10, 0, "\xD1\xE1\xEE\xF0\xEA\xE0")});
    w.AddEntity(at(top, 0, 0, 0, "\xD1\xE1\xEE\xF0\xEA\xE0 \xD1\xE1\xEE\xF0\xEA\xE0"));
    w.ComputeModel();
    return w.Write(argv[1]) ? 0 : 3;
}
