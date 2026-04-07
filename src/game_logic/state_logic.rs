/// Tracks the Game State including asset names for visualizations
use bevy::{
    gltf::GltfMesh,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
// use itertools::{Itertools, iproduct};
use crate::game_logic::board_logic::Board;
use crate::game_logic::movement_logic::{ChessMove, MoveType};

use crate::utils::{
    board_utils::queen_spawner,
    core_types::{ChessScene, Figure, PlayerColor, WHITE_KING_SP},
    idx_to_coordinates, pawn_promotion,
};
use std::ops::{Index, IndexMut};

/// "Bigger" version of a board, which also contains the asset names for gltf identification.
/// The fat_update method does both the update of the asset map as well as the standard map
pub struct FatBoard {
    pub board: Board,
    pub asset_map: [[Option<&'static str>; 8]; 8],
}

impl Index<(u8, u8)> for FatBoard {
    type Output = Option<&'static str>;
    fn index(&self, index: (u8, u8)) -> &Self::Output {
        &self.asset_map[index.0 as usize][index.1 as usize]
    }
}

impl IndexMut<(u8, u8)> for FatBoard {
    fn index_mut(&mut self, index: (u8, u8)) -> &mut Self::Output {
        &mut self.asset_map[index.0 as usize][index.1 as usize]
    }
}

impl FatBoard {
    fn new() -> Self {
        let white_pieces_str = [
            Some("Rook a1"),
            Some("Knight b1"),
            Some("Bishop c1"),
            Some("Queen d1"),
            Some("King e1"),
            Some("Bishop f1"),
            Some("Knight g1"),
            Some("Rook h1"),
        ];
        let black_pieces_str = [
            Some("Rook a8"),
            Some("Knight b8"),
            Some("Bishop c8"),
            Some("Queen d8"),
            Some("King e8"),
            Some("Bishop f8"),
            Some("Knight g8"),
            Some("Rook h8"),
        ];

        let white_pawns_str = [
            Some("Pawn a2"),
            Some("Pawn b2"),
            Some("Pawn c2"),
            Some("Pawn d2"),
            Some("Pawn e2"),
            Some("Pawn f2"),
            Some("Pawn g2"),
            Some("Pawn h2"),
        ];
        let black_pawns_str = [
            Some("Pawn a7"),
            Some("Pawn b7"),
            Some("Pawn c7"),
            Some("Pawn d7"),
            Some("Pawn e7"),
            Some("Pawn f7"),
            Some("Pawn g7"),
            Some("Pawn h7"),
        ];

        let empty_rank: [Option<&str>; 8] = [None; 8];

        let asset_map = [
            white_pieces_str,
            white_pawns_str,
            empty_rank,
            empty_rank,
            empty_rank,
            empty_rank,
            black_pawns_str,
            black_pieces_str,
        ];

        Self {
            board: Board::new(),
            asset_map: asset_map,
        }
    }

    /// Updates board of asset names as well as board with figures
    /// Basically the logic is exactly the same as for the regular board update,
    /// the index trait allows us to reuse it
    pub fn fat_update(&mut self, chess_move: &ChessMove, color: &PlayerColor) {
        // 1. Update the asset names map
        match chess_move.move_type {
            MoveType::Norm | MoveType::DoublePawn => {
                self[chess_move.to_tile] = self[chess_move.from_tile].take();
            }
            MoveType::Promoting => {
                let old_pawn = self[chess_move.from_tile]
                    .take()
                    .expect("Problem while unwrapping pawn promotion");
                let new_queen_name = pawn_promotion(old_pawn, *color);

                self[chess_move.to_tile] = Some(new_queen_name);
            }
            MoveType::Passing => {
                if let Some(last_move) = self.board.3.clone() {
                    self[last_move.to_tile] = None;
                    self[chess_move.to_tile] = self[chess_move.from_tile].take();
                } else {
                    panic!("In case of en-passant we should always have a valid last move.");
                };
            }

            MoveType::RochadeLeft => {
                self[chess_move.to_tile] = self[chess_move.from_tile].take();
                if chess_move.from_tile == WHITE_KING_SP {
                    self[(0, 3)] = self[(0, 0)].take()
                } else {
                    self[(7, 3)] = self[(7, 0)].take()
                }
            }
            MoveType::RochadeRight => {
                self[chess_move.to_tile] = self[chess_move.from_tile].take();
                if chess_move.from_tile == WHITE_KING_SP {
                    self[(0, 5)] = self[(0, 7)].take()
                } else {
                    self[(7, 5)] = self[(7, 7)].take()
                }
            }
        };

        // 2. Do the logical chess board Update
        self.board.update(chess_move, color);
    }
}

#[derive(Resource)]
pub struct GameState {
    pub fat_board: FatBoard,
    pub player_turn: PlayerColor,
    pub chosen_figure: Option<(Figure, u8, u8)>,
    pub possible_moves: Option<Vec<ChessMove>>,
    pub move_number: usize,
}

impl GameState {
    /// Despawns chess piece asset, does not update game state otherwise
    pub fn despawn_target(
        &self,
        commands: &mut Commands,
        target_names: HashSet<&str>,
        piece_query: &mut Query<(Entity, &Name, &mut Transform)>,
    ) {
        for (e, n, _t) in piece_query {
            if target_names.contains(n.as_str()) {
                commands.entity(e).despawn();
            }
        }
    }

    /// Executes the chosen move.
    /// Also handles the asset moving stuff.
    pub fn execute_move(
        &mut self,
        commands: &mut Commands,
        to_be_moved: &str,
        chess_move: &ChessMove,
        query: &mut Query<(Entity, &Name, &mut Transform)>,
        chess_scene: &Res<ChessScene>,
        gltf_assets: &Res<Assets<Gltf>>,
        gltf_meshes: &Res<Assets<GltfMesh>>,
    ) {
        move_asset(to_be_moved, query, chess_move);
        let mut to_despawn: HashSet<&str> = HashSet::new();
        if let Some(target) = self.fat_board[chess_move.to_tile].take() {
            to_despawn.insert(target);
        }

        if chess_move.move_type == MoveType::Passing {
            let double_pawn_move = self.fat_board.board.3.clone().unwrap().to_tile;
            if let Some(target) = self.fat_board[double_pawn_move].take() {
                to_despawn.insert(target);
            }
        }

        if chess_move.move_type == MoveType::Promoting {
            let color = self.fat_board.board[chess_move.from_tile]
                .unwrap()
                .player_color;
            let pawn_ass_name = self.fat_board[chess_move.from_tile].unwrap();

            let queen_ass_name = pawn_promotion(pawn_ass_name, color);

            to_despawn.insert(pawn_ass_name);
            queen_spawner(
                commands,
                chess_scene,
                gltf_assets,
                gltf_meshes,
                color,
                queen_ass_name,
                chess_move.to_tile,
            );
        }
        if !to_despawn.is_empty() {
            self.despawn_target(commands, to_despawn, query);
        }

        self.fat_board.fat_update(chess_move, &self.player_turn);
        self.player_turn = self.player_turn.other_player();
        self.move_number += 1;
    }

    // Checks if the picked tiles are valid and
    pub fn pick_is_valid(&self, from_tile: (u8, u8), to_tile: (u8, u8)) -> Option<ChessMove> {
        self.possible_moves.as_ref()?.iter().find_map(|cm| {
            if cm.from_tile == from_tile && cm.to_tile == to_tile {
                Some(cm.clone())
            } else {
                None
            }
        })
    }

    pub fn new() -> Self {
        Self {
            fat_board: FatBoard::new(),
            player_turn: PlayerColor::White,
            chosen_figure: None,
            possible_moves: None,
            move_number: 0,
        }
    }
}

fn move_asset(
    asset_name: &str,
    query: &mut Query<(Entity, &Name, &mut Transform)>,
    chess_move: &ChessMove,
) {
    let mut ass_pos = HashMap::new();
    ass_pos.insert(asset_name, chess_move.to_tile);
    match chess_move.move_type {
        MoveType::RochadeLeft => {
            if asset_name == "King e1" {
                ass_pos.insert("Rook a1", (0, 3));
            } else {
                ass_pos.insert("Rook a8", (7, 3));
            };
        }
        MoveType::RochadeRight => {
            if asset_name == "King e1" {
                ass_pos.insert("Rook h1", (0, 5));
            } else {
                ass_pos.insert("Rook h8", (7, 5));
            };
        }
        _ => (),
    };
    query
        .iter_mut()
        .filter(|(_ent, name, _t)| ass_pos.contains_key(name.as_str()))
        .for_each(|(_ent, name, mut t)| {
            let new_pos = ass_pos[name.as_str()];
            let (z, x) = idx_to_coordinates(new_pos.0, new_pos.1);
            t.as_mut().translation.x = x;
            t.as_mut().translation.z = z;
        });
}
