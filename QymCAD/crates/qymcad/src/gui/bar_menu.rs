//! THE MENU BAR IS WALKED BY HOVER once it has been opened.
//!
//! Reported behaviour: "when I click a menu and move the mouse to the next menu, it does not expand". The
//! first menu is opened by a click; after that, the pointer passing over another title of the same bar opens
//! that one in its place, until a click outside or a chosen item closes the menu - as menu bars behave
//! everywhere else.
//!
//! egui's `MenuBar` does not do this itself: every `MenuButton` toggles its own popup on a click and knows
//! nothing of its neighbours.

/// A drop-down menu of the menu bar that hands the open menu over to itself when the pointer comes to it.
pub(crate) trait BarMenu {
    fn bar_menu_button<'a, R>(&mut self, atoms: impl egui::IntoAtoms<'a>, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<Option<R>>;
}

/// The titles of one bar: the ones drawn on the previous frame, and the ones being drawn on this one.
#[derive(Clone, Default)]
struct Titles {
    pass: u64,
    last: Vec<Title>,
    building: Vec<Title>,
}

#[derive(Clone, Copy)]
struct Title {
    id: egui::Id,
    popup: egui::Id,
    rect: egui::Rect,
}

/// THE OPEN MENU FOLLOWS THE POINTER BEFORE ANYTHING IS DRAWN. Decided while drawing a title, the hand-over
/// came a frame late: the menus to its left had already been painted, so the old one hung under the new title
/// for a frame and the new one appeared on the next. Here the titles of the previous frame are checked against
/// the pointer once, ahead of the first of them.
fn hand_over(ctx: &egui::Context, layer: egui::LayerId, titles: &[Title], handed_key: egui::Id, now: f64) {
    let Some(pos) = ctx.pointer_hover_pos() else { return };
    // a press or a release belongs to the title's own toggle; opening here too would let it close again
    if ctx.input(|i| i.pointer.any_down() || i.pointer.any_released()) {
        return;
    }
    // a window lying over the bar keeps the pointer for itself
    if ctx.layer_id_at(pos) != Some(layer) {
        return;
    }
    if !titles.iter().any(|t| egui::Popup::is_id_open(ctx, t.popup)) {
        return;
    }
    if let Some(t) = titles.iter().find(|t| t.rect.contains(pos) && !egui::Popup::is_id_open(ctx, t.popup)) {
        // `open_id` closes every other popup, so the menu is handed over rather than doubled
        egui::Popup::open_id(ctx, t.popup);
        ctx.data_mut(|d| d.insert_temp(handed_key, (t.id, now)));
    }
}

impl BarMenu for egui::Ui {
    fn bar_menu_button<'a, R>(&mut self, atoms: impl egui::IntoAtoms<'a>, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<Option<R>> {
        let ctx = self.ctx().clone();
        // THE MENUS OF THIS BAR, remembered across frames: a title drawn before the open menu must still
        // know that menu is open, and an unrelated popup elsewhere (a combo box) must not count as one.
        let key = self.id().with("bar_menus");
        let handed_key = key.with("handed_over");
        let now = ctx.input(|i| i.time);
        let fade = ctx.global_style().animation_time;
        let mut titles = ctx.data(|d| d.get_temp::<Titles>(key)).unwrap_or_default();
        let pass = ctx.cumulative_pass_nr();
        if titles.pass != pass {
            // only the titles drawn last frame count: a menu hidden since then must not be opened unseen
            titles.last = std::mem::take(&mut titles.building);
            titles.pass = pass;
            hand_over(&ctx, self.layer_id(), &titles.last, handed_key, now);
        }
        // A MENU HANDED OVER BY HOVER IS THERE AT ONCE: the fade-in belongs to the first opening. The popup
        // has no switch for it, and it starts its fade inside its own drawing, so the animation time is taken
        // away for as long as that fade would last. The title's id is not known before it is drawn; the next
        // automatic id is the one the button takes, and a wrong guess only brings the fade back.
        let handed = ctx.data(|d| d.get_temp::<(egui::Id, f64)>(handed_key));
        let instant = handed.is_some_and(|(id, at)| id == self.next_auto_id() && now - at <= 3.0 * f64::from(fade));
        if instant {
            ctx.global_style_mut(|s| s.animation_time = 0.0);
        }
        let inner = self.menu_button(atoms, add_contents);
        if instant {
            ctx.global_style_mut(|s| s.animation_time = fade);
        }
        let response = &inner.response;
        titles.building.push(Title { id: response.id, popup: egui::Popup::default_response_id(response), rect: response.rect });
        ctx.data_mut(|d| d.insert_temp(key, titles));
        inner
    }
}

#[cfg(test)]
mod tests {
    use super::BarMenu;

    const FILE: &str = "File";
    const EDIT: &str = "Edit";

    /// The refresh rates the timing is checked at: what a menu does on one screen it must do on all of them.
    const RATES: [f64; 5] = [30.0, 60.0, 120.0, 144.0, 240.0];

    /// A bar of two menus driven by real pointer input, one frame per step.
    struct Bar {
        ctx: egui::Context,
        /// The refresh rate of the screen the bar is drawn on.
        hz: f64,
        file: egui::Rect,
        edit: egui::Rect,
        file_popup: egui::Id,
        edit_popup: egui::Id,
        frames: u32,
        /// How opaque the Edit menu was painted on each frame, `None` while it was not painted at all.
        edit_painted: Vec<Option<u8>>,
        file_painted: Vec<Option<u8>>,
    }

    /// The strongest fill of a menu frame painted with its corner at `corner`: a fade-in shows up as a fill
    /// lighter than the frame's own.
    fn fill_at(shapes: &[egui::epaint::ClippedShape], corner: egui::Pos2) -> Option<u8> {
        fn walk(shape: &egui::Shape, corner: egui::Pos2, best: &mut Option<u8>) {
            match shape {
                egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, corner, best)),
                egui::Shape::Rect(r) if (r.rect.min - corner).length() < 2.0 && r.fill.a() > 0 => {
                    *best = Some(best.unwrap_or(0).max(r.fill.a()));
                }
                _ => {}
            }
        }
        let mut best = None;
        shapes.iter().for_each(|c| walk(&c.shape, corner, &mut best));
        best
    }

    impl Bar {
        fn new() -> Self {
            Self::at(60.0)
        }

        fn at(hz: f64) -> Self {
            let mut bar = Bar {
                ctx: egui::Context::default(),
                hz,
                file: egui::Rect::NOTHING,
                edit: egui::Rect::NOTHING,
                file_popup: egui::Id::NULL,
                edit_popup: egui::Id::NULL,
                frames: 0,
                edit_painted: Vec::new(),
                file_painted: Vec::new(),
            };
            bar.frame(Vec::new());
            bar
        }

        fn frame(&mut self, events: Vec<egui::Event>) {
            let screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
            let (mut file, mut edit) = (None, None);
            // the clock moves frame by frame as on a real screen of this rate, or a fade-in never gets anywhere;
            // egui's own guess of the frame time follows it, since a fade starts half a frame in
            self.frames += 1;
            let time = Some(f64::from(self.frames) / self.hz);
            let predicted_dt = (1.0 / self.hz) as f32;
            let input = egui::RawInput { screen_rect: Some(screen), time, predicted_dt, events, ..Default::default() };
            let out = self.ctx.run_ui(input, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    file = Some(ui.bar_menu_button(FILE, |ui| ui.label("new")).response);
                    edit = Some(ui.bar_menu_button(EDIT, |ui| ui.label("undo")).response);
                });
            });
            let (file, edit) = (file.expect("the File title is drawn"), edit.expect("the Edit title is drawn"));
            self.file = file.rect;
            self.edit = edit.rect;
            self.file_popup = egui::Popup::default_response_id(&file);
            self.edit_popup = egui::Popup::default_response_id(&edit);
            self.edit_painted.push(fill_at(&out.shapes, self.edit.left_bottom()));
            self.file_painted.push(fill_at(&out.shapes, self.file.left_bottom()));
        }

        /// The pointer comes to a point and stays there for a few frames.
        fn hover(&mut self, at: egui::Pos2) {
            self.frame(vec![egui::Event::PointerMoved(at)]);
            self.frame(Vec::new());
            self.frame(Vec::new());
        }

        /// Frames go by with nothing happening, for as long as the given time lasts on this screen.
        fn idle(&mut self, seconds: f64) {
            for _ in 0..(seconds * self.hz).ceil() as u32 {
                self.frame(Vec::new());
            }
        }

        fn click(&mut self, at: egui::Pos2) {
            self.hover(at);
            let button = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
            self.frame(vec![button(true)]);
            self.frame(vec![button(false)]);
            self.frame(Vec::new());
        }

        fn open(&self, popup: egui::Id) -> bool {
            egui::Popup::is_id_open(&self.ctx, popup)
        }
    }

    /// ONCE A MENU IS OPEN, THE NEXT TITLE OPENS BY HOVER ALONE, and the first one gives way to it.
    #[test]
    fn an_open_menu_moves_to_the_title_under_the_pointer() {
        let mut bar = Bar::new();
        bar.click(bar.file.center());
        assert!(bar.open(bar.file_popup), "a click on File must open it");
        bar.hover(bar.edit.center());
        assert!(bar.open(bar.edit_popup), "with File open, bringing the pointer to Edit must open Edit");
        assert!(!bar.open(bar.file_popup), "only one menu of the bar is open at a time: File must give way to Edit");
        bar.hover(bar.file.center());
        assert!(bar.open(bar.file_popup), "and back again: the pointer returning to File opens File");
    }

    /// THE MENU HANDED OVER BY HOVER IS THERE AT ONCE: the fade-in belongs to the first opening, and walking
    /// along the bar must not blink each menu up out of nothing.
    ///
    /// AT EVERY REFRESH RATE AND ON EVERY FRAME, for half a second: the fade is held off for a stretch of
    /// TIME, and a stretch counted in frames would end early on a fast screen. Watching past the end of that
    /// stretch also catches the moment the animation time comes back.
    #[test]
    fn a_menu_handed_over_by_hover_shows_without_fading_in() {
        for hz in RATES {
            handed_over_without_fading_in(hz);
        }
    }

    fn handed_over_without_fading_in(hz: f64) {
        let mut bar = Bar::at(hz);
        // every menu has been shown once already: egui measures a popup invisibly the first time ever
        bar.click(bar.edit.center());
        bar.click(egui::pos2(700.0, 500.0));
        // THE PROBE SEES A FADE: opened by a click, the first frame of Edit is lighter than its last
        let from = bar.edit_painted.len();
        bar.click(bar.edit.center());
        let shown: Vec<u8> = bar.edit_painted[from..].iter().flatten().copied().collect();
        // a menu is painted in the window fill of the theme; anything lighter is a fade still under way
        let full = Some(bar.ctx.global_style().visuals.window_fill.a());
        let first = shown.first().copied();
        assert!(first < full, "setup at {hz} Hz: a menu opened by a click must fade in, or this check cannot see a fade at all ({shown:?})");
        bar.click(egui::pos2(700.0, 500.0));

        bar.click(bar.file.center());
        let from = bar.edit_painted.len();
        bar.frame(vec![egui::Event::PointerMoved(bar.edit.center())]);
        bar.idle(0.5);
        let shown = &bar.edit_painted[from..];
        let faint: Vec<(usize, Option<u8>)> = shown.iter().copied().enumerate().filter(|&(_, a)| a != full).collect();
        assert!(faint.is_empty(), "at {hz} Hz, Edit handed over from File by hover must be fully opaque ({full:?}) on every frame - frames that were not: {faint:?}");
    }

    /// THE HAND-OVER TAKES NO FRAME: on the very frame the pointer reaches Edit, Edit is drawn and File is
    /// not - no frame with the old menu still hanging under the new title, none with the bar empty.
    #[test]
    fn the_hand_over_happens_on_the_frame_the_pointer_arrives() {
        for hz in RATES {
            hand_over_on_arrival(hz);
        }
    }

    fn hand_over_on_arrival(hz: f64) {
        let mut bar = Bar::at(hz);
        bar.click(bar.edit.center());
        bar.click(egui::pos2(700.0, 500.0));
        bar.click(bar.file.center());
        let at = bar.edit_painted.len();
        bar.frame(vec![egui::Event::PointerMoved(bar.edit.center())]);
        assert!(bar.file_painted[at].is_none(), "at {hz} Hz, File is still drawn on the frame the pointer reached Edit: the menu lags a frame behind the pointer");
        assert!(bar.edit_painted[at].is_some(), "at {hz} Hz, Edit is not drawn on the frame the pointer reached it: the menu lags a frame behind the pointer");
        bar.frame(Vec::new());
        bar.frame(Vec::new());
        for (i, (f, e)) in bar.file_painted[at..].iter().zip(&bar.edit_painted[at..]).enumerate() {
            assert!(f.is_some() || e.is_some(), "at {hz} Hz, frame {i} after the pointer reached Edit draws no menu at all - the bar blinks empty");
        }
    }

    /// THE FIRST MENU STILL WANTS A CLICK: hover alone over a closed bar opens nothing.
    #[test]
    fn a_closed_bar_does_not_open_by_hover() {
        let mut bar = Bar::new();
        bar.hover(bar.file.center());
        bar.hover(bar.edit.center());
        assert!(!bar.open(bar.file_popup) && !bar.open(bar.edit_popup), "hovering a closed bar must not open a menu");
    }

    /// THE REAL MENU BAR IS BUILT FROM THESE MENUS rather than from egui's plain ones: the checks above
    /// prove nothing about the application if its bar does not go through `bar_menu_button`.
    #[test]
    fn the_application_bar_uses_the_walking_menus() {
        let panels = crate::gui::panels_source::PANELS;
        for key in ["menu-file", "menu-edit", "menu-view", "menu-windows", "menu-help"] {
            assert!(panels.contains(&format!(r#"bar_menu_button(qymcad_i18n::tr("{key}")"#)), "the \"{key}\" menu of the bar must be a `bar_menu_button`, or hover does not move the open menu to it");
        }
    }

    /// A CLICK OUTSIDE ENDS THE WALK: after it, hover opens nothing again.
    #[test]
    fn a_click_outside_ends_the_walk() {
        let mut bar = Bar::new();
        bar.click(bar.file.center());
        bar.click(egui::pos2(700.0, 500.0));
        assert!(!bar.open(bar.file_popup), "a click outside must close the menu");
        bar.hover(bar.edit.center());
        assert!(!bar.open(bar.edit_popup), "after a click outside, hovering Edit must not open it");
    }
}
