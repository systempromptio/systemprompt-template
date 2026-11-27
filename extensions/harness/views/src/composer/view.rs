//! How the composer looks: the completion list above a card holding the
//! attachment chips, the growing prompt and a toolbar with attach, the agent
//! and model pickers, the delivery hint and send or stop.

use gpui_kit::component::input::Textarea;
use gpui_kit::{
    AnyElement, Context, ExternalPaths, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Window, div, px,
};
use systemprompt_harness_shared::components::{Badge, Button, IconButton, ListRow};
use systemprompt_harness_shared::theme::{radius, space, text as type_scale, tokens};
use systemprompt_harness_shared::{IconName, Tone};

use crate::app::settings::DeliveryPref;
use crate::composer::ComposerView;
use crate::composer::popup::PopupKind;
use crate::strings::composer as text;

const POPUP_WIDTH: f32 = 520.0;

impl ComposerView {
    fn sync_input(&mut self, busy: bool, window: &mut Window, cx: &mut Context<Self>) {
        let enter_sends = self
            .services
            .settings()
            .read(|settings| settings.composer.enter_sends);
        if enter_sends != self.enter_sends {
            self.enter_sends = enter_sends;
            self.input
                .update(cx, |input, cx| input.set_submit_on_enter(enter_sends, cx));
        }
        if busy != self.busy_placeholder {
            self.busy_placeholder = busy;
            let placeholder = if busy {
                text::PLACEHOLDER_BUSY
            } else {
                text::PLACEHOLDER
            };
            self.input.update(cx, |input, cx| {
                input.set_placeholder(placeholder, window, cx);
            });
        }
    }

    fn render_popup(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let open = self.popup.as_ref()?;
        let palette = *tokens(cx);
        let empty = match open.kind {
            PopupKind::Slash => text::NO_COMMANDS,
            PopupKind::Mention { .. } => text::NO_FILES,
        };
        let prefix = if open.kind == PopupKind::Slash {
            "/"
        } else {
            "@"
        };
        let rows: Vec<AnyElement> = open
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut row =
                    ListRow::new(("composer-popup", index), format!("{prefix}{}", item.value))
                        .selected(index == open.selected)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.accept_popup(Some(index), window, cx);
                        }));
                if let Some(detail) = &item.detail {
                    row = row.subtitle(detail.clone());
                }
                if item.builtin {
                    row = row.trailing(Badge::new(text::BUILTIN));
                }
                row.into_any_element()
            })
            .collect();
        let body = if rows.is_empty() {
            div()
                .px(space::MD)
                .py(space::SM)
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .child(empty)
                .into_any_element()
        } else {
            div().flex().flex_col().children(rows).into_any_element()
        };
        Some(
            div()
                .w_full()
                .max_w(px(POPUP_WIDTH))
                .p(space::XS)
                .rounded(radius::LG)
                .border_1()
                .border_color(palette.border.strong)
                .bg(palette.surface.raised)
                .child(body)
                .into_any_element(),
        )
    }

    fn render_chips(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if self.attachments.is_empty() {
            return None;
        }
        let palette = *tokens(cx);
        let chips = self
            .attachments
            .iter()
            .enumerate()
            .map(|(index, attachment)| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(space::XXS)
                    .pl(space::SM)
                    .rounded(radius::SM)
                    .border_1()
                    .border_color(palette.border.subtle)
                    .bg(palette.surface.sunken)
                    .text_size(type_scale::CAPTION)
                    .text_color(palette.text.secondary)
                    .child(
                        div()
                            .max_w(px(220.))
                            .truncate()
                            .child(SharedString::from(attachment.label.clone())),
                    )
                    .child(
                        IconButton::new(
                            ("composer-chip-remove", index),
                            IconName::X,
                            text::REMOVE_ATTACHMENT,
                        )
                        .small()
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.remove_attachment(index, cx)),
                        ),
                    )
            });
        Some(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(space::XS)
                .px(space::MD)
                .pt(space::SM)
                .children(chips)
                .into_any_element(),
        )
    }

    fn render_toolbar(&self, busy: bool, ready: bool, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let preference = self
            .services
            .settings()
            .read(|settings| settings.composer.delivery);
        let hint = match (busy, self.enter_sends, preference) {
            (false, true, _) => text::HINT_IDLE,
            (false, false, _) => text::HINT_IDLE_CTRL,
            (true, _, DeliveryPref::Steer) => text::HINT_STEER,
            (true, _, DeliveryPref::Queue) => text::HINT_QUEUE,
        };
        let mut bar = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .px(space::SM)
            .pb(space::SM)
            .child(
                IconButton::new("composer-attach", IconName::Plus, text::ATTACH)
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.choose_files(cx))),
            )
            .child(self.agent_picker.clone())
            .child(self.model_picker.clone())
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(type_scale::CAPTION)
                    .text_color(palette.text.muted)
                    .child(hint),
            );
        if busy {
            if ready {
                let tooltip = if preference == DeliveryPref::Steer {
                    text::STEER
                } else {
                    text::QUEUE
                };
                bar = bar.child(
                    IconButton::new("composer-steer", IconName::Send, tooltip)
                        .small()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.submit(false, window, cx)),
                        ),
                );
            }
            bar = bar.child(
                Button::new("composer-stop")
                    .label(text::STOP)
                    .icon(IconName::Stop)
                    .danger()
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.stop(cx))),
            );
        } else {
            bar = bar.child(
                Button::new("composer-send")
                    .label(text::SEND)
                    .icon(IconName::Send)
                    .primary()
                    .small()
                    .disabled(!ready)
                    .on_click(cx.listener(|this, _, window, cx| this.submit(false, window, cx))),
            );
        }
        bar.into_any_element()
    }
}

impl Render for ComposerView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let busy = self.is_busy(cx);
        let has_model = self.has_model(cx);
        self.sync_input(busy, window, cx);
        let written = !self.text(cx).trim().is_empty() || !self.attachments.is_empty();
        let ready = has_model && written;
        let view = cx.weak_entity();
        let textarea = Textarea::new(&self.input)
            .appearance(false)
            .disabled(!has_model)
            .on_paste(move |item, _, cx| {
                // Why: discard-ok: a closed composer has nowhere to put the paste, so the text
                // goes in as usual.
                view.update(cx, |this, cx| this.paste(item, cx))
                    .unwrap_or(false)
            });
        let input = Self::capture_keys(div().px(space::MD).pt(space::SM), cx).child(textarea);
        let card = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .w_full()
            .rounded(radius::XL)
            .border_1()
            .border_color(palette.border.strong)
            .bg(palette.surface.raised)
            .children(self.render_chips(cx))
            .child(input)
            .child(self.render_toolbar(busy, ready, cx));
        let notice = (!has_model).then(|| {
            div()
                .px(space::SM)
                .text_size(type_scale::SMALL)
                .text_color(palette.tone(Tone::Warning).fg)
                .child(text::NO_MODEL)
        });
        let drop_tint = palette.accent.subtle;
        div()
            .id("composer")
            .flex()
            .flex_col()
            .gap(space::XS)
            .w_full()
            .px(space::LG)
            .pt(space::SM)
            .pb(space::LG)
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.bg(drop_tint))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.attach_paths(paths.paths().to_vec(), cx);
            }))
            .children(self.render_popup(cx))
            .child(card)
            .children(notice)
    }
}
