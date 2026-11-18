//! The windowed game: input, `Game::update`, drawing. Run with `cargo run`;
//! build for the web with `cargo build --release --target wasm32-unknown-unknown`.

use macroquad::prelude::*;
use playmand_macroquad_2d::{ARENA, BALL_RADIUS, Game, PADDLE};

fn window() -> Conf {
    Conf {
        window_title: "Playmand · Macroquad paddle".into(),
        window_width: ARENA.x as i32,
        window_height: ARENA.y as i32,
        ..Default::default()
    }
}

#[macroquad::main(window)]
async fn main() {
    let mut game = Game::default();
    loop {
        let steer = (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32) as f32;
        if game.over() {
            if is_key_pressed(KeyCode::Space) {
                game = Game::default();
            }
        } else {
            game.update(get_frame_time(), steer);
        }

        clear_background(Color::from_rgba(24, 24, 32, 255));
        draw_rectangle(
            game.paddle_x - PADDLE.x / 2.0,
            ARENA.y - PADDLE.y,
            PADDLE.x,
            PADDLE.y,
            Color::from_rgba(255, 179, 71, 255),
        );
        draw_circle(game.ball.x, game.ball.y, BALL_RADIUS, WHITE);
        let status = if game.over() {
            format!("Game over · score {} · Space to play again", game.score)
        } else {
            format!("Score {} · Lives {} · ← →", game.score, game.lives)
        };
        draw_text(&status, 16.0, 28.0, 24.0, LIGHTGRAY);
        next_frame().await
    }
}
