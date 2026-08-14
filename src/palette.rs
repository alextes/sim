//! catppuccin mocha color palette.
//!
//! ui colors are `egui::Color32`. the sprite-batch tint path (stage 2) will
//! add an `[f32; 4]` conversion alongside these.
//!
//! this is a palette library; not every color is used at all times.
#![allow(dead_code)]
use egui::Color32;

pub const ROSEWATER: Color32 = Color32::from_rgb(245, 224, 220);
pub const FLAMINGO: Color32 = Color32::from_rgb(242, 205, 205);
pub const PINK: Color32 = Color32::from_rgb(245, 194, 231);
pub const MAUVE: Color32 = Color32::from_rgb(203, 166, 247);
pub const RED: Color32 = Color32::from_rgb(243, 139, 168);
pub const MAROON: Color32 = Color32::from_rgb(235, 160, 172);
pub const PEACH: Color32 = Color32::from_rgb(250, 179, 135);
pub const YELLOW: Color32 = Color32::from_rgb(249, 226, 175);
pub const GREEN: Color32 = Color32::from_rgb(166, 227, 161);
pub const TEAL: Color32 = Color32::from_rgb(148, 226, 213);
pub const SKY: Color32 = Color32::from_rgb(137, 220, 235);
pub const SAPPHIRE: Color32 = Color32::from_rgb(116, 199, 236);
pub const BLUE: Color32 = Color32::from_rgb(137, 180, 250);
pub const LAVENDER: Color32 = Color32::from_rgb(180, 190, 254);
pub const TEXT: Color32 = Color32::from_rgb(205, 214, 244);
pub const SUBTEXT1: Color32 = Color32::from_rgb(186, 194, 222);
pub const SUBTEXT0: Color32 = Color32::from_rgb(166, 173, 200);
pub const OVERLAY2: Color32 = Color32::from_rgb(147, 153, 178);
pub const OVERLAY1: Color32 = Color32::from_rgb(127, 132, 156);
pub const OVERLAY0: Color32 = Color32::from_rgb(108, 112, 134);
pub const SURFACE2: Color32 = Color32::from_rgb(88, 91, 112);
pub const SURFACE1: Color32 = Color32::from_rgb(69, 71, 90);
pub const SURFACE0: Color32 = Color32::from_rgb(49, 50, 68);
pub const BASE: Color32 = Color32::from_rgb(30, 30, 46);
pub const MANTLE: Color32 = Color32::from_rgb(24, 24, 37);
pub const CRUST: Color32 = Color32::from_rgb(17, 17, 27);

// semantic aliases retained while render paths move to palette-native names.
pub const WHITE: Color32 = TEXT;
pub const BLACK: Color32 = CRUST;
pub const CYAN: Color32 = TEAL;
pub const MAGENTA: Color32 = MAUVE;
pub const BROWN: Color32 = PEACH;
pub const LGRAY: Color32 = SUBTEXT1;
pub const DGRAY: Color32 = OVERLAY0;
pub const LBLUE: Color32 = SAPPHIRE;
pub const LGREEN: Color32 = GREEN;
pub const LCYAN: Color32 = SKY;
pub const LRED: Color32 = MAROON;
pub const LMAGENTA: Color32 = PINK;
pub const GRAY: Color32 = OVERLAY2;
pub const ORANGE: Color32 = PEACH;

/// the scene clear color as a `wgpu::Color`.
///
/// the surface is a gamma-space (unorm) format, so values are written as-is and
/// interpreted as srgb by the compositor. that means we pass the raw normalized
/// srgb channels here, not a linear conversion.
pub fn clear_color() -> wgpu::Color {
    wgpu::Color {
        r: BASE.r() as f64 / 255.0,
        g: BASE.g() as f64 / 255.0,
        b: BASE.b() as f64 / 255.0,
        a: 1.0,
    }
}
