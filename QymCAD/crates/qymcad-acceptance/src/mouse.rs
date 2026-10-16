//! THE MOUSE LAYOUTS THE SETTINGS OFFER, as a person uses them: the gesture of each that turns the view, the one that
//! moves it and the one that scales it, and the way a layout is chosen. Written out here rather than asked of the
//! program: a layout is a promise to the habit a person came with, and a check that asks the program what it promises
//! checks nothing.
use qymcad::{Modifiers, PointerButton, Pos2, Session};

/// A GESTURE OF A LAYOUT: the buttons held while the pointer moves, and the modifiers held with them. No button at
/// all means the pointer moves with the modifiers alone, as the touchpad layout asks.
#[derive(Clone, Copy, Debug)]
pub struct Hold {
    pub buttons: &'static [PointerButton],
    pub mods: Modifiers,
}

pub const LEFT: PointerButton = PointerButton::Primary;
pub const RIGHT: PointerButton = PointerButton::Secondary;
pub const MIDDLE: PointerButton = PointerButton::Middle;

/// A hold of `buttons` with no modifier.
pub const fn bare(buttons: &'static [PointerButton]) -> Hold {
    Hold { buttons, mods: Modifiers::NONE }
}

/// A hold of `buttons` with `mods`.
pub const fn with(buttons: &'static [PointerButton], mods: Modifiers) -> Hold {
    Hold { buttons, mods }
}

/// Ctrl with Shift, as the touchpad layout scales with them.
const CTRL_SHIFT: Modifiers = Modifiers { alt: false, ctrl: true, shift: true, mac_cmd: false, command: true };

/// A LAYOUT: its code in the settings, the gesture that turns the view, the one that moves it, and the one that scales
/// it - `None` where the wheel does.
pub struct Layout {
    pub code: &'static str,
    pub turn: Hold,
    pub pan: Hold,
    pub zoom: Option<Hold>,
}

/// EVERY LAYOUT, as the line under the list of the settings describes it.
pub const LAYOUTS: [Layout; 11] = [
    Layout { code: "qymcad", turn: bare(&[LEFT]), pan: with(&[LEFT], Modifiers::SHIFT), zoom: None },
    Layout { code: "cad", turn: bare(&[MIDDLE, LEFT]), pan: bare(&[MIDDLE]), zoom: None },
    Layout { code: "blender", turn: bare(&[MIDDLE]), pan: with(&[MIDDLE], Modifiers::SHIFT), zoom: None },
    Layout { code: "gesture", turn: bare(&[LEFT]), pan: bare(&[RIGHT]), zoom: None },
    Layout { code: "maya", turn: with(&[LEFT], Modifiers::ALT), pan: with(&[MIDDLE], Modifiers::ALT), zoom: None },
    Layout { code: "opencascade", turn: bare(&[MIDDLE, RIGHT]), pan: bare(&[MIDDLE]), zoom: None },
    Layout { code: "openinventor", turn: bare(&[LEFT]), pan: bare(&[MIDDLE]), zoom: None },
    Layout { code: "openscad", turn: bare(&[LEFT]), pan: bare(&[RIGHT]), zoom: None },
    Layout { code: "revit", turn: with(&[MIDDLE], Modifiers::SHIFT), pan: bare(&[MIDDLE]), zoom: None },
    Layout { code: "tinkercad", turn: bare(&[RIGHT]), pan: bare(&[MIDDLE]), zoom: None },
    Layout { code: "touchpad", turn: with(&[], Modifiers::ALT), pan: with(&[], Modifiers::SHIFT), zoom: Some(with(&[], CTRL_SHIFT)) },
];

/// MAKE THE GESTURE `hold` from `from` to `to`: every button of it held at once, the modifiers with them.
pub fn make(s: &mut Session, hold: Hold, from: Pos2, to: Pos2) {
    s.drag_with(from, to, hold.buttons, hold.mods);
}

/// SCALE THE VIEW UP the way `layout` does it, about `at`: two notches of the wheel, or its gesture where it has no
/// wheel.
pub fn scale_up(s: &mut Session, layout: &Layout, at: Pos2) {
    match layout.zoom {
        Some(hold) => make(s, hold, at, qymcad::pos2(at.x, at.y - 120.0)),
        None => {
            for _ in 0..2 {
                s.wheel(at, qymcad::vec2(0.0, 50.0), Modifiers::NONE);
            }
        }
    }
}

/// OPEN THE SETTINGS at the section of the viewport.
pub fn open_the_viewport_settings(s: &mut Session) {
    let (windows, settings) = (s.word("menu-windows"), s.word("menu-settings"));
    s.menu(&[&windows, &settings]);
    let viewport = s.word("settings-sec-viewport");
    s.press_word(&viewport);
}

/// PUT THE SETTINGS AWAY by the cross of the window.
pub fn close_the_settings(s: &mut Session) {
    let title = s.word("win-settings");
    s.close_window(&title);
}

/// CHOOSE THE MOUSE LAYOUT `code` in the settings, as a person changing habits does: the list of layouts, the name
/// in it, the window closed again.
pub fn choose_the_layout(s: &mut Session, code: &str) {
    open_the_viewport_settings(s);
    let caption = s.word("settings-mouse-nav");
    let list = s.field(&caption);
    s.click(list.rect.center());
    let name = s.word(&format!("settings-mouse-{code}"));
    // THE LIST IS LONGER THAN IT IS TALL: the last layouts of it are reached with the wheel, as a person does
    for _ in 0..8 {
        if s.find(&name, list.rect.center()).is_some() {
            break;
        }
        let last = s.word("settings-mouse-tinkercad");
        let over = s.find(&last, list.rect.center()).unwrap_or_else(|| panic!("the list of layouts is open and does not show them; on screen: {:?}", s.words()));
        s.wheel(over.center(), qymcad::vec2(0.0, -50.0), Modifiers::NONE);
    }
    s.press_word_near(&name, list.rect.center());
    close_the_settings(s);
}
