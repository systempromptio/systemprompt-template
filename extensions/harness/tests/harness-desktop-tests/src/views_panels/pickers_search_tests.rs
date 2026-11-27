use systemprompt_harness_views::pickers::search::{context_size, initial, matches, step};

#[test]
fn every_word_of_the_query_must_appear() {
    assert!(matches("", &["Claude Sonnet", "sonnet-4"]));
    assert!(matches("son 4", &["Claude Sonnet", "sonnet-4"]));
    assert!(matches("CLAUDE", &["Claude Sonnet"]));
    assert!(!matches("opus", &["Claude Sonnet", "sonnet-4"]));
    assert!(!matches("sonnet gpt", &["Claude Sonnet"]));
}

#[test]
fn a_query_may_match_across_fields() {
    assert!(matches("sonnet messages", &["Sonnet", "Messages"]));
}

#[test]
fn stepping_wraps_at_both_ends() {
    assert_eq!(step(Some(0), 3, 1), Some(1));
    assert_eq!(step(Some(2), 3, 1), Some(0));
    assert_eq!(step(Some(0), 3, -1), Some(2));
}

#[test]
fn stepping_without_a_highlight_starts_at_the_edge_it_moves_from() {
    assert_eq!(step(None, 4, 1), Some(0));
    assert_eq!(step(None, 4, -1), Some(3));
    assert_eq!(step(Some(9), 4, 1), Some(0));
    assert_eq!(step(Some(1), 0, 1), None);
}

#[test]
fn the_initial_highlight_falls_back_to_the_first_row() {
    assert_eq!(initial(Some(2), 3), Some(2));
    assert_eq!(initial(Some(5), 3), Some(0));
    assert_eq!(initial(None, 3), Some(0));
    assert_eq!(initial(None, 0), None);
}

#[test]
fn context_sizes_are_compact() {
    assert_eq!(context_size(0, "K", "M"), None);
    assert_eq!(context_size(512, "K", "M").as_deref(), Some("512"));
    assert_eq!(context_size(200_000, "K", "M").as_deref(), Some("200K"));
    assert_eq!(context_size(32_768, "K", "M").as_deref(), Some("33K"));
    assert_eq!(context_size(1_000_000, "K", "M").as_deref(), Some("1M"));
    assert_eq!(context_size(1_048_576, "K", "M").as_deref(), Some("1M"));
    assert_eq!(context_size(1_500_000, "K", "M").as_deref(), Some("1.5M"));
}
