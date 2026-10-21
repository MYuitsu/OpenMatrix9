#include "ThreeDmSolidShells.h"
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepLib.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <vector>
namespace OpenMatrix9Gui::ThreeDm {
static int direction(const TopoDS_Solid& solid,double tolerance){
    if(!BRepCheck_Analyzer(solid).IsValid())throw ExchangeError("Cannot classify invalid solid orientation");
    BRepClass3d_SolidClassifier classifier(solid);classifier.PerformInfinitePoint(tolerance);
    if(classifier.State()!=TopAbs_IN&&classifier.State()!=TopAbs_OUT)throw ExchangeError("Cannot determine native solid orientation");
    return classifier.State()==TopAbs_IN?-1:1;
}
int classifiedBrepSolidOrientation(const TopoDS_Shape& shape,double tolerance){
    TopTools_IndexedMapOfShape solids,faces,ownedFaces;TopExp::MapShapes(shape,TopAbs_SOLID,solids);TopExp::MapShapes(shape,TopAbs_FACE,faces);
    if(solids.IsEmpty()||!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Cannot classify invalid or non-solid BRep orientation");
    int result=0;
    for(int i=1;i<=solids.Extent();++i){auto solid=TopoDS::Solid(solids(i));const int value=direction(solid,tolerance);if(result&&result!=value)throw ExchangeError("Mixed solid orientations cannot resolve one native BRep orientation");result=value;TopExp::MapShapes(solid,TopAbs_FACE,ownedFaces);}
    if(ownedFaces.Extent()!=faces.Extent())throw ExchangeError("Unowned faces in solid BRep classification");return result;
}
TopoDS_Shape assembleClosedBrepShells(const TopoDS_Shape& input,int declaredOrientation,double tolerance){
    BRep_Builder builder;TopoDS_Shape shape=input;
    if(shape.ShapeType()==TopAbs_FACE){TopoDS_Shell shell;builder.MakeShell(shell);builder.Add(shell,shape);shape=shell;}
    TopTools_IndexedMapOfShape mapped,faces,ownedFaces;TopExp::MapShapes(shape,TopAbs_SHELL,mapped);TopExp::MapShapes(shape,TopAbs_FACE,faces);
    // Sewing can return closed periodic faces directly, especially multiple
    // spheres with singular poles. Each independently closed face owns a shell;
    // combining them as one shell would lose their containment relationship.
    TopTools_IndexedMapOfShape shellFaces;for(int i=1;i<=mapped.Extent();++i)TopExp::MapShapes(mapped(i),TopAbs_FACE,shellFaces);
    for(int i=1;i<=faces.Extent();++i)if(!shellFaces.Contains(faces(i))){
        TopoDS_Shell shell;builder.MakeShell(shell);builder.Add(shell,faces(i));
        if(!BRep_Tool::IsClosed(shell))throw GeometryRepresentationUnavailable("Closed native BRep has an unclosed standalone converted face; native graph remains preserved");mapped.Add(shell);
    }
    if(mapped.IsEmpty()||mapped.Extent()>512)throw ExchangeError("Closed BRep shell count exceeds supported assembly limits");
    struct Shell{TopoDS_Shell shape;TopoDS_Solid probe;gp_Pnt point;int direction=0,parent=-1,depth=0;};std::vector<Shell> shells;
    for(int i=1;i<=mapped.Extent();++i){
        Shell item;item.shape=TopoDS::Shell(mapped(i));if(!BRep_Tool::IsClosed(item.shape))throw ExchangeError("Imported solid shell is open");
        item.probe=BRepBuilderAPI_MakeSolid(item.shape).Solid();item.direction=direction(item.probe,tolerance);
        TopExp_Explorer vertex(item.shape,TopAbs_VERTEX);if(!vertex.More())throw ExchangeError("Closed shell has no verified boundary witness");item.point=BRep_Tool::Pnt(TopoDS::Vertex(vertex.Current()));
        TopExp::MapShapes(item.shape,TopAbs_FACE,ownedFaces);shells.push_back(item);
    }
    if(ownedFaces.Extent()!=faces.Extent())throw ExchangeError("Solid BRep has faces outside closed shells");
    const auto n=shells.size();std::vector<std::vector<bool>> contains(n,std::vector<bool>(n,false));
    for(size_t i=0;i<n;++i)for(size_t j=i+1;j<n;++j){
        BRepExtrema_DistShapeShape distance(shells[i].shape,shells[j].shape);
        if(!distance.IsDone()||distance.Value()<=tolerance)throw ExchangeError("BRep closed shells intersect, touch or cannot be separated within tolerance");
        auto inside=[&](size_t container,size_t child){BRepClass3d_SolidClassifier classifier(shells[container].probe);classifier.Perform(shells[child].point,tolerance);
            if(classifier.State()!=TopAbs_IN&&classifier.State()!=TopAbs_OUT)throw ExchangeError("Cannot classify closed shell containment");return classifier.State()==(shells[container].direction==1?TopAbs_IN:TopAbs_OUT);};
        contains[i][j]=inside(i,j);contains[j][i]=inside(j,i);if(contains[i][j]&&contains[j][i])throw ExchangeError("Cyclic closed shell containment");
    }
    for(size_t child=0;child<n;++child)for(size_t candidate=0;candidate<n;++candidate)if(contains[candidate][child]){
        auto& parent=shells[child].parent;
        if(parent<0||contains[parent][candidate])parent=int(candidate);
        else if(!contains[candidate][parent])throw ExchangeError("Inconsistent closed shell containment forest");
    }
    for(size_t i=0;i<n;++i){int parent=shells[i].parent;while(parent>=0){if(++shells[i].depth>int(n))throw ExchangeError("Cyclic closed shell parent graph");parent=shells[parent].parent;}
        if(shells[i].parent>=0&&shells[i].direction==shells[shells[i].parent].direction)throw ExchangeError("Nested shell orientation does not describe a cavity");}
    std::vector<TopoDS_Solid> material;
    int materialDirection=0;bool mixedMaterialDirection=false;
    for(const auto& shell:shells)if(shell.depth%2==0){if(materialDirection&&materialDirection!=shell.direction)mixedMaterialDirection=true;materialDirection=shell.direction;}
    const int verifiedOrientation=mixedMaterialDirection?0:materialDirection;
    for(size_t i=0;i<n;++i)if(shells[i].depth%2==0){BRepBuilderAPI_MakeSolid make(shells[i].shape);
        if(shells[i].direction<0&&declaredOrientation!=1){for(size_t j=0;j<n;++j)if(shells[j].parent==int(i))throw GeometryRepresentationUnavailable("Inward multi-shell cavity has no verified valid editable OCCT solid; native BRep remains preserved",verifiedOrientation);}
        for(size_t j=0;j<n;++j)if(shells[j].parent==int(i))make.Add(shells[j].shape);
        auto solid=make.Solid();if(!BRepCheck_Analyzer(solid).IsValid())throw ExchangeError("Assembled multi-shell solid has invalid topology");
        if(declaredOrientation==1&&!BRepLib::OrientClosedSolid(solid))throw ExchangeError("Cannot orient declared outward solid");material.push_back(solid);}
    if(material.size()==1)return material.front();TopoDS_Compound result;builder.MakeCompound(result);for(auto& solid:material)builder.Add(result,solid);return result;
}
}
