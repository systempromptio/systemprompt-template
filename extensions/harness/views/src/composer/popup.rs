//! The completion lists of the composer.
//!
//! They decide when a typed `/` opens the command list or an `@` the file list,
//! what the query is, which entries match it and in what order, and how a
//! chosen file is written into the prompt.

use harness_engine::CommandInfo;

const POPUP_LIMIT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trigger {
    Slash { query: String },
    Mention { start: usize, query: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    Slash,
    Mention { start: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupItem {
    pub value: String,
    pub detail: Option<String>,
    pub builtin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popup {
    pub kind: PopupKind,
    pub items: Vec<PopupItem>,
    pub selected: usize,
}

impl Popup {
    pub fn step(&mut self, delta: isize) {
        let count = self.items.len();
        if count == 0 {
            return;
        }
        let count_signed = isize::try_from(count).unwrap_or(isize::MAX);
        let current = isize::try_from(self.selected).unwrap_or(0);
        self.selected = usize::try_from((current + delta).rem_euclid(count_signed)).unwrap_or(0);
    }

    pub fn current(&self) -> Option<&PopupItem> {
        self.items.get(self.selected)
    }
}

pub fn trigger(text: &str, cursor: usize) -> Option<Trigger> {
    let cursor = cursor.min(text.len());
    let before = text.get(..cursor)?;
    if let Some(query) = before.strip_prefix('/')
        && !query.contains(char::is_whitespace)
    {
        return Some(Trigger::Slash {
            query: query.to_owned(),
        });
    }
    let start = before.rfind('@')?;
    let query = &before[start + 1..];
    if query.contains(char::is_whitespace) {
        return None;
    }
    let at_boundary = before[..start]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace);
    at_boundary.then(|| Trigger::Mention {
        start,
        query: query.to_owned(),
    })
}

fn rank(query: &str, name: &str, whole: &str) -> Option<u8> {
    let name = name.to_lowercase();
    if name.starts_with(query) {
        Some(0)
    } else if name.contains(query) {
        Some(1)
    } else if whole.to_lowercase().contains(query) {
        Some(2)
    } else {
        None
    }
}

pub fn command_items(query: &str, commands: &[CommandInfo]) -> Vec<PopupItem> {
    let query = query.to_lowercase();
    let mut ranked: Vec<(u8, &CommandInfo)> = commands
        .iter()
        .filter_map(|command| {
            rank(&query, &command.name, &command.name).map(|score| (score, command))
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name.cmp(&b.1.name)));
    ranked
        .into_iter()
        .take(POPUP_LIMIT)
        .map(|(_, command)| PopupItem {
            value: command.name.clone(),
            detail: command.description.clone(),
            builtin: command.builtin,
        })
        .collect()
}

pub fn path_items(query: &str, paths: &[String]) -> Vec<PopupItem> {
    let query = query.to_lowercase();
    let mut ranked: Vec<(u8, &String)> = paths
        .iter()
        .filter_map(|path| {
            let name = path.rsplit('/').next().unwrap_or(path);
            rank(&query, name, path).map(|score| (score, path))
        })
        .collect();
    ranked.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.len().cmp(&b.1.len()))
            .then_with(|| a.1.cmp(b.1))
    });
    ranked
        .into_iter()
        .take(POPUP_LIMIT)
        .map(|(_, path)| PopupItem {
            value: path.clone(),
            detail: path.rsplit_once('/').map(|(folder, _)| folder.to_owned()),
            builtin: false,
        })
        .collect()
}

pub fn insert_mention(text: &str, start: usize, cursor: usize, path: &str) -> (String, usize) {
    let cursor = cursor.clamp(start, text.len());
    let head = text.get(..start).unwrap_or(text);
    let tail = text.get(cursor..).unwrap_or_default();
    let mention = format!("@{path} ");
    let caret = head.len() + mention.len();
    (format!("{head}{mention}{tail}"), caret)
}
