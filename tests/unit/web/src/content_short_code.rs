//! Short codes are the public half of every campaign link, so they must be a
//! fixed 8 characters drawn only from URL-safe alphanumerics — a stray
//! separator or a variable length would break the `/{short_code}` redirect
//! route — and they must not collide in practice.

use std::collections::HashSet;

use systemprompt_web_extension::services::link_generation::generate_short_code;

#[test]
fn a_short_code_is_eight_characters() {
    for _ in 0..64 {
        assert_eq!(generate_short_code().chars().count(), 8);
    }
}

#[test]
fn a_short_code_is_alphanumeric_ascii_only() {
    for _ in 0..64 {
        let code = generate_short_code();
        assert!(
            code.chars().all(|c| c.is_ascii_alphanumeric()),
            "unexpected character in {code}"
        );
    }
}

#[test]
fn short_codes_do_not_repeat_across_a_batch() {
    let codes: HashSet<String> = (0..500).map(|_| generate_short_code()).collect();
    assert_eq!(codes.len(), 500);
}
