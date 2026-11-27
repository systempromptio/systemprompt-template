//! The custom titlebar: sidebar toggle, the app's name and channel, the project
//! switcher, and on the right the entry points of the admitted extensions'
//! titlebar screens and the governance pill. The toolkit's titlebar draws the
//! window controls and moves the window.

use std::collections::HashMap;
use std::rc::Rc;

use gpui_kit::component::TitleBar;
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Subscription, Window,
    div,
};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::components::{Badge, IconButton, tooltip_with_keys};
use systemprompt_harness_shared::contract::ScreenKey;
use systemprompt_harness_shared::theme::{space, text as type_scale, tokens};

use crate::app::Services;
use crate::governance::GovernancePill;
use crate::keymap::TOGGLE_SIDEBAR_KEYS;
use crate::state::AppState;
use crate::state::extensions::Contributions;
use crate::strings::foundation as text;
use crate::{extension_entries, projects};

pub(crate) struct TitleBarView {
    app: Entity<AppState>,
    services: Services,
    pill: Entity<GovernancePill>,
    items: HashMap<ScreenKey, AnyView>,
    items_from: Option<Rc<Contributions>>,
    _observation: Subscription,
}

impl std::fmt::Debug for TitleBarView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TitleBarView").finish_non_exhaustive()
    }
}

impl TitleBarView {
    pub(crate) fn new(
        app: Entity<AppState>,
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let pill = cx.new(|cx| GovernancePill::new(app.clone(), services.clone(), window, cx));
        let observation = cx.observe(&app, |_, _, cx| cx.notify());
        Self {
            app,
            services,
            pill,
            items: HashMap::new(),
            items_from: None,
            _observation: observation,
        }
    }
}

impl TitleBarView {
    fn follow_contributions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let contributions = Rc::clone(&self.app.read(cx).contributions);
        if self
            .items_from
            .as_ref()
            .is_some_and(|from| Rc::ptr_eq(from, &contributions))
        {
            return;
        }
        self.items = extension_entries::titlebar_items(
            &self.app,
            &contributions,
            self.services.shell(),
            window,
            cx,
        );
        self.items_from = Some(contributions);
    }
}

impl Render for TitleBarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.follow_contributions(window, cx);
        let palette = *tokens(cx);
        let identity = self.services.identity();
        let app = self.app.clone();
        let toggle = IconButton::new(
            "toggle-sidebar",
            IconName::ChevronLeft,
            text::TOGGLE_SIDEBAR,
        )
        .small()
        .on_click(move |_, _, cx| app.update(cx, AppState::toggle_sidebar));
        let toggle = div()
            .id("toggle-sidebar-hint")
            .tooltip(tooltip_with_keys(text::TOGGLE_SIDEBAR, TOGGLE_SIDEBAR_KEYS))
            .child(toggle);
        let left = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .child(toggle)
            .child(
                div()
                    .text_size(type_scale::BODY)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.primary)
                    .child(identity.display_name()),
            )
            .children(
                identity
                    .channel_label()
                    .map(|label| Badge::new(label).pill()),
            )
            .child(projects::switcher(&self.app, cx));
        let right = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .pr(space::MD)
            .children(extension_entries::titlebar_entries(
                &self.app,
                &self.items,
                cx,
            ))
            .child(self.pill.clone());
        TitleBar::new().child(left).child(right)
    }
}
