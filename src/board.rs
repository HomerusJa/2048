use std::sync::LazyLock;

use crate::sliding::{calculate_row_left_to_right_slide, invert_row};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// Computing the slides four times has a negligible performance impact, but it makes the code much simpler and easier to read.
// It was verified to take less than 100ms to compute all four tables on my machine.
static SLIDE_TABLE: LazyLock<[u16; 65536]> = LazyLock::new(|| {
    let mut table = [0u16; 65536];
    for i in 0..65536 {
        table[i] = calculate_row_left_to_right_slide(i as u16).result_row;
    }
    table
});

static SCORE_TABLE: LazyLock<[u32; 65536]> = LazyLock::new(|| {
    let mut table = [0u32; 65536];
    for i in 0..65536 {
        table[i] = calculate_row_left_to_right_slide(i as u16).score;
    }
    table
});

static INVERTED_SLIDE_TABLE: LazyLock<[u16; 65536]> = LazyLock::new(|| {
    let mut table = [0u16; 65536];
    for i in 0..65536 {
        let inverted_row = invert_row(i as u16);
        let slid_row = calculate_row_left_to_right_slide(inverted_row).result_row;
        table[i] = invert_row(slid_row);
    }
    table
});

static INVERTED_SCORE_TABLE: LazyLock<[u32; 65536]> = LazyLock::new(|| {
    let mut table = [0u32; 65536];
    for i in 0..65536 {
        let inverted_row = invert_row(i as u16);
        let slid_row = calculate_row_left_to_right_slide(inverted_row);
        table[i] = slid_row.score;
    }
    table
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    /// 16 fields, 4 bits each, with the exponent of the tile (0 for empty, 1 for 2, 2 for 4, etc.)
    ///
    /// (0,0) is stored in the least significant 4 bits, (3,3) in the most significant 4 bits.
    /// This means that a position (x,y) is stored at bit index `4*(x+y*4)` to `4*(x+y*4)+3`.
    board: u64,
}

impl Board {
    pub fn new(initial_state: u64) -> Self {
        Board {
            board: initial_state,
        }
    }

    /// Get the value of the tile at (x,y). (0,0) is the top-left corner, (3,3) is the bottom-right corner.
    pub fn get(&self, x: u8, y: u8) -> u32 {
        assert!(x < 4 && y < 4, "Coordinates out of bounds");
        self.get_by_index(x + y * 4)
    }

    /// Get the value of the tile at index i (0-15). 0 is the top-left corner, 15 is the bottom-right corner.
    /// We first go through each row, then the columns.
    ///
    /// So the order is: (0,0), (1,0), (2,0), (3,0), (0,1), (1,1), (2,1), (3,1), (0,2), (1,2), (2,2), (3,2), (0,3), (1,3), (2,3), (3,3).
    pub fn get_by_index(&self, i: u8) -> u32 {
        assert!(i < 16, "Index out of bounds");
        let shift = i * 4;
        let exp = (self.board >> shift) & 0x0F;
        if exp == 0 { 0 } else { 1 << exp }
    }

    #[inline(always)]
    pub fn get_exponent(&self, x: u8, y: u8) -> u8 {
        assert!(x < 4 && y < 4, "Coordinates out of bounds");
        self.get_exponent_by_index(x + y * 4)
    }

    #[inline(always)]
    pub fn get_exponent_by_index(&self, i: u8) -> u8 {
        assert!(i < 16, "Index out of bounds");
        let shift = i * 4;
        ((self.board >> shift) & 0x0F) as u8
    }

    /// Set the value of the tile at (x,y) to 2^exponent. (0,0) is the top-left corner, (3,3) is the bottom-right corner.
    #[inline(always)]
    pub fn with_tile(&self, x: u8, y: u8, exponent: u8) -> Self {
        assert!(x < 4 && y < 4, "Coordinates out of bounds");
        self.with_tile_by_index(x + y * 4, exponent)
    }

    /// Set the value of the tile at index i (0-15) to 2^exponent. 0 is the top-left corner, 15 is the bottom-right corner.
    ///
    /// So the order is: (0,0), (1,0), (2,0), (3,0), (0,1), (1,1), (2,1), (3,1), (0,2), (1,2), (2,2), (3,2), (0,3), (1,3), (2,3), (3,3).
    #[inline(always)]
    pub fn with_tile_by_index(&self, i: u8, exponent: u8) -> Self {
        assert!(i < 16, "Index out of bounds");
        let shift = i * 4;

        let mask = 0xFu64 << shift;

        // First clear out the tile, then set it to the new value
        let new_board = (self.board & !mask) | ((exponent as u64) << shift);
        Board::new(new_board)
    }

    fn transpose(&self) -> Self {
        let mut new_board = 0u64;
        for x in 0..4 {
            for y in 0..4 {
                let exp = self.get_exponent(x, y);
                new_board |= (exp as u64) << ((y + x * 4) * 4);
            }
        }
        Board::new(new_board)
    }

    /// Slide the board in the given direction. Returns None if the board does not change.
    pub fn slide(&self, direction: Direction) -> Option<Self> {
        self.slide_with_score(direction)
            .map(|(new_board, _)| new_board)
    }

    #[inline(always)]
    fn get_row(&self, row: u8) -> u16 {
        assert!(row < 4, "Row out of bounds");
        ((self.board >> (row * 16)) & 0xFFFF) as u16
    }

    /// Slide the board in the given direction, returning the new board and the score gained from merging tiles.
    /// Returns None if the board does not change.
    pub fn slide_with_score(&self, direction: Direction) -> Option<(Self, u32)> {
        let mut new_board = 0u64;
        let mut score = 0u32;
        match direction {
            Direction::Right => {
                for row in 0..4 {
                    let row_value = self.get_row(row);
                    let slid_row = SLIDE_TABLE[row_value as usize];
                    let row_score = SCORE_TABLE[row_value as usize];
                    new_board |= (slid_row as u64) << (row * 16);
                    score += row_score;
                }
            }
            Direction::Left => {
                for row in 0..4 {
                    let row_value = self.get_row(row);
                    let slid_row = INVERTED_SLIDE_TABLE[row_value as usize];
                    let row_score = INVERTED_SCORE_TABLE[row_value as usize];
                    new_board |= (slid_row as u64) << (row * 16);
                    score += row_score;
                }
            }
            Direction::Down => {
                let transposed = self.transpose();
                for row in 0..4 {
                    let row_value = transposed.get_row(row);
                    let slid_row = SLIDE_TABLE[row_value as usize];
                    let row_score = SCORE_TABLE[row_value as usize];
                    new_board |= (slid_row as u64) << (row * 16);
                    score += row_score;
                }
            }
            Direction::Up => {
                let transposed = self.transpose();
                for row in 0..4 {
                    let row_value = transposed.get_row(row);
                    let slid_row = INVERTED_SLIDE_TABLE[row_value as usize];
                    let row_score = INVERTED_SCORE_TABLE[row_value as usize];
                    new_board |= (slid_row as u64) << (row * 16);
                    score += row_score;
                }
            }
        }
        if new_board == self.board {
            None
        } else {
            let board;
            if matches!(direction, Direction::Up | Direction::Down) {
                board = Board::new(new_board).transpose();
            } else {
                board = Board::new(new_board);
            }
            Some((board, score))
        }
    }

    /// Returns a 16-bit mask where each bit corresponds to a tile on the board.
    /// A bit is set to 1 if the corresponding tile is empty, and 0 if it is occupied.
    #[inline(always)]
    pub fn empty_mask(&self) -> u16 {
        let mut mask = 0u16;
        for i in 0..16 {
            if self.get_by_index(i) == 0 {
                mask = mask | (1 << (15 - i));
            }
        }
        mask
    }

    /// Returns true if the game is over (no more valid moves), false otherwise.
    pub fn is_game_over(&self) -> bool {
        self.slide_with_score(Direction::Left).is_none()
            && self.slide_with_score(Direction::Right).is_none()
            && self.slide_with_score(Direction::Up).is_none()
            && self.slide_with_score(Direction::Down).is_none()
    }

    /// Returns the maximum tile value on the board.
    pub fn max_tile(&self) -> u32 {
        let mut current = 0u32;
        for i in 0..16 {
            current = current.max(self.get_by_index(i));
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_board() {
        let board = Board::new(0);

        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(board.get(x, y), 0);
            }
        }
    }

    #[test]
    fn test_single_tile() {
        let board = Board::new(0).with_tile(0, 0, 1);

        assert_eq!(board.get(0, 0), 2);

        for y in 0..4 {
            for x in 0..4 {
                if (x, y) != (0, 0) {
                    assert_eq!(board.get(x, y), 0);
                }
            }
        }
    }

    #[test]
    fn test_multiple_tiles() {
        let board = Board::new(0)
            .with_tile(0, 0, 1) // 2
            .with_tile(1, 1, 2) // 4
            .with_tile(2, 2, 3) // 8
            .with_tile(3, 3, 4); // 16

        assert_eq!(board.get(0, 0), 2);
        assert_eq!(board.get(1, 1), 4);
        assert_eq!(board.get(2, 2), 8);
        assert_eq!(board.get(3, 3), 16);
    }

    #[test]
    fn test_overwrite_tile() {
        let board = Board::new(0).with_tile(2, 1, 1).with_tile(2, 1, 5);

        assert_eq!(board.get(2, 1), 32);
    }

    #[test]
    fn test_clear_tile() {
        let board = Board::new(0).with_tile(1, 2, 4).with_tile(1, 2, 0);

        assert_eq!(board.get(1, 2), 0);
    }

    #[test]
    fn test_index_mapping() {
        let board = Board::new(0)
            .with_tile_by_index(0, 1)
            .with_tile_by_index(5, 2)
            .with_tile_by_index(15, 3);

        assert_eq!(board.get(0, 0), 2);
        assert_eq!(board.get(1, 1), 4);
        assert_eq!(board.get(3, 3), 8);
    }

    #[test]
    fn test_roundtrip_all_cells() {
        for i in 0..16 {
            let board = Board::new(0).with_tile_by_index(i, 7);

            assert_eq!(board.get_by_index(i), 128);
        }
    }

    #[test]
    fn test_empty_mask_emtpy_board() {
        let board = Board::new(0);
        assert_eq!(board.empty_mask(), 0xFFFF);
    }

    #[test]
    fn test_empty_mask_full_board() {
        let board = Board::new(0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(board.empty_mask(), 0);
    }

    #[test]
    fn test_empty_mask_some_filled() {
        let board = Board::new(0)
            .with_tile_by_index(0, 1)
            .with_tile_by_index(5, 3);
        let mask = board.empty_mask();
        assert_eq!(mask, 0b0111_1011_1111_1111);
    }

    #[test]
    fn test_max_tile_empty_board() {
        let board = Board::new(0);
        assert_eq!(board.max_tile(), 0);
    }

    #[test]
    fn test_max_tile_diverse_board() {
        let board = Board::new(0)
            .with_tile(1, 2, 3)
            .with_tile(3, 0, 2)
            .with_tile(0, 0, 5)
            .with_tile(0, 1, 4);

        assert_eq!(board.max_tile(), 32)
    }

    #[test]
    fn test_basic_slide_with_score() {
        let board = Board::new(0).with_tile(0, 0, 1).with_tile(1, 0, 1);

        let (new_board, score) = board.slide_with_score(Direction::Right).unwrap();

        let expected_board = Board::new(0).with_tile(3, 0, 2);
        assert_eq!(new_board, expected_board);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_slide_right_moves_row_into_place() {
        let board = Board::new(0).with_tile(0, 2, 1).with_tile(2, 2, 2);

        let new_board = board.slide(Direction::Right).unwrap();

        let expected_board = Board::new(0).with_tile(2, 2, 1).with_tile(3, 2, 2);
        assert_eq!(new_board, expected_board);
    }

    #[test]
    fn test_slide_up_basic() {
        let board = Board::new(0).with_tile(1, 1, 1).with_tile(1, 3, 1);

        let (new_board, score) = board.slide_with_score(Direction::Up).unwrap();

        let expected_board = Board::new(0).with_tile(1, 0, 2);
        assert_eq!(new_board, expected_board);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_slide_down_basic() {
        let board = Board::new(0).with_tile(2, 0, 1).with_tile(2, 2, 1);

        let (new_board, score) = board.slide_with_score(Direction::Down).unwrap();

        let expected_board = Board::new(0).with_tile(2, 3, 2);
        assert_eq!(new_board, expected_board);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_slide_left_basic() {
        let board = Board::new(0).with_tile(3, 1, 1).with_tile(1, 1, 1);

        let (new_board, score) = board.slide_with_score(Direction::Left).unwrap();

        let expected_board = Board::new(0).with_tile(0, 1, 2);
        assert_eq!(new_board, expected_board);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_transpose() {
        let board = Board::new(0).with_tile(1, 3, 1);

        let transposed = board.transpose();

        assert_eq!(transposed.get(3, 1), 2);
    }

    #[test]
    fn test_transpose_twice() {
        let board = Board::new(0)
            .with_tile(0, 0, 1)
            .with_tile(1, 2, 2)
            .with_tile(3, 1, 3);

        let transposed_twice = board.transpose().transpose();

        assert_eq!(transposed_twice, board);
    }
}
