use harness_desktop::state::input::{Intent, Submission, classify, slash_command};
use harness_engine::{AttachmentInput, CommandInfo, Delivery};

fn commands() -> Vec<CommandInfo> {
    ["compact", "review"]
        .into_iter()
        .map(|name| CommandInfo {
            name: name.to_owned(),
            description: None,
            builtin: true,
        })
        .collect()
}

#[test]
fn known_slash_commands_are_split_from_their_arguments() {
    assert_eq!(
        slash_command("/review src/lib.rs  ", &commands()),
        Some(("review".to_owned(), "src/lib.rs".to_owned()))
    );
    assert_eq!(
        slash_command("  /compact", &commands()),
        Some(("compact".to_owned(), String::new()))
    );
}

#[test]
fn unknown_commands_and_plain_text_stay_prompts() {
    assert_eq!(slash_command("/deploy now", &commands()), None);
    assert_eq!(slash_command("please /review", &commands()), None);
    assert_eq!(slash_command("/", &commands()), None);
}

#[test]
fn a_command_with_attachments_is_sent_as_a_prompt() {
    let mut submission = Submission::text("/review", Delivery::Steer);
    assert!(matches!(
        classify(submission.clone(), &commands()),
        Intent::Command(_)
    ));
    submission
        .attachments
        .push(AttachmentInput::Path("a.png".into()));
    assert!(matches!(
        classify(submission, &commands()),
        Intent::Prompt(_)
    ));
}

#[test]
fn blank_submissions_are_empty() {
    assert!(Submission::text("  \n", Delivery::Queue).is_empty());
    assert!(!Submission::text("hi", Delivery::Queue).is_empty());
}
