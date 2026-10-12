// Native adapter only: FreeCAD 1.1 exposes Boost signals connections.
#pragma once
#include <boost/signals2/connection.hpp>
namespace fastsignals {
using connection = boost::signals2::connection;
using scoped_connection = boost::signals2::scoped_connection;
}
