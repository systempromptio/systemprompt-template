//! Building every registered shell extension and reading the contributions of
//! the ones the instance's enablement admits.

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use systemprompt_engine::run::extension::{
    Contributed, EnabledExtensions, Resolved, admitted, resolve,
};
use systemprompt_harness::extension::ExtensionError;

use crate::contract::context::ShellContext;
use crate::contract::contribution::{
    ScreenSpec, SettingsPaneSpec, TitlebarItemSpec, TranscriptAdornment,
};
use crate::contract::extension::ShellExtension;
use crate::contract::register::ShellExtensionRegistration;

type Entry = Resolved<Arc<dyn ShellExtension>>;

/// The resolved shell extensions, highest priority first.
///
/// One per id, the highest priority shadowing the rest, dependencies present
/// and acyclic. Each reader takes the enablement and returns the admitted
/// extensions' contributions in that order.
#[derive(Clone, Default)]
pub struct ShellExtensions {
    entries: Vec<Entry>,
}

impl ShellExtensions {
    pub fn from_registrations(
        registrations: &[ShellExtensionRegistration],
    ) -> Result<Self, ExtensionError> {
        let built = registrations
            .iter()
            .map(|registration| (registration.factory)());
        let extensions = Self::from_extensions(built)?;
        tracing::info!(
            count = extensions.entries.len(),
            "shell extensions registered"
        );
        Ok(extensions)
    }

    pub fn from_extensions(
        extensions: impl IntoIterator<Item = Arc<dyn ShellExtension>>,
    ) -> Result<Self, ExtensionError> {
        let candidates = extensions.into_iter().map(|value| Resolved {
            metadata: value.metadata(),
            value,
        });
        Ok(Self {
            entries: resolve(candidates)?,
        })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn screens(
        &self,
        enabled: Option<&EnabledExtensions>,
        ctx: &ShellContext,
    ) -> Vec<Contributed<ScreenSpec>> {
        self.collect(enabled, |extension| extension.screens(ctx))
    }

    pub fn titlebar_items(
        &self,
        enabled: Option<&EnabledExtensions>,
        ctx: &ShellContext,
    ) -> Vec<Contributed<TitlebarItemSpec>> {
        self.collect(enabled, |extension| extension.titlebar_items(ctx))
    }

    pub fn settings_panes(
        &self,
        enabled: Option<&EnabledExtensions>,
        ctx: &ShellContext,
    ) -> Vec<Contributed<SettingsPaneSpec>> {
        self.collect(enabled, |extension| extension.settings_panes(ctx))
    }

    pub fn adornments(
        &self,
        enabled: Option<&EnabledExtensions>,
        ctx: &ShellContext,
    ) -> Vec<Contributed<Rc<dyn TranscriptAdornment>>> {
        self.collect(enabled, |extension| extension.adornments(ctx))
    }

    fn collect<T>(
        &self,
        enabled: Option<&EnabledExtensions>,
        read: impl Fn(&dyn ShellExtension) -> Vec<T>,
    ) -> Vec<Contributed<T>> {
        admitted(&self.entries, enabled)
            .into_iter()
            .flat_map(|entry| {
                read(entry.value.as_ref())
                    .into_iter()
                    .map(|item| Contributed {
                        extension: entry.metadata.id.clone(),
                        item,
                    })
            })
            .collect()
    }
}

impl fmt::Debug for ShellExtensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.entries.iter().map(|entry| &entry.metadata))
            .finish()
    }
}
