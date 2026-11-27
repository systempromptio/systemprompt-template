//! The fact list: its filter, the pages read so far, the cursor of the next
//! page and the selected fact. Each read carries the generation it was started
//! in, so an answer to a superseded read is dropped.

use systemprompt_models::brain::BrainFact;

use systemprompt_identifiers::BrainFactId;

use systemprompt_models::brain::client::FactPage;

use super::failure::BrainFailure;
use super::filter::FactFilter;
use super::load::Load;

/// The listed facts and how far the listing has been read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactListing {
    pub filter: FactFilter,
    pub facts: Load<Vec<BrainFact>>,
    pub next: Option<String>,
    pub appending: bool,
    pub more_failed: Option<BrainFailure>,
    pub selected: Option<BrainFactId>,
    generation: u64,
}

impl FactListing {
    pub fn begin(&mut self, appending: bool) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.more_failed = None;
        self.appending = appending && self.facts.ready().is_some();
        if !self.appending && self.facts.ready().is_none() {
            self.facts = Load::Loading;
        }
        self.generation
    }

    pub fn apply(&mut self, generation: u64, page: FactPage) -> bool {
        if generation != self.generation {
            return false;
        }
        let FactPage { facts, next } = page;
        match &mut self.facts {
            Load::Ready(known) if self.appending => {
                for fact in facts {
                    if !known.iter().any(|existing| existing.id == fact.id) {
                        known.push(fact);
                    }
                }
            },
            _ => self.facts = Load::Ready(facts),
        }
        self.appending = false;
        self.next = next;
        true
    }

    pub fn fail(&mut self, generation: u64, failure: BrainFailure) -> bool {
        if generation != self.generation {
            return false;
        }
        if self.appending {
            self.more_failed = Some(failure);
        } else {
            self.facts = Load::Failed(failure);
            self.next = None;
        }
        self.appending = false;
        true
    }

    pub fn set_filter(&mut self, filter: FactFilter) -> bool {
        if self.filter == filter {
            return false;
        }
        self.filter = filter;
        self.facts = Load::Loading;
        self.next = None;
        self.appending = false;
        self.more_failed = None;
        true
    }

    pub fn fact(&self, id: &BrainFactId) -> Option<&BrainFact> {
        self.facts.ready()?.iter().find(|fact| &fact.id == id)
    }

    pub fn clear(&mut self) {
        *self = Self {
            filter: std::mem::take(&mut self.filter),
            generation: self.generation.wrapping_add(1),
            ..Self::default()
        };
    }
}
