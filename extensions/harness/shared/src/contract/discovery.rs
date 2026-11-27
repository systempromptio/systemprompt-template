//! Finding every linked shell extension and reading the contributions of the
//! ones the instance's enablement admits.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use harness_engine::extension::{Contributed, EnabledExtensions, Resolved, admitted, resolve};
use sp_extension::ExtensionError;

use crate::context::ShellContext;
use crate::contribution::{ScreenSpec, SettingsPaneSpec, TitlebarItemSpec, TranscriptAdornment};
use crate::extension::ShellExtension;
use crate::register::ShellExtensionRegistration;

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
    pub fn discover() -> Result<Self, ExtensionError> {
        let found = inventory::iter::<ShellExtensionRegistration>
            .into_iter()
            .map(|registration| (registration.factory)());
        let extensions = Self::from_extensions(found)?;
        tracing::info!(
            count = extensions.entries.len(),
            "shell extensions discovered"
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
