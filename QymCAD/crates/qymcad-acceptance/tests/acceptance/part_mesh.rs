//! A MESH BROUGHT IN AND TURNED INTO A BODY: the block is written out as a mesh of triangles, brought back in, and
//! recognised - either as the exact surfaces it was made of, or as the polyhedron the file holds.
use qymcad::{Key, Session};
use qymcad_acceptance::build;
use qymcad_acceptance::probe;

/// The bodies of the part.
fn bodies(s: &mut Session) -> Vec<qymcad::Solid> {
    s.document().bodies.into_iter().filter(|b| !b.consumed && !b.sheet).collect()
}

/// Whether a face of the body can be taken by a click at `p` - which a mesh has nothing to answer with and a body
/// made of one has.
fn a_face_can_be_clicked(s: &mut Session, p: [f64; 3]) -> bool {
    qymcad_acceptance::refusal(|| {
        let _ = s.face_at(p);
    })
    .is_empty()
}

/// Press the word `key` names in the bar of options at the top.
fn bar_word(s: &mut Session, key: &str) {
    let word = s.word(key);
    s.press_word_near(&word, qymcad::pos2(0.0, 0.0));
}

/// A NEW PROJECT HOLDING THE MESH OF A BLOCK: the block is built, written out as `name`.stl at the standard quality,
/// and brought back into a project of its own.
fn a_mesh_of_a_block(name: &str) -> Session {
    a_mesh_of(name, build::block)
}

/// A NEW PROJECT HOLDING THE MESH OF WHAT `make` BUILDS, written out as `name`.stl at the standard quality.
fn a_mesh_of(name: &str, make: fn(&mut Session)) -> Session {
    let mut s = Session::start();
    make(&mut s);
    let path = format!("{}/{name}.stl", std::env::temp_dir().display());
    let (file, export) = (s.word("menu-file"), s.word("file-export"));
    s.menu(&[&file, &export, "STL\u{2026}"]);
    let quality = s.word("stl-standard");
    s.press_word(&quality);
    s.answer_file(&path);
    assert!(std::path::Path::new(&path).exists(), "the mesh was not written out to {path}: the program says {:?}", s.status());
    let (file, new) = (s.word("menu-file"), s.word("file-new"));
    s.menu(&[&file, &new]);
    let dont_save = s.word("nav-dont-save");
    if let Some(button) = s.find(&dont_save, qymcad::pos2(0.0, 0.0)) {
        s.click(button.center());
    }
    build::into_the_first_part(&mut s);
    let (file, import) = (s.word("menu-file"), s.word("file-import"));
    s.menu(&[&file, &import]);
    s.answer_file(&path);
    let go = s.word("import-scale-import");
    s.press_word_near(&go, qymcad::pos2(1200.0, 700.0)); // the placement and the units are taken as they come
    s
}

probe! {
    /// A MESH IS BROUGHT IN AS IT IS: the triangles of the file, holding what the block held, and the program says
    /// how many came.
    fn a_mesh_is_brought_in_and_the_program_says_what_came() {
        let mut s = a_mesh_of_a_block("brought-in");
        let said = s.status();
        assert!(said.contains("12"), "nothing says how many triangles came in: the program says {said:?}");
        let mesh = bodies(&mut s);
        assert!(mesh.len() == 1, "one mesh was brought in, and the part holds {:?}", mesh.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
        assert!((mesh[0].volume - 12000.0).abs() < 1.0, "the mesh of the 40 by 30 by 10 block holds 12000, and it holds {}", mesh[0].volume);
        // a mesh brought in is a mesh: no exact body stands behind it. Its flat groups of triangles are taken as faces
        // by a click - a sketch goes on the face of an STL - but it has no edges of an exact body until it is recognised
        assert!(mesh[0].edges.is_none(), "the mesh brought in holds an exact body of {:?} edges - it was not taken as it is", mesh[0].edges);
    }
}

probe! {
    /// RECOGNISED AS EXACT SURFACES the mesh becomes the body it came from: six faces and twelve edges.
    fn a_mesh_is_recognised_as_the_exact_surfaces() {
        let mut s = a_mesh_of_a_block("exact");
        let hint = s.word("tb-recognise-hint");
        s.press_hint(&hint);
        bar_word(&mut s, "cmd-recognise-exact");
        let at = s.in_space([20.0, 15.0, 10.0]); // a mesh has no faces to name: the click is on the mesh itself
        s.click(at);
        assert!(s.status() != s.word("msg-recognise"), "the click on the mesh was not taken: the program still says {:?}", s.status());
        s.key(Key::Enter);
        let now = bodies(&mut s);
        assert!(now.len() == 1, "recognising leaves one body, and the part holds {:?}", now.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
        assert!((now[0].volume - 12000.0).abs() < 1.0, "the body made of the mesh holds 12000, and it holds {}", now[0].volume);
        assert!(now[0].faces == 6, "the block recognised exactly has six faces, and it has {}", now[0].faces);
        assert!(now[0].edges.is_some() && a_face_can_be_clicked(&mut s, [20.0, 15.0, 10.0]), "the body made of the mesh has no exact body behind it - it is still a mesh");
    }
}

probe! {
    /// RECOGNISED AS IT IS the mesh becomes a body of the triangles it holds: the same material, more faces than six.
    fn a_mesh_is_recognised_as_the_polyhedron_it_is() {
        let mut s = a_mesh_of_a_block("as-is");
        let hint = s.word("tb-recognise-hint");
        s.press_hint(&hint);
        bar_word(&mut s, "cmd-recognise-asis");
        let at = s.in_space([20.0, 15.0, 10.0]);
        s.click(at);
        assert!(s.status() != s.word("msg-recognise"), "the click on the mesh was not taken: the program still says {:?}", s.status());
        s.key(Key::Enter);
        let now = bodies(&mut s);
        assert!(now.len() == 1, "recognising leaves one body, and the part holds {:?}", now.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
        assert!((now[0].volume - 12000.0).abs() < 1.0, "the polyhedron of the mesh holds 12000, and it holds {}", now[0].volume);
        assert!(now[0].edges.is_some() && a_face_can_be_clicked(&mut s, [20.0, 15.0, 10.0]), "the polyhedron made of the mesh has no exact body behind it - it is still a mesh");
        let node = s.document().features.last().cloned().unwrap_or_else(|| panic!("the timeline is empty"));
        assert!(node.error.is_none(), "the node that made the polyhedron is red: {node:?}");
    }
}

probe! {
    /// A MESH OF A KNOWN SHAPE IS RECOGNISED AS THAT SHAPE: the cylinder 20 across and 10 tall, written out as
    /// triangles and brought back, recognised exactly - one body of three faces, a side and two caps, holding
    /// pi x 10^2 x 10 = 3141.6 mm^3 as the cylinder did, not the smaller volume of the facets.
    fn a_mesh_of_a_cylinder_is_recognised_as_the_cylinder() {
        let mut s = a_mesh_of("cylinder", qymcad_acceptance::bodies::cylinder);
        let hint = s.word("tb-recognise-hint");
        s.press_hint(&hint);
        bar_word(&mut s, "cmd-recognise-exact");
        let at = s.in_space([20.0, 15.0, 10.0]);
        s.click(at);
        s.key(Key::Enter);
        let now = bodies(&mut s);
        assert!(now.len() == 1, "recognising leaves one body, and the part holds {:?}", now.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
        let exact = std::f64::consts::PI * 100.0 * 10.0;
        assert!(
            now[0].faces == 3 && (now[0].volume - exact).abs() < exact * 0.002,
            "the cylinder recognised exactly has 3 faces and holds {exact:.1}; the body has {} faces and holds {:.1}",
            now[0].faces,
            now[0].volume
        );
    }
}

probe! {
    /// A MESH OF A ROUNDED BLOCK IS RECOGNISED AS THAT BLOCK: the block with its top front edge rounded 2, written out
    /// and brought back, recognised exactly - seven faces, the round one among them, holding
    /// 12000 - 40 x 2^2 x (1 - pi/4) = 11965.7 mm^3 as the rounded block did.
    fn a_mesh_of_a_rounded_block_is_recognised_as_that_block() {
        let mut s = a_mesh_of("rounded", qymcad_acceptance::bodies::rounded);
        let hint = s.word("tb-recognise-hint");
        s.press_hint(&hint);
        bar_word(&mut s, "cmd-recognise-exact");
        let at = s.in_space([20.0, 15.0, 10.0]);
        s.click(at);
        s.key(Key::Enter);
        let now = bodies(&mut s);
        assert!(now.len() == 1, "recognising leaves one body, and the part holds {:?}", now.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
        let exact = 12000.0 - 40.0 * 4.0 * (1.0 - std::f64::consts::FRAC_PI_4);
        assert!(
            now[0].faces == 7 && (now[0].volume - exact).abs() < 1.0,
            "the rounded block recognised exactly has 7 faces and holds {exact:.1}; the body has {} faces and holds {:.1}",
            now[0].faces,
            now[0].volume
        );
    }
}
