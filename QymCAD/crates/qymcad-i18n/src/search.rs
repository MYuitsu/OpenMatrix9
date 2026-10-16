//! SEARCH BY NAME: the rule every search box matches by - the interface language, and English besides.

/// WHERE THE QUERY STANDS IN A NAME: in the `shown` form, else in the English one `name_in` gives.
pub fn find_in_names(q: &str, shown: &str, name_in: impl Fn(&str) -> String) -> Option<usize> {
    if let Some(at) = shown.to_lowercase().find(q) {
        return Some(at);
    }
    if crate::language() == crate::FALLBACK {
        return None;
    }
    crate::keys::keys_in_text(&name_in(crate::FALLBACK)).to_lowercase().find(q)
}

/// THE QUERY AS A SEARCH COMPARES IT: without the spaces around it, in lower case.
pub fn query(typed: &str) -> String {
    typed.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::find_in_names;

    /// The name of a key in the language `code` of the catalogue, empty where it has none.
    fn name_in(key: &str) -> impl Fn(&str) -> String + '_ {
        move |code| crate::tr_in(code, key).unwrap_or_default()
    }

    /// A NAME IS FOUND BY ITS ENGLISH FORM, one way only.
    #[test]
    fn a_name_is_found_by_english_one_way() {
        let prev = crate::language();
        let key = "language-name";
        let english = crate::tr_in("en", key).expect("the English name of English").to_lowercase();
        let russian = crate::tr_in("ru", key).expect("the Russian name of Russian").to_lowercase();
        let mut wrong = Vec::new();
        for (ui, typed, found) in [("ru", &english, true), ("ru", &russian, true), ("en", &english, true), ("en", &russian, false)] {
            crate::set_language(ui);
            if find_in_names(typed, &crate::tr(key), name_in(key)).is_some() != found {
                wrong.push(format!("interface `{ui}`, `{typed}` typed: found {}, expected {found}", !found));
            }
        }
        crate::set_language(&prev);
        assert!(wrong.is_empty(), "{wrong:?}");
    }

    /// The position comes from the first form that holds the query.
    #[test]
    fn the_position_comes_from_the_first_form_that_holds_the_query() {
        let prev = crate::language();
        crate::set_language("ru");
        let english = |code: &str| if code == "en" { "Spiegel".to_string() } else { String::new() };
        let none = find_in_names("zzz", "Mirror", english);
        let at = find_in_names("ror", "Mirror", english);
        let other = find_in_names("gel", "Mirror", english);
        crate::set_language(&prev);
        assert_eq!(none, None, "a word in no form of the name is found");
        assert_eq!(at, Some(3), "`ror` stands at byte 3 of the shown form `mirror`");
        assert_eq!(other, Some(4), "`gel` stands at byte 4 of the English form `spiegel`");
    }
}
