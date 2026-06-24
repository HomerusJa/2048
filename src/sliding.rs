/// Convert a row of the 2048 [crate::board::Board] to an array of 4 tiles, with the leftmost tile at index 0 and the rightmost tile at index 3.
/// 
/// Note that the Board stores the tile (0,0) at the least significant bits, so the leftmost tile is stored in the least significant bits of the row.
/// 
/// This is the inverse of [tiles_to_row].
pub fn row_to_tiles(row: u16) -> [u8; 4] {
    [
        ((row >> 0) & 0xF) as u8,
        ((row >> 4) & 0xF) as u8,
        ((row >> 8) & 0xF) as u8,
        ((row >> 12) & 0xF) as u8,
    ]
}

/// Convert an array of 4 tiles to a row of the 2048 [crate::board::Board], with the leftmost tile at index 0 and the rightmost tile at index 3.
/// 
/// This is the inverse of [row_to_tiles].
pub fn tiles_to_row(tiles: [u8; 4]) -> u16 {
    ((tiles[0] as u16) << 0)
        | ((tiles[1] as u16) << 4)
        | ((tiles[2] as u16) << 8)
        | ((tiles[3] as u16) << 12)
}

/// Invert the order of the tiles in a row u16, so that the leftmost tile becomes the rightmost tile and vice versa.
pub fn invert_row(row: u16) -> u16 {
    let tiles = row_to_tiles(row);
    let inverted_tiles = invert_tiles(tiles);
    tiles_to_row(inverted_tiles)
}

/// Invert the order of the tiles in an array, so that the leftmost tile becomes the rightmost tile and vice versa.
pub fn invert_tiles(tiles: [u8; 4]) -> [u8; 4] {
    [tiles[3], tiles[2], tiles[1], tiles[0]]
}

pub fn calculate_row_left_to_right_slide(row: u16) -> u16 {
    let tiles = row_to_tiles(row);

    // Compress non-zero tiles preserving their left-to-right order.
    let nonzeros: Vec<u8> = tiles.iter().cloned().filter(|&t| t != 0).collect();

    // Merge tiles when sliding left->right: process from the right end of the
    // compressed list, merging adjacent equal tiles once.
    let mut merged: Vec<u8> = Vec::new();
    let mut i: isize = nonzeros.len() as isize - 1;
    while i >= 0 {
        let cur = nonzeros[i as usize];
        if i > 0 && nonzeros[(i - 1) as usize] == cur {
            // As the current tile and the previous tile are equal, merge them into a single tile with exponent +1.
            merged.push(cur + 1);
            // Skip the previous tile as it has been merged with the current tile.
            i -= 2;
        } else {
            merged.push(cur);
            i -= 1;
        }
    }

    // `merged` currently holds tiles from rightmost to leftmost; place them into
    // the result row starting at the rightmost index.
    let mut result = [0u8; 4];
    for (idx, &val) in merged.iter().enumerate() {
        if idx < 4 {
            result[3 - idx] = val;
        }
    }

    tiles_to_row(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_tiles_row_conversion() {
        let cases: Vec<u16> = vec![
            0x1234,
            0,
            0xFFFF,
            0x8000,
            0x8001,
        ];

        for case in cases {
            assert_eq!(tiles_to_row(row_to_tiles(case)), case)
        }
    }

    #[test]
    fn test_row_to_tiles() {
        let cases: Vec<(u16, [u8; 4])> = vec![
            (0x1234, [4, 3, 2, 1]),
            (0, [0, 0, 0, 0]),
            (0xFFFF, [15, 15, 15, 15]),
            (0x8000, [0, 0, 0, 8]),
            (0x8001, [1, 0, 0, 8]),
        ];

        for (case, expected) in cases {
            assert_eq!(row_to_tiles(case), expected)
        }
    }

    #[test]
    fn test_tiles_to_row() {
        let cases = vec![
            ([1, 2, 3, 4], 0x4321),
            ([0, 0, 0, 0], 0),
            ([15, 15, 15, 15], 0xFFFF),
            ([0, 0, 0, 8], 0x8000),
            ([1, 0, 0, 8], 0x8001),
        ];

        for (case, expected) in cases {
            assert_eq!(tiles_to_row(case), expected)
        }
    }

    #[test]
    fn test_invert_row() {
        let cases: Vec<(u16, u16)> = vec![
            (0x1234, 0x4321),
            (0, 0),
            (0xFFFF, 0xFFFF),
            (0x8000, 0x0008),
            (0x8001, 0x1008),
        ];

        for (case, expected) in cases {
            assert_eq!(invert_row(case), expected)
        }
    }

    #[test]
    fn test_invert_tiles() {
        let cases: Vec<([u8; 4], [u8; 4])> = vec![
            ([1, 2, 3, 4], [4, 3, 2, 1]),
            ([0, 0, 0, 0], [0, 0, 0, 0]),
            ([15, 15, 15, 15], [15, 15, 15, 15]),
            ([0, 0, 0, 8], [8, 0, 0, 0]),
            ([1, 0, 0, 8], [8, 0, 0, 1]),
        ];

        for (case, expected) in cases {
            assert_eq!(invert_tiles(case), expected)
        }
    }

    #[test]
    fn test_left_to_right_slide_some_cases() {
        let cases: Vec<([u8; 4], [u8; 4])> = vec![
            // basic sliding
            ([2, 0, 0, 0], [0, 0, 0, 2]),
            ([0, 2, 0, 0], [0, 0, 0, 2]),
            ([0, 0, 2, 0], [0, 0, 0, 2]),
            ([0, 0, 0, 2], [0, 0, 0, 2]),

            // multiple tiles
            ([2, 1, 0, 0], [0, 0, 2, 1]),
            ([2, 0, 1, 0], [0, 0, 2, 1]),
            ([0, 0, 1, 2], [0, 0, 1, 2]),
            ([1, 0, 2, 0], [0, 0, 1, 2]),
            ([1, 3, 5, 0], [0, 1, 3, 5]),
            ([1, 0, 3, 5], [0, 1, 3, 5]),
            ([0, 1, 3, 5], [0, 1, 3, 5]),
            ([5, 4, 3, 1], [5, 4, 3, 1]),
            
            // with merging
            ([1, 0, 0, 1], [0, 0, 0, 2]),
            ([1, 0, 1, 0], [0, 0, 0, 2]),
            ([1, 1, 0, 0], [0, 0, 0, 2]),
            ([2, 1, 1, 0], [0, 0, 2, 2]),
            ([2, 1, 0, 1], [0, 0, 2, 2]),
            ([2, 1, 1, 2], [0, 2, 2, 2]),
            ([1, 1, 1, 1], [0, 0, 2, 2]),
            ([2, 2, 1, 1], [0, 0, 3, 2]),
            ([1, 1, 4, 3], [0, 2, 4, 3]),
        ];

        for (case, expected) in cases {
            let result = calculate_row_left_to_right_slide(tiles_to_row(case));
            let result_tiles = row_to_tiles(result);
            assert_eq!(result_tiles, expected, "case: {:?} led to {:?}, when {:?} was expected", case, result_tiles, expected);
        }
    }
}
