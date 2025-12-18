use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContextInput {
    pub query: String,
    pub path: Option<String>,
    pub max_iterations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningDecision {
    pub analysis: String,
    pub is_complete: bool,
    #[serde(default)]
    pub next_actions: Vec<NextAction>,
    pub final_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type")]
pub enum NextAction {
    #[serde(rename = "read_files")]
    ReadFiles { paths: Vec<String> },
    #[serde(rename = "grep")]
    Grep {
        pattern: String,
        path: Option<String>,
        glob: Option<String>,
    },
    #[serde(rename = "list_directory")]
    ListDirectory { path: String, depth: Option<u32> },
    #[serde(rename = "glob_search")]
    GlobSearch {
        pattern: String,
        path: Option<String>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct AccumulatedContext {
    pub directory_tree: String,
    pub file_contents: Vec<FileContent>,
    pub search_results: Vec<SearchResult>,
    pub actions_taken: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub query: String,
    pub matches: Vec<String>,
}

impl AccumulatedContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn format_for_ai(&self) -> String {
        let mut output = String::new();

        if !self.directory_tree.is_empty() {
            output.push_str("## Directory Structure\n\n```\n");
            output.push_str(&self.directory_tree);
            output.push_str("\n```\n\n");
        }

        if !self.file_contents.is_empty() {
            output.push_str("## File Contents\n\n");
            for file in &self.file_contents {
                output.push_str(&format!("### {}\n\n", file.path));
                if file.truncated {
                    output.push_str("*(truncated)*\n\n");
                }
                output.push_str("```\n");
                output.push_str(&file.content);
                output.push_str("\n```\n\n");
            }
        }

        if !self.search_results.is_empty() {
            output.push_str("## Search Results\n\n");
            for result in &self.search_results {
                output.push_str(&format!("### Search: `{}`\n\n", result.query));
                for match_line in &result.matches {
                    output.push_str(&format!("- {}\n", match_line));
                }
                output.push_str("\n");
            }
        }

        if !self.actions_taken.is_empty() {
            output.push_str("## Actions Taken\n\n");
            for (i, action) in self.actions_taken.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, action));
            }
            output.push_str("\n");
        }

        output
    }

    pub fn estimated_tokens(&self) -> usize {
        let text = self.format_for_ai();
        text.len() / 4
    }
}
