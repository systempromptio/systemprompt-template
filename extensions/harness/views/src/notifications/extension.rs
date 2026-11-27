//! The two shell extensions.
//!
//! The inbox screen of `notifications` with the unread bell as its titlebar
//! item, and the settings pane of `slack`, each present only while its
//! instance-side half provided a handle (the shell adds them only while the
//! instance admits the extension).
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::rc::Rc;

use gpui_kit::{AnyView, App, AppContext as _, Window};
use harness_shell::{
    ExtensionMetadata, OpenScreen, Placement, ScreenSpec, SettingsPaneSpec, ShellContext,
    ShellExtension, TitlebarItemSpec, register_shell_extension,
};
use harness_ui::IconName;
use sp_notifications::{NotificationsExtension, NotificationsHandle};
use sp_slack::{SlackExtension, SlackHandle};

use crate::bell::unread_bell;
use crate::inbox_view::InboxView;
use crate::slack_pane::SlackPane;
use crate::strings as text;

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

register_shell_extension!(NotificationsShell);

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

register_shell_extension!(SlackShell);

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
