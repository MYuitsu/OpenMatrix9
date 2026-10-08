#include "ThreeDmNativeReferences.h"
#include <iostream>
#include <cmath>
#include <limits>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* what){if(!value)throw ExchangeError(what);}
int main(){try{ON::Begin();int checks=0;
    for(bool nonlinear:{false,true})for(bool reversed:{false,true}){
        ON_NurbsSurface surface(3,false,2,2,2,2);for(int d=0;d<2;++d){surface.SetKnot(d,0,0);surface.SetKnot(d,1,1);}
        for(int i=0;i<2;++i)for(int j=0;j<2;++j)surface.SetCV(i,j,ON_3dPoint(2*i,3*j,0));
        ON_Brep brep;require(brep.NewFace(surface)&&brep.IsValid(),"Native plane face");auto& trim=brep.m_T[0];auto edge=trim.Edge();
        auto curve=new ON_NurbsCurve(2,false,3,3);curve->SetCV(0,ON_3dPoint(0,0,0));curve->SetCV(1,ON_3dPoint(nonlinear?0:.5,0,0));curve->SetCV(2,ON_3dPoint(1,0,0));
        curve->SetKnot(0,101);curve->SetKnot(1,101);curve->SetKnot(2,149);curve->SetKnot(3,149);
        delete brep.m_C2[trim.m_c2i];brep.m_C2[trim.m_c2i]=curve;trim.SetProxyCurve(curve);require(trim.SetDomain(101,149)&&edge->SetDomain(11,23)&&brep.IsValid(),"Different nonlinear native trim and edge domains");
        if(reversed){require(edge->Reverse(),"Reverse native edge topology");require(edge->SetDomain(11,23),"Independent reversed edge domain");}
        const auto immutable=brep.DataCRC(0);
        for(int i=1;i<16;++i){double fraction=i/16.;double physical=reversed?1-fraction:fraction;
            auto result=nativeTrimParameterAt(trim,*edge,*brep.m_S[0],ON_Interval(101,149),11+12*fraction,1e-7);
            double expected=101+48*(nonlinear?std::sqrt(physical):physical);
            require(std::abs(result.parameter-expected)<1e-6,"Projection must recover nonlinear native trim parameter, not linear interpolation");
            require(result.deviation<1e-7,"Native evaluation independently validates projected parameter");++checks;
        }
        require(brep.DataCRC(0)==immutable,"Native parameter mapping never changes authoritative geometry");++checks;
        for(double tolerance:{-1.,std::numeric_limits<double>::quiet_NaN()}){bool invalid=false;try{nativeTrimParameterAt(trim,*edge,*brep.m_S[0],ON_Interval(101,149),17,tolerance);}catch(const ExchangeError&){invalid=true;}require(invalid,"Invalid tolerance rejected");++checks;}
        // Endpoints still agree; interior geometry is wrong. A two-endpoint oracle cannot detect this.
        curve->SetCV(1,ON_3dPoint(nonlinear?0:.5,.01,0));brep.DestroyRuntimeCache();bool rejected=false;
        try{nativeTrimParameterAt(trim,*edge,*brep.m_S[0],ON_Interval(101,149),17,1e-7);}catch(const ExchangeError&){rejected=true;}
        require(rejected,"Wrong interior trim must fail despite matching endpoints");++checks;
        curve->SetCV(1,ON_3dPoint(2,0,0));curve->SetCV(2,ON_3dPoint(0,0,0));brep.DestroyRuntimeCache();rejected=false;
        try{nativeTrimParameterAt(trim,*edge,*brep.m_S[0],ON_Interval(101,149),17,1e-7);}catch(const ExchangeError& e){rejected=std::string(e.what()).find("Ambiguous")!=std::string::npos;}
        require(rejected,"Two exact physical parameter branches must reject ambiguity");++checks;
    }
    for(bool arcParameter:{false,true})for(bool reversed:{false,true}){
        auto wire=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),2)).Edge()).Wire();
        auto face=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),wire,true).Face();auto brep=exportBrep(face,1e-7);require(brep->IsValid()&&brep->m_T.Count()==1,"Independent disk with genuine closed trim and edge");
        auto& trim=brep->m_T[0];auto edge=trim.Edge();require(edge&&edge->IsClosed()&&trim.IsClosed(),"Closed trim seam control");auto edgeDomain=edge->Domain();
        if(arcParameter){auto arc=new ON_ArcCurve(ON_Circle(ON_xy_plane,2));require(arc->ChangeDimension(2)&&arc->SetDomain(trim.Domain()[0],trim.Domain()[1]),"Independent angular native UV arc domain");delete brep->m_C2[trim.m_c2i];brep->m_C2[trim.m_c2i]=arc;trim.SetProxyCurve(arc);ON_NurbsCurve form;require(trim.GetNurbForm(form)==2&&brep->IsValid(),"Genuine type2 angular/rational parameter mismatch with valid native trim topology");}
        if(reversed){require(edge->Reverse()&&edge->SetDomain(edgeDomain),"Closed edge reversal preserves separate native parameter domain");}
        const auto immutable=brep->DataCRC(0);
        for(int i=0;i<=16;++i){auto mapping=nativeTrimParameterAt(trim,*edge,*trim.Face()->SurfaceOf(),trim.Domain(),edgeDomain.ParameterAt(i/16.),1e-7);require(mapping.deviation<=1e-7,"Closed trim seam and interior match native physical geometry");
            if(i==0||i==16)require(std::abs(mapping.parameter-trim.Domain()[trim.m_bRev3d?(i==0?1:0):(i==0?0:1)])<1e-10,"Topological seam endpoint determines parameter side despite identical physical endpoints");++checks;}
        require(brep->DataCRC(0)==immutable,"Closed seam mapping leaves original native topology unchanged");++checks;
    }
    std::cout<<"Native nonlinear trim mapping PASS "<<checks<<" checks\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
