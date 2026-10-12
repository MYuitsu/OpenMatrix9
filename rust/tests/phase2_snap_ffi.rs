use openmatrix9_rust::phase2_ffi::*;
#[test]
fn snap_ffi_copies_rows_and_bounds_caller_output_and_drops_children() {
    unsafe {
        let index = om9_phase2_snap_create();
        let key = Om9ViewKey {
            document: 1,
            view: 2,
            generation: 3,
            width: 1000,
            height: 800,
            camera: [0.; 16],
        };
        assert!(om9_phase2_snap_begin(index, &key));
        let mut row = Om9SnapRow {
            object: 8,
            bounds: [0., 0., 10., 10.],
        };
        assert!(om9_phase2_snap_add(index, &row));
        row.object = 99;
        assert!(om9_phase2_snap_finish(index));
        let cursor = [0., 0.];
        let limits = Om9SnapLimits {
            objects: 64,
            per_object: 2048,
            total: 8192,
        };
        let query = om9_phase2_query_create(index, cursor.as_ptr(), 8., 2, &limits);
        assert_ne!(query, 0);
        let mut output = [777u64; 2];
        assert_eq!(om9_phase2_query_objects(query, output.as_mut_ptr(), 0), 1);
        assert_eq!(output, [777; 2]);
        assert_eq!(om9_phase2_query_objects(query, output.as_mut_ptr(), 1), 1);
        assert_eq!(output, [8, 777]);
        assert_eq!(om9_phase2_query_budget(query, 8, 2), 2048);
        assert!(!om9_phase2_query_cached(query, 8, 2));
        let point = Om9SnapPoint {
            world: [1., 2., 3.],
            screen: [1., 0., 0.5],
        };
        assert!(om9_phase2_query_consume(query, 8, 2, &point, 1, 1, 1));
        let mut result = Om9SnapResult::default();
        assert!(om9_phase2_query_finish(query, &mut result));
        assert_eq!(result.point, [1., 2., 3.]);
        assert_eq!(result.picked, 1);
        assert_eq!(row.object, 99);
        om9_phase2_snap_drop(index);
        assert_eq!(om9_phase2_query_budget(query, 8, 2), 0);
        assert!(!om9_phase2_query_finish(query, &mut result));
        om9_phase2_query_drop(query);
    }
}
#[test]
fn partial_build_and_malformed_ffi_never_certify_a_point() {
    unsafe {
        let h = om9_phase2_snap_create();
        let key = Om9ViewKey {
            document: 1,
            view: 2,
            generation: 3,
            width: 100,
            height: 100,
            camera: [0.; 16],
        };
        assert!(om9_phase2_snap_begin(h, &key));
        assert!(!om9_phase2_snap_add(h, std::ptr::null()));
        om9_phase2_snap_abort(h);
        assert!(!om9_phase2_snap_matches(h, &key));
        let limits = Om9SnapLimits {
            objects: 64,
            per_object: 2048,
            total: 8192,
        };
        assert_eq!(
            om9_phase2_query_create(h, std::ptr::null(), 8., 2, &limits),
            0
        );
        om9_phase2_snap_drop(h);
    }
}
