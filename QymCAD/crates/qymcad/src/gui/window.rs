//! A WINDOW WITHOUT A SCREEN: whole frames of the program, fed with input on a clock of their own, and what each
//! frame drew and asked for read back.
//!
//! One driver for everything that runs the program by its frames - the hand of the checks inside the crate and
//! the session outside it - so the two cannot come to disagree about what a frame is.
use std::time::Duration;

use super::App;

/// The window: its context and clock, and what its last frame drew and asked for.
pub(crate) struct Window {
    /// THE CONTEXT the mouse and the keys go into, kept for the whole life of the window. egui tells a click from
    /// a drag, and a double click from two clicks, by what it saw in the frames before.
    pub(crate) ctx: egui::Context,
    /// The clock of the window, in seconds. A pause is time, not a count of frames.
    pub(crate) clock: f64,
    /// The size of the window, in points.
    pub(crate) screen: egui::Vec2,
    /// THE WORDS THE LAST FRAME DREW, and where: a button is pressed where its word is written.
    pub(crate) drawn: Vec<(String, egui::Rect)>,
    /// What the last frame drew, shape by shape, for a picture of the window.
    pub(crate) shapes: Vec<egui::epaint::ClippedShape>,
    /// Every texture the frames loaded and have not freed, as pictures.
    pub(crate) textures: std::collections::HashMap<egui::TextureId, super::help_raster::Tex>,
    /// THE PLATES OF THE TOOL BUTTONS the last frame drew and did not clip away: an icon button and a symbol
    /// button are both a 40x34 plate, and neither carries a word - what it does is in its hint.
    pub(crate) plates: Vec<egui::Pos2>,
    /// What the frames put on the clipboard, oldest first.
    pub(crate) copied: Vec<String>,
    /// The pages the frames asked the system to open, oldest first.
    pub(crate) urls: Vec<String>,
    /// The title the program last gave its window.
    pub(crate) title: Option<String>,
    /// THE LAST FRAME REFUSED TO CLOSE the window it was asked to close.
    pub(crate) close_refused: bool,
    /// THE PROGRAM CLOSED ITS WINDOW itself - Quit, or the answer to a question asked on closing.
    pub(crate) closed: bool,
    /// How soon the last frame asked to be drawn again; `Duration::MAX` is "not until something happens".
    pub(crate) repaint: Duration,
    /// The shape of the pointer the last frame asked for.
    pub(crate) cursor: egui::CursorIcon,
    /// THE LONGEST A FRAME HAS TAKEN since it was last asked, in wall time.
    pub(crate) worst_frame: Duration,
    /// The same, since the last step of a check was watched.
    pub(crate) worst_step_frame: Duration,
    /// WHAT CAN BE ACTED ON, as the last frame described it to assistive technology: every widget with its kind,
    /// its words, its value and where it stands.
    pub(crate) widgets: Vec<Widget>,
}

/// A widget of the window, as a screen reader is told about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Widget {
    /// What kind of thing it is.
    pub kind: Kind,
    /// The words on it: a button's caption, a checkbox's text.
    pub label: String,
    /// What it holds: the text of a field, the number of a spin box.
    pub value: String,
    /// The grey words an empty field shows.
    pub placeholder: String,
    /// Where it stands, in points.
    pub rect: egui::Rect,
    /// Can it be used now?
    pub enabled: bool,
    /// Ticked or not, for what can be ticked.
    pub checked: Option<bool>,
}

/// The kinds of widget a person tells apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Button,
    CheckBox,
    RadioButton,
    TextField,
    Number,
    ComboBox,
    Slider,
    Link,
    Label,
    Window,
    Other,
}

impl Window {
    /// A window of `screen` points, its clock at zero, nothing drawn yet.
    pub(crate) fn new(screen: egui::Vec2) -> Self {
        let ctx = egui::Context::default();
        // the frames describe their widgets as they do for a screen reader; nothing else changes with it
        ctx.enable_accesskit();
        Self {
            ctx,
            clock: 0.0,
            screen,
            drawn: Vec::new(),
            shapes: Vec::new(),
            textures: std::collections::HashMap::new(),
            plates: Vec::new(),
            copied: Vec::new(),
            urls: Vec::new(),
            title: None,
            close_refused: false,
            closed: false,
            repaint: Duration::MAX,
            cursor: egui::CursorIcon::Default,
            worst_frame: Duration::ZERO,
            worst_step_frame: Duration::ZERO,
            widgets: Vec::new(),
        }
    }

    /// HOW MANY POINTS THE WINDOW HOLDS ACROSS AND DOWN: the size on the desk divided by the interface scale.
    pub(crate) fn across(&self) -> egui::Vec2 {
        self.screen / self.ctx.zoom_factor()
    }

    /// ONE WHOLE FRAME OF THE PROGRAM with `events` in it and `modifiers` held, a sixtieth of a second after the
    /// one before; `close` carries the system's request to close the window.
    ///
    /// The frame the live window runs (`App::draw_frame`, panels and all), not a handler: the frame decides first
    /// who gets a press or a key - a popup, a field holding the keyboard, a panel lying over the canvas.
    pub(crate) fn run(&mut self, app: &mut App, modifiers: egui::Modifiers, events: Vec<egui::Event>, close: bool) {
        self.clock += 1.0 / 60.0;
        // THE WINDOW IS THE SAME SIZE ON THE DESK WHATEVER THE INTERFACE SCALE IS: the scale makes a point
        // larger, so a larger scale leaves FEWER points across - that is what a live window hands egui, and what
        // makes the setting visible at all.
        let across = self.across();
        let mut raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, across)), time: Some(self.clock), modifiers, events, ..Default::default() };
        if close {
            raw.viewports.entry(egui::ViewportId::ROOT).or_default().events.push(egui::ViewportEvent::Close);
        }
        let began = std::time::Instant::now();
        let out = self.ctx.run_ui(raw, |ui| app.draw_frame(ui));
        self.worst_frame = self.worst_frame.max(began.elapsed());
        self.worst_step_frame = self.worst_step_frame.max(began.elapsed());
        self.drawn.clear();
        self.plates.clear();
        for shape in &out.shapes {
            words(&shape.shape, &mut self.drawn);
            plates(&shape.shape, shape.clip_rect, &mut self.plates);
        }
        // NOT ONE KEY OF THE CATALOGUE ON THE SCREEN INSTEAD OF A WORD, in any frame any check draws: a key the
        // catalogue lacks is drawn as it is, and nothing else would notice it
        let leaked: Vec<String> = self.drawn.iter().flat_map(|(t, _)| super::key_leak::leaks(t).into_iter().map(move |k| format!("{k:?} in {t:?}"))).collect();
        assert!(leaked.is_empty(), "a key of the catalogue reached the screen instead of words: {}", leaked.join("; "));
        for (id, delta) in &out.textures_delta.set {
            super::help_raster::apply_delta(&mut self.textures, *id, delta);
        }
        for id in &out.textures_delta.free {
            self.textures.remove(id);
        }
        self.shapes = out.shapes;
        for command in out.platform_output.commands {
            match command {
                egui::OutputCommand::CopyText(text) => self.copied.push(text),
                egui::OutputCommand::OpenUrl(url) => self.urls.push(url.url),
                egui::OutputCommand::CopyImage(_) => {}
            }
        }
        self.cursor = out.platform_output.cursor_icon;
        if let Some(tree) = &out.platform_output.accesskit_update {
            self.widgets = tree.nodes.iter().filter_map(|(_, node)| widget(node)).collect();
        }
        self.close_refused = false;
        self.repaint = Duration::MAX;
        if let Some(root) = out.viewport_output.get(&egui::ViewportId::ROOT) {
            self.repaint = root.repaint_delay;
            for command in &root.commands {
                match command {
                    egui::ViewportCommand::Title(t) => self.title = Some(t.clone()),
                    egui::ViewportCommand::CancelClose => self.close_refused = true,
                    egui::ViewportCommand::Close => self.closed = true,
                    _ => {}
                }
            }
        }
    }
}

/// A node of the accessibility tree as a widget, when it stands somewhere on screen.
fn widget(node: &egui::accesskit::Node) -> Option<Widget> {
    use egui::accesskit::{Role, Toggled};
    let b = node.bounds()?;
    let kind = match node.role() {
        Role::Button => Kind::Button,
        Role::CheckBox => Kind::CheckBox,
        Role::RadioButton => Kind::RadioButton,
        Role::TextInput | Role::MultilineTextInput => Kind::TextField,
        Role::SpinButton => Kind::Number,
        Role::ComboBox => Kind::ComboBox,
        Role::Slider => Kind::Slider,
        Role::Link => Kind::Link,
        Role::Label => Kind::Label,
        Role::Window => Kind::Window,
        _ => Kind::Other,
    };
    // the icons of the interface are letters of a private range of the icon font, not words
    let words = |s: Option<&str>| s.unwrap_or_default().chars().filter(|c| !('\u{e000}'..='\u{f8ff}').contains(c)).collect::<String>().trim().to_string();
    // a spin box says its number with its unit (`180s`); the number itself is what it holds
    let value = match (kind, node.numeric_value(), node.value()) {
        (Kind::Number | Kind::Slider, Some(n), _) => format!("{n}"),
        (_, _, Some(v)) => words(Some(v)),
        (_, Some(n), None) => format!("{n}"),
        (_, None, None) => String::new(),
    };
    Some(Widget {
        kind,
        label: words(node.label()),
        value,
        placeholder: words(node.placeholder()),
        rect: egui::Rect::from_min_max(egui::pos2(b.x0 as f32, b.y0 as f32), egui::pos2(b.x1 as f32, b.y1 as f32)),
        enabled: !node.is_disabled(),
        checked: node.toggled().and_then(|t| match t {
            Toggled::True => Some(true),
            Toggled::False => Some(false),
            Toggled::Mixed => None,
        }),
    })
}

/// The words of a shape and the rectangles they stand in.
fn words(s: &egui::epaint::Shape, out: &mut Vec<(String, egui::Rect)>) {
    match s {
        egui::epaint::Shape::Text(t) => {
            // the icons of the interface are letters of a private range of the icon font, not words
            let text: String = t.galley.text().chars().filter(|c| !('\u{e000}'..='\u{f8ff}').contains(c)).collect();
            // A TURNED TEXT (the length of an upright dimension laid along its line) turns about its top-left corner:
            // what it covers on the screen is the box around its four turned corners, not the level box it would take
            let size = t.galley.size();
            let (c, sn) = (t.angle.cos(), t.angle.sin());
            let turn = |v: egui::Vec2| t.pos + egui::vec2(v.x * c - v.y * sn, v.x * sn + v.y * c);
            let corners = [turn(egui::Vec2::ZERO), turn(egui::vec2(size.x, 0.0)), turn(size), turn(egui::vec2(0.0, size.y))];
            out.push((text.trim().to_string(), egui::Rect::from_points(&corners)));
        }
        egui::epaint::Shape::Vec(v) => v.iter().for_each(|s| words(s, out)),
        _ => {}
    }
}

/// The centres of the 40x34 tool plates of a shape that its clip leaves visible.
fn plates(s: &egui::epaint::Shape, clip: egui::Rect, out: &mut Vec<egui::Pos2>) {
    match s {
        egui::epaint::Shape::Rect(r) if (r.rect.width() - 40.0).abs() < 0.5 && (r.rect.height() - 34.0).abs() < 0.5 && clip.contains(r.rect.center()) => out.push(r.rect.center()),
        egui::epaint::Shape::Vec(v) => v.iter().for_each(|s| plates(s, clip, out)),
        _ => {}
    }
}
