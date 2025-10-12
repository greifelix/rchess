// Add code related to minmaxing the ai here

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use std::collections::HashMap;

use crate::game_logic::{Board, FigType, GameState, PlayerColor, PossibleMoves, movement_logic};

#[derive(Copy, Clone, Debug)]
pub struct MaxMove {
    from_tile: (usize, usize),
    to_tile: (usize, usize),
}

#[derive(Copy, Clone, Debug)]
pub struct MinMaxData {
    value: i32,
    max_move: Option<MaxMove>,
}

impl MinMaxData {
    pub fn new_val(val: i32) -> MinMaxData {
        MinMaxData {
            value: val,
            max_move: None,
        }
    }
}

const MAX_DEPTH: u8 = 4;

/// This is just used as means to save the generated moves over time
#[derive(Resource)]
pub struct GeneratedMoves {
    data: HashMap<usize, Task<Option<MaxMove>>>,
}

impl GeneratedMoves {
    pub fn new() -> Self {
        GeneratedMoves {
            data: HashMap::new(),
        }
    }
}

pub fn evaluate_board(board: &Board) -> i32 {
    board
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
                    FigType::King => 10_000,
                };

                match fig.player_color {
                    PlayerColor::Black => score,
                    PlayerColor::White => -score,
                }
            } else {
                0
            }
        })
        .sum()
}

/// Spawns an asynchronous task to find a move for the black player, in case there is not one spawned yet
pub fn spawn_minmax_task(game_state: Res<GameState>, mut minmax_moves: ResMut<GeneratedMoves>) {
    if game_state.player_turn == PlayerColor::Black
        && !minmax_moves.data.contains_key(&game_state.move_number)
    {
        let task_pool = AsyncComputeTaskPool::get();
        let board = game_state.board.clone();
        let task = task_pool.spawn(async move {
            let found_move = mmax(PlayerColor::Black, MAX_DEPTH, board);
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
            let ass_name = game_state
                .board
                .get_fig_on_tile(from_row, from_col)
                .unwrap()
                .ass_name;

            // Ugly for now, but I have to add the move to the possible moves now :D
            game_state.possible_moves = Some(PossibleMoves {
                from_tile: max_move.from_tile,
                to: std::collections::HashSet::from([max_move.to_tile]),
            });

            game_state.execute_move(
                &mut commands,
                ass_name,
                (from_row, from_col),
                max_move.to_tile,
                &mut piece_query,
            );
        }
    }
}

/// Maximizing Player, right now this is hard coded to be the black player.
pub fn mmax(player: PlayerColor, depth: u8, board: Board) -> MinMaxData {
    if depth == 0 {
        return MinMaxData::new_val(evaluate_board(&board));
    }
    let moves_all_figures = movement_logic::MoveBuilder::calculate_all(&board, player);
    if moves_all_figures.len() == 0 {
        return MinMaxData::new_val(evaluate_board(&board));
    }
    let mut max_value = -100_000;
    let mut max_move: Option<MaxMove> = None;

    for moves_one_figure in moves_all_figures {
        let (from_row, from_col) = moves_one_figure.from_tile;
        for (to_row, to_col) in moves_one_figure.to {
            let mut board_copy = board.clone();
            board_copy[to_row][to_col] = board_copy[from_row][from_col].take();

            let min_val = mmin(player.other_player(), depth - 1, board_copy);
            if min_val > max_value {
                max_value = min_val;
                if depth == MAX_DEPTH {
                    max_move = Some(MaxMove {
                        from_tile: (from_row, from_col),
                        to_tile: (to_row, to_col),
                    })
                }
            }
        }
    }

    MinMaxData {
        value: max_value,
        max_move: max_move,
    }
}

pub fn mmin(
    player: PlayerColor,
    depth: u8,
    board: Board,
) -> i32 {
    if depth == 0 {
        return evaluate_board(&board);
    }
    let moves_all_figures = movement_logic::MoveBuilder::calculate_all(&board, player);
    if moves_all_figures.len() == 0 {
        return evaluate_board(&board);
    }
    let mut min_value = 100_000;

    for moves_one_figure in moves_all_figures {
        let (from_row, from_col) = moves_one_figure.from_tile;
        for (to_row, to_col) in moves_one_figure.to {
            let mut board_copy = board.clone();
            board_copy[to_row][to_col] = board_copy[from_row][from_col].take();

            let max_val = mmax(player.other_player(), depth - 1, board_copy).value;
            if max_val < min_value {
                min_value = max_val;
            }
        }
    }
    min_value
}
