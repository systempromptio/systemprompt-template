//! Importing a local file into the Drive: the file dialog, reading the file off
//! the UI thread, the upload with its progress and cancel, and the strip that
//! shows it.

use std::path::PathBuf;

use gpui_kit::{
    AnyElement, AsyncApp, Context, IntoElement, ParentElement as _, PathPromptOptions,
    SharedString, Styled as _, WeakEntity, div,
};
use harness_tasks::CancellationToken;
use harness_ui::components::{IconButton, ProgressBar};
use harness_ui::theme::{space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};
use sp_drive::view::DriveFailure;
use sp_drive::{DriveError, ImportFile, UploadProgress};
use sp_protocol::wire::drive::DriveCreated;
use tokio::sync::watch;

use super::jobs::{self, ImportReadError};
use super::{DriveView, Upload};
use crate::{bridge, strings as text};

impl DriveView {
    pub(super) fn pick_import(&mut self, cx: &Context<Self>) {
        if self.upload.is_some() {
            return;
        }
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(SharedString::new_static(text::IMPORT_PROMPT)),
        });
        let task = cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let path = match chosen.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) | Err(_) => None,
                Ok(Err(error)) => {
                    tracing::warn!(error = %error, "the import dialog failed");
                    None
                },
            };
            if let Some(path) = path
                && this
                    .update(cx, |this, cx| this.read_import(path, cx))
                    .is_err()
            {
                tracing::trace!("a file was chosen after the Drive closed");
            }
        });
        self.keep(task);
    }

    fn read_import(&mut self, path: PathBuf, cx: &Context<Self>) {
        let folder = self.browse.folder.clone();
        let task = bridge::blocking(
            cx,
            &self.runtime,
            "drive-read-import",
            move || jobs::read_import(&path, folder),
            |this, result, cx| match result {
                Ok(file) => this.start_upload(file, cx),
                Err(error) => {
                    tracing::warn!(error = %error, "a file was not read for import");
                    let message = match error {
                        ImportReadError::TooLarge { .. } => text::FILE_TOO_LARGE,
                        ImportReadError::Read { .. } => text::FILE_UNREADABLE,
                    };
                    this.notify_user(Tone::Danger, message, cx);
                },
            },
        );
        self.keep(task);
    }

    fn start_upload(&mut self, file: ImportFile, cx: &mut Context<Self>) {
        let Some(client) = self.client() else {
            return self.report(DriveFailure::SignedOut, cx);
        };
        let (sender, receiver) = watch::channel::<Option<UploadProgress>>(None);
        let cancel = CancellationToken::new();
        let name = file.name.clone();
        let job = jobs::upload(client, file, sender, cancel.clone());
        let job = bridge::run(
            cx,
            &self.runtime,
            "drive-upload",
            job,
            Self::upload_finished,
        );
        let feed = bridge::follow(cx, receiver.clone(), |_: &mut Self, cx| cx.notify());
        self.upload = Some(Upload {
            name,
            progress: receiver,
            cancel,
            _job: job,
            _feed: feed,
        });
        cx.notify();
    }

    fn upload_finished(
        &mut self,
        result: Result<DriveCreated, DriveError>,
        cx: &mut Context<Self>,
    ) {
        self.upload = None;
        match result {
            Ok(created) => {
                self.notify_user(Tone::Success, text::UPLOADED, cx);
                self.browse.selected = Some(created.document.id);
                self.reload_listing(cx);
            },
            Err(DriveError::Cancelled) => self.notify_user(Tone::Info, text::UPLOAD_CANCELLED, cx),
            Err(error) => {
                tracing::warn!(error = %error, "a file was not imported");
                self.report(DriveFailure::from(&error), cx);
            },
        }
        cx.notify();
    }

    pub(super) fn cancel_upload(&self, cx: &mut Context<Self>) {
        if let Some(upload) = &self.upload {
            upload.cancel.cancel();
            cx.notify();
        }
    }

    pub(super) fn render_upload(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let upload = self.upload.as_ref()?;
        let palette = *tokens(cx);
        let bar = ProgressBar::new("drive-upload-progress").label(text::uploading(&upload.name));
        let progress = *upload.progress.borrow();
        let bar = match progress {
            Some(progress) if progress.total_bytes > 0 => {
                bar.fraction(progress.sent_bytes as f32 / progress.total_bytes as f32)
            },
            _ => bar,
        };
        let strip = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .px(space::MD)
            .py(space::SM)
            .border_b_1()
            .border_color(palette.border.subtle)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(space::XS)
                    .child(
                        div()
                            .truncate()
                            .text_size(type_scale::SMALL)
                            .text_color(palette.text.secondary)
                            .child(text::uploading(&upload.name)),
                    )
                    .child(bar),
            )
            .child(
                IconButton::new("drive-upload-cancel", IconName::X, text::CANCEL_UPLOAD)
                    .small()
                    .disabled(upload.cancel.is_cancelled())
                    .on_click(cx.listener(|this, _, _, cx| this.cancel_upload(cx))),
            );
        Some(strip.into_any_element())
    }
}
