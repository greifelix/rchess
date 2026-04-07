pub mod board_utils;
pub mod core_types;
pub mod picking_utils;
pub mod setup_utils;
use bevy::{platform::collections::HashSet, prelude::*};
use core_types::*;

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

/// Figs are direct neighbours, horizontally, vertically or diagonal
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

pub fn pawn_promotion(pawn_name: &str, player_color: PlayerColor) -> &str {
    match player_color {
        PlayerColor::Black => _black_promotion(pawn_name),
        PlayerColor::White => _white_promotion(pawn_name),
    }
}

/// Workaround which works with static strings
pub fn _white_promotion(pawn_name: &str) -> &str {
    match pawn_name {
        "Pawn a2" => "Queen a2",
        "Pawn b2" => "Queen b2",
        "Pawn c2" => "Queen c2",
        "Pawn d2" => "Queen d2",
        "Pawn e2" => "Queen e2",
        "Pawn f2" => "Queen f2",
        "Pawn g2" => "Queen g2",
        "Pawn h2" => "Queen h2",
        _ => panic!("Invalid pawn name for promotion!"),
    }
}

/// Workaround which works with static strings
pub fn _black_promotion(pawn_name: &str) -> &str {
    match pawn_name {
        "Pawn a7" => "Queen a7",
        "Pawn b7" => "Queen b7",
        "Pawn c7" => "Queen c7",
        "Pawn d7" => "Queen d7",
        "Pawn e7" => "Queen e7",
        "Pawn f7" => "Queen f7",
        "Pawn g7" => "Queen g7",
        "Pawn h7" => "Queen h7",
        _ => panic!("Invalid pawn name for promotion!"),
    }
}
