// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;

fn frame(
    app: &mut MatrixApp,
    ctx: &egui::Context,
    size: Vec2,
    ppp: f32,
    pointer: Option<egui::Pos2>,
) -> egui::FullOutput {
    ctx.set_pixels_per_point(ppp);
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
            events: pointer.map(egui::Event::PointerMoved).into_iter().collect(),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn check(app: &MatrixApp, output: &egui::FullOutput) {
    let outer = app.sidebar_rect.unwrap();
    let content = app.sidebar_content_rect.unwrap();
    let bar = app.sidebar_scrollbar_rect.unwrap();
    assert!((bar.width() - 4.).abs() < 0.1);
    assert!(
        (outer.right() - content.right() - 6.).abs() < 0.5,
        "{outer:?} / {content:?}"
    );
    assert!(bar.left() >= content.right() + 1.5 && bar.right() <= outer.right() + 0.5);
    assert!(
        app.sidebar_content_size.unwrap().x <= content.width() + 0.5,
        "widgets exceed scroll viewport: {:?} / {content:?}",
        app.sidebar_content_size
    );
    for icon in &app.rail_icon_rects {
        assert!(
            icon.right() <= content.right() + 0.5 && icon.right() < bar.left(),
            "{icon:?} / {bar:?}"
        );
    }
    assert!(
        output.shapes.iter().any(|shape| matches!(&shape.shape,
        egui::epaint::Shape::Rect(r) if (r.rect.left() - bar.left()).abs() < 0.5
            && (r.rect.right() - bar.right()).abs() < 0.5
            && (r.rect.top() - bar.top()).abs() < 0.5
            && (r.rect.bottom() - bar.bottom()).abs() < 0.5)),
        "scrollbar must be painted even without overflow"
    );
}

#[test]
fn slim_sidebar_scrollbar_never_overlays_buttons_at_supported_widths_and_dpi() {
    for width in [240., 360., 460.] {
        for ppp in [1., 1.25, 1.5] {
            let ctx = egui::Context::default();
            let mut app = MatrixApp::new(Document::new("scroll gutter")).unwrap();
            app.dock.set_size(PanelId::LeftRail, width);
            let size = Vec2::new(1280., 720.);
            for _ in 0..3 {
                frame(&mut app, &ctx, size, ppp, None);
            }
            let output = frame(&mut app, &ctx, size, ppp, None);
            check(&app, &output);
            let bar = app.sidebar_scrollbar_rect.unwrap();
            let hovered = frame(&mut app, &ctx, size, ppp, Some(bar.center()));
            check(&app, &hovered);
            assert_eq!(
                app.sidebar_scrollbar_rect.unwrap(),
                bar,
                "hover must not widen the bar"
            );
            println!(
                "OM9_SCROLL_GUTTER width={width} ppp={ppp} content={:?} bar={bar:?}",
                app.sidebar_content_rect
            );
        }
    }
}

#[test]
fn slim_sidebar_scrollbar_stays_visible_when_all_content_fits() {
    let ctx = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("short contents")).unwrap();
    for section in app.dock.left_sections() {
        app.dock.set_section_collapsed(section, true);
    }
    let size = Vec2::new(1280., 900.);
    for _ in 0..3 {
        frame(&mut app, &ctx, size, 1., None);
    }
    let output = frame(&mut app, &ctx, size, 1., None);
    assert!(app.sidebar_content_size.unwrap().y < app.sidebar_content_rect.unwrap().height());
    check(&app, &output);
}

#[test]
fn sidebar_wheel_scroll_is_retained_on_following_frames() {
    let ctx = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("scroll persistence")).unwrap();
    let size = Vec2::new(1280., 720.);
    for _ in 0..3 {
        frame(&mut app, &ctx, size, 1., None);
    }
    let pointer = app.sidebar_content_rect.unwrap().center();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
            events: vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: Vec2::new(0., -180.),
                    phase: egui::TouchPhase::Move,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    for _ in 0..4 {
        frame(&mut app, &ctx, size, 1., Some(pointer));
    }
    let offset = app.sidebar_scroll;
    assert!(offset > 10., "wheel must move contents: {offset}");
    for _ in 0..4 {
        frame(&mut app, &ctx, size, 1., None);
    }
    assert!(
        app.sidebar_scroll >= offset - 0.5,
        "idle frames must not reset scrolling"
    );
}
