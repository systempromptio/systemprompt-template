//! A picker's open surface: the search field, the option rows and the empty
//! state.

use super::popup::{
    EmptyCopy, LIST_HEIGHT, PickerHost, PickerOption, SURFACE_WIDTH, choose_value, move_active,
    set_open,
};
use gpui_kit::component::input::{Escape, MoveDown, MoveUp};
use gpui_kit::{
    AnyElement, ClickEvent, Context, FontWeight, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, div, px,
};
use systemprompt_harness_shared::components::{Badge, Divider, ListRow};
use systemprompt_harness_shared::theme::{size, space, text as type_scale, tokens};
use systemprompt_harness_shared::{Icon, IconName};

pub(super) fn render_surface<H: PickerHost>(
    host: &H,
    empty: &EmptyCopy,
    cx: &Context<H>,
) -> AnyElement {
    let popup = host.popup();
    let options = host.options(&popup.query, cx);
    let list = if options.is_empty() {
        render_empty(empty, cx)
    } else {
        let rows: Vec<AnyElement> = options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                render_row(
                    popup.ids.row,
                    index,
                    option,
                    popup.active == Some(index),
                    cx,
                )
            })
            .collect();
        div()
            .id(popup.ids.list)
            .flex()
            .flex_col()
            .max_h(px(LIST_HEIGHT))
            .overflow_y_scroll()
            .track_scroll(&popup.scroll)
            .children(rows)
            .into_any_element()
    };
    div()
        .flex()
        .flex_col()
        .gap(space::SM)
        .w(px(SURFACE_WIDTH))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .capture_action(cx.listener(|host: &mut H, _: &MoveUp, _, cx| {
            move_active(host, -1, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|host: &mut H, _: &MoveDown, _, cx| {
            move_active(host, 1, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|host: &mut H, _: &Escape, window, cx| {
            set_open(host, false, window, cx);
            cx.stop_propagation();
        }))
        .child(popup.field.clone())
        .child(Divider::horizontal())
        .child(list)
        .into_any_element()
}

pub(super) fn render_row<H: PickerHost>(
    id: &'static str,
    index: usize,
    option: PickerOption,
    active: bool,
    cx: &Context<H>,
) -> AnyElement {
    let palette = *tokens(cx);
    let leading = if option.current {
        Icon::new(IconName::Check)
            .size(size::ICON_SM)
            .color(palette.text.accent)
            .into_any_element()
    } else {
        div().size(size::ICON_SM).into_any_element()
    };
    let badges = div()
        .flex()
        .flex_row()
        .gap(space::XS)
        .children(option.badges.into_iter().map(Badge::new));
    let value = option.value;
    let mut row = ListRow::new((id, index), option.title)
        .leading(leading)
        .trailing(badges)
        .selected(active)
        .on_click(
            cx.listener(move |host: &mut H, _: &ClickEvent, window, cx| {
                choose_value(host, value.clone(), window, cx);
            }),
        );
    if let Some(detail) = option.detail {
        row = row.subtitle(detail);
    }
    row.into_any_element()
}

pub(super) fn render_empty<H: PickerHost>(empty: &EmptyCopy, cx: &Context<H>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_col()
        .gap(space::XS)
        .px(space::MD)
        .py(space::LG)
        .child(
            div()
                .text_size(type_scale::BODY)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text.primary)
                .child(empty.title.clone()),
        )
        .children(empty.body.clone().map(|body| {
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.secondary)
                .child(body)
        }))
        .into_any_element()
}
