// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{Error, mesh_triangulation::triangulate_face};

#[test]
fn concave_faces_preserve_area_and_winding_and_refuse_invalid_input() {
    let points = [[0.,0.,0.],[3.,0.,0.],[3.,1.,0.],[1.,1.,0.],[1.,3.,0.],[0.,3.,0.]];
    for (face, expected) in [(vec![0,1,2,3,4,5], 5.), (vec![5,4,3,2,1,0], -5.)] {
        let triangles = triangulate_face(&points, &face, 1e-6, &mut 0, 1000).unwrap();
        assert_eq!(triangles.len(), 4);
        let area: f64 = triangles.iter().map(|t| {
            let [a,b,c] = t.map(|i| points[i as usize]);
            ((b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]))/2.
        }).sum();
        assert!((area-expected).abs() < 1e-12);
    }
    let bow = [[0.,0.,0.],[2.,2.,0.],[0.,2.,0.],[2.,0.,0.]];
    assert!(triangulate_face(&bow, &[0,1,2,3], 1e-6, &mut 0, 1000).is_err());
    let mut nonplanar = points;
    nonplanar[3][2] = 0.1;
    assert!(triangulate_face(&nonplanar, &[0,1,2,3,4,5], 1e-6, &mut 0, 1000).is_err());
    assert!(matches!(triangulate_face(&points, &[0,1,2,3,4,5], 1e-6, &mut 0, 1), Err(Error::Budget(_))));
}

#[test]
fn cancellable_triangulation_matches_and_refuses_canceled_work() {
    use om9_core::{mesh_triangulation::triangulate_face_cancellable, transaction::Cancellation};
    let points = [[0.,0.,0.],[3.,0.,0.],[3.,1.,0.],[1.,1.,0.],[1.,3.,0.],[0.,3.,0.]];
    let face = [0,1,2,3,4,5];
    let cancel = Cancellation::default();
    let mut normal_work = 0;
    let mut cancellable_work = 0;
    let normal = triangulate_face(&points, &face, 1e-6, &mut normal_work, 1000).unwrap();
    let actual = triangulate_face_cancellable(&points, &face, 1e-6, &mut cancellable_work, 1000, &cancel).unwrap();
    assert_eq!((actual, cancellable_work), (normal, normal_work));
    cancel.cancel();
    assert!(matches!(triangulate_face_cancellable(&points, &face, 1e-6, &mut 0, 1000, &cancel), Err(Error::Canceled)));

    let points: Vec<_> = (0..1400).map(|i| {
        let angle = std::f64::consts::TAU * i as f64 / 1400.;
        [angle.cos(), angle.sin(), 0.]
    }).collect();
    let face: Vec<_> = (0..1400).collect();
    let cancel = Cancellation::default();
    let worker_cancel = cancel.clone();
    let (started, ready) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        started.send(()).unwrap();
        triangulate_face_cancellable(&points, &face, 1e-8, &mut 0, 4_000_000, &worker_cancel)
    });
    ready.recv().unwrap();
    cancel.cancel();
    assert!(matches!(worker.join().unwrap(), Err(Error::Canceled)));
}
