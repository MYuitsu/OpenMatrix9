//! WHAT A COMMAND BAR SAYS GOES ON ITS OWN LINE.
//!
//! Reported behaviour: "messages like these must be written on a new line. The text with the
//! information - a new line; if there is text with errors, that goes on a new line too, so on the
//! third one. And this is scale 1 - very small and unreadable even on my 2K monitor."
//!
//! MEASURED ON THE THREAD BAR, which is the one in the report. Opened with its own defaults - M10,
//! coarse pitch, 0.20 mm of fit - it says four sentences: the geometry of the size, what the mating
//! part needs, and two things that will not build as typed. All four were laid BESIDE the buttons in
//! one wrapping row, 470 characters of prose in the middle of a row of controls, breaking wherever the
//! row happened to end - mid-sentence, and with the second half starting under a button.
//!
//! And they were drawn at the small text size, which is what "unreadable" is about. Small was the
//! price of standing in a row of controls; on a line of their own the sentences pay no such price.
//!
//! THE ROWS ARE MEASURED, NOT THE CALLS. Where a label is written in the source says nothing about
//! where it lands: the row wraps by itself. So the frame is run and the drawn text is banded by its y.

use qymcad_ui_state::Bench;

/// A text the frame drew: what it says, where it landed, its colour and its size.
struct Drawn {
    text: String,
    rect: egui::Rect,
    colour: egui::Color32,
    size: f32,
}

/// Everything the frame drew.
fn texts(shapes: &[egui::epaint::ClippedShape]) -> Vec<Drawn> {
    fn walk(s: &egui::epaint::Shape, out: &mut Vec<Drawn>) {
        match s {
            egui::epaint::Shape::Text(t) => {
                let fmt = t.galley.job.sections.first().map(|s| s.format.clone()).unwrap_or_default();
                let colour = t.override_text_color.unwrap_or(fmt.color);
                out.push(Drawn { text: t.galley.text().to_string(), rect: egui::Rect::from_min_size(t.pos, t.galley.size()), colour, size: fmt.font_id.size });
            }
            egui::epaint::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for cs in shapes {
        walk(&cs.shape, &mut out);
    }
    out
}

/// The rows of the bar: drawn text banded by the y it landed on.
///
/// Banded rather than compared pairwise, because "on its own line" is a statement about rows, and two
/// labels of different heights on one row do not share a top edge.
fn rows(mut drawn: Vec<Drawn>) -> Vec<Vec<Drawn>> {
    drawn.sort_by(|a, b| a.rect.center().y.total_cmp(&b.rect.center().y));
    let mut out: Vec<Vec<_>> = Vec::new();
    for t in drawn {
        match out.last_mut() {
            Some(row) if row.iter().any(|p: &Drawn| p.rect.intersects(t.rect) || (p.rect.center().y - t.rect.center().y).abs() < 4.0) => row.push(t),
            _ => out.push(vec![t]),
        }
    }
    out
}

/// The bench with the thread command open, and the rows its bar was drawn in.
struct ThreadBar {
    bench: Bench,
    rows: Vec<Vec<Drawn>>,
}

/// The thread command as a person meets it: opened, with the defaults of its own fields.
fn the_thread_bar() -> ThreadBar {
    let mut b = Bench { mode_3d: true, ..Default::default() };
    b.cmd.open(&mut b.armed, 24, true);
    qymcad_ui_state::set_thread_params(&mut b.cmd, b.thread);

    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1600.0, 800.0));
    let full = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| {
        qymcad_part::feat_command_bar(&mut b.part_ctx(), ui);
    });
    let r = rows(texts(&full.shapes));
    ThreadBar { bench: b, rows: r }
}

/// THE SENTENCES ARE NOT IN THE ROW OF CONTROLS, and the errors are not in the row of the sentences.
#[test]
fn the_information_and_the_errors_each_get_a_line() {
    let ThreadBar { bench: b, rows } = the_thread_bar();
    let (note, wrong) = (b.scheme.pal.hint(), b.scheme.pal.error_mild());

    let row_of = |c: egui::Color32| rows.iter().enumerate().filter(|(_, r)| r.iter().any(|t| t.colour == c)).map(|(i, _)| i).collect::<Vec<_>>();
    let notes = row_of(note);
    let wrongs = row_of(wrong);
    assert!(!notes.is_empty(), "GUARD: the thread bar must say something for there to be a line to place");
    assert!(!wrongs.is_empty(), "GUARD: the thread defaults must not build, or there is no error line to place");

    assert!(!notes.contains(&0), "what the tool says shares the row of controls: {:?}", rows[0].iter().map(|t| &t.text).collect::<Vec<_>>());
    assert!(!wrongs.contains(&0), "what is wrong shares the row of controls");
    assert!(wrongs.iter().min() > notes.iter().max(), "the errors must come after what the tool says, on a line of their own: information on rows {notes:?}, errors on rows {wrongs:?}");
}

/// AND THEY ARE WRITTEN AT THE ORDINARY SIZE.
///
/// The small size was what squeezing a sentence into a row of controls cost. Off that row the sentence
/// is read, not glanced at, and it is written like the rest of the text.
#[test]
fn the_sentences_are_written_at_the_ordinary_size() {
    let ThreadBar { bench: b, rows } = the_thread_bar();
    let body = rows[0].iter().map(|t| t.size).fold(0.0f32, f32::max);
    let said: Vec<_> = rows.iter().flatten().filter(|t| t.colour == b.scheme.pal.hint() || t.colour == b.scheme.pal.error_mild()).collect();
    assert!(!said.is_empty(), "GUARD: nothing was said, so nothing was measured");

    let small: Vec<_> = said.iter().filter(|t| t.size < body).map(|t| format!("{} at {} against {body}", t.text, t.size)).collect();
    assert!(small.is_empty(), "a sentence written smaller than the controls beside it is the unreadable one:\n{}", small.join("\n"));
}
