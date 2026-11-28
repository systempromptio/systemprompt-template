//! The question dock's reply and the composer's completion lists.

use systemprompt_engine::run::{Question, QuestionOption};
use systemprompt_harness_views::composer::popup::{Trigger, trigger};
use systemprompt_harness_views::docks::question_draft::QuestionDraft;

fn question(multiple: bool) -> Question {
    Question {
        question: "Which?".to_owned(),
        header: String::new(),
        options: ["Red", "Green", "Blue"]
            .into_iter()
            .map(|label| QuestionOption {
                label: label.to_owned(),
                description: String::new(),
            })
            .collect(),
        multiple,
    }
}

#[test]
fn a_multiselect_reply_holds_the_choices_and_the_typed_answer() {
    let questions = vec![question(true)];
    let mut draft = QuestionDraft::new(&questions);
    draft.toggle(0, 0);
    draft.toggle(0, 2);
    let typed = vec!["  Purple, edited  ".to_owned()];
    assert_eq!(
        draft.answers(&questions, &typed),
        vec![vec![
            "Red".to_owned(),
            "Blue".to_owned(),
            "Purple, edited".to_owned()
        ]]
    );
}

#[test]
fn picking_a_single_choice_answer_replaces_the_earlier_pick() {
    let questions = vec![question(false)];
    let mut draft = QuestionDraft::new(&questions);
    draft.toggle(0, 0);
    draft.toggle(0, 1);
    assert!(draft.has_answer(&[]));
    assert_eq!(
        draft.answers(&questions, &[]),
        vec![vec!["Green".to_owned()]]
    );
}

#[test]
fn slash_and_at_open_their_lists_and_typing_a_space_closes_them() {
    assert_eq!(
        trigger("/rev", 4),
        Some(Trigger::Slash {
            query: "rev".to_owned()
        })
    );
    assert_eq!(trigger("/review ", 8), None);
    assert_eq!(
        trigger("look at @src", 12),
        Some(Trigger::Mention {
            start: 8,
            query: "src".to_owned()
        })
    );
    assert_eq!(trigger("look at @src now", 16), None);
    assert_eq!(trigger("mail me@host", 12), None);
}
