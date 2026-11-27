//! The projection of a palette onto the toolkit's theme colours, so the
//! toolkit's own controls match.

use gpui_kit::Hsla;
use gpui_kit::component::ThemeColor;

use super::color::Color;
use super::tokens::Palette;

pub(super) fn apply(p: &Palette, c: &mut ThemeColor) {
    surface_colors(c, p);
    action_colors(c, p);
    status_colors(c, p);
    navigation_colors(c, p);
    data_colors(c, p);
    hue_colors(c, p);
}

fn shade(color: Color, amount: f32) -> Hsla {
    color.mix(Color::hex(0x00_00_00), amount).hsla()
}

fn surface_colors(c: &mut ThemeColor, p: &Palette) {
    let h = Color::hsla;
    c.background = h(p.surface.app);
    c.foreground = h(p.text.primary);
    c.border = h(p.border.subtle);
    c.input = h(p.border.strong);
    c.ring = h(p.border.focus);
    c.caret = h(p.text.primary);
    c.selection = h(p.selection);
    c.overlay = h(p.surface.overlay);
    c.window_border = h(p.border.subtle);
    c.drag_border = h(p.accent.base);
    c.drop_target = h(p.accent.base.with_alpha(0.16));

    c.muted = h(p.surface.hover);
    c.muted_foreground = h(p.text.muted);
    c.accent = h(p.surface.hover);
    c.accent_foreground = h(p.text.primary);
    c.popover = h(p.surface.raised);
    c.popover_foreground = h(p.text.primary);
    c.accordion = h(p.surface.raised);
    c.group_box = h(p.surface.panel);
    c.group_box_foreground = h(p.text.primary);
    c.description_list_label = h(p.surface.panel);
    c.description_list_label_foreground = h(p.text.secondary);
    c.skeleton = h(p.surface.hover);
}

fn action_colors(c: &mut ThemeColor, p: &Palette) {
    let h = Color::hsla;
    c.primary = h(p.accent.base);
    c.primary_hover = h(p.accent.hover);
    c.primary_active = h(p.accent.active);
    c.primary_foreground = h(p.text.on_accent);
    c.secondary = h(p.surface.hover);
    c.secondary_hover = h(p.surface.active);
    c.secondary_active = h(p.surface.active);
    c.secondary_foreground = h(p.text.primary);

    c.button = h(p.surface.raised);
    c.button_hover = h(p.surface.hover);
    c.button_active = h(p.surface.active);
    c.button_foreground = h(p.text.primary);
    c.button_primary = c.primary;
    c.button_primary_hover = c.primary_hover;
    c.button_primary_active = c.primary_active;
    c.button_primary_foreground = c.primary_foreground;
    c.button_secondary = c.secondary;
    c.button_secondary_hover = c.secondary_hover;
    c.button_secondary_active = c.secondary_active;
    c.button_secondary_foreground = c.secondary_foreground;
}

fn status_colors(c: &mut ThemeColor, p: &Palette) {
    let h = Color::hsla;
    c.danger = h(p.danger.solid);
    c.danger_hover = shade(p.danger.solid, 0.12);
    c.danger_active = shade(p.danger.solid, 0.22);
    c.danger_foreground = h(p.danger.on_solid);
    c.success = h(p.success.solid);
    c.success_hover = shade(p.success.solid, 0.12);
    c.success_active = shade(p.success.solid, 0.22);
    c.success_foreground = h(p.success.on_solid);
    c.warning = h(p.warning.solid);
    c.warning_hover = shade(p.warning.solid, 0.12);
    c.warning_active = shade(p.warning.solid, 0.22);
    c.warning_foreground = h(p.warning.on_solid);
    c.info = h(p.info.solid);
    c.info_hover = shade(p.info.solid, 0.12);
    c.info_active = shade(p.info.solid, 0.22);
    c.info_foreground = h(p.info.on_solid);
    c.button_danger = c.danger;
    c.button_danger_hover = c.danger_hover;
    c.button_danger_active = c.danger_active;
    c.button_danger_foreground = c.danger_foreground;
    c.button_success = c.success;
    c.button_success_hover = c.success_hover;
    c.button_success_active = c.success_active;
    c.button_success_foreground = c.success_foreground;
    c.button_warning = c.warning;
    c.button_warning_hover = c.warning_hover;
    c.button_warning_active = c.warning_active;
    c.button_warning_foreground = c.warning_foreground;
    c.button_info = c.info;
    c.button_info_hover = c.info_hover;
    c.button_info_active = c.info_active;
    c.button_info_foreground = c.info_foreground;
}

fn navigation_colors(c: &mut ThemeColor, p: &Palette) {
    let h = Color::hsla;
    let clear = Hsla::transparent_black();
    c.link = h(p.text.accent);
    c.link_hover = h(p.accent.hover.mix(p.text.accent, 0.5));
    c.link_active = h(p.text.accent);

    c.list = h(p.surface.raised);
    c.list_even = h(p.surface.raised);
    c.list_head = h(p.surface.panel);
    c.list_hover = h(p.surface.hover);
    c.list_active = h(p.surface.selected);
    c.list_active_border = h(p.border.focus);

    c.sidebar = h(p.surface.panel);
    c.sidebar_foreground = h(p.text.primary);
    c.sidebar_border = h(p.border.subtle);
    c.sidebar_accent = h(p.surface.hover);
    c.sidebar_accent_foreground = h(p.text.primary);
    c.sidebar_primary = h(p.accent.base);
    c.sidebar_primary_foreground = h(p.text.on_accent);

    c.title_bar = h(p.surface.panel);
    c.title_bar_border = h(p.border.subtle);
    c.status_bar = h(p.surface.panel);
    c.status_bar_border = h(p.border.subtle);

    c.tab = clear;
    c.tab_bar = h(p.surface.panel);
    c.tab_bar_segmented = h(p.surface.hover);
    c.tab_active = h(p.surface.app);
    c.tab_active_foreground = h(p.text.primary);
    c.tab_foreground = h(p.text.secondary);
}

fn data_colors(c: &mut ThemeColor, p: &Palette) {
    let h = Color::hsla;
    let clear = Hsla::transparent_black();
    c.table = h(p.surface.raised);
    c.table_even = h(p.surface.raised);
    c.table_head = h(p.surface.panel);
    c.table_head_foreground = h(p.text.secondary);
    c.table_foot = h(p.surface.panel);
    c.table_foot_foreground = h(p.text.secondary);
    c.table_hover = h(p.surface.hover);
    c.table_active = h(p.surface.selected);
    c.table_active_border = h(p.border.focus);
    c.table_row_border = h(p.border.divider);

    c.scrollbar = clear;
    c.scrollbar_thumb = h(p.scrollbar);
    c.scrollbar_thumb_hover = h(p.scrollbar.with_alpha((p.scrollbar.alpha() * 1.6).min(1.0)));
    c.switch = h(p.border.strong);
    c.switch_thumb = h(Color::hex(0xFF_FF_FF));
    c.slider_bar = h(p.accent.base);
    c.slider_thumb = h(p.surface.raised);
    c.progress_bar = h(p.accent.base);

    c.chart_1 = h(p.accent.base);
    c.chart_2 = h(p.info.fg);
    c.chart_3 = h(p.success.fg);
    c.chart_4 = h(p.warning.fg);
    c.chart_5 = h(p.danger.fg);
    c.chart_bullish = h(p.success.fg);
    c.chart_bearish = h(p.danger.fg);
    c.chart_grid = h(p.border.divider);
}

fn hue_colors(c: &mut ThemeColor, p: &Palette) {
    let h = Color::hsla;
    c.red = h(p.danger.fg);
    c.red_light = h(p.danger.bg);
    c.green = h(p.success.fg);
    c.green_light = h(p.success.bg);
    c.blue = h(p.info.fg);
    c.blue_light = h(p.info.bg);
    c.yellow = h(p.warning.fg);
    c.yellow_light = h(p.warning.bg);
    c.magenta = h(p.syntax.keyword);
    c.magenta_light = h(p.accent.subtle);
    c.cyan = h(p.syntax.type_name);
    c.cyan_light = h(p.info.bg);
}
