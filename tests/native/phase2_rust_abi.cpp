#include "Phase2Rust.h"
#include <array>
#include <cmath>
#include <iostream>
#include <limits>
static_assert(sizeof(void*)==8);
static_assert(sizeof(Om9BasisInput)==96&&offsetof(Om9BasisInput,poles)==16);
static_assert(sizeof(Om9BasisOutput)==96&&offsetof(Om9BasisOutput,last)==88);
static_assert(sizeof(Om9WitnessInput)==40&&sizeof(Om9WireInput)==40);
static_assert(sizeof(Om9ViewKey)==160&&sizeof(Om9SnapRow)==40);
static_assert(sizeof(Om9SnapLimits)==24&&sizeof(Om9SnapPoint)==48&&sizeof(Om9SnapResult)==72);
void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
int main(){try{
    std::array<double,9> poles{};std::array<double,3> weights{1,0.7,1};std::array<double,2> knots{0,1},mults{3,3};
    Om9BasisInput basis{2,0,poles.data(),poles.size(),weights.data(),weights.size(),knots.data(),knots.size(),mults.data(),mults.size(),0,1};
    require(om9_phase2_basis_validate(&basis),"C++ basis layout does not match Rust");basis.degree=2.5;require(!om9_phase2_basis_validate(&basis),"Fractional degree was truncated");basis.degree=2;
    const std::string signature="native-ABI";Om9WitnessInput witness{1,2,3,reinterpret_cast<const std::uint8_t*>(signature.data()),signature.size()};const auto h=om9_phase2_session_create(&witness,&basis);require(h!=0,"Cannot create Rust session");
    poles[0]=std::numeric_limits<double>::quiet_NaN();require(om9_phase2_session_check(h,&witness),"Caller mutation invalidated owned Rust copy");require(!om9_phase2_session_replace(h,&basis),"Invalid draft accepted");
    Om9BasisOutput out{};require(!om9_phase2_session_copy(h,&out)&&out.poles_len==9,"Required output dimensions not reported");std::array<double,9> copied{};std::array<double,3> w{};std::array<double,2> k{},m{};out.poles=copied.data();out.weights=w.data();out.knots=k.data();out.multiplicities=m.data();require(om9_phase2_session_copy(h,&out)&&copied[0]==0&&w[1]==0.7,"C++ could not copy Rust-owned numeric output");++witness.generation;require(!om9_phase2_session_check(h,&witness),"Stale request accepted");om9_phase2_session_drop(h);require(!om9_phase2_session_check(h,&witness),"Dropped handle resurrected");
    const auto index=om9_phase2_snap_create();Om9ViewKey key{1,2,3,100,100,{}};require(om9_phase2_snap_begin(index,&key),"C++ view layout does not match Rust");Om9SnapRow row{7,{0,0,10,10}};require(om9_phase2_snap_add(index,&row)&&om9_phase2_snap_finish(index),"Cannot publish numeric index");
    double cursor[2]={};Om9SnapLimits limits{64,2048,8192};const auto query=om9_phase2_query_create(index,cursor,8,2,&limits);require(query!=0&&om9_phase2_query_budget(query,7,2)==2048,"Rust budget ABI mismatch");Om9SnapPoint point{{2,3,4},{0,0,0.5}};require(om9_phase2_query_consume(query,7,2,&point,1,1,1),"C++ point layout mismatch");Om9SnapResult result{};require(om9_phase2_query_finish(query,&result)&&result.complete&&result.picked&&result.point[2]==4,"C++ result layout/ranking mismatch");om9_phase2_query_drop(query);om9_phase2_snap_drop(index);
    std::cout<<"Rust phase2 native ABI ownership/layout/stale/numeric contracts PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
