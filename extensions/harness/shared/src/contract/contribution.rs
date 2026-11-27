//! What a shell extension contributes: screens, the titlebar items that open
//! them, settings panes and transcript adornments, each built from the shell
//! context.

use std::fmt;
use std::rc::Rc;

use crate::IconName;
use gpui_kit::{AnyElement, AnyView, App, Task, Window};
use systemprompt_engine::store::MessageWithParts;
use systemprompt_identifiers::engine::ExtensionId;

use crate::contract::context::ShellContext;

/// Runs on the UI thread only: the result is an `Rc`, never sent to the
/// runtime.
pub type ViewBuilder = Rc<dyn Fn(&ShellContext, &mut Window, &mut App) -> AnyView>;

/// Opens or closes one screen; the shell hands it to the titlebar item it
/// placed for that screen.
pub type OpenScreen = Rc<dyn Fn(&mut Window, &mut App)>;

/// Runs on the UI thread only; the item keeps the [`OpenScreen`] it is given
/// for as long as it is shown.
pub type TitlebarItemBuilder =
    Rc<dyn Fn(&ShellContext, OpenScreen, &mut Window, &mut App) -> AnyView>;

/// Asks the shell to draw an extension's contribution again.
pub type OnChange = Rc<dyn Fn(&mut App)>;

/// Where a screen's entry point sits in the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    Sidebar,
    Titlebar,
}

/// The route of one extension screen: the extension and the screen's key
/// within it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScreenKey {
    pub extension: ExtensionId,
    pub screen: &'static str,
}

/// A screen an extension adds: its key among the extension's screens, the
/// title and icon of its entry point, where that entry point sits, and the
/// builder of its view.
#[derive(Clone)]
pub struct ScreenSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub icon: IconName,
    pub placement: Placement,
    pub build: ViewBuilder,
}

impl fmt::Debug for ScreenSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreenSpec")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("icon", &self.icon)
            .field("placement", &self.placement)
            .finish_non_exhaustive()
    }
}

/// The entry point an extension draws itself for one of its titlebar screens,
/// in place of the shell's plain icon button, such as a mark with a live count.
///
/// `screen` is the key of that screen among the extension's screens. The
/// shell builds the item once while the screen is admitted and hands it the
/// [`OpenScreen`] that opens the screen, or closes it when it is open.
#[derive(Clone)]
pub struct TitlebarItemSpec {
    pub screen: &'static str,
    pub build: TitlebarItemBuilder,
}

impl fmt::Debug for TitlebarItemSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TitlebarItemSpec")
            .field("screen", &self.screen)
            .finish_non_exhaustive()
    }
}

/// A page an extension adds to Settings: its key, the title and icon in the
/// settings navigation, and the builder of its view.
#[derive(Clone)]
pub struct SettingsPaneSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub icon: IconName,
    pub build: ViewBuilder,
}

impl fmt::Debug for SettingsPaneSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SettingsPaneSpec")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("icon", &self.icon)
            .finish_non_exhaustive()
    }
}

/// Draws an extension's mark under a message in the transcript.
///
/// Held as `dyn` on the UI thread. `adorn` answers `None` for a message the
/// extension has nothing to say about; it reads the message and its parts as
/// stored, and state of its own. An adornment whose mark follows state of its
/// own returns from `follow` a task that calls `on_change` whenever that state
/// changed; the transcript keeps the task while it shows the adornment and
/// draws again on every call.
pub trait TranscriptAdornment {
    fn adorn(
        &self,
        message: &MessageWithParts,
        ctx: &ShellContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement>;

    fn follow(&self, _on_change: OnChange, _cx: &mut App) -> Option<Task<()>> {
        None
    }
}
