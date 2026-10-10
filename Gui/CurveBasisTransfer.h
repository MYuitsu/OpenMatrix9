#pragma once
#include "Phase2Rust.h"
#include <QJsonObject>
#include <QJsonArray>
#include <vector>
#include <limits>
namespace OpenMatrix9Gui {
// Syntax decoding only. Numeric ranges, integers, basis relations and budgets
// have a single implementation in Rust; kernel constructibility stays native.
struct CurveBasisTransfer {
    std::vector<double> poles,weights,knots,mults;
    double degree=0,first=0,last=0;std::uint32_t periodic=0;bool malformed=false;
    explicit CurveBasisTransfer(const QJsonObject& input) {
        degree=input.value("degree").toDouble(NAN);first=input.value("first").toDouble(NAN);last=input.value("last").toDouble(NAN);
        periodic=input.value("periodic").isBool()?unsigned(input.value("periodic").toBool()):2;
        for(const auto& p:input.value("poles").toArray()){const auto xyz=p.toArray();if(xyz.size()!=3)malformed=true;for(const auto& n:xyz)poles.push_back(n.toDouble(NAN));}
        for(const auto& n:input.value("weights").toArray())weights.push_back(n.toDouble(NAN));
        for(const auto& n:input.value("knots").toArray())knots.push_back(n.toDouble(NAN));
        for(const auto& n:input.value("multiplicities").toArray())mults.push_back(n.toDouble(NAN));
    }
    explicit CurveBasisTransfer(std::uint64_t session) {
        Om9BasisOutput o{};om9_phase2_session_copy(session,&o);
        if(o.poles_len==0)phase2Require(false,session);
        poles.resize(o.poles_len);weights.resize(o.weights_len);knots.resize(o.knots_len);mults.resize(o.multiplicities_len);
        o.poles=poles.data();o.weights=weights.data();o.knots=knots.data();o.multiplicities=mults.data();phase2Require(om9_phase2_session_copy(session,&o),session);
        degree=o.degree;periodic=o.periodic;first=o.first;last=o.last;
    }
    Om9BasisInput input() const {return {degree,periodic,poles.data(),malformed?std::numeric_limits<std::size_t>::max():poles.size(),weights.data(),weights.size(),knots.data(),knots.size(),mults.data(),mults.size(),first,last};}
    QJsonObject json()const {QJsonArray p,w,k,m;for(std::size_t i=0;i+2<poles.size();i+=3)p.append(QJsonArray{poles[i],poles[i+1],poles[i+2]});for(auto v:weights)w.append(v);for(auto v:knots)k.append(v);for(auto v:mults)m.append(v);return {{"degree",degree},{"periodic",bool(periodic)},{"poles",p},{"weights",w},{"knots",k},{"multiplicities",m},{"first",first},{"last",last}};}
};
}
