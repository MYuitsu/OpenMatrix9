#![cfg(feature = "occt-brep")]
use om9_app::{display::ViewportDetail, ui::MatrixApp};
use om9_core::{Error, budget::{Usage,memory::MemoryClass}, geometry::Geometry, model::Document,
    transaction::CapturedBrepSource};

fn fixture(detail: ViewportDetail, count: usize) -> MatrixApp {
    let mut app=MatrixApp::new(Document::with_template_tolerance("Boolean request capture")).unwrap();
    app.controller.request_viewport_detail(detail).unwrap();
    let pairs=count/2;
    let columns=(pairs as f64).sqrt().ceil() as usize;
    let rows=pairs.div_ceil(columns);
    for index in 0..count {
        let pair=index/2;
        let x=(pair%columns) as f64*20.-(columns-1) as f64*10.+(index%2) as f64*5.;
        let y=(pair/columns) as f64*16.-(rows-1) as f64*8.;
        app.controller.solid_sphere_points([x,y,0.],5.).unwrap();
    }
    app
}

#[test]
fn batch_capture_preserves_requested_order_and_refuses_atomically_without_losing_redo() {
    let mut app=fixture(ViewportDetail::Light,4);
    app.controller.solid_sphere_points([100.,0.,0.],1.).unwrap();
    app.controller.undo().unwrap();
    let mut ids=app.controller.document().entities.keys().copied().collect::<Vec<_>>();ids.reverse();
    let before=app.controller.document().clone();
    let coordinator=app.controller.coordinator();
    let undo=coordinator.undo_len();let redo=coordinator.redo_len();assert_eq!(redo,1);
    let bytes=Usage::array::<CapturedBrepSource>(ids.len()).unwrap().bytes;
    let mut scratch=coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,bytes,None,Default::default()).unwrap();
    let mut captures=Vec::new();captures.try_reserve_exact(ids.len()).unwrap();
    scratch.set_allocated_bytes(Usage::array::<CapturedBrepSource>(captures.capacity()).unwrap().bytes).unwrap();
    let mut invalid=ids.clone();invalid[2]=om9_core::identity::ObjectId::new();
    assert!(coordinator.capture_brep_sources_into(before.id,&invalid,&mut captures).is_err());
    assert!(captures.is_empty());
    let pressure=coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,Default::default()).unwrap();
    assert!(matches!(coordinator.capture_brep_sources_into(before.id,&ids,&mut captures),Err(Error::Budget(_))));
    assert!(captures.is_empty());drop(pressure);
    coordinator.capture_brep_sources_into(before.id,&ids,&mut captures).unwrap();
    for (id,capture) in ids.iter().zip(&captures) {
        let Geometry::Brep(source)=&before.entities[id].geometry else {panic!("BRep")};
        assert_eq!(capture.source_identity(),source.identity());
        assert_eq!(capture.source(),source.source());
        assert!(capture.belongs_to(coordinator));
    }
    assert_eq!(coordinator.undo_len(),undo);assert_eq!(coordinator.redo_len(),redo);
    assert_eq!(app.controller.document(),&before);
    drop(captures);drop(scratch);
    app.controller.redo().unwrap();assert_eq!(app.controller.document().entities.len(),5);
}

#[test]
#[ignore = "diagnostic request-only timing; no completed Boolean/FPS claim"]
fn profile_real_sphere_request_then_cancel() {
    let count=std::env::var("OM9_REQUEST_PROFILE_OPERANDS").ok().and_then(|s|s.parse().ok()).unwrap_or(128usize);
    assert!(count>=4 && count<=128 && count%2==0);
    for detail in [ViewportDetail::High,ViewportDetail::Light] {
        let mut app=fixture(detail,count);
        let ids=app.controller.document().entities.keys().copied().collect::<Vec<_>>();
        app.controller.select_objects(&ids).unwrap();
        let before=app.controller.document().clone();
        let undo=app.controller.coordinator().undo_len();
        app.controller.submit("BooleanUnion").unwrap();
        app.controller.submit("DeleteInput=Yes").unwrap();
        let start=std::time::Instant::now();app.controller.submit("").unwrap();
        eprintln!("request_probe profile={detail:?} operands={count} elapsed_ms={:.3} capture={:?}",
            start.elapsed().as_secs_f64()*1000.,app.controller.boolean_union_request_capture());
        app.controller.cancel();
        assert_eq!(app.controller.document(),&before);
        assert_eq!(app.controller.coordinator().undo_len(),undo);
    }
}
