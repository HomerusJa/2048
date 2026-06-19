#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    board: u64, // 16 fields, 4 bits each
}

impl Board {
    pub fn new(initial_state: u64) -> Self {
        Board {
            board: initial_state,
        }
    }

    pub fn as_u64(&self) -> u64 {
        self.board
    }

    pub fn get(&self, x: u8, y: u8) -> u32 {
        assert!(x < 4 && y < 4, "Coordinates out of bounds");
        self.get_by_index(x + y*4)
    }

    #[inline(always)]
    pub fn get_by_index(&self, i: u8) -> u32 {
        assert!(i < 16, "Index out of bounds");
        let shift = i * 4;
        let exp = (self.board >> shift) & 0x0F;
        if exp == 0 { 0 } else { 1 << exp }
    }

    pub fn with_tile(&self, x: u8, y: u8, exponent: u8) -> Self {
        assert!(x < 4 && y < 4, "Coordinates out of bounds");
        self.with_tile_by_index(x + y*4, exponent)
    }

    #[inline(always)]
    pub fn with_tile_by_index(&self, i: u8, exponent: u8) -> Self {
        assert!(i < 16, "Index out of bounds");
        let shift = i*4;
        
        let mask = 0xFu64 << shift;
        
        // First clear out the tile, then set it to the new value
        let new_board = (self.board & !mask) | ((exponent as u64) << shift);
        Board::new(new_board)
    }

    pub fn slide(&self, direction: Direction) -> Option<Self> {
        todo!("To be implemented!")
    }

    pub fn slide_with_score(&self, direction: Direction) -> Option<(Self, u32)> {
        todo!("To be implemented!")
    }

    #[inline(always)]
    pub fn empty_mask(&self) -> u16 {
        todo!("To be implemented")
    }

    pub fn is_game_over(&self) -> bool {
        todo!("To be implemented")
    }

    pub fn max_tile(&self) -> (u8, u8) {
        todo!("To be implemented")
    }
}
