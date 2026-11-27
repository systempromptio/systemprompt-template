//! The browsing panes: the scope switch and folder tree on the left, and the
//! document list with its search, sort, New, Import and Refresh controls in the
//! middle. The list takes the arrow keys, Enter and Escape.

use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use harness_ui::components::{
    Activate, Button, Dismiss, IconButton, LIST_CONTEXT, ListRow, Menu, MenuEntry, MenuItem,
    SectionHeader, Select, SelectNext, SelectOption, SelectPrevious, Spinner,
};
use harness_ui::theme::{space, tokens};
use harness_ui::{Icon, IconName};
use sp_drive::DocumentKind;

use super::DriveView;
use super::dialogs::NewItem;
use crate::model::{BrowseScope, Load, SortKey, folder_rows};
use crate::strings as text;

impl DriveView {
    pub(super) fn render_folders(&self, cx: &Context<Self>) -> AnyElement {
        let scopes = div()
            .flex()
            .flex_row()
            .gap(space::XXS)
            .p(space::SM)
            .children(
                BrowseScope::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(index, scope)| {
                        Button::new(("drive-scope", index))
                            .label(text::scope(scope))
                            .small()
                            .ghost()
                            .selected(self.browse.folder.is_none() && self.browse.scope == scope)
                            .on_click(cx.listener(move |this, _, _, cx| this.set_scope(scope, cx)))
                    }),
            );
        let all = ListRow::new("drive-folder-all", text::ALL_DOCUMENTS)
            .icon(IconName::Cloud)
            .selected(self.browse.folder.is_none())
            .on_click(cx.listener(|this, _, _, cx| this.select_folder(None, cx)));
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(scopes)
            .child(SectionHeader::new(text::FOLDERS))
            .child(
                div()
                    .id("drive-folders")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .px(space::XS)
                    .overflow_y_scroll()
                    .child(all)
                    .children(self.folder_tree(cx)),
            )
            .into_any_element()
    }

    fn folder_tree(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let tree = match &self.browse.folders {
            Load::Ready(tree) => tree,
            Load::Loading => {
                return vec![div().p(space::MD).child(Spinner::new()).into_any_element()];
            },
            Load::Failed(_) | Load::Idle => return Vec::new(),
        };
        if tree.is_empty() {
            let palette = tokens(cx);
            return vec![
                div()
                    .px(space::MD)
                    .py(space::SM)
                    .text_color(palette.text.muted)
                    .child(text::NO_FOLDERS)
                    .into_any_element(),
            ];
        }
        let palette = *tokens(cx);
        folder_rows(tree, &self.browse.expanded)
            .into_iter()
            .enumerate()
            .map(|(index, row)| {
                let leading = div()
                    .flex()
                    .pl(px(row.depth as f32 * 12.0))
                    .child(Icon::new(IconName::Folder).color(palette.text.secondary));
                let target = row.id.clone();
                let mut item = ListRow::new(("drive-folder", index), row.name.clone())
                    .leading(leading)
                    .selected(self.browse.folder.as_ref() == Some(&row.id))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_folder(Some(target.clone()), cx);
                    }));
                if row.has_children {
                    let (icon, tooltip) = if row.expanded {
                        (IconName::ChevronDown, text::COLLAPSE_FOLDER)
                    } else {
                        (IconName::ChevronRight, text::EXPAND_FOLDER)
                    };
                    let toggled = row.id;
                    item = item.trailing(
                        IconButton::new(("drive-folder-toggle", index), icon, tooltip)
                            .small()
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.toggle_folder(&toggled, cx)),
                            ),
                    );
                }
                item.into_any_element()
            })
            .collect()
    }

    pub(super) fn render_list(&self, cx: &Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.toolbar(cx))
            .children(self.render_upload(cx))
            .child(
                div()
                    .id("drive-documents")
                    .key_context(LIST_CONTEXT)
                    .track_focus(&self.list_focus)
                    .on_action(
                        cx.listener(|this, _: &SelectNext, _, cx| this.step_selection(true, cx)),
                    )
                    .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| {
                        this.step_selection(false, cx);
                    }))
                    .on_action(cx.listener(|this, _: &Activate, window, cx| {
                        this.attach_to_composer(window, cx);
                    }))
                    .on_action(cx.listener(|this, _: &Dismiss, window, cx| {
                        this.search.update(cx, |field, cx| field.focus(window, cx));
                    }))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .p(space::XS)
                    .overflow_y_scroll()
                    .child(self.documents(cx)),
            )
            .into_any_element()
    }

    fn toolbar(&self, cx: &Context<Self>) -> AnyElement {
        let palette = tokens(cx);
        let sorts = SortKey::ALL
            .into_iter()
            .map(|sort| SelectOption::new(sort.key(), sort_label(sort)));
        let sort = Select::new("drive-sort", sorts)
            .selected(Some(SharedString::new_static(self.browse.sort.key())))
            .small()
            .on_select(cx.listener(|this, value: &SharedString, _, cx| {
                if let Some(sort) = SortKey::from_key(value) {
                    this.set_sort(sort, cx);
                }
            }));
        let controls = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::XS)
            .child(Self::new_menu(cx))
            .child(
                IconButton::new("drive-import", IconName::Plus, text::IMPORT)
                    .small()
                    .disabled(self.upload.is_some())
                    .on_click(cx.listener(|this, _, _, cx| this.pick_import(cx))),
            )
            .child(div().flex_1())
            .child(sort)
            .child(
                IconButton::new("drive-refresh", IconName::Sync, text::REFRESH)
                    .small()
                    .loading(self.browse.documents.is_loading())
                    .on_click(cx.listener(|this, _, _, cx| this.reload_all(cx))),
            );
        div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .p(space::SM)
            .border_b_1()
            .border_color(palette.border.subtle)
            .child(self.search.clone())
            .child(controls)
            .into_any_element()
    }

    fn new_menu(cx: &Context<Self>) -> Menu {
        let view = cx.weak_entity();
        let entry = move |item: NewItem, label: &'static str, icon: IconName| -> MenuEntry {
            let view = view.clone();
            MenuItem::new(label, move |window, cx| {
                if view
                    .update(cx, |_, cx| Self::prompt_new(item, window, cx))
                    .is_err()
                {
                    tracing::trace!("a new item was asked for after the Drive closed");
                }
            })
            .icon(icon)
            .into()
        };
        let mut entries = vec![
            entry(NewItem::Folder, text::NEW_FOLDER, IconName::Folder),
            MenuEntry::Separator,
        ];
        entries.push(MenuEntry::Label(SharedString::new_static(
            text::NEW_DOCUMENT_HEADING,
        )));
        entries.extend(
            DocumentKind::ALL
                .into_iter()
                .map(|kind| entry(NewItem::Document(kind), text::kind(kind), IconName::File)),
        );
        Menu::labeled("drive-new", text::NEW, entries)
    }
}

const fn sort_label(sort: SortKey) -> &'static str {
    match sort {
        SortKey::Updated => text::SORT_UPDATED,
        SortKey::Name => text::SORT_NAME,
        SortKey::Size => text::SORT_SIZE,
    }
}
