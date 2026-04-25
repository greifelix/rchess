/// Credits:
/// Thanks to the Chess Programming Wiki and its contributors for Piece Square Tables and material values.
/// The PST values and material scores are derived from the Simplified Evaluation Function:
///   https://www.chessprogramming.org/Simplified_Evaluation_Function
/// Originally authored by Tomasz Michniewski. Values have been reorganized into Rust const arrays.
/// The Chess Programming Wiki text is licensed under CC BY-SA 3.0: https://creativecommons.org/licenses/by-sa/3.0/
use crate::game_logic::board_logic::Board;
use crate::utils::core_types::{FigType, Figure, PlayerColor};
use itertools::iproduct;

// +++ Move ratings are following the most valuable victim / least valuable attacker logic and are used for ordering only +++

// Move ordering via move rating
pub fn rate_promotion() -> u8 {
    50
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
        FigType::King => 0,
        FigType::Queen => 1,
        FigType::Rook => 2,
        FigType::Bishop => 3,
        FigType::Knight => 4,
        FigType::Pawn => 5,
    }
}

fn _taken_val(taken: FigType) -> u8 {
    match taken {
        FigType::King => 0, // This should never happen, but in case we use naive moves for filtering invalid moves or something, we dont want to panic, so no panic
        FigType::Pawn => 10,
        FigType::Knight => 20,
        FigType::Bishop => 30,
        FigType::Rook => 40,
        FigType::Queen => 50,
    }
}

// +++ Board evaluation and Piece Square Tables +++
pub fn evaluate_board(board: &Board, maximizer: &PlayerColor) -> i16 {
    board
        .0
        .iter()
        .flatten()
        .map(|maybe_fig| {
            if let Some(fig) = maybe_fig {
                let score = match fig.fig_type {
                    FigType::Pawn => 100,
                    FigType::Knight => 320,
                    FigType::Bishop => 330,
                    FigType::Rook => 500,
                    FigType::Queen => 900,
                    FigType::King => 20_000, // King does not matter as it is never hit
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
        .sum::<i16>()
        + pst_board_val(board)
}

/// Compute the piece square table of the entire board.
pub fn pst_board_val(board: &Board) -> i16 {
    iproduct!(0..8, 0..8)
        .filter_map(|tile| board[tile].map(|fig| (tile, fig)))
        .map(|(tile, fig)| pst_piece_val(&tile, &fig))
        .sum()
}

/// Compute the piece square table of one piece (and its tile).
fn pst_piece_val(tile: &(u8, u8), fig: &Figure) -> i16 {
    let (row, col, factor) = match fig.player_color {
        PlayerColor::White => (tile.0, tile.1, 1),
        PlayerColor::Black => (tile.0, 7 - tile.1, -1),
    };

    let val = match fig.fig_type {
        FigType::Pawn => PST_PAWN[row as usize][col as usize],
        FigType::Knight => PST_KNIGHT[row as usize][col as usize],
        FigType::Bishop => PST_BISHOP[row as usize][col as usize],
        FigType::Rook => PST_ROOK[row as usize][col as usize],
        FigType::Queen => PST_QUEEN[row as usize][col as usize],
        FigType::King => PST_KING[row as usize][col as usize],
    };
    val * factor
}

const fn reverse_rows(arr: [[i16; 8]; 8]) -> [[i16; 8]; 8] {
    [
        arr[7], arr[6], arr[5], arr[4], arr[3], arr[2], arr[1], arr[0],
    ]
}

// +++ All PST's here are for the white player. Index recompute is used otherwise. +++
const PST_PAWN: [[i16; 8]; 8] = reverse_rows([
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
]);

const PST_KNIGHT: [[i16; 8]; 8] = reverse_rows([
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
]);

const PST_BISHOP: [[i16; 8]; 8] = reverse_rows([
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
]);

const PST_ROOK: [[i16; 8]; 8] = reverse_rows([
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
]);

const PST_QUEEN: [[i16; 8]; 8] = reverse_rows([
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10], // Hier Asymetrisch
    [-20, -10, -10, -5, -5, -10, -10, -20],
]);

const PST_KING: [[i16; 8]; 8] = reverse_rows([
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [20, 30, 10, 0, 0, 10, 30, 20],
]);
