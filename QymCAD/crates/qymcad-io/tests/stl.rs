//! A test of the STL import.

use qymcad_io::{import_stl, import_stl_named};
use std::io::Write;

/// A folder for the files, under `target`: nothing goes to `/tmp`, which lives in memory.
fn file(name: &str, bytes: &[u8]) -> String {
    let dir = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/stl-probe"));
    std::fs::create_dir_all(&dir).expect("a folder for the check");
    let p = dir.join(name);
    std::fs::write(&p, bytes).expect("written");
    p.to_string_lossy().into_owned()
}

/// A binary STL written by hand: an 80-byte header beginning with `header`, then every triangle - its nine
/// coordinates - with its attribute word.
fn binary(header: &[u8], tris: &[([f32; 9], u16)]) -> Vec<u8> {
    let mut b = header.to_vec();
    b.resize(80, b' ');
    b.extend((tris.len() as u32).to_le_bytes());
    for (p, attr) in tris {
        b.extend([0u8; 12]);
        for c in p {
            b.extend(c.to_le_bytes());
        }
        b.extend(attr.to_le_bytes());
    }
    b
}

const TRI: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];

/// An ASCII STL: a 10×10 square at z = 2, of two faces.
const FIXTURE: &str = "\
solid s
facet normal 0 0 1
 outer loop
  vertex 0 0 2
  vertex 10 0 2
  vertex 10 10 2
 endloop
endfacet
facet normal 0 0 1
 outer loop
  vertex 0 0 2
  vertex 10 10 2
  vertex 0 10 2
 endloop
endfacet
endsolid s
";

#[test]
fn imports_ascii_stl_square() {
    let path = std::env::temp_dir().join("qymcad_test_square.stl");
    std::fs::File::create(&path).unwrap().write_all(FIXTURE.as_bytes()).unwrap();

    let mesh = import_stl(path.to_str().unwrap()).expect("import ok");
    assert_eq!(mesh.tris.len(), 2, "two faces");
    assert!(mesh.verts.len() >= 3);

    let b = mesh.bounds().expect("bounds");
    assert!((b.min.z - 2.0).abs() < 1e-6 && (b.max.z - 2.0).abs() < 1e-6, "the plane z = 2");
    assert!((b.max.x - 10.0).abs() < 1e-6 && (b.max.y - 10.0).abs() < 1e-6);
}

/// SEVERAL SOLIDS IN ONE TEXT FILE come as pieces under their names - a CAD writes an assembly so - and all of their
/// triangles come in, not the first solid's alone.
#[test]
fn several_solids_come_as_pieces_under_their_names() {
    let pin = "solid pin\nfacet normal 0 0 1\n outer loop\n  vertex 20 0 0\n  vertex 21 0 0\n  vertex 20 1 0\n endloop\nendfacet\nendsolid pin\n";
    let p = file("two-solids.stl", format!("{}{pin}", FIXTURE.replace("solid s", "solid plate").replace("endsolid s", "endsolid plate")).as_bytes());
    assert_eq!(import_stl(&p).expect("reads").tris.len(), 3, "the second solid's triangle did not come in");
    let back = import_stl_named(&p).expect("reads");
    assert_eq!(back.iter().map(|b| (b.name.as_str(), b.mesh.tris.len())).collect::<Vec<_>>(), [("plate", 2), ("pin", 1)], "the solids do not come as pieces under their names");
}

/// A BINARY FILE WHOSE HEADER BEGINS WITH "solid" reads as the binary file it is, whatever its bytes: several CADs
/// write the part's name there, and a file told by its first line was taken for text wherever that line read as text -
/// ten triangles, and the count's first byte is a newline.
#[test]
fn a_binary_file_whose_header_begins_with_solid_reads() {
    let p = file("solid-header.stl", &binary(b"solid part, binary", &[(TRI, 0); 10]));
    let m = import_stl(&p).expect("the binary file reads");
    assert_eq!((m.tris.len(), m.verts.len()), (10, 3));
}

/// A BINARY FILE BRINGS ITS COLOURS, by either rule it can be written in. With "COLOR=" in the header (Materialise)
/// the object takes the colour after it, and a triangle with bit 15 clear its own, red in the low bits; without it
/// (VisCAM, SolidView) a triangle with bit 15 set has a colour of its own, blue in the low bits.
#[test]
fn a_binary_file_brings_its_colours() {
    let mut head = b"COLOR=".to_vec();
    head.extend([204, 26, 26, 255]);
    let p = file("magics.stl", &binary(&head, &[(TRI, 0x8000), (TRI, 31 << 5)]));
    let back = import_stl_named(&p).expect("reads");
    assert_eq!(back[0].color, Some([204, 26, 26]), "the object's colour in the header was not read");
    assert_eq!(back[0].tri_colors, [[204, 26, 26], [0, 255, 0]], "the triangles' colours were not read by the header's rule");
    let p = file("viscam.stl", &binary(b"", &[(TRI, 0x8000 | (31 << 10)), (TRI, 0x8000 | (31 << 10)), (TRI, 0x8000 | 31)]));
    let back = import_stl_named(&p).expect("reads");
    assert_eq!(back[0].color, Some([255, 0, 0]), "the piece is not in the colour most of its triangles have");
    assert_eq!(back[0].tri_colors, [[255, 0, 0], [255, 0, 0], [0, 0, 255]], "the triangles' colours were not read by the other rule");
}

/// THE OWNER'S STL READS AS ITS BYTES SAY, where it is at hand (`QYM_CONDOR_STL`): every triangle the header counts,
/// and a vertex for every distinct point, counted here from the bytes themselves.
#[test]
#[ignore = "the owner's file"]
fn the_owners_stl_reads_as_its_bytes_say() {
    let Ok(path) = std::env::var("QYM_CONDOR_STL") else { return };
    let b = std::fs::read(&path).expect("the file reads");
    let n = u32::from_le_bytes([b[80], b[81], b[82], b[83]]) as usize;
    let mut points = std::collections::HashSet::new();
    for k in 0..n {
        for i in 0..3 {
            let at = 84 + 50 * k + 12 + 12 * i;
            points.insert([0, 1, 2].map(|c| (f32::from_le_bytes(b[at + 4 * c..at + 4 * c + 4].try_into().expect("four bytes")) + 0.0).to_bits()));
        }
    }
    let m = import_stl(&path).expect("reads");
    println!("{} triangles, {} vertices; the bytes say {n} and {}", m.tris.len(), m.verts.len(), points.len());
    assert_eq!((m.tris.len(), m.verts.len()), (n, points.len()));
}

/// A LONE SOLID IS NAMED AFTER ITS FILE: the name on its `solid` line is most often the writer's stamp ("OpenSCAD_Model",
/// "Exported from Blender-...") and the file's own name says more; a solid's name is kept where it tells solids apart.
#[test]
fn a_lone_solid_is_named_after_its_file() {
    let back = import_stl_named(&file("lone-solid.stl", FIXTURE.as_bytes())).expect("reads");
    assert_eq!(back.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), [""], "a lone solid came in under its stamp, not its file's name");
}
