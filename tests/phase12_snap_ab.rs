// Same-process alternating A/B; unchanged public Rust APIs and owned fixtures.
#[path = "H:/FreeCAD-src/build/om9-dev/rust/src/phase2_snap.rs"]
mod baseline;
#[path = "H:/FreeCAD-src/build/om9-perf-dev/rust/src/phase2_snap.rs"]
mod candidate;
use std::time::Instant;
use std::hint::black_box;

macro_rules! measure {
    ($module:ident, $dense:expr) => {{
        let mut index = $module::SnapIndex::default();
        let limits = $module::Limits { objects: 64, per_object: 2048, total: 8192 };
        let dense = $dense;
        let objects = if dense { 64 } else { 4096 };
        let points = if dense { 128 } else { 32 };
        index.begin_build($module::ViewKey { document: 1, view: 2, generation: 3, width: 8192, height: 8192, camera: [0.; 16] }).unwrap();
        for i in 0..objects {
            let (x,y) = if dense { (16.,16.) } else { ((i%64) as f64*64.+16., (i/64) as f64*64.+16.) };
            index.add_row($module::Row { object: i as u64+1, bounds: [x,y,x+1.,y+1.] }).unwrap();
        }
        index.finish_build().unwrap();
        for i in 0..if dense { 1 } else { objects } {
            let (x,y) = if dense { (16.,16.) } else { ((i%64) as f64*64.+16., (i/64) as f64*64.+16.) };
            let mut query = index.start_query([x,y],8.,2,limits).unwrap();
            for token in query.objects().to_vec() {
                let data = (0..points).map(|p| ([token as f64,p as f64,0.], [x+p as f64/100.,y,0.5])).collect::<Vec<_>>();
                query.consume(token,2,&data,true,points).unwrap();
            }
            assert!(index.publish_query(query).unwrap().complete);
        }
        let mut timings = Vec::new();
        for sample in 0..1050 {
            let started = Instant::now();
            let mut query = index.start_query([16.,16.],8.,2,limits).unwrap();
            for token in query.objects().to_vec() { assert!(query.cached(token,2).unwrap()); }
            let result = index.publish_query(query).unwrap();
            let elapsed = started.elapsed().as_secs_f64()*1e6;
            assert!(result.complete && result.picked);
            assert_eq!(result.points.len(),if dense {8192} else {32});
            assert_eq!(result.point, [if dense {64.} else {1.},0.,0.]);
            black_box(result);
            if sample >= 50 { timings.push(elapsed); }
        }
        timings.sort_by(f64::total_cmp);
        (timings[500],timings[949])
    }}
}

fn main() {
    for dense in [false,true] {
        for pair in 0..6 {
            // Reverse order to reduce drift/order bias. Each run has 50 warmups.
            let (base,next) = if pair%2==0 {(measure!(baseline,dense),measure!(candidate,dense))}
                              else {let next=measure!(candidate,dense);(measure!(baseline,dense),next)};
            println!("{{\"scenario\":\"{}\",\"pair\":{},\"queries_per_mode\":1000,\"baseline_median_us\":{},\"baseline_p95_us\":{},\"candidate_median_us\":{},\"candidate_p95_us\":{}}}",if dense {"dense8192"} else {"cache4096"},pair,base.0,base.1,next.0,next.1);
        }
    }
}
