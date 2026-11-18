//! A tiny Macroquad paddle game, split so the rules are plain Rust: `Game`
//! knows nothing about windows or drawing, so it is tested headless, and
//! `main.rs` only reads input, calls `Game::update` and draws.
//!
//! Macroquad suits small 2D games and the smallest WebAssembly builds. It is
//! a separate engine: a Macroquad project never pulls in Bevy.

use macroquad::math::{Vec2, vec2};

pub const ARENA: Vec2 = vec2(640.0, 400.0);
pub const PADDLE: Vec2 = vec2(96.0, 12.0);
pub const BALL_RADIUS: f32 = 8.0;
/// The rules always advance in 1/120 s steps, whatever the frame rate.
pub const STEP: f32 = 1.0 / 120.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub paddle_x: f32,
    pub ball: Vec2,
    pub velocity: Vec2,
    pub score: u32,
    pub lives: u32,
    leftover: f32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            paddle_x: ARENA.x / 2.0,
            ball: vec2(ARENA.x / 2.0, 80.0),
            velocity: vec2(150.0, 220.0),
            score: 0,
            lives: 3,
            leftover: 0.0,
        }
    }
}

impl Game {
    /// Advances by a frame of `dt` seconds with the paddle steered by `steer`
    /// (-1 left, 0 still, 1 right). Long frames are cut into fixed steps so the
    /// result does not depend on the frame rate.
    pub fn update(&mut self, dt: f32, steer: f32) {
        self.leftover += dt.min(0.25);
        while self.leftover >= STEP {
            self.leftover -= STEP;
            self.step(steer);
        }
    }

    fn step(&mut self, steer: f32) {
        let half = PADDLE.x / 2.0;
        self.paddle_x = (self.paddle_x + steer * 420.0 * STEP).clamp(half, ARENA.x - half);
        self.ball += self.velocity * STEP;
        // Walls and ceiling: reflect, and put the ball back inside.
        if self.ball.x < BALL_RADIUS || self.ball.x > ARENA.x - BALL_RADIUS {
            self.velocity.x = -self.velocity.x;
            self.ball.x = self.ball.x.clamp(BALL_RADIUS, ARENA.x - BALL_RADIUS);
        }
        if self.ball.y < BALL_RADIUS {
            self.velocity.y = self.velocity.y.abs();
            self.ball.y = BALL_RADIUS;
        }
        // The paddle sits at the bottom; y grows downwards, as on screen.
        let paddle_top = ARENA.y - PADDLE.y;
        if self.velocity.y > 0.0 && self.ball.y + BALL_RADIUS >= paddle_top {
            if (self.ball.x - self.paddle_x).abs() <= half + BALL_RADIUS {
                self.velocity.y = -self.velocity.y.abs();
                self.ball.y = paddle_top - BALL_RADIUS;
                self.score += 1;
            } else if self.ball.y > ARENA.y + BALL_RADIUS {
                self.lives = self.lives.saturating_sub(1);
                let (score, lives) = (self.score, self.lives);
                *self = Self {
                    score,
                    lives,
                    ..Self::default()
                };
            }
        }
    }

    pub fn over(&self) -> bool {
        self.lives == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_do_not_depend_on_the_frame_rate() {
        let mut at_30 = Game::default();
        let mut at_120 = Game::default();
        for _ in 0..90 {
            at_30.update(1.0 / 30.0, 1.0);
        }
        for _ in 0..360 {
            at_120.update(1.0 / 120.0, 1.0);
        }
        assert!((at_30.ball - at_120.ball).length() < 1e-3);
        assert!((at_30.paddle_x - at_120.paddle_x).abs() < 1e-3);
    }

    #[test]
    fn the_ball_bounces_off_the_walls_and_stays_in_the_arena() {
        let mut game = Game {
            velocity: vec2(-600.0, -500.0),
            ..Game::default()
        };
        for _ in 0..240 {
            game.update(1.0 / 60.0, 0.0);
            assert!(game.ball.x >= BALL_RADIUS && game.ball.x <= ARENA.x - BALL_RADIUS);
            assert!(game.ball.y >= BALL_RADIUS);
        }
    }

    #[test]
    fn catching_the_ball_scores_and_sends_it_back_up() {
        let mut game = Game {
            ball: vec2(320.0, ARENA.y - 40.0),
            velocity: vec2(0.0, 300.0),
            paddle_x: 320.0,
            ..Game::default()
        };
        game.update(0.2, 0.0);
        assert_eq!(game.score, 1);
        assert!(game.velocity.y < 0.0);
    }

    #[test]
    fn missing_the_ball_costs_a_life_and_restarts_the_serve() {
        let mut game = Game {
            ball: vec2(40.0, ARENA.y - 40.0),
            velocity: vec2(0.0, 300.0),
            paddle_x: 600.0,
            ..Game::default()
        };
        game.update(0.5, 0.0);
        assert_eq!(game.lives, 2);
        assert_eq!(game.ball, Game::default().ball);
        for _ in 0..2 {
            game.lives -= 1;
        }
        assert!(game.over());
    }
}
