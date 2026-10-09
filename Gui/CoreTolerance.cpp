// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreTolerance.h"
#include <sstream>
#include <stdexcept>
#include <vector>
extern "C" unsigned om9_tolerance_resolve(const char*,std::uint32_t,double,bool,double,OpenMatrix9Gui::ToleranceValues*);
extern "C" std::size_t om9_tolerance_report(const OpenMatrix9Gui::ToleranceValues*,char*,std::size_t);
namespace OpenMatrix9Gui {
ToleranceSnapshot resolveTolerance(const std::string& context,unsigned operation,
    const std::optional<double>& overrideMm,double worldCoordinateBound) {
    if(context.find('\0')!=std::string::npos)throw std::runtime_error("Invalid tolerance context");
    ToleranceSnapshot result;result.context=context;
    const auto code=om9_tolerance_resolve(context.c_str(),operation,overrideMm.value_or(0),
        overrideMm.has_value(),worldCoordinateBound,&result.values);
    if(code){
        const char* reasons[]={"","Unsupported tolerance operation","Invalid tolerance context",
            "Effective Join tolerance must be finite and within 1e-9..1e6 mm",
            "Invalid world-coordinate bound","ToleranceBelowResolution","Invalid tolerance ABI output","Tolerance resolver failed"};
        std::ostringstream error;error<<(code<8?reasons[code]:"Unknown tolerance error")
            <<" (world-coordinate bound "<<worldCoordinateBound<<" mm)";
        throw std::runtime_error(error.str());
    }
    const auto count=om9_tolerance_report(&result.values,nullptr,0);
    if(!count||count>4096)throw std::runtime_error("Cannot serialize effective tolerance report");
    std::vector<char> buffer(count);
    if(om9_tolerance_report(&result.values,buffer.data(),buffer.size())!=count)
        throw std::runtime_error("Cannot serialize effective tolerance report");
    result.report=buffer.data();return result;
}
}
