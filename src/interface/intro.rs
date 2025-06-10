use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::SpriteSheetRenderer;
use crate::GameState;

const SIM_ASCII_ART: &str = r#"
   ____  _  __  __
  / __/ (_)/ / / /
 / _/  / / / /_/ /
/___/ /_/  \____/

"#;

const LOADING_DURATION: Duration = Duration::from_secs(3);

pub struct IntroState {
    pub start_time: Instant,
}

impl IntroState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

pub fn render_intro_screen(
    canvas: &mut Canvas<Window>,
    sprite_renderer: &SpriteSheetRenderer,
    game_state: &Arc<Mutex<GameState>>,
    intro_state: &IntroState,
) {
    let elapsed = intro_state.start_time.elapsed();
    let loading_progress = (elapsed.as_secs_f64() / LOADING_DURATION.as_secs_f64()).min(1.0);

    let art_lines: Vec<(String, sdl2::pixels::Color)> = SIM_ASCII_ART
        .lines()
        .map(|line| (line.to_string(), crate::colors::WHITE))
        .collect();

    if loading_progress < 1.0 {
        canvas.set_draw_color(crate::colors::BASE);
        canvas.clear();
        super::draw_centered_window(canvas, sprite_renderer, &art_lines);

        let screen_width = canvas.window().size().0;
        let screen_height = canvas.window().size().1;

        let bar_width = screen_width / 2;
        let bar_height = 20;
        let bar_x = (screen_width - bar_width) / 2;
        let bar_y = screen_height / 2 + 50;

        // Draw loading bar background
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas
            .fill_rect(Rect::new(
                bar_x as i32,
                bar_y as i32,
                bar_width,
                bar_height,
            ))
            .unwrap();

        // Draw loading bar progress
        canvas.set_draw_color(Color::RGB(100, 200, 100));
        canvas
            .fill_rect(Rect::new(
                bar_x as i32,
                bar_y as i32,
                (bar_width as f64 * loading_progress) as u32,
                bar_height,
            ))
            .unwrap();
    } else {
        let mut state_guard = game_state.lock().unwrap();
        *state_guard = GameState::GameMenu;
    }
}