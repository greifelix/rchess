pub mod movement_logic;

use bevy::prelude::*;
use itertools::iproduct;
use std::cmp::Ordering;

use crate::utils::idx_to_coordinates;
use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FigType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerColor {
    Black,
    White,
}

impl PlayerColor {
    fn other_player(&self) -> PlayerColor {
        match self {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Figure {
    pub fig_type: FigType,
    pub ass_name: &'static str,
    pub player_color: PlayerColor,
}

pub struct PossibleMoves {
    pub from_tile: (usize, usize),
    pub to: Vec<(usize, usize)>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Attacker {
    fig: Figure,
    tile: (usize, usize),
}

#[derive(Copy, Clone, PartialEq, Eq, DerefMut, Deref)]
pub struct Board(pub [[Option<Figure>; 8]; 8]);

impl Board {
    pub fn get_king_position(&self, fig_color: PlayerColor) -> (usize, usize) {
        iproduct!(0..8, 0..8)
            .find(|(r, c)| match self[*r][*c] {
                Some(fig) => fig.player_color == fig_color && fig.fig_type == FigType::King,
                None => false,
            })
            .expect("There will always be a king, so this should not panic.")
    }
}

#[derive(Resource)]
pub struct GameState {
    pub board: Board,
    pub player_turn: PlayerColor,
    pub chosen_figure: Option<(Figure, usize, usize)>,
    pub possible_moves: Option<PossibleMoves>,
    pub under_attack: Option<Attacker>,
}

impl GameState {
    pub fn get_figure_name(&self, row: usize, col: usize) -> Option<&'static str> {
        match self.board[row][col] {
            None => None,
            Some(fig) => Some(fig.ass_name),
        }
    }

    /// Kills target asset
    pub fn despawn_target(
        &self,
        commands: &mut Commands,
        target_name: &str,
        piece_query: &mut Query<(Entity, &Name, &mut Transform)>,
    ) {
        for (e, n, _t) in piece_query {
            if n.as_str() == target_name {
                commands.entity(e).despawn();
            }
        }
    }

    /// Executes the chosen move, if it is valid. In case the move is invalid, nothing will happen d
    pub fn execute_move(
        &mut self,
        commands: &mut Commands,
        to_be_moved: &str,
        from_tile: (usize, usize),
        to_tile: (usize, usize),
        query: &mut Query<(Entity, &Name, &mut Transform)>,
    ) {
        if self.move_is_valid(from_tile, to_tile) {
            let (from_row, from_col) = from_tile;
            let (to_row, to_col) = to_tile;

            if let Some(target) = self.board[to_row][to_col].take() {
                self.despawn_target(commands, target.ass_name, query);
            }
            self.board[to_row][to_col] = self.board[from_row][from_col].take();
            self.under_attack = self.enemy_under_attack(to_tile);

            move_asset(to_be_moved, query, to_tile);
            self.player_turn = self.player_turn.other_player();
        }
        self.chosen_figure = None;
        self.possible_moves = None;
    }

    pub fn get_fig_on_tile(&self, row: usize, col: usize) -> Option<Figure> {
        self.board[row][col]
    }

    pub fn move_is_valid(&self, from_tile: (usize, usize), to_tile: (usize, usize)) -> bool {
        if let Some(moves) = &self.possible_moves {
            from_tile == moves.from_tile && moves.to.contains(&to_tile)
        } else {
            false
        }
    }

    /// Meant to be called after an own move to see whether enemy is in own naive possible moves
    pub fn enemy_under_attack(&self, attacker_tile: (usize, usize)) -> Option<Attacker> {
        let king_pos = self
            .board
            .get_king_position(self.player_turn.other_player());

        // Here we assume that the move was already executed!
        if movement_logic::calculate_naive_moves(&self.board, attacker_tile).contains(&king_pos) {
            println!("Check!!!");
            Some(Attacker {
                fig: self
                    .get_fig_on_tile(attacker_tile.0, attacker_tile.1)
                    .unwrap(),
                tile: attacker_tile,
            })
        } else {
            None
        }
    }

    pub fn block_selfchecking_moves(
        &self,
        fig_pos: (usize, usize),
        fig_moves: Vec<(usize, usize)>,
    ) -> Vec<(usize, usize)> {
        let (from_row, from_col) = fig_pos;
        let mut out_moves = Vec::new();
        let king_pos = self.board.get_king_position(self.player_turn);

        // Refactor dies in "validate king movement"
        if king_pos == fig_pos {
            for (to_row, to_col) in fig_moves.into_iter() {
                let mut board_clone = self.board.clone();
                board_clone[to_row][to_col] = board_clone[from_row][from_col].take();

                let enemy_tiles =
                    movement_logic::get_busy_tiles(&board_clone, self.player_turn.other_player());

                if enemy_tiles.into_iter().any(|enemy_move| {
                    movement_logic::calculate_naive_moves(&board_clone, enemy_move)
                        .contains(&(to_row, to_col))
                }) {
                    continue;
                } else {
                    out_moves.push((to_row, to_col));
                }
            }
        // Refactor in validate_figure_movement
        } else {
            let possible_threat_direction = movement_logic::pos_rel_to_king(fig_pos, king_pos);

            for (to_row, to_col) in self.filter_moves_in_check(fig_moves).into_iter() {
                let mut test_board = self.board.clone();
                test_board[to_row][to_col] = test_board[from_row][from_col].take();

                if movement_logic::threats_detected(
                    king_pos,
                    test_board,
                    possible_threat_direction,
                    self.player_turn.other_player(),
                ) {
                    continue;
                } else {
                    out_moves.push((to_row, to_col));
                }
            }
        }
        out_moves
    }

    /// Only allows moves which stop the attack (Only non-king moves); For every figure seperately
    pub fn filter_moves_in_check(&self, moves: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        let Some(attacker) = self.under_attack else {
            return moves;
        };

        let king_pos = self.board.get_king_position(self.player_turn);
        let attack_angle = movement_logic::pos_rel_to_king(attacker.tile, king_pos);

        let stopping_moves: Vec<(usize, usize)> = match attacker.fig.fig_type {
            FigType::King => panic!("Attacker can never be the enemy king."),
            FigType::Knight | FigType::Pawn => Vec::from([attacker.tile]), // Pawns and Knights can only be stopped by killing move
            FigType::Rook | FigType::Bishop | FigType::Queen => {
                movement_logic::get_tiles_between(attack_angle, king_pos, attacker.tile).collect()
            }
        };

        let filter_set: HashSet<(usize, usize)> = stopping_moves.into_iter().collect();
        let out: HashSet<(usize, usize)> = moves.into_iter().collect();

        out.intersection(&filter_set).copied().collect()
    }

    pub fn new() -> Self {
        let white_pieces = [
            Some(Figure {
                fig_type: FigType::Rook,
                ass_name: "Rook a1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Knight,
                ass_name: "Knight b1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Bishop,
                ass_name: "Bishop c1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Queen,
                ass_name: "Queen d1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::King,
                ass_name: "King e1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Bishop,
                ass_name: "Bishop f1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Knight,
                ass_name: "Knight g1",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Rook,
                ass_name: "Rook h1",
                player_color: PlayerColor::White,
            }),
        ];

        let black_pieces = [
            Some(Figure {
                fig_type: FigType::Rook,
                ass_name: "Rook a8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Knight,
                ass_name: "Knight b8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Bishop,
                ass_name: "Bishop c8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Queen,
                ass_name: "Queen d8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::King,
                ass_name: "King e8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Bishop,
                ass_name: "Bishop f8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Knight,
                ass_name: "Knight g8",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Rook,
                ass_name: "Rook h8",
                player_color: PlayerColor::Black,
            }),
        ];

        let white_pawns = [
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn a2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn b2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn c2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn d2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn e2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn f2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn g2",
                player_color: PlayerColor::White,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn h2",
                player_color: PlayerColor::White,
            }),
        ];

        let black_pawns = [
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn a7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn b7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn c7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn d7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn e7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn f7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn g7",
                player_color: PlayerColor::Black,
            }),
            Some(Figure {
                fig_type: FigType::Pawn,
                ass_name: "Pawn h7",
                player_color: PlayerColor::Black,
            }),
        ];

        let empty_rank: [Option<Figure>; 8] = [None; 8];

        let board = [
            white_pieces,
            white_pawns,
            empty_rank,
            empty_rank,
            empty_rank,
            empty_rank,
            black_pawns,
            black_pieces,
        ];

        Self {
            board: Board(board),
            player_turn: PlayerColor::White,
            chosen_figure: None,
            possible_moves: None,
            under_attack: None,
        }
    }
}

// Todo: Replace filter with find or something
fn move_asset(
    asset_name: &str,
    query: &mut Query<'_, '_, (Entity, &Name, &mut Transform)>,
    to_tile: (usize, usize),
) {
    let (to_row, to_col) = to_tile;
    query
        .iter_mut()
        .filter(|(ent, name, t)| name.as_str() == asset_name)
        .for_each(|(ent, name, mut t)| {
            let (z, x) = idx_to_coordinates(to_row, to_col);

            t.as_mut().translation.x = x;
            t.as_mut().translation.z = z;
        });
}
