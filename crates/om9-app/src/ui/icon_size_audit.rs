// SPDX-License-Identifier: LGPL-2.1-or-later
//! Actual egui-frame measurements, not OS per-monitor DPI acceptance.
use super::*;

fn frame(app: &mut MatrixApp, ctx: &egui::Context, physical: Vec2, ppp: f32) {
    ctx.set_pixels_per_point(ppp);
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, physical / ppp)),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn palette_rects(app: &MatrixApp, section: SectionId, next: SectionId) -> Vec<Rect> {
    let top = app.section_rect(section.key()).unwrap().bottom();
    let bottom = app.section_rect(next.key()).unwrap().top();
    app.rail_icon_rects
        .iter()
        .copied()
        .filter(|r| r.top() >= top && r.bottom() <= bottom)
        .collect()
}

fn check_slots(rects: &[Rect], sidebar: Rect, label: &str) {
    for (i, rect) in rects.iter().enumerate() {
        assert!(
            rect.is_finite() && (rect.width() - rect.height()).abs() < 0.1,
            "{label}: {rect:?}"
        );
        assert!(rect.width() >= 18., "{label}: too small: {rect:?}");
        assert!(
            rect.left() >= sidebar.left() - 0.5 && rect.right() <= sidebar.right() + 0.5,
            "{label}: horizontal overflow: {rect:?} / {sidebar:?}"
        );
        for other in &rects[i + 1..] {
            let overlap = rect.intersect(*other);
            assert!(
                overlap.width() <= 0.5 || overlap.height() <= 0.5,
                "{label}: overlapping icon slots: {rect:?} / {other:?}"
            );
        }
    }
}

#[test]
fn icons_resize_in_the_same_window_and_restore_without_changing_slots() {
    for ppp in [1., 1.25, 1.5] {
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("live resize")).unwrap();
        let measure = |app: &mut MatrixApp, size: Vec2| {
            // Keep this audit about icon scaling rather than whatever rail width
            // a previous fullscreen frame persisted.
            app.dock.set_size(
                crate::dock_layout::PanelId::LeftRail,
                crate::dock_layout::LEFT_RAIL_DEFAULT,
            );
            for _ in 0..4 {
                frame(app, &ctx, size * ppp, ppp);
            }
            let mut edges = vec![app.toolbar_icon_edge()];
            for (section, next, count) in [
                (SectionId::MainMenu, SectionId::Display, 20),
                (SectionId::Display, SectionId::Snaps, 11),
                (SectionId::Snaps, SectionId::InfoSettings, 16),
                (SectionId::InfoSettings, SectionId::Layers, 22),
            ] {
                let rects = palette_rects(app, section, next);
                assert_eq!(rects.len(), count);
                check_slots(&rects, app.sidebar_rect.unwrap(), "live resize");
                edges.push(rects[0].width());
            }
            assert!(
                (app.left_rail_outer_px - 300.).abs() <= 4.,
                "ppp={ppp} rail={}",
                app.left_rail_outer_px
            );
            edges
        };
        let small = measure(&mut app, Vec2::new(1280., 720.));
        let restored = measure(&mut app, Vec2::new(1280., 900.));
        let full = measure(&mut app, Vec2::new(1920., 1080.));
        for i in 0..full.len() {
            assert!(
                full[i] > restored[i] + 0.5,
                "group {i} at {ppp}: restored={restored:?}, full={full:?}"
            );
            assert!(
                restored[i] > small[i] + 0.5,
                "group {i} at {ppp}: small={small:?}, restored={restored:?}"
            );
        }
        let again = measure(&mut app, Vec2::new(1280., 720.));
        assert!(
            small
                .iter()
                .zip(&again)
                .all(|(before, after)| (before - after).abs() <= 0.5),
            "resize must be reversible at {ppp}: small={small:?} again={again:?}"
        );
        println!(
            "OM9_LIVE_ICON_RESIZE ppp={ppp} small={small:?} restored={restored:?} fullscreen={full:?} returned={again:?}"
        );
        assert!(app.controller.document().entities.is_empty());
    }
}

#[test]
fn icon_size_audit_all_categories_at_common_resolutions_and_scales() {
    let categories = [
        "Curve",
        "Surface",
        "Solid",
        "Transform",
        "SubD",
        "Art",
        "Builder",
        "Tools",
        "Gems",
        "Setting",
        "Cutters",
        "Render",
    ];
    let tabs = [
        "Standard",
        "CPlanes",
        "Set View",
        "Display",
        "Select",
        "Viewport Layout",
        "Visibility",
        "Transform",
        "Curve Tools",
        "Surface Tools",
        "Solid Tools",
        "Mesh Tools",
        "Render Tools",
        "Drafting",
        "SubD",
        "New in V5",
        "Emboss",
        "Sculpt",
        "Tools",
        "Settings",
    ];
    // Literal 100% anchors independently pin the current rendered slot sizes.
    for (width, height, main_edge, toolbar_edge) in [
        (1280., 720., 24., 17.25),
        (1366., 768., 24., 17.25),
        (1920., 1080., 34., 24.),
        (2560., 1440., 34., 24.),
    ] {
        for ppp in [1., 1.25, 1.5] {
            let label = format!("{width}x{height}@{ppp}");
            let ctx = egui::Context::default();
            let mut app = MatrixApp::new(Document::new("icon size audit")).unwrap();
            let physical = Vec2::new(width, height);
            for _ in 0..4 {
                frame(&mut app, &ctx, physical, ppp);
            }
            assert!(
                app.region_error.is_none(),
                "{label}: {:?}",
                app.region_error
            );
            assert!(
                (app.left_rail_outer_px - 300.).abs() <= 4.,
                "{label} rail={}",
                app.left_rail_outer_px
            );
            if ppp == 1. {
                assert_eq!(app.left_icon_edge, main_edge, "{label}");
                assert_eq!(app.toolbar_icon_edge, toolbar_edge, "{label}");
            }
            let sidebar = app.sidebar_rect.unwrap();
            for (section, next, count, top_count) in [
                (SectionId::Display, SectionId::Snaps, 11, 11),
                (SectionId::Snaps, SectionId::InfoSettings, 16, 9),
                (SectionId::InfoSettings, SectionId::Layers, 22, 11),
            ] {
                let rects = palette_rects(&app, section, next);
                assert_eq!(rects.len(), count, "{label}/{section:?}");
                check_slots(&rects, sidebar, &label);
                assert!(
                    rects[..top_count]
                        .iter()
                        .all(|r| (r.top() - rects[0].top()).abs() < 0.1)
                );
                if top_count < count {
                    assert!(
                        rects[top_count..]
                            .iter()
                            .all(|r| (r.top() - rects[top_count].top()).abs() < 0.1)
                    );
                }
            }
            let fixed = palette_rects(&app, SectionId::Display, SectionId::Snaps)[0].width();
            let main = palette_rects(&app, SectionId::MainMenu, SectionId::Display)[0].width();
            let history =
                palette_rects(&app, SectionId::IconHistory, SectionId::MainMenu)[0].width();
            let toolbar = app
                .file_toolbar_rect(crate::file_lifecycle::FileKind::New)
                .unwrap()
                .width();
            println!(
                "OM9_ICON_SIZE {}",
                serde_json::json!({
                    "resolution": [width, height], "scale": ppp,
                    "client_points": [width / ppp, height / ppp],
                    "dock_slot_pt": app.left_icon_edge, "dock_art_pt": app.left_icon_edge - 3.,
                    "main_slot_pt": main, "main_art_pt": main - 3.,
                    "history_slot_pt": history,
                    "toolbar_slot_pt": toolbar, "toolbar_art_pt": toolbar - 2.25,
                    "fixed_slot_pt": fixed, "fixed_art_pt": fixed - 3.,
                    "dock_slot_device_px": app.left_icon_edge * ppp,
                    "main_slot_device_px": main * ppp,
                    "toolbar_slot_device_px": app.toolbar_icon_edge * ppp,
                    "fixed_slot_device_px": fixed * ppp,
                })
            );
            let mut checked_pages = 0;
            for (index, name) in categories.iter().enumerate() {
                app.main_category = index;
                let total = crate::menu_catalog::category(name).unwrap().len();
                for page in 0..total.div_ceil(20).max(1) {
                    ctx.memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new(("main-menu-page", *name)), page)
                    });
                    for _ in 0..2 {
                        frame(&mut app, &ctx, physical, ppp);
                    }
                    let rects = palette_rects(&app, SectionId::MainMenu, SectionId::Display);
                    assert_eq!(
                        rects.len(),
                        (total - page * 20).min(20),
                        "{label}/{name}/page{page}"
                    );
                    check_slots(
                        &app.rail_icon_rects,
                        app.sidebar_rect.unwrap(),
                        &format!("{label}/{name}/page{page}"),
                    );
                    assert!(
                        rects.iter().all(|r| (r.width() - main).abs() < 0.1),
                        "{label}/{name}"
                    );
                    checked_pages += 1;
                }
                assert_eq!(
                    app.main_category_rects.len(),
                    categories.len(),
                    "{label}/{name}"
                );
            }
            println!(
                "OM9_ICON_PAGES {label} categories={} pages={checked_pages} tabs={}",
                categories.len(),
                tabs.len()
            );
            for (index, tab) in tabs.iter().enumerate() {
                app.toolbar_tab = index;
                for _ in 0..2 {
                    frame(&mut app, &ctx, physical, ppp);
                }
                assert!(
                    app.toolbar_rect.unwrap().height() >= app.toolbar_icon_edge,
                    "{label}/{tab}"
                );
            }
            assert_eq!(app.view_rects.len(), 4, "{label}");
            assert!(app.controller.document().entities.is_empty());
            assert!(!app.controller.is_running());
        }
    }
}
