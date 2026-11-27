//! The document list of the Drive: its rows and the state shown when the
//! listing failed.

use super::DriveView;
use crate::model::Load;
use crate::strings as text;
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div};
use harness_ui::IconName;
use harness_ui::components::{Button, EmptyState, ListRow, Spinner};
use harness_ui::theme::space;
use sp_drive::view::{DocumentItem, DriveFailure};

impl DriveView {
    pub(super) fn documents(&self, cx: &Context<Self>) -> AnyElement {
        let items = match &self.browse.documents {
            Load::Ready(items) => items,
            Load::Loading | Load::Idle => {
                return div().p(space::LG).child(Spinner::new()).into_any_element();
            },
            Load::Failed(DriveFailure::NotFound) => {
                return EmptyState::new(text::NO_DRIVE_TITLE)
                    .icon(IconName::Cloud)
                    .description(text::NO_DRIVE_BODY)
                    .into_any_element();
            },
            Load::Failed(failure) => return Self::listing_failure(*failure, cx),
        };
        if items.is_empty() {
            let searching = !self.browse.query.trim().is_empty();
            let state = if searching {
                EmptyState::new(text::NO_MATCHES)
            } else {
                EmptyState::new(text::NO_DOCUMENTS_TITLE).description(text::NO_DOCUMENTS_BODY)
            };
            return state.icon(IconName::File).into_any_element();
        }
        let more = self.browse.next_cursor.is_some().then(|| {
            Button::new("drive-load-more")
                .label(text::LOAD_MORE)
                .small()
                .ghost()
                .full_width()
                .on_click(cx.listener(|this, _, _, cx| this.load_more(cx)))
        });
        div()
            .flex()
            .flex_col()
            .gap(space::XXS)
            .children(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| self.document_row(index, item, cx)),
            )
            .children(more)
            .into_any_element()
    }

    pub(super) fn document_row(
        &self,
        index: usize,
        item: &DocumentItem,
        cx: &Context<Self>,
    ) -> AnyElement {
        let now = chrono::Local::now();
        let mut detail = text::when(&item.updated_at, now);
        if let Some(bytes) = item.size_bytes {
            detail.push_str(" · ");
            detail.push_str(&text::size(bytes));
        }
        let target = item.id.clone();
        ListRow::new(("drive-document", index), item.name.clone())
            .icon(IconName::File)
            .subtitle(detail)
            .tall()
            .selected(self.browse.selected.as_ref() == Some(&item.id))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.list_focus.focus(window, cx);
                this.select_document(target.clone(), cx);
            }))
            .into_any_element()
    }

    pub(super) fn listing_failure(failure: DriveFailure, cx: &Context<Self>) -> AnyElement {
        EmptyState::new(text::failure(failure))
            .icon(IconName::Alert)
            .action(
                Button::new("drive-retry")
                    .label(text::REFRESH)
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.reload_all(cx))),
            )
            .into_any_element()
    }
}
