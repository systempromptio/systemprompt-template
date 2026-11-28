use gpui_kit::WindowAppearance;
use systemprompt_harness_brand::Brand;
use systemprompt_harness_shared::theme::{
    AA_LARGE, AA_TEXT, BrandTheme, Color, Palette, Scheme, Tone, contrast_ratio, highlight_theme,
};

use crate::TestResult;

fn palettes() -> Vec<(&'static str, &'static Palette)> {
    BrandTheme::all()
        .into_iter()
        .flat_map(|theme| [(theme.id, &theme.light), (theme.id, &theme.dark)])
        .collect()
}

fn assert_aa(name: &str, what: &str, fg: Color, bg: Color, minimum: f64) {
    let ratio = contrast_ratio(fg, bg);
    assert!(
        ratio >= minimum,
        "{name}: {what} is {ratio:.2}:1 ({} on {}), needs {minimum}",
        fg.to_hex(),
        bg.to_hex()
    );
}

#[test]
fn text_levels_meet_aa_on_every_surface() {
    for (name, p) in palettes() {
        for (surface, bg) in p.text_surfaces() {
            for (level, fg) in [
                ("primary", p.text.primary),
                ("secondary", p.text.secondary),
                ("muted", p.text.muted),
                ("accent", p.text.accent),
            ] {
                assert_aa(name, &format!("text.{level} on {surface}"), fg, bg, AA_TEXT);
            }
        }
    }
}

#[test]
fn status_text_meets_aa_on_surfaces_and_its_own_tint() {
    for (name, p) in palettes() {
        for tone in [
            Tone::Neutral,
            Tone::Accent,
            Tone::Success,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
        ] {
            let colors = p.tone(tone);
            assert_aa(
                name,
                &format!("{tone:?} fg on its bg"),
                colors.fg,
                colors.bg,
                AA_TEXT,
            );
            assert_aa(
                name,
                &format!("{tone:?} on_solid on solid"),
                colors.on_solid,
                colors.solid,
                AA_TEXT,
            );
            for (surface, bg) in p.text_surfaces() {
                assert_aa(
                    name,
                    &format!("{tone:?} fg on {surface}"),
                    colors.fg,
                    bg,
                    AA_TEXT,
                );
            }
        }
    }
}

#[test]
fn accent_fills_keep_their_label_readable() {
    for (name, p) in palettes() {
        for (state, fill) in [
            ("base", p.accent.base),
            ("hover", p.accent.hover),
            ("active", p.accent.active),
        ] {
            assert_aa(
                name,
                &format!("on_accent on accent.{state}"),
                p.text.on_accent,
                fill,
                AA_TEXT,
            );
        }
    }
}

#[test]
fn focus_ring_and_control_borders_meet_non_text_contrast() {
    for (name, p) in palettes() {
        for (surface, bg) in [
            ("app", p.surface.app),
            ("panel", p.surface.panel),
            ("raised", p.surface.raised),
            ("sunken", p.surface.sunken),
        ] {
            assert_aa(
                name,
                &format!("focus ring on {surface}"),
                p.border.focus,
                bg,
                AA_LARGE,
            );
            assert_aa(
                name,
                &format!("control border on {surface}"),
                p.border.strong,
                bg,
                AA_LARGE,
            );
        }
    }
}

#[test]
fn syntax_colours_meet_aa_on_code_and_diff_backgrounds() {
    for (name, p) in palettes() {
        for (token, fg) in p.syntax.named() {
            for (surface, bg) in [
                ("sunken", p.surface.sunken),
                ("raised", p.surface.raised),
                ("diff added", p.diff.added_bg),
                ("diff removed", p.diff.removed_bg),
            ] {
                assert_aa(
                    name,
                    &format!("syntax.{token} on {surface}"),
                    fg,
                    bg,
                    AA_TEXT,
                );
            }
        }
    }
}

#[test]
fn diff_text_and_gutters_meet_aa() {
    for (name, p) in palettes() {
        for (surface, bg) in [
            ("added", p.diff.added_bg),
            ("removed", p.diff.removed_bg),
            ("hunk", p.diff.hunk_bg),
        ] {
            for (level, fg) in [
                ("primary", p.text.primary),
                ("secondary", p.text.secondary),
                ("muted", p.text.muted),
            ] {
                assert_aa(
                    name,
                    &format!("text.{level} on diff {surface}"),
                    fg,
                    bg,
                    AA_TEXT,
                );
            }
        }
        assert_aa(
            name,
            "added sign",
            p.diff.added_fg,
            p.diff.added_bg,
            AA_TEXT,
        );
        assert_aa(
            name,
            "removed sign",
            p.diff.removed_fg,
            p.diff.removed_bg,
            AA_TEXT,
        );
    }
}

#[test]
fn palettes_declare_their_appearance() {
    for theme in BrandTheme::all() {
        assert!(!theme.light.dark);
        assert!(theme.dark.dark);
        assert!(
            theme.light.surface.app.relative_luminance()
                > theme.dark.surface.app.relative_luminance()
        );
    }
}

#[test]
fn brands_map_to_their_themes() {
    assert_eq!(BrandTheme::for_brand(Brand::Systemprompt).id, "graphite");
}

#[test]
fn scheme_keys_round_trip_and_resolve() {
    for scheme in Scheme::ALL {
        assert_eq!(Scheme::from_key(scheme.key()), Some(scheme));
    }
    assert_eq!(Scheme::from_key("sepia"), None);
    assert!(Scheme::System.is_dark(WindowAppearance::Dark));
    assert!(!Scheme::System.is_dark(WindowAppearance::Light));
    assert!(Scheme::Dark.is_dark(WindowAppearance::Light));
    assert!(!Scheme::Light.is_dark(WindowAppearance::Dark));
}

#[test]
fn syntax_palette_deserialises_into_the_toolkit_theme() -> TestResult {
    for theme in BrandTheme::all() {
        for palette in [&theme.light, &theme.dark] {
            let highlight = highlight_theme(theme.id, palette)
                .ok_or_else(|| format!("{} did not produce a highlight theme", theme.id))?;
            let keyword = highlight.style("keyword").and_then(|style| style.color);
            assert_eq!(keyword, Some(palette.syntax.keyword.hsla()));
        }
    }
    Ok(())
}
