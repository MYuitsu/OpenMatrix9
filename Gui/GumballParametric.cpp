// SPDX-License-Identifier: LGPL-2.1-or-later
// Necessary native bridge: FreeCAD owns property metadata and expression ASTs;
// safe Rust owns affine representation admission. This planner never mutates.
#include "GumballParametric.h"
#include <App/DocumentObject.h>
#include <App/Expression.h>
#include <App/ObjectIdentifier.h>
#include <App/PropertyGeo.h>
#include <App/PropertyStandard.h>
#include <App/PropertyUnits.h>
#include <charconv>
#include <cmath>
#include <limits>
#include <set>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
std::string number(double value){
    char buffer[64];
    const auto result=std::to_chars(buffer,buffer+sizeof(buffer),value,
        std::chars_format::general,std::numeric_limits<double>::max_digits10);
    if(result.ec!=std::errc())throw std::runtime_error("Cannot encode native Gumball dimension factor");
    return std::string(buffer,result.ptr);
}
bool writable(const App::Property& property,const App::DocumentObject& object){
    return property.getContainer()==&object&&!property.isReadOnly()
        &&!(property.getType()&App::Prop_ReadOnly)&&!property.testStatus(App::Property::Output);
}
bool bound(App::DocumentObject& object,const char* key){
    return bool(object.getExpression(App::ObjectIdentifier(&object,std::string(key))).expression);
}
}
std::optional<GumballParametricScale> planGumballParametricScale(
    App::DocumentObject& object,const double* affine,const double* world){
    const std::string type(object.getTypeId().getName());
    unsigned kind=0;
    if(type=="Part::Box"||type=="Part::Plane"||type=="Part::Ellipsoid"||type=="Part::Wedge")kind=1;
    else if(type=="Part::Sphere"||type=="Part::Torus")kind=2;
    else if(type=="Part::Cylinder"||type=="Part::Cone"||type=="Part::Prism")kind=4;
    else return std::nullopt;
    auto* placement=object.getPropertyByName<App::PropertyPlacement>("Placement");
    if(!placement||!writable(*placement,object))return std::nullopt;
    // A direct Placement setter cannot override attachment or a bound component.
    // The associative result path preserves that existing governance instead.
    for(const auto& [path,expression]:object.ExpressionEngine.getExpressions())
        if(expression&&path.getPropertyName()=="Placement")return std::nullopt;
    if(auto* map=object.getPropertyByName<App::PropertyEnumeration>("MapMode"))
        if(map->getValue()!=0)return std::nullopt;
    GumballParametricScale plan;
    const double oldScale[3]={1,1,1};
    if(om9_gumball_owner_scale(affine,world,oldScale,kind,&plan.owner))return std::nullopt;
    const auto* factor=plan.owner.factors;
    // Oblique Cylinder/Prism need angle composition under unequal axial/radial
    // scale. Route to affine output rather than preserving an incorrect slope.
    if((type=="Part::Cylinder"||type=="Part::Prism")&&std::abs(factor[0]-factor[2])>1e-12){
        for(const char* key:{"FirstAngle","SecondAngle"}){
            auto* angle=object.getPropertyByName<App::PropertyFloat>(key);
            if(!angle||std::abs(angle->getValue())>1e-12||bound(object,key))return std::nullopt;
        }
    }
    auto edit=[&](const char* key,double multiplier)->bool{
        if(multiplier==1)return true;
        auto* property=object.getPropertyByName<App::PropertyFloat>(key);
        if(!property||!writable(*property,object))return false;
        GumballParametricEdit change;
        change.key=key;change.factor=multiplier;change.before=property->getValue();
        change.value=change.before*multiplier;
        if(!std::isfinite(change.value)||std::abs(change.value)>1e12)return false;
        const auto info=object.getExpression(App::ObjectIdentifier(&object,std::string(key)));
        if(info.expression){
            change.beforeExpression=info.expression->toString(true);
            change.expressionText="("+change.beforeExpression+") * "+number(multiplier);
            change.expression=std::shared_ptr<App::Expression>(App::Expression::parse(&object,change.expressionText));
        }
        plan.edits.push_back(std::move(change));return true;
    };
    bool valid=true;
    if(type=="Part::Box")valid=edit("Length",factor[0])&&edit("Width",factor[1])&&edit("Height",factor[2]);
    else if(type=="Part::Plane")valid=edit("Length",factor[0])&&edit("Width",factor[1]);
    else if(type=="Part::Sphere")valid=edit("Radius",factor[0]);
    else if(type=="Part::Torus")valid=edit("Radius1",factor[0])&&edit("Radius2",factor[0]);
    else if(type=="Part::Cylinder")valid=edit("Radius",factor[0])&&edit("Height",factor[2]);
    else if(type=="Part::Cone")valid=edit("Radius1",factor[0])&&edit("Radius2",factor[0])&&edit("Height",factor[2]);
    else if(type=="Part::Prism")valid=edit("Circumradius",factor[0])&&edit("Height",factor[2]);
    else if(type=="Part::Ellipsoid"){
        auto* radius3=object.getPropertyByName<App::PropertyFloat>("Radius3");
        auto* radius2=object.getPropertyByName<App::PropertyFloat>("Radius2");
        if(!radius3||!radius2)return std::nullopt;
        valid=edit("Radius1",factor[2])&&edit("Radius2",factor[0]);
        if(radius3->getValue()>=1e-7)valid=valid&&edit("Radius3",factor[1]);
        else if(factor[0]!=factor[1]){
            // Native Radius3=0 means Y follows X (Radius2). Preserve that
            // dependency as an explicit ratio after independent axis scaling.
            if(!writable(*radius3,object)||bound(object,"Radius3"))return std::nullopt;
            GumballParametricEdit change;
            change.key="Radius3";change.before=radius3->getValue();
            change.value=radius2->getValue()*factor[1];
            change.expressionText="Radius2 * "+number(factor[1]/factor[0]);
            change.expression=std::shared_ptr<App::Expression>(App::Expression::parse(&object,change.expressionText));
            plan.edits.push_back(std::move(change));
        }
    }else if(type=="Part::Wedge"){
        for(const char* key:{"Xmin","Xmax","X2min","X2max"})valid=valid&&edit(key,factor[0]);
        for(const char* key:{"Ymin","Ymax"})valid=valid&&edit(key,factor[1]);
        for(const char* key:{"Zmin","Zmax","Z2min","Z2max"})valid=valid&&edit(key,factor[2]);
    }
    if(!valid)return std::nullopt;
    std::set<std::string> touched;
    for(const auto& change:plan.edits)touched.insert(change.key);
    // Multiplying a bound dimension which reads another changed dimension
    // compounds that upstream factor, even during uniform scaling. Keep the
    // untouched native definition associative instead of guessing compensation.
    for(const auto& [path,expression]:object.ExpressionEngine.getExpressions()){
        if(!expression)continue;
        const auto dependencies=expression->getDeps(App::Expression::DepAll);
        const auto own=dependencies.find(&object);
        if(own==dependencies.end())continue;
        for(const auto& [key,identifiers]:own->second)
            if(touched.contains(key))return std::nullopt;
    }
    return std::optional<GumballParametricScale>(std::move(plan));
}
}
