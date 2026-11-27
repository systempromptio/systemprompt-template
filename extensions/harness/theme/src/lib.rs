//! Semantic design tokens, the two brand themes, and the bridge onto the
//! toolkit's theme.
//!
//! Components never read raw toolkit colours: they call [`tokens`] and paint
//! with the [`Palette`] of the theme that [`apply`] installed. `apply` also
//! projects the palette onto the toolkit's `Theme`, so the toolkit's own
//! controls (inputs, menus, dialogs, notifications, scrollbars, the markdown
//! view) match.

mod color;
mod highlight;
mod palettes;
mod scale;
mod tokens;
mod toolkit_colors;

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Global, SharedString, WindowAppearance};
use sp_brand::Brand;

pub use color::{AA_LARGE, AA_TEXT, Color, contrast_ratio};
pub use highlight::highlight_theme;
pub use scale::{font, radius, size, space, text};
pub use tokens::{
    Accent, Borders, DiffColors, Palette, StatusColors, Surfaces, SyntaxPalette, TextColors, Tone,
};

/// The user's appearance preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scheme {
    #[default]
    System,
    Light,
    Dark,
}

impl Scheme {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    #[must_use]
    pub fn is_dark(self, appearance: WindowAppearance) -> bool {
        match self {
            Self::System => ThemeMode::from(appearance).is_dark(),
            Self::Light => false,
            Self::Dark => true,
        }
    }

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scheme| scheme.key() == key)
    }
}

/// A brand's palettes for both appearances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrandTheme {
    pub id: &'static str,
    pub light: Palette,
    pub dark: Palette,
}

pub static GRAPHITE: BrandTheme = BrandTheme {
    id: "graphite",
    light: palettes::GRAPHITE_LIGHT,
    dark: palettes::GRAPHITE_DARK,
};

pub static ASTOUND: BrandTheme = BrandTheme {
    id: "astound",
    light: palettes::ASTOUND_LIGHT,
    dark: palettes::ASTOUND_DARK,
};

impl BrandTheme {
    #[must_use]
    pub fn for_brand(brand: Brand) -> &'static Self {
        match brand {
            Brand::Systemprompt => &GRAPHITE,
            Brand::Astound => &ASTOUND,
        }
    }

    #[must_use]
    pub fn current() -> &'static Self {
        Self::for_brand(sp_brand::identity().brand())
    }

    #[must_use]
    pub fn all() -> [&'static Self; 2] {
        [&GRAPHITE, &ASTOUND]
    }

    #[must_use]
    pub const fn palette(&self, dark: bool) -> &Palette {
        if dark { &self.dark } else { &self.light }
    }
}

#[derive(Debug, Clone)]
struct Fonts {
    ui: SharedString,
    mono: Option<SharedString>,
}

#[derive(Debug, Clone)]
struct Applied {
    theme: BrandTheme,
    scheme: Scheme,
    palette: Palette,
    fonts: Fonts,
}

impl Global for Applied {}

pub fn apply(cx: &mut App, theme: &BrandTheme, scheme: Scheme) {
    let dark = scheme.is_dark(cx.window_appearance());
    let palette = *theme.palette(dark);
    let fonts = cx
        .try_global::<Applied>()
        .map_or_else(|| resolve_fonts(cx), |applied| applied.fonts.clone());
    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };

    Theme::change(mode, None, cx);
    let highlight = highlight_theme(theme.id, &palette);
    let ui_font = fonts.ui.clone();
    let mono_font = fonts.mono.clone();
    Theme::update(cx, move |toolkit| {
        toolkit_colors::apply(&palette, &mut toolkit.colors);
        toolkit.font_family = ui_font;
        if let Some(mono) = mono_font {
            toolkit.mono_font_family = mono;
        }
        toolkit.font_size = text::BODY;
        toolkit.mono_font_size = text::MONO;
        toolkit.radius = radius::MD;
        toolkit.radius_lg = radius::XL;
        toolkit.shadow = true;
        toolkit.focus_ring = true;
        if let Some(highlight) = highlight {
            toolkit.highlight_theme = highlight;
        }
    });

    cx.set_global(Applied {
        theme: *theme,
        scheme,
        palette,
        fonts,
    });
    cx.refresh_windows();
    tracing::debug!(
        theme = theme.id,
        scheme = scheme.key(),
        dark,
        "theme applied"
    );
}

pub fn refresh(cx: &mut App) {
    let (theme, scheme) = cx.try_global::<Applied>().map_or_else(
        || (*BrandTheme::current(), Scheme::System),
        |applied| (applied.theme, applied.scheme),
    );
    apply(cx, &theme, scheme);
}

pub fn set_scheme(cx: &mut App, scheme: Scheme) {
    let theme = cx
        .try_global::<Applied>()
        .map_or_else(|| *BrandTheme::current(), |applied| applied.theme);
    apply(cx, &theme, scheme);
}

#[must_use]
pub fn tokens(cx: &App) -> &Palette {
    cx.try_global::<Applied>()
        .map_or_else(|| &BrandTheme::current().light, |applied| &applied.palette)
}

#[must_use]
pub fn scheme(cx: &App) -> Scheme {
    cx.try_global::<Applied>()
        .map_or(Scheme::System, |applied| applied.scheme)
}

#[must_use]
pub fn mono_font(cx: &App) -> SharedString {
    cx.try_global::<Applied>()
        .and_then(|applied| applied.fonts.mono.clone())
        .unwrap_or_else(|| Theme::global(cx).mono_font_family.clone())
}

fn resolve_fonts(cx: &App) -> Fonts {
    let installed = cx.text_system().all_font_names();
    let ui = font::pick(font::UI_STACK, &installed).unwrap_or(".SystemUIFont");
    let mono = font::pick(font::MONO_STACK, &installed);
    Fonts {
        ui: SharedString::new_static(ui),
        mono: mono.map(SharedString::new_static),
    }
}
