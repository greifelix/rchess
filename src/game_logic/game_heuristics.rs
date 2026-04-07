use crate::utils::core_types::{FigType, PlayerColor};

use crate::game_logic::board_logic::Board;

// Board evaluation
pub fn evaluate_board(board: &Board, maximizer: &PlayerColor) -> i16 {
    board
        .0
        .iter()
        .flatten()
        .map(|maybe_fig| {
            if let Some(fig) = maybe_fig {
                let score = match fig.fig_type {
                    FigType::Pawn => 1,
                    FigType::Rook => 5,
                    FigType::Queen => 8,
                    FigType::Bishop => 3,
                    FigType::Knight => 3,
                    FigType::King => 0, // King does not matter as it is never hit
                };

                if fig.player_color == *maximizer {
                    score
                } else {
                    -score
                }
            } else {
                0
            }
        })
        .sum()
}

// Move ordering via move rating

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
