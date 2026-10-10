#include "Phase3RustBridge.h"
#include <iostream>
#include <stdexcept>
#include <string>
static void check(bool value,const char* message){if(!value)throw std::runtime_error(message);}
int main(){try{
    static_assert(sizeof(Om9Phase3Facts)==12);static_assert(alignof(Om9Phase3Facts)==4);
    static_assert(sizeof(Om9Phase3Snapshot)==4*sizeof(void*));
    Om9Phase3Facts inputs[2]={{5,1,1,0,0,6},{5,1,1,0,0,6}};
    check(om9_phase3_capability(4,inputs,2,true)==0,"ABI closed solid Difference allowed");
    inputs[1].kind=4;inputs[1].closed=0;
    check(om9_phase3_capability(4,inputs,2,true)==4,"ABI open-shell Boolean rejected");
    check(om9_phase3_capability(4,nullptr,2,true)!=0,"ABI null arrays rejected");
    std::string doc="OwnedDocument",name="Profile.Edge1",signature="geometry-world-placement";
    Om9Phase3Snapshot snapshot{reinterpret_cast<const std::uint8_t*>(name.data()),name.size(),reinterpret_cast<const std::uint8_t*>(signature.data()),signature.size()};
    auto handle=om9_phase3_request_begin(reinterpret_cast<const std::uint8_t*>(doc.data()),doc.size(),&snapshot,1);
    check(handle!=0,"ABI request created");
    check(om9_phase3_request_validate(handle,reinterpret_cast<const std::uint8_t*>(doc.data()),doc.size(),&snapshot,1,true)==0,"ABI owned snapshots match");
    signature[0]='X';
    check(om9_phase3_request_validate(handle,reinterpret_cast<const std::uint8_t*>(doc.data()),doc.size(),&snapshot,1,true)==7,"ABI changed native data stale");
    signature[0]='g';om9_phase3_request_cancel(handle);
    check(om9_phase3_request_validate(handle,reinterpret_cast<const std::uint8_t*>(doc.data()),doc.size(),&snapshot,1,true)==9,"ABI cancelled request rejected");
    std::cout<<"Phase3 Rust ABI: 7 assertions PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
