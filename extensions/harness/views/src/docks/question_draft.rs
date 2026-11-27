//! The answers being put together for a question request: which options are
//! chosen for each question, single or several, and the answer lists the engine
//! receives, typed text included.

use std::collections::BTreeSet;

use harness_engine::Question;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionDraft {
    chosen: Vec<BTreeSet<usize>>,
    multiple: Vec<bool>,
}

impl QuestionDraft {
    pub fn new(questions: &[Question]) -> Self {
        Self {
            chosen: vec![BTreeSet::new(); questions.len()],
            multiple: questions.iter().map(|question| question.multiple).collect(),
        }
    }

    pub fn toggle(&mut self, question: usize, option: usize) {
        let multiple = self.multiple.get(question).copied().unwrap_or(false);
        let Some(chosen) = self.chosen.get_mut(question) else {
            return;
        };
        if chosen.remove(&option) {
            return;
        }
        if !multiple {
            chosen.clear();
        }
        chosen.insert(option);
    }

    pub fn is_chosen(&self, question: usize, option: usize) -> bool {
        self.chosen
            .get(question)
            .is_some_and(|chosen| chosen.contains(&option))
    }

    pub fn answers(&self, questions: &[Question], typed: &[String]) -> Vec<Vec<String>> {
        questions
            .iter()
            .enumerate()
            .map(|(index, question)| {
                let own = typed
                    .get(index)
                    .map(|text| text.trim())
                    .filter(|text| !text.is_empty());
                let picked = self.chosen.get(index).into_iter().flatten();
                let labels = picked.filter_map(|option| {
                    question
                        .options
                        .get(*option)
                        .map(|choice| choice.label.clone())
                });
                match own {
                    Some(own) if !question.multiple => vec![own.to_owned()],
                    Some(own) => labels.chain(std::iter::once(own.to_owned())).collect(),
                    None => labels.collect(),
                }
            })
            .collect()
    }

    pub fn has_answer(&self, typed: &[String]) -> bool {
        self.chosen.iter().any(|chosen| !chosen.is_empty())
            || typed.iter().any(|text| !text.trim().is_empty())
    }
}
