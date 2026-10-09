// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CircleTangent.h"
#include "RustBridge.h"
#include <Base/Interpreter.h>
#include <algorithm>
#include <cmath>
#include <stdexcept>
namespace {
using namespace OpenMatrix9Gui;
using Ref=CurvePyRef;
struct NativeCurve {PyObject* edge;double first,last;};
struct Context {std::map<std::size_t,NativeCurve> curves;};
double number(PyObject* o,const char* key){Ref v(PyObject_GetAttrString(o,key));const double n=PyFloat_AsDouble(v.value);if(PyErr_Occurred()||!std::isfinite(n))throw std::runtime_error("Nonfinite native curve value");return n;}
SolidPoint point(PyObject* o){return {number(o,"x"),number(o,"y"),number(o,"z")};}
double dot(SolidPoint a,SolidPoint b){double n=0;for(unsigned i=0;i<3;++i)n+=a[i]*b[i];return n;}
// Synchronous ABI callback. No pointer/context is retained. Never allow native
// or Python exceptions to unwind across Rust; caller owns refs and holds GIL.
bool evaluate(void* opaque,std::size_t index,double fraction,double* out) noexcept {
    try {
        if(!opaque||!out||!std::isfinite(fraction)||fraction<0||fraction>1)return false;
        const auto& c=static_cast<Context*>(opaque)->curves.at(index);
        const double u=c.first+fraction*(c.last-c.first);
        Ref p(PyObject_CallMethod(c.edge,"valueAt","d",u)),t(PyObject_CallMethod(c.edge,"tangentAt","d",u));
        const auto xyz=point(p.value);auto tangent=point(t.value);const double length=std::sqrt(dot(tangent,tangent));
        if(!std::isfinite(length)||length<=1e-14)return false;
        for(unsigned i=0;i<3;++i){out[i]=xyz[i];out[i+3]=tangent[i]/length;}return true;
    }catch(...){if(PyErr_Occurred())PyErr_Clear();return false;}
}
}
namespace OpenMatrix9Gui {
std::array<double,7> circleSolveNative(const std::map<std::size_t,SolidCurveReference>& refs,const double* b,std::size_t count,const double* frame,int solution,bool fromFirst,bool vertical){
    Base::PyGILStateLocker lock;
    if(!b||!frame||count<2||count>3)throw std::runtime_error("Invalid native Circle tangent constraints");
    std::vector<double> evidence;Context context;double seeds[3]={};
    for(std::size_t i=0;i<count;++i){
        if(b[i*4+3]){evidence.insert(evidence.end(),b+i*4,b+i*4+3);continue;}
        const auto& ref=refs.at(i);const auto points=solidCurvePlanePoints(ref);
        for(const auto& p:points)evidence.insert(evidence.end(),p.begin(),p.end());
        const double first=number(ref.edge->value,"FirstParameter"),last=number(ref.edge->value,"LastParameter");
        if(last-first<=1e-12)throw std::runtime_error("Degenerate native Tangent edge range");
        seeds[i]=solidCurveFraction(ref,{b[i*4],b[i*4+1],b[i*4+2]});context.curves.emplace(i,NativeCurve{ref.edge->value,first,last});
    }
    double spatial[13];spatial[12]=frame[12];
    if(om9_circle_plane_frame(evidence.data(),evidence.size()/3,frame,spatial)){
        if(vertical&&std::abs(spatial[9]*frame[9]+spatial[10]*frame[10]+spatial[11]*frame[11])>1e-8)throw std::runtime_error("Tangent Vertical requires a plane perpendicular to CPlane");
        double contacts[9]={};const auto [center,radius]=nativeTangentCircle(refs,b,count,spatial,spatial[12],solution,fromFirst,contacts);
        const SolidPoint normal{spatial[9],spatial[10],spatial[11]};
        bool verified=true;
        for(std::size_t i=0;i<count;++i){
            SolidPoint p{b[i*4],b[i*4+1],b[i*4+2]},t{};
            if(!b[i*4+3]){const auto contact=solidOnCurve(refs.at(i),{contacts[i*3],contacts[i*3+1],contacts[i*3+2]});p=contact.first;t=contact.second;}
            SolidPoint delta;for(unsigned j=0;j<3;++j)delta[j]=p[j]-center[j];
            if(std::abs(dot(delta,normal))>1e-6||std::abs(std::sqrt(dot(delta,delta))-radius)>1e-6*std::max(1.,radius)||(!b[i*4+3]&&(std::abs(dot(t,normal))>1e-8||std::abs(dot(delta,t)/radius)>1e-8)))verified=false;
            if(fromFirst&&i==0){SolidPoint shift;for(unsigned j=0;j<3;++j)shift[j]=p[j]-b[j];if(std::sqrt(dot(shift,shift))>1e-6)verified=false;}
        }
        if(verified)return {center[0],center[1],center[2],spatial[9],spatial[10],spatial[11],radius};
    }
    double output[10];char error[1024]={};
    if(!om9_circle_spatial_solve(b,seeds,count,frame,fromFirst,vertical,solution,evaluate,&context,output,error,sizeof(error)))throw std::runtime_error(error[0]?error:"Spatial Tangent failed");
    double validated[7];if(!om9_circle_validate_plan(output,validated))throw std::runtime_error("Invalid spatial Tangent plan");
    const SolidPoint normal{validated[3],validated[4],validated[5]};
    for(std::size_t i=0;i<count;++i){
        double contact[6];if(!b[i*4+3]){
            if(fromFirst&&i==0&&std::abs(output[7]-seeds[0])>1e-12)throw std::runtime_error("Spatial Tangent changed FromFirstPoint");
            if(!evaluate(&context,i,output[7+i],contact))throw std::runtime_error("Spatial Tangent source changed during verification");
            const auto smooth=solidOnCurve(refs.at(i),{0,0,0},output[7+i]);for(unsigned j=0;j<3;++j){contact[j]=smooth.first[j];contact[j+3]=smooth.second[j];}
        }else std::copy(b+i*4,b+i*4+3,contact);
        SolidPoint delta;for(unsigned j=0;j<3;++j)delta[j]=contact[j]-validated[j];
        if(std::abs(dot(delta,normal))>1e-6||std::abs(std::sqrt(dot(delta,delta))-validated[6])>1e-6*std::max(1.,validated[6]))throw std::runtime_error("Spatial Tangent contact failed exact native verification");
        if(!b[i*4+3]){const SolidPoint t{contact[3],contact[4],contact[5]};if(std::abs(dot(t,normal))>1e-8||std::abs(dot(delta,t)/validated[6])>1e-8)throw std::runtime_error("Spatial Tangent direction failed exact native verification");}
    }
    return {validated[0],validated[1],validated[2],validated[3],validated[4],validated[5],validated[6]};
}
}
