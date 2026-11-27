//! The name prompts behind New folder and New document: a dialog with one text
//! field, Enter to create.

use gpui_kit::{App, AppContext as _, Context, Entity, Window};
use systemprompt_harness::members::drive::DocumentKind;
use systemprompt_harness_shared::components::{ButtonKind, DialogAction, DialogSpec, TextField};

use super::DriveView;
use crate::drive::strings as text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NewItem {
    Folder,
    Document(DocumentKind),
}

impl NewItem {
    const fn title(self) -> &'static str {
        match self {
            Self::Folder => text::NEW_FOLDER_TITLE,
            Self::Document(kind) => text::kind(kind),
        }
    }

    const fn placeholder(self) -> &'static str {
        match self {
            Self::Folder => text::NEW_FOLDER_PLACEHOLDER,
            Self::Document(_) => text::NEW_DOCUMENT_PLACEHOLDER,
        }
    }
}

impl DriveView {
    pub(super) fn prompt_new(item: NewItem, window: &mut Window, cx: &mut Context<Self>) {
        let field: Entity<TextField> = cx.new(|cx| {
            TextField::new(window, cx)
                .label(text::NAME_LABEL)
                .placeholder(item.placeholder())
        });
        let view = cx.weak_entity();
        let confirm_field = field.clone();
        let confirm = DialogAction::new(text::CREATE, move |_window: &mut Window, cx: &mut App| {
            let name = confirm_field.read(cx).text(cx).trim().to_owned();
            if name.is_empty() {
                return false;
            }
            let created = view.update(cx, |this, cx| match item {
                NewItem::Folder => this.create_folder(&name, cx),
                NewItem::Document(kind) => this.create_document(kind, &name, cx),
            });
            if created.is_err() {
                tracing::trace!("a name was confirmed after the Drive closed");
            }
            true
        })
        .kind(ButtonKind::Primary);
        let body_field = field.clone();
        DialogSpec::new(item.title())
            .body(move |_, _| body_field.clone())
            .confirm(confirm)
            .cancel(DialogAction::new(text::CANCEL, |_, _| true))
            .open(window, cx);
        field.update(cx, |field, cx| field.focus(window, cx));
    }
}
