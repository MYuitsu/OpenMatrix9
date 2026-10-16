//! THE SOLVE OF A SKETCH TAKES THE CORES THE KERNEL IS GIVEN: one setting of the program, "cores for computation",
//! handed over where a rebuild starts (`set_parallel`), rules both. One core is a single thread for both.

#[test]
fn the_setting_of_the_cores_reaches_the_solve_of_a_sketch() {
    qymcad_kernel::set_parallel(false, 1);
    assert_eq!(qymcad_core::solver::threads(), 1, "a single core for the kernel left the solve of a sketch on more");
    qymcad_kernel::set_parallel(true, 3);
    assert_eq!(qymcad_core::solver::threads(), 3, "three cores for the kernel are not three for the solve");
    qymcad_kernel::set_parallel(true, 0);
    let all_but_one = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).saturating_sub(1).max(1);
    assert_eq!(qymcad_core::solver::threads(), all_but_one, "zero is all the cores but one, for the solve as for the kernel");
}
