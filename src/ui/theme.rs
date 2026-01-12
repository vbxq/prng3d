use egui::{Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

pub const BG_PURE_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
pub const BG_PANEL: Color32 = Color32::from_rgb(5, 5, 7);
pub const BG_WIDGET: Color32 = Color32::from_rgb(15, 15, 20);
pub const BG_WIDGET_HOVER: Color32 = Color32::from_rgb(25, 25, 35);
pub const BG_WIDGET_ACTIVE: Color32 = Color32::from_rgb(35, 35, 50);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(168, 168, 171);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 107, 112);
pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(220, 220, 225);

pub const ACCENT_GREEN: Color32 = Color32::from_rgb(46, 172, 35);
pub const ACCENT_RED: Color32 = Color32::from_rgb(172, 35, 35);
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(84, 102, 206);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(131, 23, 213);
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(172, 117, 35);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(50, 51, 113, 77);
pub const BORDER_ACCENT: Color32 = Color32::from_rgb(84, 102, 206);

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(TEXT_PRIMARY),

        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: BG_WIDGET,
                weak_bg_fill: BG_PANEL,
                bg_stroke: Stroke::new(1.0, BORDER_SUBTLE),
                rounding: Rounding::same(4.0),
                fg_stroke: Stroke::new(1.0, TEXT_MUTED),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: BG_WIDGET,
                weak_bg_fill: BG_WIDGET,
                bg_stroke: Stroke::new(1.0, BORDER_SUBTLE),
                rounding: Rounding::same(4.0),
                fg_stroke: Stroke::new(1.0, TEXT_PRIMARY),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: BG_WIDGET_HOVER,
                weak_bg_fill: BG_WIDGET_HOVER,
                bg_stroke: Stroke::new(1.0, BORDER_ACCENT),
                rounding: Rounding::same(4.0),
                fg_stroke: Stroke::new(1.0, TEXT_BRIGHT),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: BG_WIDGET_ACTIVE,
                weak_bg_fill: BG_WIDGET_ACTIVE,
                bg_stroke: Stroke::new(2.0, ACCENT_PURPLE),
                rounding: Rounding::same(4.0),
                fg_stroke: Stroke::new(1.0, TEXT_BRIGHT),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: BG_WIDGET_ACTIVE,
                weak_bg_fill: BG_WIDGET_ACTIVE,
                bg_stroke: Stroke::new(1.0, BORDER_ACCENT),
                rounding: Rounding::same(4.0),
                fg_stroke: Stroke::new(1.0, TEXT_BRIGHT),
                expansion: 0.0,
            },
        },

        selection: egui::style::Selection {
            bg_fill: ACCENT_PURPLE.gamma_multiply(0.4),
            stroke: Stroke::new(1.0, ACCENT_PURPLE),
        },

        hyperlink_color: ACCENT_BLUE,
        faint_bg_color: BG_PANEL,
        extreme_bg_color: BG_PURE_BLACK,
        code_bg_color: BG_PURE_BLACK,
        warn_fg_color: ACCENT_ORANGE,
        error_fg_color: ACCENT_RED,

        window_rounding: Rounding::same(6.0),
        window_shadow: egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_black_alpha(128),
        },
        window_fill: BG_PANEL,
        window_stroke: Stroke::new(1.0, BORDER_SUBTLE),

        panel_fill: BG_PANEL,

        popup_shadow: egui::epaint::Shadow {
            offset: egui::vec2(0.0, 2.0),
            blur: 8.0,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        },

        resize_corner_size: 12.0,
        text_cursor: egui::style::TextCursorStyle {
            stroke: Stroke::new(2.0, ACCENT_PURPLE),
            ..Default::default()
        },
        clip_rect_margin: 3.0,
        button_frame: true,
        collapsing_header_frame: false,
        indent_has_left_vline: true,
        striped: false,
        slider_trailing_fill: true,
        handle_shape: egui::style::HandleShape::Circle,
        interact_cursor: None,
        image_loading_spinners: true,
        numeric_color_space: egui::style::NumericColorSpace::GammaByte,
        menu_rounding: Rounding::same(4.0),
        window_highlight_topmost: true,
    };

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.indent = 18.0;
    style.spacing.slider_width = 200.0;

    let mut fonts = egui::FontDefinitions::default();

    fonts.families.insert(
        FontFamily::Monospace,
        vec!["Hack".to_owned(), "monospace".to_owned()],
    );

    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(18.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();

    ctx.set_style(style);
}
