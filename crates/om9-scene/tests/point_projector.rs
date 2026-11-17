use om9_scene::camera::{Camera, Viewport};

#[test]
fn prepared_projection_preserves_every_point_and_clipping_bit_for_bit() {
    let viewport=Viewport { origin:[17.5,-23.],size:[649.,410.] };
    for mut camera in [
        Camera::orthographic([0.,0.,100.],[0.;3],[0.,1.,0.],40.).unwrap(),
        Camera::perspective([0.,0.,100.],[0.;3],[0.,1.,0.],0.8).unwrap(),
        Camera::perspective([51.,-29.,83.],[3.,4.,-7.],[0.2,1.,0.3],1.2).unwrap(),
    ] {
        for _ in 0..3 {
            let projector=camera.projector(viewport).unwrap();
            for i in -100..=100 {
                let point=[i as f64*0.31,(i*i%47) as f64-23.,i as f64*5.];
                let old=camera.project(point,viewport).unwrap();
                let new=projector.project(point).unwrap();
                assert_eq!(old.map(|p|[p.screen[0].to_bits(),p.screen[1].to_bits(),p.depth.to_bits()]),
                    new.map(|p|[p.screen[0].to_bits(),p.screen[1].to_bits(),p.depth.to_bits()]));
            }
            camera.pan([3.,-2.]).unwrap();
        }
    }
    let mut camera=Camera::orthographic([0.,0.,100.],[0.;3],[0.,1.,0.],40.).unwrap();
    camera.near_mm=1.;camera.far_mm=200.;
    let projector=camera.projector(viewport).unwrap();
    assert_eq!(projector.project([10.,0.,0.]).unwrap().unwrap().screen,[444.5,182.]);
    assert_eq!(projector.project([0.,0.,99.]).unwrap().unwrap().depth,1.);
    assert_eq!(projector.project([0.,0.,-100.]).unwrap().unwrap().depth,200.);
    assert!(projector.project([0.,0.,99.1]).unwrap().is_none());
    assert!(projector.project([0.,0.,-100.1]).unwrap().is_none());
}

#[test]
fn prepared_projection_keeps_invalid_camera_viewport_and_point_checks() {
    let viewport=Viewport { origin:[0.;2],size:[649.,410.] };
    let mut camera=Camera::orthographic([0.,0.,100.],[0.;3],[0.,1.,0.],40.).unwrap();
    let projector=camera.projector(viewport).unwrap();
    assert!(projector.project([f64::NAN,0.,0.]).is_err());
    assert!(projector.project([0.,f64::INFINITY,0.]).is_err());
    assert!(camera.projector(Viewport { size:[0.,410.],..viewport }).is_err());
    camera.up=[0.;3];assert!(camera.projector(viewport).is_err());
    camera.up=[0.,1.,0.];camera.near_mm=0.;assert!(camera.projector(viewport).is_err());
}
