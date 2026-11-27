//! Choosing the project: the folder dialog and the project switcher menu.

use std::path::PathBuf;

use gpui_kit::{App, AsyncApp, Entity, PathPromptOptions, SharedString};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::components::{Menu, MenuEntry, MenuItem};

use crate::app::paths::display_name;
use crate::state::AppState;
use crate::strings::foundation as text;

pub(crate) fn pick_folder(app: Entity<AppState>, cx: &App) {
    let chosen = cx.prompt_for_paths(PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some(SharedString::new_static(text::OPEN_FOLDER_PROMPT)),
    });
    cx.spawn(async move |cx: &mut AsyncApp| {
        let directory = match chosen.await {
            Ok(Ok(Some(paths))) => paths.into_iter().next(),
            Ok(Ok(None)) | Err(_) => None,
            Ok(Err(error)) => {
                tracing::warn!(error = %error, "folder dialog failed");
                None
            },
        };
        if let Some(directory) = directory {
            app.update(cx, |app, cx| app.open_project(directory, cx));
        }
    })
    .detach();
}

pub(crate) fn switcher(app: &Entity<AppState>, cx: &App) -> Menu {
    let state = app.read(cx);
    let label = state.project.as_ref().map_or_else(
        || text::NO_PROJECT_NAME.to_owned(),
        |project| project.name.clone(),
    );
    let current = state
        .project
        .as_ref()
        .map(|project| project.directory.clone());
    let recent: Vec<PathBuf> = state
        .services()
        .settings()
        .read(|settings| settings.recent_projects.clone());
    let mut entries: Vec<MenuEntry> = vec![MenuEntry::Label(SharedString::new_static(
        text::RECENT_PROJECTS,
    ))];
    entries.extend(recent.into_iter().map(|directory| {
        let target = app.clone();
        let checked = current.as_ref() == Some(&directory);
        MenuItem::new(display_name(&directory), move |_, cx| {
            let directory = directory.clone();
            target.update(cx, |app, cx| app.open_project(directory, cx));
        })
        .icon(IconName::Folder)
        .checked(checked)
        .into()
    }));
    entries.push(MenuEntry::Separator);
    let target = app.clone();
    entries.push(
        MenuItem::new(text::OPEN_FOLDER, move |_, cx| {
            pick_folder(target.clone(), cx);
        })
        .icon(IconName::Plus)
        .into(),
    );
    Menu::labeled("project-switcher", label, entries)
}
