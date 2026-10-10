#pragma once
#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <string>
// Borrowed input/output buffers must remain valid and aligned during the call.
// Rust handles own numeric copies only; native CAD/Qt/document lifetime stays here.
extern "C" {
bool om9_modeling_snap_allowed(unsigned,bool,bool,unsigned);
struct Om9BasisInput {double degree;std::uint32_t periodic;const double* poles;std::size_t poles_len;const double* weights;std::size_t weights_len;const double* knots;std::size_t knots_len;const double* multiplicities;std::size_t multiplicities_len;double first,last;};
struct Om9BasisOutput {double degree;std::uint32_t periodic;double* poles;std::size_t poles_len;double* weights;std::size_t weights_len;double* knots;std::size_t knots_len;double* multiplicities;std::size_t multiplicities_len;double first,last;};
struct Om9WitnessInput {std::uint64_t document,object,generation;const std::uint8_t* signature;std::size_t signature_len;};
struct Om9WireInput {const std::uint64_t* vertex_degrees;std::size_t vertex_degrees_len;const std::uint64_t* ordered_edges;std::size_t ordered_edges_len;std::size_t expected_edges;};
struct Om9RebuildOptions {std::size_t degree,point_count;std::uint32_t delete_input;};
struct Om9RebuildInput {Om9WitnessInput witness;std::uint32_t has_dependents;};
struct Om9ViewKey {std::uint64_t document,view,generation;std::uint32_t width,height;double camera[16];};
struct Om9SnapRow {std::uint64_t object;double bounds[4];};
struct Om9SnapLimits {std::size_t objects,per_object,total;};
struct Om9SnapPoint {double world[3],screen[3];};
struct Om9SnapResult {std::uint32_t complete,picked;double point[3];std::size_t nearby_objects,visited_objects,visited_topology,generated,max_per_object;};
std::uint64_t om9_phase2_snap_create();
void om9_phase2_snap_drop(std::uint64_t);
bool om9_phase2_snap_begin(std::uint64_t,const Om9ViewKey*);
bool om9_phase2_snap_add(std::uint64_t,const Om9SnapRow*);
bool om9_phase2_snap_finish(std::uint64_t);
void om9_phase2_snap_abort(std::uint64_t);
void om9_phase2_snap_invalidate(std::uint64_t);
bool om9_phase2_snap_matches(std::uint64_t,const Om9ViewKey*);
std::uint64_t om9_phase2_query_create(std::uint64_t,const double*,double,std::uint32_t,const Om9SnapLimits*);
std::size_t om9_phase2_query_objects(std::uint64_t,std::uint64_t*,std::size_t);
std::size_t om9_phase2_query_budget(std::uint64_t,std::uint64_t,std::uint32_t);
bool om9_phase2_query_cached(std::uint64_t,std::uint64_t,std::uint32_t);
bool om9_phase2_query_consume(std::uint64_t,std::uint64_t,std::uint32_t,const Om9SnapPoint*,std::size_t,std::uint32_t,std::size_t);
void om9_phase2_query_abort(std::uint64_t);
bool om9_phase2_query_finish(std::uint64_t,Om9SnapResult*);
std::size_t om9_phase2_query_points(std::uint64_t,double*,std::size_t);
void om9_phase2_query_drop(std::uint64_t);
bool om9_phase2_basis_validate(const Om9BasisInput*);
bool om9_phase2_wire_validate(const Om9WireInput*);
bool om9_phase2_join_options(std::size_t,std::size_t);
bool om9_phase2_join_selection(const std::uint64_t*,std::size_t,std::size_t);
std::uint64_t om9_phase2_session_create(const Om9WitnessInput*,const Om9BasisInput*);
bool om9_phase2_session_replace(std::uint64_t,const Om9BasisInput*);
bool om9_phase2_session_check(std::uint64_t,const Om9WitnessInput*);
bool om9_phase2_session_copy(std::uint64_t,Om9BasisOutput*);
void om9_phase2_session_drop(std::uint64_t);
std::uint64_t om9_phase2_rebuild_create(const Om9RebuildInput*,std::size_t,const Om9RebuildOptions*);
bool om9_phase2_rebuild_replace(std::uint64_t,const Om9RebuildOptions*);
bool om9_phase2_rebuild_check(std::uint64_t,const Om9RebuildInput*,std::size_t);
void om9_phase2_rebuild_drop(std::uint64_t);
bool om9_phase2_rebuild_input_count(std::size_t);
bool om9_phase2_rebuild_get(std::uint64_t,Om9RebuildOptions*);
std::size_t om9_phase2_error(std::uint64_t,char*,std::size_t);
}
namespace OpenMatrix9Gui {
inline std::string phase2Error(std::uint64_t handle=0) {const auto n=om9_phase2_error(handle,nullptr,0);if(n>4096)return "Rust operation failed";std::string value(n,'\0');om9_phase2_error(handle,value.data(),n);if(!value.empty())value.pop_back();return value;}
inline void phase2Require(bool success,std::uint64_t handle=0) {if(!success)throw std::runtime_error(phase2Error(handle));}
}
