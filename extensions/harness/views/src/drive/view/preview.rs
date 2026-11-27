//! The document pane: the name with its actions (attach to the composer, open
//! in the browser, refresh), the tabs, and the Preview, Revisions and Sharing
//! tabs. Comments are drawn by `comments`.

use crate::drive::items::{DriveFailure, ShareState};
use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div,
};
use systemprompt_harness_shared::components::{
    Badge, Banner, Button, EmptyState, IconButton, SectionHeader, Spinner,
};
use systemprompt_harness_shared::theme::{space, text as type_scale, tokens};
use systemprompt_harness_shared::{IconName, Markdown, Tone};

use super::preview_parts::{fact, grant_row, muted, revisions};
use super::{DriveView, OpenDocument, PreviewTab};
use crate::drive::model::{DriveLoad, PreviewText};
use crate::drive::strings as text;

impl DriveView {
    pub(super) fn render_preview(&self, cx: &Context<Self>) -> AnyElement {
        let Some(open) = &self.open else {
            return EmptyState::new(text::NOTHING_SELECTED_TITLE)
                .icon(IconName::File)
                .description(text::NOTHING_SELECTED_BODY)
                .into_any_element();
        };
        let body = match self.tab {
            PreviewTab::Preview => self.preview_body(open, cx),
            PreviewTab::Comments => self.render_comments(open, cx),
            PreviewTab::Revisions => revisions(open, cx),
            PreviewTab::Sharing => Self::sharing(open, cx),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(Self::header(open, cx))
            .child(self.tabs(open, cx))
            .child(
                div()
                    .id("drive-preview-body")
                    .flex_1()
                    .min_h_0()
                    .p(space::LG)
                    .overflow_y_scroll()
                    .child(body),
            )
            .into_any_element()
    }

    fn header(open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let palette = tokens(cx);
        let access = open
            .detail
            .ready()
            .map(|detail| detail.my_access.clone())
            .or_else(|| open.item.my_access.clone());
        let id = open.id.clone();
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .px(space::LG)
            .py(space::MD)
            .border_b_1()
            .border_color(palette.border.subtle)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(type_scale::TITLE)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(open.item.name.clone())),
            )
            .child(Badge::new(open.item.format.clone()))
            .children(access.map(|access| Badge::new(access).tone(Tone::Accent)))
            .child(
                IconButton::new("drive-attach", IconName::Send, text::ATTACH)
                    .small()
                    .on_click(
                        cx.listener(|this, _, window, cx| this.attach_to_composer(window, cx)),
                    ),
            )
            .child(
                IconButton::new(
                    "drive-open-browser",
                    IconName::ExternalLink,
                    text::OPEN_IN_BROWSER,
                )
                .small()
                .on_click(cx.listener(move |this, _, _, cx| this.open_in_browser(Some(&id), cx))),
            )
            .child(
                IconButton::new("drive-refresh-document", IconName::Sync, text::REFRESH)
                    .small()
                    .loading(open.detail.is_loading())
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_document(cx))),
            )
            .into_any_element()
    }

    fn tabs(&self, open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let palette = tokens(cx);
        let count = open.comments.ready().map(|threads| {
            threads
                .iter()
                .filter(|thread| !thread.is_resolved())
                .count()
        });
        div()
            .flex()
            .flex_row()
            .gap(space::XXS)
            .px(space::MD)
            .py(space::XS)
            .border_b_1()
            .border_color(palette.border.subtle)
            .children(PreviewTab::ALL.into_iter().enumerate().map(|(index, tab)| {
                let label = match (tab, count) {
                    (PreviewTab::Comments, Some(count)) if count > 0 => {
                        format!("{} {count}", tab.label())
                    },
                    _ => tab.label().to_owned(),
                };
                Button::new(("drive-tab", index))
                    .label(label)
                    .small()
                    .ghost()
                    .selected(self.tab == tab)
                    .on_click(cx.listener(move |this, _, _, cx| this.set_tab(tab, cx)))
            }))
            .into_any_element()
    }

    fn preview_body(&self, open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let preview: &PreviewText = match &open.preview {
            DriveLoad::Ready(preview) => preview,
            DriveLoad::Loading | DriveLoad::Idle => return Spinner::new().into_any_element(),
            DriveLoad::Failed(failure) => return self.unavailable(*failure, cx),
        };
        if preview.markdown.trim().is_empty() {
            return muted(text::PREVIEW_EMPTY, cx);
        }
        let id = open.id.clone();
        let view = cx.weak_entity();
        div()
            .flex()
            .flex_col()
            .gap(space::LG)
            .child(Markdown::new("drive-preview", &open.markdown).copy_button(text::COPY_CODE))
            .children(preview.truncated.then(|| {
                Banner::info("drive-preview-truncated", text::PREVIEW_TRUNCATED).action(
                    text::OPEN_IN_BROWSER,
                    move |_, cx| {
                        if view
                            .update(cx, |this, cx| this.open_in_browser(Some(&id), cx))
                            .is_err()
                        {
                            tracing::trace!("the browser was asked for after the Drive closed");
                        }
                    },
                )
            }))
            .into_any_element()
    }

    fn unavailable(&self, failure: DriveFailure, cx: &Context<Self>) -> AnyElement {
        let message = if failure == DriveFailure::Failed {
            text::PREVIEW_UNAVAILABLE
        } else {
            text::failure(failure)
        };
        let id = self.open.as_ref().map(|open| open.id.clone());
        EmptyState::new(message)
            .icon(IconName::Info)
            .action(
                Button::new("drive-preview-browser")
                    .label(text::OPEN_IN_BROWSER)
                    .icon(IconName::ExternalLink)
                    .small()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.open_in_browser(id.as_ref(), cx)),
                    ),
            )
            .into_any_element()
    }

    fn sharing(open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let access: &ShareState = match &open.access {
            DriveLoad::Ready(access) => access,
            DriveLoad::Loading | DriveLoad::Idle => return Spinner::new().into_any_element(),
            DriveLoad::Failed(failure) => return muted(text::failure(*failure), cx),
        };
        let id = open.id.clone();
        let facts = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(fact(text::OWNER, &access.owner, cx))
            .child(fact(text::YOUR_ACCESS, &access.my_access, cx))
            .children(
                access
                    .agent_cap
                    .as_ref()
                    .map(|cap| fact(text::AGENT_ACCESS, &cap.cap, cx)),
            );
        let mut column = div().flex().flex_col().gap(space::MD).child(facts);
        column =
            column.child(SectionHeader::new(text::PEOPLE_WITH_ACCESS).count(access.grants.len()));
        if access.grants.is_empty() {
            column = column.child(muted(text::NOT_SHARED, cx));
        }
        column = column.children(
            access
                .grants
                .iter()
                .enumerate()
                .map(|(index, grant)| grant_row("drive-grant", index, grant)),
        );
        if !access.inherited.is_empty() {
            column = column
                .child(SectionHeader::new(text::INHERITED_ACCESS).count(access.inherited.len()))
                .children(
                    access
                        .inherited
                        .iter()
                        .enumerate()
                        .map(|(index, grant)| grant_row("drive-inherited", index, grant)),
                );
        }
        column
            .child(
                Button::new("drive-manage-sharing")
                    .label(text::MANAGE_SHARING)
                    .icon(IconName::ExternalLink)
                    .small()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.open_in_browser(Some(&id), cx)),
                    ),
            )
            .into_any_element()
    }
}
