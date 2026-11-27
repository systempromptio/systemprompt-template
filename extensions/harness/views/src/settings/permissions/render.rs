//! How the Permissions page draws: the add-rule form over one card per
//! permission, each rule with its action and where it comes from, and a remove
//! button on the user's own rules.

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    Window, div, px,
};
use systemprompt_engine::config::Action;
use systemprompt_engine::config::permission::{
    EDIT, EXTERNAL_DIRECTORY, FETCH, GLOB, GREP, QUESTION, READ, SHELL, SKILL, TASK, TODO,
};
use systemprompt_harness_shared::components::{
    Badge, Banner, Button, IconButton, Select, SelectOption,
};
use systemprompt_harness_shared::theme::{space, text as type_scale, tokens};
use systemprompt_harness_shared::{IconName, Tone};

use crate::settings::layout;
use crate::settings::permissions::load::RulesView;
use crate::settings::permissions::rules::{Group, ShownRule, Source};
use crate::settings::permissions::{Loading, PermissionsPage};
use crate::strings::settings as text;

const ANY: &str = "*";
const PERMISSIONS: [&str; 12] = [
    ANY,
    READ,
    EDIT,
    SHELL,
    GLOB,
    GREP,
    FETCH,
    TODO,
    TASK,
    SKILL,
    QUESTION,
    EXTERNAL_DIRECTORY,
];
const ACTIONS: [Action; 3] = [Action::Allow, Action::Ask, Action::Deny];
const SELECT_WIDTH: f32 = 170.0;
const SOURCE_WIDTH: f32 = 240.0;
const IDS_PER_GROUP: usize = 10_000;

fn permission_label(permission: &str) -> SharedString {
    if permission == ANY {
        SharedString::new_static(text::ANY_PERMISSION)
    } else {
        SharedString::from(permission.to_owned())
    }
}

const fn tone(action: Action) -> Tone {
    match action {
        Action::Allow => Tone::Success,
        Action::Ask => Tone::Warning,
        Action::Deny => Tone::Danger,
    }
}

fn source_label(source: Option<&Source>) -> String {
    match source {
        Some(Source::Defaults(agent)) => text::with_detail(text::SOURCE_DEFAULTS, agent),
        Some(Source::User(path)) => {
            text::with_detail(text::SOURCE_USER, &path.display().to_string())
        },
        None => text::SOURCE_USER.to_owned(),
        Some(Source::Project(path)) => {
            text::with_detail(text::SOURCE_PROJECT, &path.display().to_string())
        },
        Some(Source::Agent(agent)) => text::with_detail(text::SOURCE_AGENT, agent),
    }
}

impl PermissionsPage {
    fn render_rules(&self, rules: &RulesView, cx: &Context<Self>) -> Vec<AnyElement> {
        let mut body = Vec::new();
        if let Some(agent) = &rules.agent {
            body.push(layout::note(text::with_detail(text::RULES_FOR, agent), cx));
        }
        for (index, problem) in rules.problems.iter().enumerate() {
            let file = text::with_detail(text::RULE_PROBLEM, &problem.path.display().to_string());
            let message = text::with_detail(&file, &problem.kind.to_string());
            body.push(Banner::warning(("rules-problem", index), message).into_any_element());
        }
        body.push(self.render_form(rules.user.editable, cx));
        if rules.groups.is_empty() {
            body.push(layout::note(text::NO_RULES, cx));
        }
        for (index, group) in rules.groups.iter().enumerate() {
            body.push(Self::render_group(index, group, &rules.sources, cx));
        }
        body
    }

    fn render_form(&self, editable: bool, cx: &Context<Self>) -> AnyElement {
        let permission = Select::new(
            "rule-permission",
            PERMISSIONS.map(|name| SelectOption::new(name, permission_label(name))),
        )
        .selected(Some(self.permission.clone()))
        .width(px(SELECT_WIDTH))
        .disabled(!editable)
        .on_select(cx.listener(|page, value: &SharedString, _, cx| {
            page.permission = value.clone();
            cx.notify();
        }));
        let action = Select::new(
            "rule-action",
            ACTIONS.map(|action| SelectOption::new(action.as_str(), text::action(action))),
        )
        .selected(Some(SharedString::new_static(self.action.as_str())))
        .width(px(SELECT_WIDTH))
        .disabled(!editable)
        .on_select(cx.listener(|page, value: &SharedString, _, cx| {
            if let Some(action) = Action::parse(value) {
                page.action = action;
                cx.notify();
            }
        }));
        let add = Button::new("rule-add")
            .label(text::ADD)
            .icon(IconName::Plus)
            .primary()
            .small()
            .disabled(!editable)
            .on_click(cx.listener(|page, _, window, cx| page.add_rule(window, cx)));
        let form = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .w_full()
            .child(permission)
            .child(div().flex_1().min_w_0().child(self.field.clone()))
            .child(action)
            .child(add)
            .into_any_element();
        let mut rows = vec![form];
        if !editable {
            rows.push(layout::note(text::RULES_NOT_EDITABLE, cx));
        }
        layout::card(text::ADD_RULE, rows)
    }

    fn render_group(
        group_index: usize,
        group: &Group,
        sources: &[Source],
        cx: &Context<Self>,
    ) -> AnyElement {
        let rows = group
            .rules
            .iter()
            .enumerate()
            .map(|(index, shown)| {
                Self::render_rule(group_index * IDS_PER_GROUP + index, shown, sources, cx)
            })
            .collect();
        layout::card(permission_label(&group.permission), rows)
    }

    fn render_rule(
        id: usize,
        shown: &ShownRule,
        sources: &[Source],
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let removal = shown.removal.clone().map(|edit| {
            IconButton::new(("rule-remove", id), IconName::Trash, text::REMOVE_RULE)
                .small()
                .on_click(cx.listener(move |page, _, _, cx| page.apply(edit.clone(), cx)))
        });
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .w_full()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(layout::mono(shown.rule.pattern.clone(), cx)),
            )
            .child(Badge::new(text::action(shown.rule.action)).tone(tone(shown.rule.action)))
            .child(
                div()
                    .w(px(SOURCE_WIDTH))
                    .truncate()
                    .text_size(type_scale::SMALL)
                    .text_color(palette.text.muted)
                    .child(SharedString::from(source_label(sources.get(shown.source)))),
            )
            .children(removal)
            .into_any_element()
    }
}

impl Render for PermissionsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match (&self.scope, &self.loading, &self.rules) {
            (None, _, _) => vec![layout::empty(text::NO_PROJECT)],
            (Some(_), Loading::Failed, _) => vec![layout::empty(text::LOAD_FAILED)],
            (Some(_), _, Some(rules)) => self.render_rules(rules, cx),
            (Some(_), Loading::Busy | Loading::Idle, None) => vec![layout::loading()],
        };
        layout::page(
            "settings-permissions",
            text::PERMISSIONS,
            text::PERMISSIONS_BODY,
            body,
            cx,
        )
    }
}
