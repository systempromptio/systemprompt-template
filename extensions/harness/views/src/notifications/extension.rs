//! The two shell extensions.
//!
//! The inbox screen of `notifications` with the unread bell as its titlebar
//! item, and the settings pane of `slack`, each present only while its
//! instance-side half provided a handle (the shell adds them only while the
//! instance admits the extension).

use std::rc::Rc;

use gpui_kit::{AnyView, App, AppContext as _, Window};
use systemprompt_harness::members::notifications::{NotificationsExtension, NotificationsHandle};
use systemprompt_harness::members::slack::{SlackExtension, SlackHandle};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::contract::{
    ExtensionMetadata, OpenScreen, Placement, ScreenSpec, SettingsPaneSpec, ShellContext,
    ShellExtension, TitlebarItemSpec,
};

use crate::notifications::bell::unread_bell;
use crate::notifications::inbox_view::InboxView;
use crate::notifications::slack_pane::SlackPane;
use crate::notifications::strings as text;

pub const INBOX_SCREEN: &str = "inbox";

pub const SLACK_PANE: &str = "slack";

/// The notifications inbox as a shell extension: one titlebar screen.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotificationsShell;

impl ShellExtension for NotificationsShell {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata::new(NotificationsExtension::ID, text::NOTIFICATIONS_NAME)
    }

    fn screens(&self, ctx: &ShellContext) -> Vec<ScreenSpec> {
        if !ctx.handles.contains::<NotificationsHandle>() {
            return Vec::new();
        }
        vec![ScreenSpec {
            key: INBOX_SCREEN,
            title: text::INBOX_TITLE,
            icon: IconName::Bell,
            placement: Placement::Titlebar,
            build: Rc::new(build_inbox),
        }]
    }

    fn titlebar_items(&self, ctx: &ShellContext) -> Vec<TitlebarItemSpec> {
        if !ctx.handles.contains::<NotificationsHandle>() {
            return Vec::new();
        }
        vec![TitlebarItemSpec {
            screen: INBOX_SCREEN,
            build: Rc::new(build_bell),
        }]
    }
}

/// Slack as a shell extension: one settings pane, depending on
/// `notifications`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SlackShell;

impl ShellExtension for SlackShell {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata::new(SlackExtension::ID, text::SLACK_NAME)
            .depends_on(NotificationsExtension::ID)
    }

    fn settings_panes(&self, ctx: &ShellContext) -> Vec<SettingsPaneSpec> {
        if !ctx.handles.contains::<SlackHandle>() {
            return Vec::new();
        }
        vec![SettingsPaneSpec {
            key: SLACK_PANE,
            title: text::SLACK_TITLE,
            icon: IconName::Plug,
            build: Rc::new(build_slack),
        }]
    }
}

pub(crate) fn notifications_handle(ctx: &ShellContext) -> NotificationsHandle {
    ctx.handles
        .get::<NotificationsHandle>()
        .map_or_else(NotificationsHandle::new, |handle| {
            NotificationsHandle::clone(&handle)
        })
}

fn build_inbox(ctx: &ShellContext, _window: &mut Window, cx: &mut App) -> AnyView {
    let handle = notifications_handle(ctx);
    let runtime = ctx.runtime.clone();
    cx.new(|cx| InboxView::new(handle, runtime, cx)).into()
}

fn build_bell(ctx: &ShellContext, open: OpenScreen, _window: &mut Window, cx: &mut App) -> AnyView {
    unread_bell(ctx, move |window, cx| open(window, cx), cx)
}

fn build_slack(ctx: &ShellContext, _window: &mut Window, cx: &mut App) -> AnyView {
    let handle = ctx
        .handles
        .get::<SlackHandle>()
        .map(|handle| SlackHandle::clone(&handle));
    cx.new(|cx| SlackPane::new(handle, cx)).into()
}
