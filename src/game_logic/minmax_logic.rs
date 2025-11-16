use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use crate::game_logic::{
    Board, FigType, GameState, PlayerColor,
    movement_logic::{self, ChessMove},
};
use bevy::platform::collections::HashMap;


// pub fn singleplayer_plugin(app: &mut App) {
//     app.insert_resource(GeneratedMoves::new())
//         .add_systems(Update, (spawn_minmax_task, retrieve_and_exec_minmax_result));
// }

#[derive(Clone)]
pub struct MinMaxData {
    value: i16,
    max_move: Option<ChessMove>,
}

impl MinMaxData {
    pub fn new_val(val: i16) -> MinMaxData {
        MinMaxData {
            value: val,
            max_move: None,
        }
    }
}

const MAX_DEPTH: u8 = 6;
const MAXIMIZER: PlayerColor = PlayerColor::Black;

/// This is just used as means to save the generated moves over time
#[derive(Resource)]
pub struct GeneratedMoves {
    data: HashMap<usize, Task<Option<ChessMove>>>,
}

impl GeneratedMoves {
    pub fn new() -> Self {
        GeneratedMoves {
            data: HashMap::new(),
        }
    }
}

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

/// Spawns an asynchronous task to find a move for the black player, in case there is not one spawned yet
pub fn spawn_minmax_task(game_state: Res<GameState>, mut minmax_moves: ResMut<GeneratedMoves>) {
    if game_state.player_turn == MAXIMIZER
        && !minmax_moves.data.contains_key(&game_state.move_number)
    {
        let task_pool = AsyncComputeTaskPool::get();

        let board = game_state.board.clone();
        let task = task_pool.spawn(async move {
            let found_move = mmax(MAXIMIZER, MAX_DEPTH, &board, -1000, 1000);
            found_move.max_move
        });

        minmax_moves.data.insert(game_state.move_number, task);
    }
}

pub fn retrieve_and_exec_minmax_result(
    mut commands: Commands,
    mut minmax_moves: ResMut<GeneratedMoves>,
    mut game_state: ResMut<GameState>,
    mut piece_query: Query<(Entity, &Name, &mut Transform)>,
) {
    if let Some(t) = minmax_moves.data.get_mut(&game_state.move_number) {
        let status = block_on(future::poll_once(t));
        if status.is_some() {
            println!("The task finished!")
        }
        if let Some(max_move) = status.flatten() {
            println!("Also we found a move, nice!  - Lets try to execute it!");

            let (from_row, from_col) = max_move.from_tile;
            let ass_name = game_state.board[(from_row, from_col)].unwrap().ass_name;

            game_state.execute_move(&mut commands, ass_name, &max_move, &mut piece_query);
        }
    }
}

pub fn mmax(player: PlayerColor, depth: u8, board: &Board, alpha: i16, beta: i16) -> MinMaxData {
    let maxplayer_moves = movement_logic::calculate_all_smarter(board, player);

    let num_moves_left = maxplayer_moves.len();
    let mut max_value = alpha;

    if depth == 0 || num_moves_left == 0 {
        if num_moves_left > 0 {
            return MinMaxData::new_val(evaluate_board(board, &MAXIMIZER));
        } else {
            return MinMaxData::new_val(evaluate_board(board, &MAXIMIZER) + max_value);
        }
    }

    let mut max_move: Option<ChessMove> = None;

    for chess_move in maxplayer_moves {
        let mut board_copy = board.clone();

        board_copy.update(&chess_move, &player);

        let mmin_val = mmin(
            player.other_player(),
            depth - 1,
            &board_copy,
            max_value,
            beta,
        );

        if mmin_val > max_value {
            max_value = mmin_val;
            if depth == MAX_DEPTH {
                max_move = Some(chess_move.clone());
            }

            if max_value >= beta {
                return MinMaxData {
                    value: max_value,
                    max_move: max_move,
                };
            }
        }
    }

    MinMaxData {
        value: max_value,
        max_move: max_move,
    }
}

pub fn mmin(player: PlayerColor, depth: u8, board: &Board, alpha: i16, beta: i16) -> i16 {
    let minplayer_moves = movement_logic::calculate_all_smarter(board, player);
    let num_moves_left = minplayer_moves.len();

    let mut min_value = beta;
    if depth == 0 || num_moves_left == 0 {
        if num_moves_left > 0 {
            return evaluate_board(board, &MAXIMIZER);
        } else {
            return evaluate_board(board, &MAXIMIZER) + min_value;
        }
    }

    for chess_move in minplayer_moves {
        let mut board_copy = board.clone();

        board_copy.update(&chess_move, &player);

        let mmax_val = mmax(
            player.other_player(),
            depth - 1,
            &board_copy,
            alpha,
            min_value,
        )
        .value;
        if mmax_val < min_value {
            min_value = mmax_val;
        }

        if min_value <= alpha {
            return min_value;
        }
    }

    min_value
}
