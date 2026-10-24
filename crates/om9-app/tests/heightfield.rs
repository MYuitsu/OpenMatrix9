#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

#[test]
fn heightfield_builds_a_surface_from_an_image() {
    let path = std::env::temp_dir().join("om9-heightfield-test.png");
    let mut image = image::RgbaImage::new(64, 64);
    for (x, _, pixel) in image.enumerate_pixels_mut() {
        let value = x as u8 * 4; *pixel = image::Rgba([value, value, value, 255]);
    }
    image.save(&path).unwrap();
    let mut c = Controller::new(Document::new("heightfield"), Limits::default()).unwrap();
    c.submit("Heightfield").unwrap();
    for expected in ["File=", "Height=", "Size="] {
        assert!(c
            .heightfield_option_tokens()
            .iter()
            .any(|(_, command)| *command == expected));
    }
    c.submit(&format!("File={}", path.display())).unwrap();
    c.submit("Height=10").unwrap();
    c.submit("Size=20").unwrap();
    c.submit("").unwrap();
    let surface = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Heightfield")
        .unwrap();
    let Geometry::Surface(surface) = &surface.geometry else {
        panic!("surface");
    };
    let z_max = surface.poles.iter().map(|point| point[2]).fold(f64::MIN, f64::max);
    assert!((z_max - 10.0).abs() < 0.5, "{z_max}");
    let _ = std::fs::remove_file(path);
}
