//! The Drive settings pane: the base folder of the local sync, whether the user
//! chose it or it is the default under Documents, the folder the sync writes
//! into now, and a native folder picker to choose another or go back to the
//! default. The choice is stored in the Drive's own preferences, which the
//! sync service follows; the pane reads them again on every change.

use std::path::PathBuf;

use gpui_kit::{
    AnyElement, App, AsyncApp, Context, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, PathPromptOptions, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, Task, WeakEntity, Window, div, px,
};
use systemprompt_drive::DriveStatus;
use systemprompt_harness::extension::ExtensionSettings;
use systemprompt_harness::members::drive::{DriveExtension, DriveHandle, DriveSettings};
use systemprompt_harness_shared::components::{Badge, Banner, Button, Card};
use systemprompt_harness_shared::contract::ShellContext;
use systemprompt_harness_shared::theme::{space, text as type_scale, tokens};
use systemprompt_harness_shared::{Icon, IconName, Tone};
use tokio::sync::watch;

use crate::drive::bridge;
use crate::drive::model::{SyncFolder, default_folder};
use crate::drive::strings::settings as text;

const PAGE_WIDTH: f32 = 760.0;

pub(crate) struct DriveSettingsPane {
    handle: DriveHandle,
    settings: ExtensionSettings,
    default: Option<PathBuf>,
    sync_status: watch::Receiver<DriveStatus>,
    picking: bool,
    failure: Option<&'static str>,
    tasks: Vec<Task<()>>,
    _follow: [Task<()>; 3],
}

impl DriveSettingsPane {
    pub(crate) fn new(handle: DriveHandle, ctx: &ShellContext, cx: &Context<Self>) -> Self {
        let settings = ctx.settings_for(&DriveExtension::ID);
        let notify = |_: &mut Self, cx: &mut Context<Self>| cx.notify();
        let follow = [
            bridge::follow(cx, settings.changes(), notify),
            bridge::follow(cx, handle.sync().subscribe(), notify),
            bridge::follow(cx, ctx.enablement.subscribe(), notify),
        ];
        Self {
            sync_status: handle.sync().subscribe(),
            default: default_folder(ctx.paths.documents.as_deref(), ctx.identity.product()),
            handle,
            settings,
            picking: false,
            failure: None,
            tasks: Vec::new(),
            _follow: follow,
        }
    }

    fn folder(&self) -> SyncFolder {
        let preference = self.settings.get::<DriveSettings>();
        if let Err(error) = &preference {
            tracing::debug!(error = %error, "the Drive folder preference does not decode");
        }
        SyncFolder::resolve(preference.as_ref().cloned(), self.default.as_deref())
    }

    fn choose(&mut self, cx: &mut Context<Self>) {
        if self.picking {
            return;
        }
        self.picking = true;
        self.failure = None;
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(SharedString::new_static(text::CHOOSE_PROMPT)),
        });
        let task = cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let picked = match chosen.await {
                Ok(Ok(paths)) => Ok(paths.and_then(|paths| paths.into_iter().next())),
                Ok(Err(error)) => {
                    tracing::warn!(error = %error, "the Drive folder picker failed");
                    Err(())
                },
                Err(_) => Ok(None),
            };
            let applied = this.update(cx, |this, cx| {
                this.picking = false;
                match picked {
                    Ok(Some(folder)) => this.store(Some(folder), cx),
                    Ok(None) => cx.notify(),
                    Err(()) => {
                        this.failure = Some(text::PICKER_FAILED);
                        cx.notify();
                    },
                }
            });
            if applied.is_err() {
                tracing::trace!("a Drive folder was chosen after the pane closed");
            }
        });
        self.tasks.retain(|task| !task.is_ready());
        self.tasks.push(task);
        cx.notify();
    }

    fn store(&mut self, folder: Option<PathBuf>, cx: &mut Context<Self>) {
        match self.settings.set(&DriveSettings { folder }) {
            Ok(()) => self.failure = None,
            Err(error) => {
                tracing::warn!(error = %error, "the Drive folder was not saved");
                self.failure = Some(text::SAVE_FAILED);
            },
        }
        cx.notify();
    }

    fn render_banners(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let mut banners = Vec::new();
        if self.handle.is_withheld() {
            banners
                .push(Banner::info("drive-settings-withheld", text::WITHHELD).into_any_element());
        }
        if let Some(message) = self.failure {
            let view = cx.weak_entity();
            banners.push(
                Banner::error("drive-settings-failed", message)
                    .dismissible(text::DISMISS, move |_, cx| {
                        let dismissed = view.update(cx, |this, cx| {
                            this.failure = None;
                            cx.notify();
                        });
                        if dismissed.is_err() {
                            tracing::trace!("a failure was dismissed after the pane closed");
                        }
                    })
                    .into_any_element(),
            );
        }
        banners
    }

    fn render_folder(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let folder = self.folder();
        let badge = match &folder {
            SyncFolder::Chosen(_) => Some(Badge::new(text::CHOSEN).tone(Tone::Accent)),
            SyncFolder::Default(_) => Some(Badge::new(text::DEFAULT)),
            SyncFolder::Unknown | SyncFolder::Unreadable => None,
        };
        let path = folder.path().map(|path| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .child(Icon::new(IconName::Folder).color(palette.text.secondary))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(type_scale::BODY)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.text.primary)
                        .child(path.display().to_string()),
                )
                .children(badge)
        });
        let in_use = self
            .sync_status
            .borrow()
            .root
            .clone()
            .map(|root| caption(format!("{} {}", text::IN_USE, root.display()), cx));
        let actions = div()
            .flex()
            .flex_row()
            .gap(space::SM)
            .child(
                Button::new("drive-settings-choose")
                    .label(text::CHOOSE)
                    .icon(IconName::Folder)
                    .primary()
                    .small()
                    .loading(self.picking)
                    .on_click(cx.listener(|this, _, _, cx| this.choose(cx))),
            )
            .child(
                Button::new("drive-settings-default")
                    .label(text::USE_DEFAULT)
                    .ghost()
                    .small()
                    .disabled(!folder.can_reset())
                    .on_click(cx.listener(|this, _, _, cx| this.store(None, cx))),
            );
        Card::new()
            .title(text::FOLDER_TITLE)
            .children(path)
            .child(paragraph(text::folder_state(&folder), cx))
            .child(caption(text::FOLDER_NOTE, cx))
            .children(in_use)
            .child(actions)
            .into_any_element()
    }
}

fn paragraph(message: &'static str, cx: &App) -> AnyElement {
    div()
        .text_size(type_scale::BODY)
        .text_color(tokens(cx).text.secondary)
        .child(message)
        .into_any_element()
}

fn caption(message: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .text_size(type_scale::SMALL)
        .text_color(tokens(cx).text.muted)
        .child(message.into())
        .into_any_element()
}

impl Render for DriveSettingsPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = *tokens(cx);
        let header = div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(
                div()
                    .text_size(type_scale::HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.primary)
                    .child(text::TITLE),
            )
            .child(paragraph(text::DESCRIPTION, cx));
        div()
            .id("settings-drive")
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::XL)
                    .w_full()
                    .max_w(px(PAGE_WIDTH))
                    .p(space::XXL)
                    .child(header)
                    .children(self.render_banners(cx))
                    .child(self.render_folder(cx)),
            )
    }
}
