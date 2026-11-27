//! Parsed content the transcript keeps between frames: one markdown document
//! per text part, re-split only at its tail as text streams in, and one parsed
//! patch per diff, re-parsed only when the diff text changes.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher as _};
use std::sync::Arc;

use systemprompt_engine::store::PartId;
use systemprompt_harness_shared::{MarkdownDocument, Patch};

#[derive(Debug, Default)]
pub(crate) struct DocumentCache {
    documents: HashMap<PartId, MarkdownDocument>,
}

impl DocumentCache {
    pub(crate) fn document(&mut self, id: &PartId, text: &str) -> &MarkdownDocument {
        let document = self.documents.entry(id.clone()).or_default();
        if document.text() != text && !document.set_text(text) {
            tracing::trace!("markdown document unchanged");
        }
        document
    }

    pub(crate) fn retain(&mut self, live: &HashSet<PartId>) {
        self.documents.retain(|id, _| live.contains(id));
    }
}

#[derive(Debug)]
pub(crate) struct PatchCache<K> {
    patches: HashMap<K, (u64, Arc<Patch>)>,
}

impl<K> Default for PatchCache<K> {
    fn default() -> Self {
        Self {
            patches: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq + Clone> PatchCache<K> {
    pub(crate) fn patch(&mut self, key: &K, text: &str) -> Arc<Patch> {
        let digest = digest(text);
        if let Some((known, patch)) = self.patches.get(key)
            && *known == digest
        {
            return Arc::clone(patch);
        }
        let patch = Arc::new(Patch::parse(text));
        self.patches
            .insert(key.clone(), (digest, Arc::clone(&patch)));
        patch
    }

    pub(crate) fn retain(&mut self, keep: impl Fn(&K) -> bool) {
        self.patches.retain(|key, _| keep(key));
    }
}

pub(crate) fn digest(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
