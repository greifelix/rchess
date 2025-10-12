pub mod minmax_logic;
pub mod movement_logic;

use bevy::prelude::*;
use itertools::iproduct;
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::{game_logic::movement_logic::MoveBuilder, utils::idx_to_coordinates};

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
    pub to: HashSet<(usize, usize)>,
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
            .expect("There will always be a king, so this should never panic.")
    }
    pub fn get_fig_on_tile(&self, row: usize, col: usize) -> Option<Figure> {
        self[row][col]
    }

    pub fn get_busy_tiles(&self, player_color: PlayerColor) -> Vec<(usize, usize)> {
        iproduct!(0..8, 0..8)
            .filter(|(r, c)| match self[*r][*c] {
                Some(fig) if fig.player_color == player_color => true,
                _ => false,
            })
            .collect()
    }    
}

#[derive(Resource)]
pub struct GameState {
    pub board: Board,
    pub player_turn: PlayerColor,
    pub chosen_figure: Option<(Figure, usize, usize)>,
    pub possible_moves: Option<PossibleMoves>,
    pub under_attack: Option<Attacker>,
    pub move_number:usize
}

impl GameState {
    /// Despawns chess piece asset, does not update game state otherwise
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

    /// Executes the chosen move, if it is valid. In case the move is invalid, nothing will happen
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
            move_asset(to_be_moved, query, to_tile);

            if let Some(target) = self.board[to_row][to_col].take() {
                self.despawn_target(commands, target.ass_name, query);
            }
            self.board[to_row][to_col] = self.board[from_row][from_col].take();
            self.under_attack = self.enemy_in_check(to_tile);

            if self.enemy_in_checkmate(){
                println!("Game over, {:?}",self.player_turn.other_player());
            }

            self.player_turn = self.player_turn.other_player();
        }
        self.chosen_figure = None;
        self.possible_moves = None;
        self.move_number+=1;
    }

    pub fn get_fig_on_tile(&self, row: usize, col: usize) -> Option<Figure> {
        self.board[row][col]
    }

    // Checks if one of the picked moves of the PLAYER is valid
    pub fn move_is_valid(&self, from_tile: (usize, usize), to_tile: (usize, usize)) -> bool {
        if let Some(moves) = &self.possible_moves {
            from_tile == moves.from_tile && moves.to.contains(&to_tile)
        } else {
            false
        }
    }

    /// Meant to be called after an own move to see whether enemy is in own naive possible moves
    pub fn enemy_in_check(&self, attacker_tile: (usize, usize)) -> Option<Attacker> {
        let king_pos = self
            .board
            .get_king_position(self.player_turn.other_player());

        // Here we assume that the move was already executed!
        if MoveBuilder::new(attacker_tile, self.board.clone())
            .calculate_naive_moves()
            .extract()
            .to
            .contains(&king_pos)
        {
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

    pub fn enemy_in_checkmate(&self) -> bool {
        match self.under_attack {
            Some(attacker) => !self
                .board
                .get_busy_tiles(self.player_turn.other_player())
                .into_iter()
                .any(|(r, c)| {
                    MoveBuilder::new((r, c), self.board.clone())
                        .calculate_naive_moves()
                        .filter_moves_in_check(Some(attacker))
                        .block_selfchecking_moves()
                        .extract()
                        .to
                        .iter()
                        .count()
                        > 0
                }),
            None => false,
        }
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

        let raw_board = [
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
            board: Board(raw_board),
            player_turn: PlayerColor::White,
            chosen_figure: None,
            possible_moves: None,
            under_attack: None,
            move_number:0
        }
    }
}

fn move_asset(
    asset_name: &str,
    query: &mut Query<'_, '_, (Entity, &Name, &mut Transform)>,
    to_tile: (usize, usize),
) {
    let (to_row, to_col) = to_tile;
    query
        .iter_mut()
        .filter(|(_ent, name, _t)| name.as_str() == asset_name)
        .for_each(|(_ent, _name, mut t)| {
            let (z, x) = idx_to_coordinates(to_row, to_col);

            t.as_mut().translation.x = x;
            t.as_mut().translation.z = z;
        });
}
