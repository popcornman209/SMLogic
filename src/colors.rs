use crate::egui::{Color32, epaint, style};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ColorPallet {
    pub label: Cow<'static, str>,
    pub grid: Color32,
    pub grid_lines: Color32,
    pub text: Color32,
    pub base: Color32,
    pub button: Color32,
    pub button_hover: Color32,
    pub button_pushed: Color32,
    pub selection: Color32,
    pub selection_text: Color32,
}

impl ColorPallet {
    pub const DEFAULT_PALLET: Self = ColorPallet {
        label: Cow::Borrowed("Default"),
        grid: Color32::from_gray(40),
        grid_lines: Color32::from_gray(75),
        text: Color32::from_gray(210),
        base: Color32::from_gray(35),
        button: Color32::from_gray(50),
        button_hover: Color32::from_gray(40),
        button_pushed: Color32::from_gray(20),
        selection: Color32::from_rgb(77, 151, 255),
        selection_text: Color32::from_gray(15),
    };

    pub const CATPPUCCIN_MOCHA: Self = ColorPallet {
        label: Cow::Borrowed("Catppuccin Mocha"),
        grid: Color32::from_rgb(30, 30, 46),           //base
        grid_lines: Color32::from_rgb(88, 91, 112),    //surface 2
        text: Color32::from_rgb(205, 214, 244),        //text
        base: Color32::from_rgb(24, 24, 37),           //mantle
        button: Color32::from_rgb(49, 50, 68),         //surface 0
        button_hover: Color32::from_rgb(24, 24, 37),   //mantle
        button_pushed: Color32::from_rgb(17, 17, 27),  //crust
        selection: Color32::from_rgb(245, 194, 231),   //pink
        selection_text: Color32::from_rgb(17, 17, 27), //crust
    };

    pub const NORD: Self = ColorPallet {
        label: Cow::Borrowed("Nord"),
        grid: Color32::from_rgb(67, 76, 94),
        grid_lines: Color32::from_rgb(76, 86, 106),
        text: Color32::from_rgb(236, 239, 244),
        base: Color32::from_rgb(59, 66, 82),
        button: Color32::from_rgb(76, 86, 106),
        button_hover: Color32::from_rgb(67, 76, 94),
        button_pushed: Color32::from_rgb(46, 52, 64),
        selection: Color32::from_rgb(129, 161, 193),
        selection_text: Color32::from_rgb(46, 52, 64),
    };

    pub const COLOR_SCHEMES: &[&'static str] = &["Defaut", "Catppuccin Mocha", "Nord", "Custom"];

    pub fn from_label(label: &str) -> Self {
        match label {
            "Catppuccin Mocha" => ColorPallet::CATPPUCCIN_MOCHA,
            "Nord" => ColorPallet::NORD,
            _ => ColorPallet::DEFAULT_PALLET,
        }
    }

    // next 3 fn's are modified from https://github.com/catppuccin/egui
    fn make_widget_visual(
        &self,
        old: style::WidgetVisuals,
        bg_fill: egui::Color32,
    ) -> style::WidgetVisuals {
        style::WidgetVisuals {
            bg_fill,
            weak_bg_fill: bg_fill,
            bg_stroke: egui::Stroke {
                color: self.selection,
                ..old.bg_stroke
            },
            fg_stroke: egui::Stroke {
                color: self.text,
                ..old.fg_stroke
            },
            ..old
        }
    }

    fn visuals(&self, old: egui::Visuals) -> egui::Visuals {
        // let shadow_color = if is_latte {
        //     egui::Color32::from_black_alpha(25)
        // } else {
        //     egui::Color32::from_black_alpha(96)
        // };

        egui::Visuals {
            hyperlink_color: self.selection,
            faint_bg_color: self.grid,
            extreme_bg_color: self.base,
            window_fill: self.base,
            panel_fill: self.base,
            window_stroke: egui::Stroke {
                color: self.selection,
                ..old.window_stroke
            },
            widgets: style::Widgets {
                noninteractive: self.make_widget_visual(old.widgets.noninteractive, self.base),
                inactive: self.make_widget_visual(old.widgets.inactive, self.button),
                hovered: self.make_widget_visual(old.widgets.hovered, self.button_hover),
                active: self.make_widget_visual(old.widgets.active, self.button_pushed),
                open: self.make_widget_visual(old.widgets.open, self.button),
            },
            selection: style::Selection {
                bg_fill: self.selection,
                stroke: egui::Stroke {
                    color: self.selection_text,
                    ..old.selection.stroke
                },
            },
            window_shadow: epaint::Shadow::NONE,
            popup_shadow: epaint::Shadow::NONE,
            dark_mode: true,
            ..old
        }
    }
    pub fn apply_theme(&self, ctx: &egui::Context) {
        let old = ctx.style().visuals.clone();
        ctx.set_visuals(self.visuals(old));
    }
}

impl Default for ColorPallet {
    fn default() -> Self {
        Self::DEFAULT_PALLET
    }
}

pub const DEFAULT_GATE_COLOR: Color32 = Color32::from_rgb(0xDF, 0x7F, 0x01);

/// paint tool palette; 10 base colors × 4 shades.
pub const SM_PALETTE: [[Color32; 4]; 10] = [
    // Gray
    [
        Color32::from_rgb(0xEE, 0xEE, 0xEE),
        Color32::from_rgb(0x7F, 0x7F, 0x7F),
        Color32::from_rgb(0x4A, 0x4A, 0x4A),
        Color32::from_rgb(0x22, 0x22, 0x22),
    ],
    // Yellow
    [
        Color32::from_rgb(0xF5, 0xF0, 0x71),
        Color32::from_rgb(0xE2, 0xDB, 0x13),
        Color32::from_rgb(0x81, 0x7C, 0x00),
        Color32::from_rgb(0x32, 0x30, 0x00),
    ],
    // Lime
    [
        Color32::from_rgb(0xCB, 0xF6, 0x6F),
        Color32::from_rgb(0xA0, 0xEA, 0x00),
        Color32::from_rgb(0x57, 0x7D, 0x07),
        Color32::from_rgb(0x37, 0x50, 0x00),
    ],
    // Green
    [
        Color32::from_rgb(0x68, 0xFF, 0x88),
        Color32::from_rgb(0x19, 0xE7, 0x53),
        Color32::from_rgb(0x0E, 0x80, 0x31),
        Color32::from_rgb(0x06, 0x40, 0x23),
    ],
    // Cyan
    [
        Color32::from_rgb(0x7E, 0xED, 0xED),
        Color32::from_rgb(0x2C, 0xE6, 0xE6),
        Color32::from_rgb(0x11, 0x87, 0x87),
        Color32::from_rgb(0x0A, 0x44, 0x44),
    ],
    // Blue
    [
        Color32::from_rgb(0x4C, 0x6F, 0xE3),
        Color32::from_rgb(0x0A, 0x3E, 0xE2),
        Color32::from_rgb(0x0F, 0x2E, 0x91),
        Color32::from_rgb(0x0A, 0x1D, 0x5A),
    ],
    // Violet
    [
        Color32::from_rgb(0xAE, 0x79, 0xF0),
        Color32::from_rgb(0x75, 0x14, 0xED),
        Color32::from_rgb(0x50, 0x0A, 0xA6),
        Color32::from_rgb(0x35, 0x08, 0x6C),
    ],
    // Magenta
    [
        Color32::from_rgb(0xEE, 0x7B, 0xF0),
        Color32::from_rgb(0xCF, 0x11, 0xD2),
        Color32::from_rgb(0x72, 0x0A, 0x74),
        Color32::from_rgb(0x52, 0x06, 0x53),
    ],
    // Red
    [
        Color32::from_rgb(0xF0, 0x67, 0x67),
        Color32::from_rgb(0xD0, 0x25, 0x25),
        Color32::from_rgb(0x7C, 0x00, 0x00),
        Color32::from_rgb(0x56, 0x02, 0x02),
    ],
    // Orange
    [
        Color32::from_rgb(0xEE, 0xAF, 0x5C),
        Color32::from_rgb(0xDF, 0x7F, 0x00),
        Color32::from_rgb(0x67, 0x3B, 0x00),
        Color32::from_rgb(0x47, 0x28, 0x00),
    ],
];

pub const UNPOWERED_COLOR: Color32 = Color32::from_gray(35);
pub const POWERED_COLOR: Color32 = Color32::from_rgb(0, 149, 255);
