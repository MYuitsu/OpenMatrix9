//! THE HELP AND THE WINDOWS AROUND THE WORK: F1, the articles and their search, the search of commands, the start
//! screen, the About window, the report of a problem and the notice of a crash in the run before.
use qymcad::{Key, Machine, Session};
use qymcad_acceptance::{build, probe};

/// The block of the first part, with nothing in hand.
fn a_block() -> Session {
    let mut s = Session::start();
    build::block(&mut s);
    s.key(Key::Escape);
    s
}

probe! {
    /// F1 OPENS THE HELP at an article of its own, with something written in it.
    fn f1_opens_the_help() {
        let mut s = a_block();
        s.key(Key::F1);
        let title = s.word("help-title");
        assert!(s.windows().iter().any(|w| w.contains(&title)), "F1 was pressed and the help did not open: the windows are {:?}", s.windows());
        let words = s.words().len();
        assert!(words > 40, "the help opened and there is next to nothing in the window: {words} words");
    }
}

probe! {
    /// THE HELP IS SEARCHED AND AN ARTICLE IS READ: what is typed narrows the contents, and the article opened
    /// tells about what was asked for.
    fn the_help_is_searched_and_an_article_is_read() {
        let mut s = a_block();
        s.key(Key::F1); // the help of the place a person stands in
        let contents = s.words();
        // the field says what it is for in grey inside itself, so it is clicked where those words stand
        let search = s.word("help-search");
        let at = s.find(&search, qymcad::pos2(200.0, 200.0)).unwrap_or_else(|| panic!("the help has no search; on screen: {:?}", s.words()));
        s.click(at.center());
        s.type_text("sketch").key(Key::Enter);
        let found = s.words();
        assert!(found != contents, "the help was searched for \"sketch\" and the contents did not change");
        let article = found
            .iter()
            .find(|w| w.to_lowercase().contains("sketch") && w.len() > 6)
            .cloned()
            .unwrap_or_else(|| panic!("the search for \"sketch\" found no article: on screen {found:?}"));
        s.press_word_near(&article, qymcad::pos2(200.0, 300.0));
        let read = s.words();
        assert!(read.len() > found.len() / 2, "an article was opened and there is nothing in it: {read:?}");
    }
}

probe! {
    /// THE SEARCH OF COMMANDS FINDS A TOOL BY A PIECE OF ITS NAME AND STARTS IT.
    fn the_search_of_commands_finds_a_tool_and_starts_it() {
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        build::rectangle_on_xy(&mut s);
        let finish = s.word("wb-finish");
        s.press_word(&finish); // a sketch to work on, so the tool has what to take
        s.key(Key::Space);
        let hint = s.word("cs-hint");
        s.fill_empty(&hint, "extr");
        let extrude = s.word("cmd-extrude");
        assert!(s.shows(&extrude), "the search of commands was asked for \"extr\" and does not offer {extrude:?}; on screen: {:?}", s.words());
        s.key(Key::Enter);
        assert!(s.in_hand().first().is_some_and(|w| w.contains(&extrude)), "the command was chosen in the search and the bar says {:?}", s.in_hand());
    }
}

probe! {
    /// THE START SCREEN IS PUT AWAY AND COMES BACK FROM THE MENU, offering what it offers.
    fn the_start_screen_is_put_away_and_comes_back() {
        let mut s = Session::start();
        let (title, in_the_menu) = (s.word("start-title"), s.word("win-start"));
        assert!(s.shows(&title), "a first start does not show the start screen; on screen: {:?}", s.words());
        s.key(Key::Escape);
        assert!(!s.shows(&title), "Esc was pressed and the start screen stayed; on screen: {:?}", s.words());
        let windows = s.word("menu-windows");
        s.menu(&[&windows, &in_the_menu]);
        assert!(s.shows(&title), "the start screen was asked for in the Windows menu and did not come; on screen: {:?}", s.words());
        for key in ["start-new-part", "start-new-assembly", "start-recent"] {
            let word = s.word(key);
            assert!(s.shows(&word), "the start screen does not offer {word:?}; on screen: {:?}", s.words());
        }
    }
}

probe! {
    /// THE ABOUT WINDOW NAMES THE PROGRAM, ITS BUILD AND WHOSE IT IS, and closes again.
    fn the_about_window_names_the_program() {
        let mut s = a_block();
        let (help, about) = (s.word("menu-help"), s.word("help-about"));
        s.menu(&[&help, &about]);
        let author = s.word("about-author-name");
        assert!(s.shows(&author), "the About window does not name the author; on screen: {:?}", s.words());
        let licence = s.word("about-license");
        assert!(s.shows(&licence), "the About window does not name the licence; on screen: {:?}", s.words());
        let version = s.words().iter().any(|w| w.chars().any(|c| c.is_ascii_digit()) && w.contains('.'));
        assert!(version, "the About window names no version of the program; on screen: {:?}", s.words());
        let close = s.word("close");
        s.press_word_near(&close, qymcad::pos2(640.0, 400.0));
        assert!(!s.shows(&author), "the About window was closed and is still there");
    }
}

probe! {
    /// A PROBLEM IS REPORTED: the window asks what happened, what was expected and how to repeat it.
    fn a_problem_is_reported() {
        let mut s = a_block();
        let (help, report) = (s.word("menu-help"), s.word("help-report"));
        s.menu(&[&help, &report]);
        for key in ["report-title", "report-what", "report-expected", "report-steps"] {
            let word = s.word(key);
            assert!(s.shows(&word), "the window of a problem report does not ask {word:?}; on screen: {:?}", s.words());
        }
        let what = s.word("report-what-hint");
        s.fill_empty(&what, "the fillet does not take");
    }
}

probe! {
    /// A CRASH IN THE RUN BEFORE IS TOLD ABOUT ONCE: the report left on the machine is named at the next start,
    /// and the start after it says nothing.
    fn a_crash_in_the_run_before_is_told_about_once() {
        let home = std::env::temp_dir().join(format!("qymcad-crash-probe-{}", std::process::id()));
        let crashes = home.join("crashes"); // the one folder of the person's files, the crash reports in it
        std::fs::create_dir_all(&crashes).expect("the folder of the reports is made");
        std::fs::write(crashes.join("crash_20260101_000000.txt"), "step: extrude\nmessage: the check wrote this\n").expect("the report of the run before is written");
        let mut s = Session::start_on(Machine { home: home.clone(), ..Machine::default() });
        let title = s.word("crash-title");
        assert!(s.shows(&title), "a report of a crash lay on the machine and the program says nothing of it; on screen: {:?}", s.words());
        let what = s.word("crash-what");
        assert!(s.shows(&what), "the notice of the crash does not say what the report is for; on screen: {:?}", s.words());
        s.key(Key::Escape); // the start screen of a first start is put away, so the notice stands alone
        let at = s.find(&title, qymcad::pos2(640.0, 400.0)).unwrap_or_else(|| panic!("the notice of the crash is not on screen"));
        // the button of the notice itself, not another window's word of the same name
        let close = s.word("close");
        let button = s
            .widgets()
            .into_iter()
            .filter(|w| w.kind == qymcad::Kind::Button && w.label.contains(&close) && w.rect.center().y > at.max.y && w.rect.center().y < at.max.y + 260.0 && (w.rect.center().x - at.center().x).abs() < 260.0)
            .min_by(|a, b| a.rect.center().y.total_cmp(&b.rect.center().y))
            .unwrap_or_else(|| panic!("the notice of the crash has no {close:?} to press; on screen: {:?}", s.words()));
        s.click(button.rect.center());
        assert!(!s.shows(&s.word("crash-title").clone()), "the notice of the crash was closed and is still there");
        let kept = match s.quit() {
            Ok(kept) => kept,
            Err(mut s) => {
                let dont_save = s.word("nav-dont-save");
                s.press_word(&dont_save);
                s.quit().unwrap_or_else(|_| panic!("the window would not give what it keeps"))
            }
        };
        let mut s = Session::start_on(Machine { home, kept, ..Machine::default() });
        assert!(!s.shows(&s.word("crash-title").clone()), "the report was seen and the program tells of it again; on screen: {:?}", s.words());
    }
}

probe! {
    /// THE NOTICE OF A CRASH IS THE TOP THING ON SCREEN: at a first start it stands over the start screen, and its
    /// Close can be pressed without putting anything else away first.
    fn the_notice_of_a_crash_stands_over_the_start_screen() {
        let home = std::env::temp_dir().join(format!("qymcad-crash-over-{}", std::process::id()));
        let crashes = home.join("crashes"); // the one folder of the person's files, the crash reports in it
        std::fs::create_dir_all(&crashes).expect("the folder of the reports is made");
        let report = crashes.join("crash_20260101_000000.txt");
        std::fs::write(&report, "step: extrude\n").expect("the report of the run before is written");
        let mut s = Session::start_on(Machine { home, ..Machine::default() });
        let title = s.word("crash-title");
        let at = s.find(&title, qymcad::pos2(640.0, 400.0)).unwrap_or_else(|| panic!("a report lay on the machine and the program says nothing of it; on screen: {:?}", s.words()));
        let close = s.word("close");
        let button = s
            .widgets()
            .into_iter()
            .filter(|w| w.kind == qymcad::Kind::Button && w.label.contains(&close) && w.rect.center().y > at.max.y && w.rect.center().y < at.max.y + 260.0 && (w.rect.center().x - at.center().x).abs() < 260.0)
            .min_by(|a, b| a.rect.center().y.total_cmp(&b.rect.center().y))
            .unwrap_or_else(|| panic!("the notice of the crash has no {close:?} to press; on screen: {:?}", s.words()));
        s.click(button.rect.center());
        assert!(!s.shows(&title), "the Close of the crash notice was pressed at a first start and the notice is still there: the start screen lies over it and takes the click");
    }
}
