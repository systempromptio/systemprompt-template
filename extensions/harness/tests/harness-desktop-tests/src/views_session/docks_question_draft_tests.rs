use systemprompt_engine::run::{Question, QuestionOption};
use systemprompt_harness_views::docks::question_draft::QuestionDraft;

fn question(multiple: bool) -> Question {
    Question {
        question: "Which?".to_owned(),
        header: String::new(),
        options: vec![
            QuestionOption {
                label: "One".to_owned(),
                description: String::new(),
            },
            QuestionOption {
                label: "Two".to_owned(),
                description: String::new(),
            },
        ],
        multiple,
    }
}

#[test]
fn single_choice_keeps_one_option() {
    let questions = vec![question(false)];
    let mut draft = QuestionDraft::new(&questions);
    draft.toggle(0, 0);
    draft.toggle(0, 1);
    assert!(!draft.is_chosen(0, 0));
    assert!(draft.is_chosen(0, 1));
    assert_eq!(
        draft.answers(&questions, &[String::new()]),
        vec![vec!["Two".to_owned()]]
    );
}

#[test]
fn multiple_choice_collects_options_and_typed_text() {
    let questions = vec![question(true)];
    let mut draft = QuestionDraft::new(&questions);
    draft.toggle(0, 1);
    draft.toggle(0, 0);
    let answers = draft.answers(&questions, &["Three".to_owned()]);
    assert_eq!(
        answers,
        vec![vec!["One".to_owned(), "Two".to_owned(), "Three".to_owned()]]
    );
}

#[test]
fn typed_text_replaces_a_single_choice() {
    let questions = vec![question(false), question(false)];
    let mut draft = QuestionDraft::new(&questions);
    assert!(!draft.has_answer(&[String::new(), String::new()]));
    draft.toggle(0, 0);
    let answers = draft.answers(&questions, &["Mine".to_owned(), String::new()]);
    assert_eq!(answers, vec![vec!["Mine".to_owned()], Vec::<String>::new()]);
    assert!(draft.has_answer(&[String::new(), String::new()]));
}
