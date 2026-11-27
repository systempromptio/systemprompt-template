//! The popover both pickers share: a trigger showing the current choice, a
//! search field and a list of options. Up and Down move the highlight, Enter
//! chooses it, Escape closes; the list scrolls to follow the highlight.
//!
//! A picker supplies its options through `PickerHost`; this module owns
//! opening, filtering, the highlight and the layout, so both pickers behave the
//! same.

use super::popup_render::render_surface;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button as ToolkitButton, ButtonVariants as _};
use gpui_kit::component::popover::Popover;
use gpui_kit::{
    Anchor, AnyElement, App, AppContext as _, Context, Entity, Focusable as _, IntoElement,
    ParentElement as _, ScrollHandle, SharedString, Window,
};
use systemprompt_harness_shared::components::TextField;

use crate::pickers::search;

pub(super) const SURFACE_WIDTH: f32 = 380.0;
pub(super) const LIST_HEIGHT: f32 = 320.0;

#[derive(Debug, Clone, Copy)]
pub(super) struct PickerIds {
    pub(super) popover: &'static str,
    pub(super) trigger: &'static str,
    pub(super) list: &'static str,
    pub(super) row: &'static str,
}

#[derive(Debug, Clone)]
pub(super) struct PickerOption {
    pub(super) value: Option<String>,
    pub(super) title: SharedString,
    pub(super) detail: Option<SharedString>,
    pub(super) badges: Vec<SharedString>,
    pub(super) current: bool,
}

#[derive(Debug, Clone)]
pub(super) struct EmptyCopy {
    pub(super) title: SharedString,
    pub(super) body: Option<SharedString>,
}

#[derive(Debug, Clone)]
pub(super) struct TriggerCopy {
    pub(super) label: SharedString,
    pub(super) tooltip: &'static str,
}

pub(super) trait PickerHost: Sized + 'static {
    fn popup(&self) -> &PickerPopup;

    fn popup_mut(&mut self) -> &mut PickerPopup;

    fn options(&self, query: &str, cx: &App) -> Vec<PickerOption>;

    fn choose(&mut self, value: Option<String>, cx: &mut Context<Self>);
}

pub(super) struct PickerPopup {
    pub(super) ids: PickerIds,
    open: bool,
    pub(super) field: Entity<TextField>,
    pub(super) query: String,
    pub(super) active: Option<usize>,
    pub(super) scroll: ScrollHandle,
}

impl std::fmt::Debug for PickerPopup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PickerPopup")
            .field("open", &self.open)
            .field("active", &self.active)
            .finish_non_exhaustive()
    }
}

impl PickerPopup {
    pub(super) fn new<H: PickerHost>(
        ids: PickerIds,
        placeholder: &'static str,
        window: &mut Window,
        cx: &mut Context<H>,
    ) -> Self {
        let changed = cx.weak_entity();
        let submitted = cx.weak_entity();
        let field = cx.new(|cx| {
            TextField::new(window, cx)
                .placeholder(placeholder)
                .on_change(move |value, _, cx| {
                    let query = value.to_string();
                    if changed
                        .update(cx, |host, cx| query_changed(host, query, cx))
                        .is_err()
                    {
                        tracing::trace!("picker closed before its search changed");
                    }
                })
                .on_submit(move |_, window, cx| {
                    let host = submitted.clone();
                    window.defer(cx, move |window, cx| {
                        if host
                            .update(cx, |host, cx| choose_active(host, window, cx))
                            .is_err()
                        {
                            tracing::trace!("picker closed before its choice ran");
                        }
                    });
                })
        });
        Self {
            ids,
            open: false,
            field,
            query: String::new(),
            active: None,
            scroll: ScrollHandle::new(),
        }
    }
}

fn query_changed<H: PickerHost>(host: &mut H, query: String, cx: &mut Context<H>) {
    let count = host.options(&query, cx).len();
    let popup = host.popup_mut();
    popup.query = query;
    popup.active = search::initial(None, count);
    popup.scroll.scroll_to_item(0);
    cx.notify();
}

pub(super) fn set_open<H: PickerHost>(
    host: &mut H,
    open: bool,
    window: &mut Window,
    cx: &mut Context<H>,
) {
    if host.popup().open == open {
        return;
    }
    host.popup_mut().open = open;
    if open {
        let field = host.popup().field.clone();
        field.update(cx, |field, cx| {
            field.set_text(SharedString::default(), window, cx);
            field.focus(window, cx);
        });
        let options = host.options("", cx);
        let current = options.iter().position(|option| option.current);
        let popup = host.popup_mut();
        popup.query.clear();
        popup.active = search::initial(current, options.len());
        if let Some(index) = popup.active {
            popup.scroll.scroll_to_item(index);
        }
    }
    cx.notify();
}

pub(super) fn move_active<H: PickerHost>(host: &mut H, delta: isize, cx: &mut Context<H>) {
    let count = host.options(&host.popup().query, cx).len();
    let popup = host.popup_mut();
    popup.active = search::step(popup.active, count, delta);
    if let Some(index) = popup.active {
        popup.scroll.scroll_to_item(index);
    }
    cx.notify();
}

fn choose_active<H: PickerHost>(host: &mut H, window: &mut Window, cx: &mut Context<H>) {
    let options = host.options(&host.popup().query, cx);
    let Some(option) = host
        .popup()
        .active
        .and_then(|index| options.into_iter().nth(index))
    else {
        return;
    };
    choose_value(host, option.value, window, cx);
}

pub(super) fn choose_value<H: PickerHost>(
    host: &mut H,
    value: Option<String>,
    window: &mut Window,
    cx: &mut Context<H>,
) {
    set_open(host, false, window, cx);
    host.choose(value, cx);
}

pub(super) fn render<H: PickerHost>(
    host: &H,
    trigger: TriggerCopy,
    empty: &EmptyCopy,
    cx: &Context<H>,
) -> AnyElement {
    let popup = host.popup();
    let focus = popup.field.read(cx).focus_handle(cx);
    let toggled = cx.weak_entity();
    let button = ToolkitButton::new(popup.ids.trigger)
        .ghost()
        .small()
        .label(trigger.label)
        .tooltip(trigger.tooltip)
        .dropdown_caret(true);
    let mut popover = Popover::new(popup.ids.popover)
        .anchor(Anchor::BottomLeft)
        .open(popup.open)
        .track_focus(&focus)
        .on_open_change(move |open, window, cx| {
            if toggled
                .update(cx, |host, cx| set_open(host, *open, window, cx))
                .is_err()
            {
                tracing::trace!("picker closed before it opened");
            }
        })
        .trigger(button);
    if popup.open {
        popover = popover.child(render_surface(host, empty, cx));
    }
    popover.into_any_element()
}
