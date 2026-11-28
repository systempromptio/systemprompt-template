//! What the composer submits: an empty submission does nothing, and a
//! slash command is told apart from a prompt.

use systemprompt_engine::run::{CommandInfo, Delivery};
use systemprompt_harness_views::state::input::{Intent, Submission, classify, slash_command};

#[test]
fn an_empty_or_blank_submission_sends_nothing() {
    assert!(Submission::text("", Delivery::Steer).is_empty());
    assert!(Submission::text("  \n\t", Delivery::Queue).is_empty());
    assert!(!Submission::text("hello", Delivery::Steer).is_empty());
}

#[test]
fn a_known_slash_command_runs_as_a_command_and_anything_else_is_a_prompt() {
    let commands = [CommandInfo {
        name: "compact".to_owned(),
        description: None,
        builtin: true,
    }];
    assert_eq!(
        slash_command("/compact now", &commands),
        Some(("compact".to_owned(), "now".to_owned()))
    );
    assert!(matches!(
        classify(Submission::text("/compact", Delivery::Steer), &commands),
        Intent::Command(_)
    ));
    assert!(matches!(
        classify(Submission::text("/unknown", Delivery::Steer), &commands),
        Intent::Prompt(_)
    ));
}
