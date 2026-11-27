//! The facts tab: the filters and the paged fact list on the left, the open
//! fact on the right.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use harness_ui::components::{Badge, Banner, Button, EmptyState, ListRow, SectionHeader};
use harness_ui::theme::{space, tokens};
use harness_ui::{IconName, Tone};
use sp_protocol::wire::brain::{BrainFact, FactStatus};

use super::BrainScreen;
use crate::model::Load;
use crate::strings;
use crate::view::tone;

const LIST_PANE: f32 = 380.0;

impl BrainScreen {
    pub(super) fn render_facts_tab(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let list = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(px(LIST_PANE))
            .h_full()
            .border_r_1()
            .border_color(palette.border.subtle)
            .child(self.render_filters(cx))
            .child(
                div()
                    .id("brain-facts")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .px(space::XS)
                    .pb(space::MD)
                    .overflow_y_scroll()
                    .child(self.render_list(cx)),
            );
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(list)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_detail(cx)),
            )
            .into_any_element()
    }

    fn render_filters(&self, cx: &Context<Self>) -> AnyElement {
        let statuses = FactStatus::ALL
            .into_iter()
            .enumerate()
            .map(|(index, status)| {
                Button::new(("brain-status", index))
                    .label(strings::status(status))
                    .small()
                    .ghost()
                    .selected(self.listing.filter.has_status(status))
                    .on_click(cx.listener(move |this, _, _, cx| this.toggle_status(status, cx)))
            });
        div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .p(space::MD)
            .child(
                SectionHeader::new(strings::FILTERS).trailing(
                    Button::new("brain-filters-clear")
                        .label(strings::CLEAR_FILTERS)
                        .small()
                        .ghost()
                        .disabled(self.listing.filter.is_empty())
                        .on_click(
                            cx.listener(|this, _, window, cx| this.clear_filters(window, cx)),
                        ),
                ),
            )
            .child(self.kind_field.clone())
            .child(self.subject_field.clone())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(space::XXS)
                    .children(statuses),
            )
            .child(
                Button::new("brain-filters-apply")
                    .label(strings::APPLY_FILTERS)
                    .small()
                    .full_width()
                    .on_click(cx.listener(|this, _, _, cx| this.apply_filters(cx))),
            )
            .into_any_element()
    }

    fn render_list(&self, cx: &Context<Self>) -> AnyElement {
        let facts = match &self.listing.facts {
            Load::Ready(facts) => facts,
            Load::Idle | Load::Loading => return Self::waiting(),
            Load::Failed(failure) => return Self::failed(*failure, cx),
        };
        if facts.is_empty() {
            let state = if self.listing.filter.is_empty() {
                EmptyState::new(strings::NO_FACTS_TITLE).description(strings::NO_FACTS_BODY)
            } else {
                EmptyState::new(strings::NO_MATCHES)
            };
            return state.icon(IconName::Sparkle).into_any_element();
        }
        let more = self.listing.next.is_some().then(|| {
            Button::new("brain-facts-more")
                .label(strings::LOAD_MORE)
                .small()
                .ghost()
                .full_width()
                .loading(self.listing.appending)
                .on_click(cx.listener(|this, _, _, cx| this.load_more(cx)))
        });
        let more_failed = self.listing.more_failed.map(|failure| {
            Banner::error("brain-facts-more-failed", strings::failure(failure))
                .title(strings::MORE_FAILED)
        });
        div()
            .flex()
            .flex_col()
            .gap(space::XXS)
            .children(
                facts
                    .iter()
                    .enumerate()
                    .map(|(index, fact)| self.fact_row(index, fact, cx)),
            )
            .children(more_failed)
            .children(more)
            .into_any_element()
    }

    fn fact_row(&self, index: usize, fact: &BrainFact, cx: &Context<Self>) -> AnyElement {
        let target = fact.id.clone();
        let trailing = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::XS)
            .children(
                fact.pinned
                    .then(|| Badge::new(strings::PINNED).tone(Tone::Accent)),
            )
            .child(Badge::new(strings::status(fact.status)).tone(tone::status(fact.status)));
        ListRow::new(("brain-fact", index), fact.statement.clone())
            .subtitle(strings::fact_line(
                fact.kind.as_str(),
                &fact.subject.to_string(),
            ))
            .tall()
            .selected(self.listing.selected.as_ref() == Some(&fact.id))
            .trailing(trailing)
            .on_click(cx.listener(move |this, _, _, cx| this.select_fact(target.clone(), cx)))
            .into_any_element()
    }
}
