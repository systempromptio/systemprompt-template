//! The English text of the model and agent pickers.

use systemprompt_engine::run::Protocol;

pub(crate) const MODEL_TOOLTIP: &str = "Choose the model";
pub(crate) const AGENT_TOOLTIP: &str = "Choose the agent";
pub(crate) const SEARCH_MODELS: &str = "Search models";
pub(crate) const SEARCH_AGENTS: &str = "Search agents";

pub(crate) const NO_MODEL: &str = "No model";
pub(crate) const DEFAULT_MODEL: &str = "Instance default";
pub(crate) const DEFAULT_AGENT: &str = "Default agent";
pub(crate) const DEFAULT_AGENT_DETAIL: &str = "The project's default, or the first primary agent";

pub(crate) const NO_MODELS_TITLE: &str = "No models available";
pub(crate) const NO_MODELS_BODY: &str = "Models come from the instance you are signed in to. Sign in, or ask your administrator to offer models to this app.";
pub(crate) const NO_AGENTS_TITLE: &str = "No agents found";
pub(crate) const NO_AGENTS_BODY: &str =
    "Agents come from the built-in set and the agents folders of this project.";
pub(crate) const NO_MATCHES: &str = "Nothing matches your search";

pub(crate) const THOUSAND_TOKENS: &str = "K";
pub(crate) const MILLION_TOKENS: &str = "M";

pub(crate) fn default_model_detail(name: &str) -> String {
    format!("Currently {name}")
}

pub(crate) fn context_badge(size: &str) -> String {
    format!("{size} context")
}

pub(crate) const fn protocol(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::Messages => "Messages",
        Protocol::ChatCompletions => "Chat Completions",
    }
}
