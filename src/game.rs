use rand::{RngExt, SeedableRng};
use std::fs::OpenOptions;
use std::io::Write;

use crate::board::Board;
pub use crate::board::Direction;

#[derive(Debug)]
pub struct Game {
    board: Board,
    score: u32,
    seed: Option<u64>,
    rng: rand::rngs::SmallRng,
    debug_log_path: Option<String>,
    move_number: u32,
}

impl Game {
    pub fn from_seed(seed: u64) -> Self {
        let mut instance = Self {
            board: Board::default(),
            score: 0,
            seed: Some(seed),
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
            debug_log_path: None,
            move_number: 0,
        };
        instance.spawn_tile();
        instance
    }

    pub fn debug_log(&mut self, path: String) {
        self.debug_log_path = Some(path);

        if let Some(path) = &self.debug_log_path {
            // Clear the file and log the seed used for the RNG for later reproduction of the game state.
            let _ = std::fs::write(
                path,
                format!(
                    "Seed: {}\n\n",
                    self.seed
                        .map_or_else(|| "Unknown".to_string(), |v| v.to_string())
                ),
            );
            let _ = std::fs::write(path, format!("Initial board:\n"));
        }
        self.log_board();
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
            // .filter(|i| mask & (1 << (15 - i)) != 0)
            .nth(idx as usize)
            .unwrap();

        let exponent = if self.rng.random_bool(0.9) { 1 } else { 2 };

        if !(self.board.get_by_index(chosen_i) == 0) {
            if let Some(path) = &self.debug_log_path {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(file, "Error: Trying to spawn a tile on a non-empty space");
                }
            }
            panic!("Trying to spawn a tile on a non-empty space");
        }
        self.board = self.board.with_tile_by_index(chosen_i, exponent);
    }

    /// attempts to make a move in the given direction, returning true if the move was successful
    pub fn make_move(&mut self, direction: Direction) -> bool {
        if let Some((board, move_score)) = self.board.slide_with_score(direction) {
            self.board = board;
            self.score += move_score;

            self.move_number += 1;
            self.log_metadata(self.move_number, self.score, Some(direction));
            self.spawn_tile();
            self.log_board();
            true
        } else {
            false
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    fn log_metadata(&self, move_num: u32, score: u32, direction: Option<Direction>) {
        if let Some(path) = &self.debug_log_path {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "Move: {}", move_num);
                if let Some(direction) = direction {
                    let _ = writeln!(file, "Direction: {:?}", direction);
                }
                let _ = writeln!(file, "Score: {}", score);
            }
        }
    }

    fn log_board(&self) {
        if let Some(path) = &self.debug_log_path {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                for row in 0..4 {
                    let tiles = self.board.get_row_tiles(row);
                    let _ = writeln!(
                        file,
                        "{:5} {:5} {:5} {:5}",
                        tiles[0], tiles[1], tiles[2], tiles[3]
                    );
                }
                let _ = writeln!(file, "");
            }
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.board.is_game_over()
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::from_seed(rand::random())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_tile_bug_seed_15004643031978470866() {
        let mut game = Game::from_seed(15004643031978470866);
        game.make_move(Direction::Right);
        // This should not panic: attempting to spawn a tile on a non-empty space
        game.make_move(Direction::Right);
    }

    #[test]
    fn test_spawn_tile_bug_seed_15004643031978470866_up_right_right_up_right_right_right_up_up_right()
     {
        let mut game = Game::from_seed(15004643031978470866);
        game.make_move(Direction::Up);
        game.make_move(Direction::Right);
        game.make_move(Direction::Right);
        game.make_move(Direction::Up);
        game.make_move(Direction::Right);
        game.make_move(Direction::Right);
        game.make_move(Direction::Right);
        game.make_move(Direction::Up);
        game.make_move(Direction::Up);
        // This should not panic: attempting to spawn a tile on a non-empty space
        game.make_move(Direction::Right);
    }

    #[test]
    fn test_spawn_tile_bug_seed_15004643031978470866_down_down_down_right() {
        let mut game = Game::from_seed(15004643031978470866);
        game.make_move(Direction::Down);
        game.make_move(Direction::Down);
        game.make_move(Direction::Down);
        // This should not panic: attempting to spawn a tile on a non-empty space
        game.make_move(Direction::Right);
    }

    #[test]
    fn test_spawn_tile_bug_seed_15004643031978470866_right_down_right_right_down() {
        let mut game = Game::from_seed(15004643031978470866);
        game.make_move(Direction::Right);
        game.make_move(Direction::Down);
        game.make_move(Direction::Right);
        game.make_move(Direction::Right);
        // This should not panic: attempting to spawn a tile on a non-empty space
        game.make_move(Direction::Down);
    }

    #[test]
    fn test_spawn_tile_bug_seed_15004643031978470866_right_down_down() {
        let mut game = Game::from_seed(15004643031978470866);
        game.make_move(Direction::Right);
        game.make_move(Direction::Down);
        // This should not panic: attempting to spawn a tile on a non-empty space
        game.make_move(Direction::Down);
    }

    #[test]
    fn test_spawn_tile_bug_seed_15004643031978470866_right_right_down() {
        let mut game = Game::from_seed(15004643031978470866);
        game.make_move(Direction::Right);
        game.make_move(Direction::Right);
        // This should not panic: attempting to spawn a tile on a non-empty space
        game.make_move(Direction::Down);
    }
}
