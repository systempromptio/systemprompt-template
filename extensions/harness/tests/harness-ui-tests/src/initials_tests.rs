use harness_ui::components::initials;

#[test]
fn initials_take_first_and_last_words() {
    assert_eq!(initials("Ada Lovelace"), "AL");
    assert_eq!(initials("grace brewster murray hopper"), "GH");
    assert_eq!(initials("ada.lovelace@example.com"), "AL");
}

#[test]
fn single_words_use_two_letters() {
    assert_eq!(initials("edward"), "ED");
    assert_eq!(initials("x"), "X");
}

#[test]
fn empty_and_symbol_only_names_have_no_initials() {
    assert_eq!(initials(""), "");
    assert_eq!(initials("  -- "), "");
}
