//! The model picker: the models the signed-in instance offers, searchable by
//! name, id and wire protocol, each with its protocol family and context size.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window};

use crate::app::Services;
use crate::state::{AppEvent, AppState};
use crate::strings::pickers as text;
use crate::views::pickers::popup::{
    self, EmptyCopy, PickerHost, PickerIds, PickerOption, PickerPopup, TriggerCopy,
};
use crate::views::pickers::search;

const IDS: PickerIds = PickerIds {
    popover: "model-picker",
    trigger: "model-picker-trigger",
    list: "model-picker-list",
    row: "model-option",
};

pub(crate) struct ModelPicker {
    app: Entity<AppState>,
    _services: Services,
    popup: PickerPopup,
    _observation: Subscription,
}

impl std::fmt::Debug for ModelPicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelPicker")
            .field("popup", &self.popup)
            .finish_non_exhaustive()
    }
}

impl ModelPicker {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        let popup = PickerPopup::new(IDS, text::SEARCH_MODELS, window, cx);
        Self {
            app,
            _services: services,
            popup,
            _observation: observation,
        }
    }
}

impl PickerHost for ModelPicker {
    fn popup(&self) -> &PickerPopup {
        &self.popup
    }

    fn popup_mut(&mut self) -> &mut PickerPopup {
        &mut self.popup
    }

    fn options(&self, query: &str, cx: &App) -> Vec<PickerOption> {
        let state = self.app.read(cx);
        let Some(first) = state.models.first() else {
            return Vec::new();
        };
        let mut options = Vec::with_capacity(state.models.len() + 1);
        let default_detail = text::default_model_detail(&first.name);
        if search::matches(query, &[text::DEFAULT_MODEL, default_detail.as_str()]) {
            options.push(PickerOption {
                value: None,
                title: SharedString::new_static(text::DEFAULT_MODEL),
                detail: Some(SharedString::from(default_detail)),
                badges: Vec::new(),
                current: state.model.is_none(),
            });
        }
        for model in &state.models {
            let protocol = text::protocol(model.protocol);
            if !search::matches(query, &[model.name.as_str(), model.id.as_str(), protocol]) {
                continue;
            }
            let mut badges = vec![SharedString::new_static(protocol)];
            if let Some(size) = search::context_size(
                model.context_window,
                text::THOUSAND_TOKENS,
                text::MILLION_TOKENS,
            ) {
                badges.push(SharedString::from(text::context_badge(&size)));
            }
            options.push(PickerOption {
                value: Some(model.id.clone()),
                title: SharedString::from(model.name.clone()),
                detail: (model.id != model.name).then(|| SharedString::from(model.id.clone())),
                badges,
                current: state.model.as_ref() == Some(&model.id),
            });
        }
        options
    }

    fn choose(&mut self, value: Option<String>, cx: &mut Context<Self>) {
        self.app.update(cx, |app, cx| {
            app.set_model(value, cx);
            cx.emit(AppEvent::FocusComposer);
        });
    }
}

impl Render for ModelPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app.read(cx);
        let label = state.current_model().map_or_else(
            || SharedString::new_static(text::NO_MODEL),
            |model| SharedString::from(model.name.clone()),
        );
        let empty = if state.models.is_empty() {
            EmptyCopy {
                title: SharedString::new_static(text::NO_MODELS_TITLE),
                body: Some(SharedString::new_static(text::NO_MODELS_BODY)),
            }
        } else {
            EmptyCopy {
                title: SharedString::new_static(text::NO_MATCHES),
                body: None,
            }
        };
        let trigger = TriggerCopy {
            label,
            tooltip: text::MODEL_TOOLTIP,
        };
        popup::render(&*self, trigger, &empty, cx)
    }
}
