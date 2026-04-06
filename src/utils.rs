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

pub fn pawn_promotion(pawn_name: &str, player_color: PlayerColor) -> Figure {
    match player_color {
        PlayerColor::Black => _black_promotion(pawn_name),
        PlayerColor::White => _white_promotion(pawn_name),
    }
}

/// Workaround which works with static strings
pub fn _white_promotion(pawn_name: &str) -> Figure {
    match pawn_name {
        "Pawn a2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen a2",
            player_color: PlayerColor::White,
        },
        "Pawn b2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen b2",
            player_color: PlayerColor::White,
        },
        "Pawn c2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen c2",
            player_color: PlayerColor::White,
        },
        "Pawn d2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen d2",
            player_color: PlayerColor::White,
        },
        "Pawn e2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen e2",
            player_color: PlayerColor::White,
        },
        "Pawn f2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen f2",
            player_color: PlayerColor::White,
        },
        "Pawn g2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen g2",
            player_color: PlayerColor::White,
        },
        "Pawn h2" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen h2",
            player_color: PlayerColor::White,
        },
        _ => panic!("Invalid pawn name for promotion!"),
    }
}

/// Workaround which works with static strings
pub fn _black_promotion(pawn_name: &str) -> Figure {
    match pawn_name {
        "Pawn a7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen a7",
            player_color: PlayerColor::Black,
        },
        "Pawn b7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen b7",
            player_color: PlayerColor::Black,
        },
        "Pawn c7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen c7",
            player_color: PlayerColor::Black,
        },
        "Pawn d7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen d7",
            player_color: PlayerColor::Black,
        },
        "Pawn e7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen e7",
            player_color: PlayerColor::Black,
        },
        "Pawn f7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen f7",
            player_color: PlayerColor::Black,
        },
        "Pawn g7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen g7",
            player_color: PlayerColor::Black,
        },
        "Pawn h7" => Figure {
            fig_type: FigType::Queen,
            ass_name: "Queen h7",
            player_color: PlayerColor::Black,
        },
        _ => panic!("Invalid pawn name for promotion!"),
    }
}
