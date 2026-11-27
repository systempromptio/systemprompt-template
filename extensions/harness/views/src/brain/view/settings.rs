//! The Brain's settings pane: the recall switch, written through the
//! extension's own settings, and where the brain stands. It reads the stored
//! preference on every render and follows the preference store, the brain's
//! feed and the instance's enablement.

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Render, Styled as _, Task, Window, div,
};
use systemprompt_harness::extension::{Enablement, ExtensionSettings};
use systemprompt_harness::members::brain::{BRAIN_EXTENSION, BrainService, BrainSettings};
use systemprompt_harness_shared::components::{Banner, Card, StatusDot, Toggle};
use systemprompt_harness_shared::contract::ShellContext;
use systemprompt_harness_shared::theme::{space, text, tokens};

use super::{bridge, tone};
use crate::brain::model::Availability;
use crate::brain::strings;

pub(crate) struct BrainSettingsPane {
    settings: ExtensionSettings,
    service: BrainService,
    enablement: Enablement,
    write_failed: bool,
    _feeds: [Task<()>; 3],
}

impl BrainSettingsPane {
    pub(crate) fn new(ctx: &ShellContext, service: BrainService, cx: &Context<Self>) -> Self {
        let settings = ctx.settings_for(&BRAIN_EXTENSION);
        let feeds = [
            bridge::follow(cx, settings.changes(), |_, _, cx| cx.notify()),
            bridge::follow(cx, service.subscribe(), |_, _, cx| cx.notify()),
            bridge::follow(cx, ctx.enablement.subscribe(), |_, _, cx| cx.notify()),
        ];
        Self {
            settings,
            service,
            enablement: ctx.enablement.clone(),
            write_failed: false,
            _feeds: feeds,
        }
    }

    fn set_recall(&mut self, recall: bool, cx: &mut Context<Self>) {
        match self.settings.set(&BrainSettings { recall }) {
            Ok(()) => self.write_failed = false,
            Err(error) => {
                tracing::warn!(error = %error, "brain settings could not be saved");
                self.write_failed = true;
            },
        }
        cx.notify();
    }

    fn recall_row(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let stored = self.settings.get::<BrainSettings>();
        let unreadable = stored.is_err();
        let recall = stored.is_ok_and(|settings| settings.recall);
        let weak = cx.weak_entity();
        let toggle = Toggle::new("brain-recall")
            .on(recall)
            .on_change(move |on, _, cx| {
                if weak.update(cx, |this, cx| this.set_recall(on, cx)).is_err() {
                    tracing::trace!("recall switched after the Brain settings closed");
                }
            });
        let label = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(space::XXS)
            .child(
                div()
                    .text_size(text::BODY)
                    .text_color(palette.text.primary)
                    .child(strings::SETTINGS_RECALL),
            )
            .child(
                div()
                    .text_size(text::SMALL)
                    .text_color(palette.text.secondary)
                    .child(strings::SETTINGS_RECALL_BODY),
            );
        div()
            .flex()
            .flex_col()
            .gap(space::MD)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(space::LG)
                    .child(label)
                    .child(toggle),
            )
            .children(unreadable.then(|| {
                Banner::warning("brain-settings-unreadable", strings::SETTINGS_UNREADABLE)
            }))
            .children(
                self.write_failed
                    .then(|| Banner::error("brain-settings-write", strings::SETTINGS_WRITE_FAILED)),
            )
            .into_any_element()
    }

    fn availability_row(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let availability =
            Availability::of(self.enablement.current().as_ref(), self.service.status());
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .child(StatusDot::new(
                "brain-availability",
                tone::availability(availability),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::XXS)
                    .child(
                        div()
                            .text_size(text::BODY)
                            .text_color(palette.text.primary)
                            .child(strings::availability_title(availability)),
                    )
                    .child(
                        div()
                            .text_size(text::SMALL)
                            .text_color(palette.text.secondary)
                            .child(strings::availability_body(availability)),
                    ),
            )
            .into_any_element()
    }
}

impl Render for BrainSettingsPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(space::LG)
            .child(Card::new().child(self.recall_row(cx)))
            .child(
                Card::new()
                    .title(strings::SETTINGS_AVAILABILITY)
                    .child(self.availability_row(cx)),
            )
    }
}
