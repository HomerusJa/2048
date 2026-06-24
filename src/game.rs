use rand::RngExt;

use crate::board::{Board, Direction};

pub struct Game {
    board: Board,
    score: u32,
    rng: rand::rngs::SmallRng,
}

impl Game {
    pub fn new(rng: rand::rngs::SmallRng) -> Self {
        let mut instance = Self {
            board: Board::new(0 as u64),
            score: 0,
            rng,
        };
        instance.spawn_tile();
        instance
    }

    fn spawn_tile(&mut self) {
        let mask = self.board.empty_mask();
        let empty_count = mask.count_ones();

        if empty_count == 0 {
            return;
        }

        let idx = self.rng.random_range(0..empty_count);

        let chosen_i = (0..16)
            .filter(|i| mask & (1 << i) != 0)
            .nth(idx as usize)
            .unwrap();

        let exponent = if self.rng.random_bool(0.9) { 1 } else { 2 };
        self.board = self.board.with_tile_by_index(chosen_i, exponent)
    }

    pub fn make_move(&mut self, direction: Direction) {
        if let Some((board, move_score)) = self.board.slide_with_score(direction) {
            self.board = board;
            self.score += move_score;

            self.spawn_tile();
        }
    }

    pub fn board(&self) -> Board {
        self.board
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn is_game_over(&self) -> bool {
        self.board.is_game_over()
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(rand::make_rng())
    }
}
