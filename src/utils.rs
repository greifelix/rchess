use bevy::{platform::collections::HashSet, prelude::*};

use crate::game_logic::FigType;
pub fn tile_to_indices(tile_name: &str) -> (u8, u8) {
    let sub_strings: Vec<&str> = tile_name.split_terminator('_').collect();
    (
        sub_strings[1].parse::<u8>().unwrap(),
        sub_strings[2].parse::<u8>().unwrap(),
    )
}

pub fn idx_to_coordinates(row: u8, col: u8) -> (f32, f32) {
    let square_size = 0.05;
    let offset = 0.025;

    let row_offset: f32 = offset + 3.0 * square_size - row as f32 * square_size;
    let col_offset: f32 = -offset - 3.0 * square_size + col as f32 * square_size;
    (row_offset, col_offset)
}

/// Figs are direct neighbots, horizontally, vertically or diagonal
pub fn figs_adjacent(f1: (u8, u8), f2: (u8, u8)) -> bool {
    (f1.0.abs_diff(f2.0)) < 2 && (f1.1.abs_diff(f2.1) < 2)
}

pub fn knights_reach(from_pos: (u8, u8)) -> HashSet<(u8, u8)> {
    let (from_row, from_col) = from_pos;
    let mut cands = HashSet::new();
    // 2-hoch 1-links/rechts
    if from_row + 2 < 8 {
        if from_col + 1 < 8 {
            cands.insert((from_row + 2, from_col + 1));
        }
        if from_col.saturating_sub(1) < from_col {
            cands.insert((from_row + 2, from_col - 1));
        }
    }
    // 2-runter 1-links/rechts
    if from_row.saturating_sub(2) + 2 == from_row {
        if from_col + 1 < 8 {
            cands.insert((from_row - 2, from_col + 1));
        }
        if from_col.saturating_sub(1) < from_col {
            cands.insert((from_row - 2, from_col - 1));
        }
    }
    // 2-rechts 1-oben/unten
    if from_col + 2 < 8 {
        if from_row + 1 < 8 {
            cands.insert((from_row + 1, from_col + 2));
        }
        if from_row.saturating_sub(1) < from_row {
            cands.insert((from_row - 1, from_col + 2));
        }
    }

    // 2 links 1-oben/unten
    if from_col.saturating_sub(2) + 2 == from_col {
        if from_row + 1 < 8 {
            cands.insert((from_row + 1, from_col - 2));
        }
        if from_row.saturating_sub(1) < from_row {
            cands.insert((from_row - 1, from_col - 2));
        }
    }
    cands
}

pub fn rate_promotion() -> u8 {
    16
}


/// Rate standard_move for move ordering
pub fn rate_standard_move(moving: FigType, taken: Option<FigType>) -> u8 {
    let Some(taken) = taken else {
        return 0;
    };

    _moving_val(moving) + _taken_val(taken)
}

fn _moving_val(moving: FigType) -> u8 {
    match moving {
        FigType::Pawn => 4,
        FigType::Bishop => 3,
        FigType::Knight => 3,
        FigType::Rook => 2,
        FigType::Queen => 1,
        FigType::King => 0,
    }
}

fn _taken_val(taken: FigType) -> u8 {
    match taken {
        FigType::Pawn => 1,
        FigType::Bishop => 3,
        FigType::Knight => 3,
        FigType::Rook => 5,
        FigType::Queen => 8,
        FigType::King => 0, // This should never happen, but in case we use naive moves for filtering invalid moves or something, we dont want to panic, so no panic
    }
}
