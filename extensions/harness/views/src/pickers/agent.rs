//! The agent picker: the project's primary agents with their descriptions, plus
//! the project default.

use gpui_kit::{App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window};

use crate::app::Services;
use crate::state::{AppEvent, AppState};
use crate::strings::pickers as text;
use crate::views::pickers::popup::{
    self, EmptyCopy, PickerHost, PickerIds, PickerOption, PickerPopup, TriggerCopy,
};
use crate::views::pickers::search;

const IDS: PickerIds = PickerIds {
    popover: "agent-picker",
    trigger: "agent-picker-trigger",
    list: "agent-picker-list",
    row: "agent-option",
};

pub(crate) struct AgentPicker {
    app: Entity<AppState>,
    _services: Services,
    popup: PickerPopup,
    _observation: Subscription,
}

impl std::fmt::Debug for AgentPicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentPicker")
            .field("popup", &self.popup)
            .finish_non_exhaustive()
    }
}

impl AgentPicker {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        let popup = PickerPopup::new(IDS, text::SEARCH_AGENTS, window, cx);
        Self {
            app,
            _services: services,
            popup,
            _observation: observation,
        }
    }
}

impl PickerHost for AgentPicker {
    fn popup(&self) -> &PickerPopup {
        &self.popup
    }

    fn popup_mut(&mut self) -> &mut PickerPopup {
        &mut self.popup
    }

    fn options(&self, query: &str, cx: &App) -> Vec<PickerOption> {
        let state = self.app.read(cx);
        if state.agents.is_empty() {
            return Vec::new();
        }
        let mut options = Vec::with_capacity(state.agents.len() + 1);
        if search::matches(query, &[text::DEFAULT_AGENT, text::DEFAULT_AGENT_DETAIL]) {
            options.push(PickerOption {
                value: None,
                title: SharedString::new_static(text::DEFAULT_AGENT),
                detail: Some(SharedString::new_static(text::DEFAULT_AGENT_DETAIL)),
                badges: Vec::new(),
                current: state.agent.is_none(),
            });
        }
        for agent in &state.agents {
            let description = agent.description.as_deref().unwrap_or_default();
            if !search::matches(query, &[agent.name.as_str(), description]) {
                continue;
            }
            options.push(PickerOption {
                value: Some(agent.name.clone()),
                title: SharedString::from(agent.name.clone()),
                detail: agent.description.clone().map(SharedString::from),
                badges: Vec::new(),
                current: state.agent.as_ref() == Some(&agent.name),
            });
        }
        options
    }

    fn choose(&mut self, value: Option<String>, cx: &mut Context<Self>) {
        self.app.update(cx, |app, cx| {
            app.set_agent(value, cx);
            cx.emit(AppEvent::FocusComposer);
        });
    }
}

impl Render for AgentPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app.read(cx);
        let label = state.agent.clone().map_or_else(
            || SharedString::new_static(text::DEFAULT_AGENT),
            SharedString::from,
        );
        let empty = if state.agents.is_empty() {
            EmptyCopy {
                title: SharedString::new_static(text::NO_AGENTS_TITLE),
                body: Some(SharedString::new_static(text::NO_AGENTS_BODY)),
            }
        } else {
            EmptyCopy {
                title: SharedString::new_static(text::NO_MATCHES),
                body: None,
            }
        };
        let trigger = TriggerCopy {
            label,
            tooltip: text::AGENT_TOOLTIP,
        };
        popup::render(&*self, trigger, &empty, cx)
    }
}
