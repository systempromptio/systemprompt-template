//! Getting files into a prompt: the file picker, pasted images and copied
//! files, and files dropped on the composer. Disk work runs on the blocking
//! pool; a pasted image is written to the app's scratch folder and attached by
//! path.

use std::path::PathBuf;

use gpui_kit::{
    AsyncApp, ClipboardEntry, ClipboardItem, Context, PathPromptOptions, SharedString, WeakEntity,
};
use systemprompt_engine::run::AttachmentInput;
use systemprompt_engine::store::now_ms;
use systemprompt_harness_shared::Tone;

use crate::app::runtime;
use crate::composer::ComposerView;
use crate::composer::drafts::{self, Attachment};
use crate::state::AppState;
use crate::strings::composer as text;

impl ComposerView {
    pub(super) fn choose_files(&mut self, cx: &Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(SharedString::from(text::ATTACH_PROMPT)),
        });
        let task = cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let chosen = match picked.await {
                Ok(Ok(chosen)) => chosen,
                Ok(Err(error)) => {
                    tracing::warn!(error = %error, "file picker failed");
                    if this
                        .update(cx, |this, cx| this.warn_user(text::DIALOG_FAILED, cx))
                        .is_err()
                    {
                        tracing::trace!("composer closed before the picker returned");
                    }
                    None
                },
                Err(_) => {
                    tracing::trace!("file picker closed without an answer");
                    None
                },
            };
            if let Some(paths) = chosen
                && this
                    .update(cx, |this, cx| this.attach_paths(paths, cx))
                    .is_err()
            {
                tracing::trace!("composer closed before the picker returned");
            }
        });
        self.keep(task);
    }

    pub(super) fn attach_paths(&mut self, paths: Vec<PathBuf>, cx: &Context<Self>) {
        if paths.is_empty() {
            return;
        }
        let work = move || {
            paths
                .into_iter()
                .filter(|path| path.is_file())
                .collect::<Vec<_>>()
        };
        let task = runtime::blocking(cx, self.services.runtime(), work, |this, files, cx| {
            if drafts::add(
                &mut this.attachments,
                files.iter().map(|path| Attachment::file(path)),
            ) {
                cx.notify();
            }
        });
        self.keep(task);
    }

    pub(super) fn paste(&mut self, item: &ClipboardItem, cx: &Context<Self>) -> bool {
        let mut consumed = false;
        let mut paths = Vec::new();
        for entry in item.entries() {
            match entry {
                ClipboardEntry::Image(image) => {
                    self.save_pasted(image.bytes.clone(), image.format.extension(), cx);
                    consumed = true;
                },
                ClipboardEntry::ExternalPaths(external) => {
                    paths.extend(external.paths().iter().cloned());
                    consumed = true;
                },
                ClipboardEntry::String(_) => {},
            }
        }
        self.attach_paths(paths, cx);
        consumed
    }

    fn save_pasted(&mut self, bytes: Vec<u8>, extension: &'static str, cx: &Context<Self>) {
        self.pastes = self.pastes.saturating_add(1);
        let label = format!("{} {}", text::PASTED_IMAGE, self.pastes);
        let path = self.services.paths().tmp.join(format!(
            "pasted-{}-{}.{extension}",
            now_ms(),
            self.pastes
        ));
        let work = move || std::fs::write(&path, &bytes).map(|()| path);
        let task = runtime::blocking(
            cx,
            self.services.runtime(),
            work,
            move |this, written, cx| match written {
                Ok(path) => {
                    let attachment = Attachment {
                        label,
                        input: AttachmentInput::Path(path),
                    };
                    if drafts::add(&mut this.attachments, [attachment]) {
                        cx.notify();
                    }
                },
                Err(error) => {
                    tracing::warn!(error = %error, "pasted image not saved");
                    this.warn_user(text::PASTE_FAILED, cx);
                },
            },
        );
        self.keep(task);
    }

    pub(super) fn remove_attachment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.attachments.len() {
            self.attachments.remove(index);
            cx.notify();
        }
    }

    fn warn_user(&self, message: &'static str, cx: &mut Context<Self>) {
        self.app.update(cx, |_, cx| {
            AppState::notify_user(Tone::Warning, message, cx);
        });
    }
}
