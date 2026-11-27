//! Pieces of the document preview: the revision list, share grants, labelled
//! facts and muted notes.

use super::{DriveView, OpenDocument};
use crate::model::Load;
use crate::strings as text;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
};
use harness_ui::components::{Badge, ListRow, Spinner};
use harness_ui::theme::{space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};
use sp_drive::view::{RevisionItem, ShareGrant};

pub(super) fn revisions(open: &OpenDocument, cx: &Context<DriveView>) -> AnyElement {
    let items: &Vec<RevisionItem> = match &open.revisions {
        Load::Ready(items) => items,
        Load::Loading | Load::Idle => return Spinner::new().into_any_element(),
        Load::Failed(failure) => return muted(text::failure(*failure), cx),
    };
    if items.is_empty() {
        return muted(text::NO_REVISIONS, cx);
    }
    let head = open
        .detail
        .ready()
        .map(|detail| detail.document.head_revision_id.clone());
    let now = chrono::Local::now();
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .children(items.iter().enumerate().map(|(index, revision)| {
            let author = if revision.actor_kind == "agent" {
                text::REVISION_BY_AGENT
            } else {
                text::REVISION_BY_PERSON
            };
            let title = revision
                .message
                .clone()
                .filter(|message| !message.trim().is_empty())
                .unwrap_or_else(|| author.to_owned());
            let mut row = ListRow::new(("drive-revision", index), title)
                .icon(if revision.actor_kind == "agent" {
                    IconName::Sparkle
                } else {
                    IconName::User
                })
                .subtitle(format!(
                    "{author} · {}",
                    text::when(&revision.created_at, now)
                ));
            if head.as_ref() == Some(&revision.id) {
                row = row.trailing(Badge::new(text::HEAD_REVISION).tone(Tone::Success));
            }
            row
        }))
        .into_any_element()
}

pub(super) fn grant_row(prefix: &'static str, index: usize, grant: &ShareGrant) -> ListRow {
    let name = grant
        .display_name
        .clone()
        .unwrap_or_else(|| grant.subject_value.clone());
    ListRow::new((prefix, index), name)
        .icon(IconName::User)
        .trailing(Badge::new(grant.level.clone()))
}

pub(super) fn fact(label: &'static str, value: &str, cx: &Context<DriveView>) -> AnyElement {
    let palette = tokens(cx);
    div()
        .flex()
        .flex_row()
        .gap(space::MD)
        .text_size(type_scale::BODY)
        .child(
            div()
                .w(gpui_kit::px(120.))
                .text_color(palette.text.secondary)
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .child(SharedString::from(value.to_owned())),
        )
        .into_any_element()
}

pub(super) fn muted(message: &'static str, cx: &Context<DriveView>) -> AnyElement {
    div()
        .text_color(tokens(cx).text.muted)
        .child(message)
        .into_any_element()
}
