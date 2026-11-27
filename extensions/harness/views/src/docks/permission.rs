//! The permission dock: which tool asks for what, the command, diff, address or
//! path it would act on, what an "always" answer would cover, and Allow once,
//! Always allow and Reject with optional feedback.

use std::sync::Arc;

use gpui_kit::{
    AnyElement, Context, Div, FontWeight, IntoElement, ParentElement as _, SharedString,
    Styled as _, div, px,
};
use systemprompt_engine::run::{PermissionAsk, PermissionId, Reply};
use systemprompt_harness_shared::components::{Badge, Button, KeyHint};
use systemprompt_harness_shared::theme::{
    mono_font, radius, size, space, text as type_scale, tokens,
};
use systemprompt_harness_shared::{CodeBlock, DiffView, Icon, IconName, Patch, Tone};

use crate::docks::DocksView;
use crate::docks::permission_info::{self, Preview};
use crate::strings::docks as text;

const PREVIEW_HEIGHT: f32 = 240.0;

pub(super) fn dock_card(cx: &Context<DocksView>) -> Div {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_col()
        .gap(space::MD)
        .w_full()
        .p(space::LG)
        .rounded(radius::XL)
        .border_1()
        .border_color(palette.tone(Tone::Accent).border)
        .bg(palette.surface.raised)
}

pub(super) fn dock_title(title: &'static str, waiting: usize, cx: &Context<DocksView>) -> Div {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::SM)
        .child(
            Icon::new(IconName::Shield)
                .size(size::ICON_MD)
                .color(palette.text.accent),
        )
        .child(
            div()
                .flex_1()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text.primary)
                .child(title),
        )
        .children((waiting > 0).then(|| Badge::new(text::waiting(waiting)).pill()))
}

pub(super) fn with_key(button: Button, key: &'static str) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::XS)
        .child(button)
        .child(KeyHint::new([key]))
}

impl DocksView {
    pub(super) fn render_permission(
        &mut self,
        ask: &PermissionAsk,
        waiting: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        let mono = mono_font(cx);
        let mut card = dock_card(cx).child(
            dock_title(text::PERMISSION_TITLE, waiting, cx).child(Badge::new(ask.tool.clone())),
        );
        card = card.child(
            div()
                .text_size(type_scale::BODY)
                .text_color(palette.text.primary)
                .child(SharedString::from(ask.title.clone())),
        );
        if !ask.patterns.is_empty() {
            let chips = ask.patterns.iter().map(|pattern| {
                div()
                    .px(space::SM)
                    .rounded(radius::SM)
                    .bg(palette.surface.sunken)
                    .font_family(mono.clone())
                    .text_size(type_scale::MONO)
                    .child(SharedString::from(pattern.clone()))
            });
            card = card.child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(space::XS)
                    .text_size(type_scale::SMALL)
                    .text_color(palette.text.muted)
                    .child(text::PATTERNS)
                    .children(chips),
            );
        }
        if let Some(preview) = permission_info::preview(&ask.metadata) {
            card = card.child(self.render_preview(&ask.id, preview, cx));
        }
        card = card.child(
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .child(SharedString::from(text::always_covers(
                    &ask.permission,
                    &ask.always,
                ))),
        );
        card.child(self.render_permission_actions(ask, cx))
            .into_any_element()
    }

    fn render_preview(
        &mut self,
        id: &PermissionId,
        preview: Preview,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = *tokens(cx);
        match preview {
            Preview::Command(command) => CodeBlock::new("dock-command", command)
                .max_height(px(PREVIEW_HEIGHT))
                .into_any_element(),
            Preview::Diff(source) => {
                let patch = self.diff_for(id, &source);
                DiffView::new("dock-diff", patch)
                    .max_height(px(PREVIEW_HEIGHT))
                    .into_any_element()
            },
            Preview::Url(value) | Preview::Path(value) => div()
                .font_family(mono_font(cx))
                .text_size(type_scale::MONO)
                .text_color(palette.text.secondary)
                .truncate()
                .child(SharedString::from(value))
                .into_any_element(),
        }
    }

    fn diff_for(&mut self, id: &PermissionId, source: &str) -> Arc<Patch> {
        if let Some((known, patch)) = &self.diff
            && known == id
        {
            return Arc::clone(patch);
        }
        let patch = Arc::new(Patch::parse(source));
        self.diff = Some((id.clone(), Arc::clone(&patch)));
        patch
    }

    fn render_rejection(&self, ask: &PermissionAsk, cx: &Context<Self>) -> AnyElement {
        let send = ask.id.clone();
        let buttons = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .child(with_key(
                Button::new("dock-send-rejection")
                    .label(text::SEND_REJECTION)
                    .danger()
                    .small()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.reject_with_feedback(&send, cx)),
                    ),
                text::KEY_ENTER,
            ))
            .child(with_key(
                Button::new("dock-cancel-rejection")
                    .label(text::CANCEL)
                    .ghost()
                    .small()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.rejecting = false;
                        this.focus.focus(window, cx);
                        cx.notify();
                    })),
                text::KEY_REJECT,
            ));
        div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .child(self.feedback.clone())
            .child(buttons)
            .into_any_element()
    }

    fn render_permission_actions(&self, ask: &PermissionAsk, cx: &Context<Self>) -> AnyElement {
        if self.rejecting {
            return self.render_rejection(ask, cx);
        }
        let (once, always, reject) = (ask.id.clone(), ask.id.clone(), ask.id.clone());
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(space::MD)
            .child(with_key(
                Button::new("dock-allow-once")
                    .label(text::ALLOW_ONCE)
                    .primary()
                    .small()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.reply(&once, Reply::Once, cx)),
                    ),
                text::KEY_ENTER,
            ))
            .child(with_key(
                Button::new("dock-allow-always")
                    .label(text::ALLOW_ALWAYS)
                    .small()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.reply(&always, Reply::Always, cx)),
                    ),
                text::KEY_ALWAYS,
            ))
            .child(with_key(
                Button::new("dock-reject")
                    .label(text::REJECT)
                    .ghost()
                    .small()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.reply(&reject, Reply::Reject { feedback: None }, cx);
                    })),
                text::KEY_REJECT,
            ))
            .child(with_key(
                Button::new("dock-reject-feedback")
                    .label(text::REJECT_WITH_FEEDBACK)
                    .ghost()
                    .small()
                    .on_click(cx.listener(|this, _, window, cx| this.start_rejecting(window, cx))),
                text::KEY_FEEDBACK,
            ))
            .into_any_element()
    }
}
