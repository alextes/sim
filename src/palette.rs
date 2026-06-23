//! color palette, ported from the old sdl `colors` module.
//!
//! ui colors are `egui::Color32`. the sprite-batch tint path (stage 2) will
//! add an `[f32; 4]` conversion alongside these.
//!
//! colors were originally taken from the lee v2 dwarf fortress color set.
use egui::Color32;

pub const BASE: Color32 = Color32::from_rgb(20, 22, 30);
pub const BLUE: Color32 = Color32::from_rgb(138, 173, 244);
pub const WHITE: Color32 = Color32::from_rgb(202, 211, 245);
pub const BLACK: Color32 = Color32::from_rgb(21, 19, 15);
pub const GREEN: Color32 = Color32::from_rgb(80, 135, 20);
pub const CYAN: Color32 = Color32::from_rgb(25, 140, 140);
pub const RED: Color32 = Color32::from_rgb(160, 20, 10);
pub const MAGENTA: Color32 = Color32::from_rgb(135, 60, 130);
pub const BROWN: Color32 = Color32::from_rgb(150, 75, 55);
pub const LGRAY: Color32 = Color32::from_rgb(178, 175, 172);
pub const DGRAY: Color32 = Color32::from_rgb(116, 110, 113);
pub const LBLUE: Color32 = Color32::from_rgb(105, 135, 225);
pub const LGREEN: Color32 = Color32::from_rgb(125, 185, 55);
pub const LCYAN: Color32 = Color32::from_rgb(60, 205, 190);
pub const LRED: Color32 = Color32::from_rgb(220, 50, 20);
pub const LMAGENTA: Color32 = Color32::from_rgb(190, 110, 185);
pub const YELLOW: Color32 = Color32::from_rgb(230, 170, 30);
pub const GRAY: Color32 = Color32::from_rgb(128, 128, 128);
pub const ORANGE: Color32 = Color32::from_rgb(255, 165, 0);

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
