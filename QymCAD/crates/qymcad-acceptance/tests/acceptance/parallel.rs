//! SESSIONS KEEP TO THEMSELVES: two programs taking turns in one check build what each was asked, in the language of
//! each machine.
use qymcad::{Key, Machine, Session};
use qymcad_acceptance::{build, probe};

probe! {
    /// TWO SESSIONS TAKING TURNS build two different blocks, each in its own document and its own language.
    fn two_sessions_taking_turns_keep_to_themselves() {
        let mut a = Session::start();
        let mut b = Session::start_on(Machine { locale: "ru".into(), ..Machine::default() });
        let (hint_a, hint_b) = (a.word("g-start-hint"), b.word("g-start-hint"));
        assert_ne!(hint_a, hint_b, "the two machines speak one language");
        assert_eq!((a.status(), b.status()), (hint_a, hint_b), "a program started after another on the same thread speaks the other's language");
        build::into_the_first_part(&mut a);
        build::into_the_first_part(&mut b);
        for s in [&mut a, &mut b] {
            let xy = s.word("plane-xy-table");
            s.press_word(&xy);
        }
        for (s, far) in [(&mut a, (40.0, 30.0)), (&mut b, (20.0, 10.0))] {
            let rect = s.word("tb-rect-hint");
            s.press_hint(&rect).click_on_sketch(0.0, 0.0).click_on_sketch(far.0, far.1);
        }
        for s in [&mut a, &mut b] {
            let finish = s.word("wb-finish");
            s.press_word(&finish);
            let extrude = s.word("tb-extrude-hint");
            s.press_hint(&extrude).key(Key::Enter);
        }
        let volume = |s: &mut Session| s.document().bodies.iter().map(|b| b.volume).collect::<Vec<_>>();
        let (va, vb) = (volume(&mut a), volume(&mut b));
        assert!(va.len() == 1 && (va[0] - 12000.0).abs() < 1e-6, "the first session's block holds {va:?} mm^3, not 12000");
        assert!(vb.len() == 1 && (vb[0] - 2000.0).abs() < 1e-6, "the second session's block holds {vb:?} mm^3, not 2000");
        let (en, ru) = (a.word("wb-finish"), b.word("wb-finish"));
        assert_ne!(en, ru, "the two machines speak one language");
        assert!(a.words().contains(&a.word("menu-file")) && b.words().contains(&b.word("menu-file")), "a session draws in the other's language");
    }
}
