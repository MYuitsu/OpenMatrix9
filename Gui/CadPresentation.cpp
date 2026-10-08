#include "CadPresentation.h"
#include <App/DocumentObject.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <App/PropertyStandard.h>
#include <Base/Console.h>
#include <Gui/Document.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <Inventor/actions/SoGLRenderAction.h>
#include <Inventor/actions/SoRayPickAction.h>
#include <Inventor/actions/SoPickAction.h>
#include <Inventor/elements/SoInt32Element.h>
#include <Inventor/elements/SoDrawStyleElement.h>
#include <Inventor/nodes/SoGroup.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoLightModel.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoInfo.h>
#include <Inventor/nodes/SoIndexedFaceSet.h>
#include <Inventor/misc/SoState.h>
#include <Inventor/fields/SoSFBool.h>
#include <Inventor/elements/SoLazyElement.h>
#include <Inventor/elements/SoMaterialBindingElement.h>
#include <BRep_Tool.hxx>
#include <BRepTools.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GCPnts_QuasiUniformDeflection.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopLoc_Location.hxx>
#include <algorithm>
#include <cmath>
#include <unordered_set>
#include <vector>
#include <functional>

namespace OpenMatrix9Gui {
class CadModeElement : public SoInt32Element {
    SO_ELEMENT_HEADER(CadModeElement);
public:
    static void initClass(){SO_ELEMENT_INIT_CLASS(CadModeElement,SoInt32Element);}
    static void put(SoState* state,SoNode* node,int mode){set(classStackIndex,state,node,mode);}
    static int value(SoState* state){return get(classStackIndex,state);}
};
SO_ELEMENT_SOURCE(CadModeElement);
SO_NODE_SOURCE(CadPresentation);

// A group (not a separator) preserves native selection/material state and paths.
class CadFaceGate : public SoGroup {
    SO_NODE_HEADER(CadFaceGate);
public:
    SoSFBool cad;
    static void initClass(){SO_NODE_INIT_CLASS(CadFaceGate,SoGroup,"Group");}
    CadFaceGate(){SO_NODE_CONSTRUCTOR(CadFaceGate);SO_NODE_ADD_FIELD(cad,(true));setName("OM9CadFaceGate");}
    void GLRender(SoGLRenderAction* action) override{
        if(CadModeElement::value(action->getState())!=1)SoGroup::GLRender(action);
        else if(!cad.getValue()){auto* state=action->getState();state->push();SoDrawStyleElement::set(state,SoDrawStyleElement::LINES);SoGroup::GLRender(action);state->pop();}
    }
    void rayPick(SoRayPickAction* action) override{
        if(!cad.getValue()||CadModeElement::value(action->getState())!=1)SoGroup::rayPick(action);
    }
    void pick(SoPickAction* action) override{
        if(!cad.getValue()||CadModeElement::value(action->getState())!=1)SoGroup::pick(action);
    }
protected:~CadFaceGate() override=default;
};
SO_NODE_SOURCE(CadFaceGate);

class CadEdgeGate : public SoGroup {
    SO_NODE_HEADER(CadEdgeGate);
    SoColorPacker packer;
public:
    static void initClass(){SO_NODE_INIT_CLASS(CadEdgeGate,SoGroup,"Group");}
    CadEdgeGate(){SO_NODE_CONSTRUCTOR(CadEdgeGate);setName("OM9CadEdgeGate");}
    void GLRender(SoGLRenderAction* action) override{
        auto* state=action->getState();const auto& color=SoLazyElement::getDiffuse(state,0);
        const bool contrast=CadModeElement::value(state)==1&&std::max({color[0],color[1],color[2]})<.12f;
        const SbColor light(.8f,.8f,.8f);
        if(contrast){state->push();SoLazyElement::setDiffuse(state,this,1,&light,&packer);SoMaterialBindingElement::set(state,SoMaterialBindingElement::OVERALL);}
        SoGroup::GLRender(action);if(contrast)state->pop();
    }
protected:~CadEdgeGate() override=default;
};
SO_NODE_SOURCE(CadEdgeGate);

class CadIsocurves : public SoGroup {
    SO_NODE_HEADER(CadIsocurves);
public:
    TopoDS_Shape snapshot;
    int density=-999;
    Base::Color color;
    static void initClass(){SO_NODE_INIT_CLASS(CadIsocurves,SoGroup,"Group");}
    CadIsocurves(){SO_NODE_CONSTRUCTOR(CadIsocurves);setName("OM9CadIsocurves");}
    void GLRender(SoGLRenderAction* action) override{
        if(CadModeElement::value(action->getState())==1)SoGroup::GLRender(action);
    }
    void rayPick(SoRayPickAction*) override{} // Display guides, never fabricated CAD edges.
    void pick(SoPickAction*) override{}
protected:~CadIsocurves() override=default;
};
SO_NODE_SOURCE(CadIsocurves);

void CadPresentation::initClass(){
    static bool initialized=false;if(initialized)return;initialized=true;
    CadModeElement::initClass();CadFaceGate::initClass();CadEdgeGate::initClass();CadIsocurves::initClass();
    SO_NODE_INIT_CLASS(CadPresentation,SoDrawStyle,"DrawStyle");
    SO_ENABLE(SoGLRenderAction,CadModeElement);SO_ENABLE(SoPickAction,CadModeElement);SO_ENABLE(SoRayPickAction,CadModeElement);
}
CadPresentation::CadPresentation(){
    SO_NODE_CONSTRUCTOR(CadPresentation);SO_NODE_ADD_FIELD(mode,(1));
    style.setIgnored(true);pointSize.setIgnored(true);lineWidth.setIgnored(true);linePattern.setIgnored(true);
}
void CadPresentation::GLRender(SoGLRenderAction* a){CadModeElement::put(a->getState(),this,mode.getValue());}
void CadPresentation::rayPick(SoRayPickAction* a){CadModeElement::put(a->getState(),this,mode.getValue());}
void CadPresentation::pick(SoPickAction* a){CadModeElement::put(a->getState(),this,mode.getValue());}

namespace {
void wrapFaces(SoGroup* root,std::unordered_set<SoNode*>& visited){
    if(!visited.insert(root).second)return;
    for(int i=0;i<root->getNumChildren();++i){auto* child=root->getChild(i);
        if(dynamic_cast<CadFaceGate*>(child)||dynamic_cast<CadEdgeGate*>(child)||dynamic_cast<CadIsocurves*>(child))continue;
        if(child->getTypeId().getName()==SbName("SoBrepEdgeSet")){
            auto* gate=new CadEdgeGate;gate->addChild(child);root->replaceChild(i,gate);continue;
        }
        if(child->isOfType(SoIndexedFaceSet::getClassTypeId())||child->getTypeId().getName()==SbName("SoFCMeshObjectShape")||child->getTypeId().getName()==SbName("SoFCMeshSegmentShape")){
            auto* gate=new CadFaceGate;gate->cad=child->getTypeId().getName()==SbName("SoBrepFaceSet");gate->addChild(child);root->replaceChild(i,gate);
        }else if(auto* group=dynamic_cast<SoGroup*>(child))wrapFaces(group,visited);
    }
}
void unwrap(SoGroup* root,std::unordered_set<SoNode*>& visited){
    if(!visited.insert(root).second)return;
    for(int i=root->getNumChildren()-1;i>=0;--i){auto* child=root->getChild(i);
        if(auto* gate=dynamic_cast<CadFaceGate*>(child)){if(gate->getNumChildren())root->replaceChild(i,gate->getChild(0));}
        else if(auto* gate=dynamic_cast<CadEdgeGate*>(child)){if(gate->getNumChildren())root->replaceChild(i,gate->getChild(0));}
        else if(dynamic_cast<CadIsocurves*>(child))root->removeChild(i);
        else if(child->getName()==SbName("OM9CadOriginalOverride"))root->removeChild(i);
        else if(auto* group=dynamic_cast<SoGroup*>(child))unwrap(group,visited);
    }
}
// Rhino wire density: boundary only (-1), knot wires (0), one wire in knot-free
// spans (1), or density-1 interior wires per span (>=2). End boundaries are CAD edges.
std::vector<double> parameters(double lo,double hi,const Handle(Geom_BSplineSurface)& spline,bool u,int density){
    std::vector<double> knots{lo,hi};
    if(!spline.IsNull())for(int k=1;k<=(u?spline->NbUKnots():spline->NbVKnots());++k){double p=u?spline->UKnot(k):spline->VKnot(k);if(p>lo+1e-9&&p<hi-1e-9)knots.push_back(p);}
    std::sort(knots.begin(),knots.end());knots.erase(std::unique(knots.begin(),knots.end(),[](double a,double b){return std::abs(a-b)<1e-9;}),knots.end());
    std::vector<double> result(knots.begin()+1,knots.end()-1);
    int interiors=density>=2?density-1:(density==1&&result.empty()?1:0);
    for(std::size_t k=1;k<knots.size();++k)for(int j=1;j<=interiors;++j)result.push_back(knots[k-1]+(knots[k]-knots[k-1])*j/(interiors+1));
    return result;
}
void addIso(const TopoDS_Face& face,const Handle(Geom_Surface)& surface,bool u,double fixed,double lo,double hi,std::vector<SbVec3f>& coords,std::vector<int32_t>& lengths){
    if(!std::isfinite(lo)||!std::isfinite(hi)||hi-lo<=1e-9)return;
    Handle(Geom2d_Line) line=new Geom2d_Line(gp_Pnt2d(u?fixed:0,u?0:fixed),gp_Dir2d(u?0:1,u?1:0));
    Handle(Geom2d_TrimmedCurve) finite=new Geom2d_TrimmedCurve(line,lo,hi);
    std::vector<double> cuts{lo,hi};
    for(TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()){
        double a,b;auto pc=BRep_Tool::CurveOnSurface(TopoDS::Edge(e.Current()),face,a,b);if(pc.IsNull()||!std::isfinite(a)||!std::isfinite(b))continue;
        Handle(Geom2d_TrimmedCurve) boundary=new Geom2d_TrimmedCurve(pc,a,b);
        Geom2dAPI_InterCurveCurve intersections(finite,boundary,1e-9);
        for(int k=1;k<=intersections.NbPoints();++k){auto p=intersections.Point(k);double t=u?p.Y():p.X();if(t>lo&&t<hi)cuts.push_back(t);}
    }
    std::sort(cuts.begin(),cuts.end());cuts.erase(std::unique(cuts.begin(),cuts.end(),[](double a,double b){return std::abs(a-b)<1e-8;}),cuts.end());
    auto curve=u?surface->UIso(fixed):surface->VIso(fixed);
    for(std::size_t k=1;k<cuts.size();++k){double a=cuts[k-1],b=cuts[k];if(b-a<1e-9)continue;double mid=(a+b)*.5;
        BRepClass_FaceClassifier classify(face,gp_Pnt2d(u?fixed:mid,u?mid:fixed),1e-8);
        if(classify.State()!=TopAbs_IN)continue;
        GeomAdaptor_Curve adaptor(curve,a,b);GCPnts_QuasiUniformDeflection points(adaptor,0.005,a,b);
        if(!points.IsDone()||points.NbPoints()<2)continue;
        // Bound preview complexity without silently generating an enormous scene.
        if(points.NbPoints()>65536||coords.size()+points.NbPoints()>2000000)throw Standard_Failure("OM9 isocurve preview exceeds display limit");
        if(points.Value(1).Distance(points.Value(points.NbPoints()))<1e-10&&points.NbPoints()==2)continue;
        lengths.push_back(points.NbPoints());for(int j=1;j<=points.NbPoints();++j){auto p=points.Value(j);coords.emplace_back(float(p.X()),float(p.Y()),float(p.Z()));}
    }
}
void rebuild(CadIsocurves* overlay,const TopoDS_Shape& shape,int density,const Base::Color& color){
    std::vector<SbVec3f> coords;std::vector<int32_t> lengths;
    if(density>=0)try{
        for(TopExp_Explorer f(shape,TopAbs_FACE);f.More();f.Next()){
            auto face=TopoDS::Face(f.Current());BRepAdaptor_Surface adapted(face);
            if(adapted.GetType()==GeomAbs_Plane)continue; // Rhino flat faces need boundaries only.
            double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
            if(!std::isfinite(u0)||!std::isfinite(u1)||!std::isfinite(v0)||!std::isfinite(v1)||u1-u0<1e-9||v1-v0<1e-9)continue;
            auto surface=BRep_Tool::Surface(face);auto basis=surface;if(auto trimmed=Handle(Geom_RectangularTrimmedSurface)::DownCast(basis);!trimmed.IsNull())basis=trimmed->BasisSurface();
            auto spline=Handle(Geom_BSplineSurface)::DownCast(basis);
            auto us=parameters(u0,u1,spline,true,density),vs=parameters(v0,v1,spline,false,density);
            if(us.size()+vs.size()>4096)throw Standard_Failure("OM9 isocurve preview exceeds parameter limit");
            for(double u:us)addIso(face,surface,true,u,v0,v1,coords,lengths);
            for(double v:vs)addIso(face,surface,false,v,u0,u1,coords,lengths);
        }
    }catch(const Standard_Failure& error){coords.clear();lengths.clear();Base::Console().warning("OM9 isocurve preview unavailable: {}\n",error.GetMessageString());}
    auto* sep=new SoSeparator;sep->renderCaching=SoSeparator::OFF;
    auto* light=new SoLightModel;light->model=SoLightModel::BASE_COLOR;sep->addChild(light);
    auto* tint=new SoBaseColor;const bool dark=std::max({color.r,color.g,color.b})<.12f;tint->rgb.setValue(dark?.8f:color.r,dark?.8f:color.g,dark?.8f:color.b);sep->addChild(tint);
    auto* style=new SoDrawStyle;style->style=SoDrawStyle::LINES;style->lineWidth=1;sep->addChild(style);
    auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;sep->addChild(pick);
    auto* positions=new SoCoordinate3;positions->point.setNum(int(coords.size()));if(!coords.empty())positions->point.setValues(0,int(coords.size()),coords.data());sep->addChild(positions);
    auto* lines=new SoLineSet;lines->numVertices.setNum(int(lengths.size()));if(!lengths.empty())lines->numVertices.setValues(0,int(lengths.size()),lengths.data());sep->addChild(lines);
    overlay->removeAllChildren();overlay->addChild(sep);overlay->snapshot=shape;overlay->density=density;overlay->color=color;
}
}
void adaptCadProviders(Gui::Document* doc,const std::map<std::string,std::string>& originalModes){
    std::unordered_set<Gui::Document*> documents;
    std::function<void(Gui::Document*,bool)> adapt=[&](Gui::Document* doc,bool external){
    if(!doc||doc->isAboutToClose())return;
    if(!documents.insert(doc).second)return;
    CadPresentation::initClass();std::unordered_set<SoNode*> visited;
    // Snapshot foreign modes before ViewProviderLink forwards Flat Lines.
    for(auto* base:doc->getViewProvidersOfType(Gui::ViewProvider::getClassTypeId())){
        auto* objectProvider=dynamic_cast<Gui::ViewProviderDocumentObject*>(base);
        auto* object=objectProvider?objectProvider->getObject():nullptr;
        // Links forward override writes; only the canonical source owns a mode.
        const bool forwards=object&&object->getLinkedObject(true)!=object;
        if(external&&!forwards){auto* root=base->getRoot();bool remembered=false;for(int i=0;i<root->getNumChildren();++i)if(root->getChild(i)->getName()==SbName("OM9CadOriginalOverride"))remembered=true;
            if(!remembered){auto* note=new SoInfo;note->setName("OM9CadOriginalOverride");auto mode=originalModes.find(doc->getDocument()->getName());note->string=(mode==originalModes.end()?base->getOverrideMode():mode->second).c_str();root->addChild(note);}
        }
        if(auto* p=dynamic_cast<Gui::ViewProviderDocumentObject*>(base);p&&p->getObject())if(auto* linked=p->getObject()->getLinkedObject(true);linked&&linked->getDocument()!=doc->getDocument())adapt(Gui::Application::Instance->getDocument(linked->getDocument()),true);
    }
    for(auto* base:doc->getViewProvidersOfType(Gui::ViewProvider::getClassTypeId())){
        auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(base);if(!provider||!provider->getObject())continue;
        auto* root=provider->getRoot();wrapFaces(root,visited);
        auto* prop=dynamic_cast<Part::PropertyPartShape*>(provider->getObject()->getPropertyByName("Shape"));if(!prop)continue;
        // The native mode switch owns visibility; overlays must live inside it.
        auto* display=dynamic_cast<SoGroup*>(provider->getDisplayMaskMode("Flat Lines"));if(!display)continue;
        CadIsocurves* overlay=nullptr;for(int i=0;i<display->getNumChildren();++i)if(auto* n=dynamic_cast<CadIsocurves*>(display->getChild(i)))overlay=n;
        if(!overlay){overlay=new CadIsocurves;display->addChild(overlay);}
        int density=1;if(auto* p=dynamic_cast<App::PropertyInteger*>(provider->getPropertyByName("OM9IsoCurveDensity")))density=int(p->getValue());
        Base::Color color(0.8f,0.8f,0.8f);if(auto* p=dynamic_cast<App::PropertyColor*>(provider->getPropertyByName("LineColor")))color=p->getValue();
        auto shape=prop->getValue();shape.Location(TopLoc_Location()); // Provider already applies object placement.
        if(!shape.IsSame(overlay->snapshot)||density!=overlay->density||color!=overlay->color){
            if(density>32)Base::Console().warning("OM9 displays at most density 32; requested density {} is retained.\n",density);
            rebuild(overlay,shape,std::min(density,32),color);overlay->density=density;
        }
    }
    };
    adapt(doc,false);
}
void restoreCadProviders(){
    std::vector<Gui::ViewProvider*> providers;
    std::vector<std::pair<Gui::ViewProvider*,std::string>> modes;
    // Container and link graphs share real child roots. Collect every note
    // across documents before removing any nodes or forwarding any writes.
    for(auto* doc:App::GetApplication().getDocuments())if(auto* gui=Gui::Application::Instance->getDocument(doc))
        for(auto* p:gui->getViewProvidersOfType(Gui::ViewProvider::getClassTypeId())){
            providers.push_back(p);auto* root=p->getRoot();
            for(int i=0;i<root->getNumChildren();++i)
                if(auto* note=dynamic_cast<SoInfo*>(root->getChild(i));note&&note->getName()==SbName("OM9CadOriginalOverride")){
                    auto* vp=dynamic_cast<Gui::ViewProviderDocumentObject*>(p);auto* obj=vp?vp->getObject():nullptr;
                    if(!obj||obj->getLinkedObject(true)==obj)modes.emplace_back(p,note->string.getValue().getString());
                }
        }
    for(auto& [provider,mode]:modes)provider->setOverrideMode(mode);
    std::unordered_set<SoNode*> visited;
    for(auto* provider:providers)unwrap(provider->getRoot(),visited);
}
}
