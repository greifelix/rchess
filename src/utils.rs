use bevy::{math::ops::abs, platform::collections::HashSet, prelude::*};

use crate::game_logic::Direction;
use std::cmp::Ordering;
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

/// NOTE: We do not check knight positions here
pub fn king_prox(fig_pos: (u8, u8), king_pos: (u8, u8)) -> Direction {
    let (king_row, king_col) = king_pos;
    let (fig_row, fig_col) = fig_pos;
    match (fig_row.cmp(&king_row), fig_col.cmp(&king_col)) {
        (Ordering::Equal, Ordering::Greater) => Direction::R,
        (Ordering::Equal, Ordering::Less) => Direction::L,
        (Ordering::Greater, Ordering::Equal) => Direction::A,
        (Ordering::Less, Ordering::Equal) => Direction::B,
        (Ordering::Greater, Ordering::Greater) => {
            if fig_row - king_row == fig_col - king_col {
                Direction::AR
            } else {
                Direction::Unrelated
            }
        }
        (Ordering::Less, Ordering::Less) => {
            if king_row - fig_row == king_col - fig_col {
                Direction::BL
            } else {
                Direction::Unrelated
            }
        }
        (Ordering::Greater, Ordering::Less) => {
            if fig_row - king_row == king_col - fig_col {
                Direction::AL
            } else {
                Direction::Unrelated
            }
        }
        (Ordering::Less, Ordering::Greater) => {
            if king_row - fig_row == fig_col - king_col {
                Direction::BR
            } else {
                Direction::Unrelated
            }
        }
        _ => Direction::Unrelated,
    }
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
