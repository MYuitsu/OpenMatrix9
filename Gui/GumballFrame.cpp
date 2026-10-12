// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballFrame.h"
#include "GumballGeometry.h"
#include <QJsonArray>
#include <algorithm>
#include <limits>
#include <Standard_Failure.hxx>
#include <Base/Exception.h>
namespace OpenMatrix9Gui {
GumballFrameObservations captureGumballFrame(const GumballInputs& inputs,Gui::View3DInventor*,unsigned alignment){
    GumballFrameObservations result;
    try{
        unsigned effectiveAlignment=alignment;
        const bool components=inputs.ambiguousComponents||std::any_of(inputs.roots.begin(),inputs.roots.end(),[](const auto& root){return !root.subelement.empty();});
        if(om9_gumball_selection_alignment(alignment,inputs.roots.size(),unsigned(components),&effectiveAlignment))throw std::runtime_error("Invalid Gumball selection frame request");
        const auto rows=gumballInputFacts(inputs).value("items").toArray();double low[3]={INFINITY,INFINITY,INFINITY},high[3]={-INFINITY,-INFINITY,-INFINITY};
        const char* minimum[]={"xmin","ymin","zmin"};const char* size[]={"bounds_x","bounds_y","bounds_z"};
        for(const auto& value:rows){auto row=value.toObject();for(unsigned i=0;i<3;++i){if(!row.contains(minimum[i]))throw std::runtime_error("Object frame has no finite geometry bounds");low[i]=std::min(low[i],row[minimum[i]].toDouble());high[i]=std::max(high[i],row[minimum[i]].toDouble()+row[size[i]].toDouble());}}
        double origin[3],x[3]={1,0,0},normal[3]={0,0,1};for(unsigned i=0;i<3;++i)origin[i]=low[i]+(high[i]-low[i])/2;
        if(effectiveAlignment==0){for(unsigned i=0;i<3;++i){x[i]=inputs.coordinates.frame[3+i];normal[i]=inputs.coordinates.frame[9+i];}}
        else if(effectiveAlignment==1){const auto facts=gumballObjectFrameFacts(inputs);auto tangent=facts["x"].toArray(),n=facts["normal"].toArray(),p=facts["origin"].toArray();for(unsigned i=0;i<3;++i){x[i]=tangent[i].toDouble();normal[i]=n[i].toDouble();if(p.size()==3)origin[i]=p[i].toDouble();}}
        else if(effectiveAlignment!=2)throw std::runtime_error("Invalid gumball alignment");
        if(om9_gumball_frame_from_directions(origin,x,normal,result.frame.data()))throw std::runtime_error("Object tangent/normal does not resolve a gumball frame");
    }catch(const std::exception& e){result.unresolved=QString::fromUtf8(e.what());}catch(const Base::Exception& e){result.unresolved=QString::fromUtf8(e.what());}catch(const Standard_Failure& e){result.unresolved=QString::fromUtf8(e.GetMessageString());}
    return result;
}
}
