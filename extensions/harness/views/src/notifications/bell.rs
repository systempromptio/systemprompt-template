//! The titlebar bell.
//!
//! The inbox's entry point with the unread count, read straight from the
//! notifications handle's feed. No count shows while the feed is withheld or
//! unserved.

use std::rc::Rc;

use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement as _, Render, Styled as _,
    Task, Window, div, px,
};
use systemprompt_harness::members::notifications::Inbox;
use systemprompt_harness_shared::components::{ActionHandler, Badge, IconButton};
use systemprompt_harness_shared::contract::ShellContext;
use systemprompt_harness_shared::{IconName, Tone};
use tokio::sync::watch;

use crate::notifications::extension::notifications_handle;
use crate::notifications::model::shows_badge;
use crate::notifications::{bridge, strings as text};

const BADGE_OFFSET: f32 = -4.0;

pub fn unread_bell(
    ctx: &ShellContext,
    on_open: impl Fn(&mut Window, &mut App) + 'static,
    cx: &mut App,
) -> AnyView {
    let handle = notifications_handle(ctx);
    let on_open: ActionHandler = Rc::new(on_open);
    cx.new(|cx| {
        let follow = bridge::follow(cx, handle.subscribe(), |_: &mut UnreadBell, cx| cx.notify());
        UnreadBell {
            feed: handle.subscribe(),
            on_open,
            _follow: follow,
        }
    })
    .into()
}

struct UnreadBell {
    feed: watch::Receiver<Inbox>,
    on_open: ActionHandler,
    _follow: Task<()>,
}

impl Render for UnreadBell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (status, unread) = {
            let inbox = self.feed.borrow();
            (inbox.status(), inbox.unread())
        };
        let on_open = Rc::clone(&self.on_open);
        div()
            .relative()
            .child(
                IconButton::new(
                    "notifications-bell",
                    IconName::Bell,
                    text::bell_tooltip(unread),
                )
                .on_click(move |_, window, cx| on_open(window, cx)),
            )
            .children(shows_badge(status, unread).then(|| {
                div()
                    .absolute()
                    .top(px(BADGE_OFFSET))
                    .right(px(BADGE_OFFSET))
                    .child(
                        Badge::new(text::badge(unread))
                            .tone(Tone::Accent)
                            .solid()
                            .pill(),
                    )
            }))
    }
}
