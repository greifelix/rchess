pub mod minmax_logic;
pub mod movement_logic;
use bevy::{platform::collections::HashMap, prelude::*};
use itertools::{Itertools, iproduct};
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};
// use bevy::platform::collections::HashSet;
use std::collections::HashSet;

const WHITE_KING_SP: (u8, u8) = (0, 4);
const BLACK_KING_SP: (u8, u8) = (7, 5);
const WHITE_ROOK_R: Option<Figure> = Some(Figure {
    fig_type: FigType::Rook,
    ass_name: "Rook h1",
    player_color: PlayerColor::White,
});
const WHITE_ROOK_L: Option<Figure> = Some(Figure {
    fig_type: FigType::Rook,
    ass_name: "Rook a1",
    player_color: PlayerColor::White,
});
const BLACK_ROOK_R: Option<Figure> = Some(Figure {
    fig_type: FigType::Rook,
    ass_name: "Rook a7",
    player_color: PlayerColor::Black,
});
const BLACK_ROOK_L: Option<Figure> = Some(Figure {
    fig_type: FigType::Rook,
    ass_name: "Rook h7",
    player_color: PlayerColor::Black,
});

use crate::game_logic::movement_logic::{ChessMove, MoveType};
use crate::{
    game_logic::movement_logic::MoveBuilder,
    utils::{self, idx_to_coordinates, king_prox},
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FigType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl FigType {
    pub fn pins_in_direction(&self, dir: Direction) -> bool {
        match (*self, dir) {
            (
                Self::Bishop | Self::Queen,
                Direction::AL | Direction::AR | Direction::BL | Direction::BR,
            ) => true,
            (
                Self::Rook | Self::Queen,
                Direction::A | Direction::B | Direction::L | Direction::R,
            ) => true,
            _ => false,
        }
    }
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
/// Right, AboveRight,Above,AboveLeft,Left,BelowLeft,Below,BelowRight
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    R,
    AR,
    A,
    AL,
    L,
    BL,
    B,
    BR,
    Unrelated,
}

impl Direction {
    pub fn determine_direction_from_to(source_pos: (u8, u8), target_pos: (u8, u8)) -> Self {
        match (
            target_pos.0.cmp(&source_pos.0),
            target_pos.1.cmp(&source_pos.1),
        ) {
            (Ordering::Equal, Ordering::Greater) => Self::R,
            (Ordering::Equal, Ordering::Less) => Self::L,
            (Ordering::Greater, Ordering::Equal) => Self::A,
            (Ordering::Less, Ordering::Equal) => Self::B,
            (Ordering::Greater, Ordering::Greater) => {
                if target_pos.0 - source_pos.0 == target_pos.1 - source_pos.1 {
                    Self::AR
                } else {
                    Self::Unrelated
                }
            }
            (Ordering::Less, Ordering::Less) => {
                if source_pos.0 - target_pos.0 == source_pos.1 - target_pos.1 {
                    Self::BL
                } else {
                    Self::Unrelated
                }
            }
            (Ordering::Greater, Ordering::Less) => {
                if target_pos.0 - source_pos.0 == source_pos.1 - target_pos.1 {
                    Self::AL
                } else {
                    Self::Unrelated
                }
            }
            (Ordering::Less, Ordering::Greater) => {
                if source_pos.0 - target_pos.0 == target_pos.1 - source_pos.1 {
                    Self::BR
                } else {
                    Self::Unrelated
                }
            }
            _ => Self::Unrelated,
        }
    }
}

pub struct PossibleMoves {
    pub from_tile: (u8, u8),
    pub to: HashSet<ChessMove>,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RochadeTracker {
    player: PlayerColor,
    king_moved: bool,
    right_rook_moved: bool,
    left_rook_moved: bool,
}

impl RochadeTracker {
    pub fn new(player: PlayerColor) -> Self {
        Self {
            player: player,
            king_moved: false,
            right_rook_moved: false,
            left_rook_moved: false,
        }
    }

    pub fn rochade_possible(&self, board: &Board, dir: Direction) -> bool {
        let (right_rook, left_rook, king_pos) = match self.player {
            PlayerColor::Black => (BLACK_ROOK_R, BLACK_ROOK_L, BLACK_KING_SP),
            PlayerColor::White => (WHITE_ROOK_R, WHITE_ROOK_L, WHITE_KING_SP),
        };

        match dir {
            // Left==Long Rochade
            Direction::L => {
                !self.king_moved
                    && !self.left_rook_moved
                    && board
                        .get_first_fig_in_direction(king_pos, Direction::L, (0, 7))
                        .map(|x| x.0)
                        == left_rook
                    && self._rochade_path_guarded(board.clone(), dir) // Check ob wir auf dem Weg und im Ziel jemals im Shach wären
            }
            // Right == Short Rochade
            Direction::R => {
                self.king_moved
                    || self.right_rook_moved
                        && board
                            .get_first_fig_in_direction(king_pos, Direction::R, (0, 7))
                            .map(|x| x.0)
                            == right_rook
                        && self._rochade_path_guarded(board.clone(), dir)
            }
            _ => panic!("Rochade only possible left and right!"), // Macht halt keinen Sinn
        }
    }

    // Note: We previously checked whether path is figure-free; no check whether it is not bedroht
    fn _rochade_path_guarded(&self, mut board: Board, dir: Direction) -> bool {
        match (self.player, dir) {
            (PlayerColor::White, Direction::L) => {
                board.player_in_check(self.player).is_none()
                    && {
                        board[(WHITE_KING_SP.0, WHITE_KING_SP.1 - 1)] =
                            board[(WHITE_KING_SP.0, WHITE_KING_SP.1)].take();
                        board.player_in_check(self.player).is_none()
                    }
                    && {
                        board[(WHITE_KING_SP.0, WHITE_KING_SP.1 - 2)] =
                            board[(WHITE_KING_SP.0, WHITE_KING_SP.1 - 1)].take();
                        board.player_in_check(self.player).is_none()
                    }
            }
            (PlayerColor::White, Direction::R) => {
                board.player_in_check(self.player).is_none()
                    && {
                        board[(WHITE_KING_SP.0, WHITE_KING_SP.1 + 1)] =
                            board[(WHITE_KING_SP.0, WHITE_KING_SP.1)].take();
                        board.player_in_check(self.player).is_none()
                    }
                    && {
                        board[(WHITE_KING_SP.0, WHITE_KING_SP.1 + 2)] =
                            board[(WHITE_KING_SP.0, WHITE_KING_SP.1 + 1)].take();
                        board.player_in_check(self.player).is_none()
                    }
            }
            (PlayerColor::Black, Direction::L) => {
                board.player_in_check(self.player).is_none()
                    && {
                        board[(BLACK_KING_SP.0, BLACK_KING_SP.1 + 1)] =
                            board[(BLACK_KING_SP.0, BLACK_KING_SP.1)].take();
                        board.player_in_check(self.player).is_none()
                    }
                    && {
                        board[(BLACK_KING_SP.0, BLACK_KING_SP.1 + 2)] =
                            board[(BLACK_KING_SP.0, BLACK_KING_SP.1 + 1)].take();
                        board.player_in_check(self.player).is_none()
                    }
            }
            (PlayerColor::Black, Direction::R) => {
                board.player_in_check(self.player).is_none()
                    && {
                        board[(BLACK_KING_SP.0, BLACK_KING_SP.1 - 1)] =
                            board[(BLACK_KING_SP.0, BLACK_KING_SP.1)].take();
                        board.player_in_check(self.player).is_none()
                    }
                    && {
                        board[(BLACK_KING_SP.0, BLACK_KING_SP.1 - 2)] =
                            board[(BLACK_KING_SP.0, BLACK_KING_SP.1 - 1)].take();
                        board.player_in_check(self.player).is_none()
                    }
            }
            _ => panic!("Rochade guard in unexpected panic!"),
        }
    }

    // Updates rochade tracker.
    pub fn _update_tracker(&mut self, moved: &(u8, u8)) {
        if self.king_moved || self.right_rook_moved || self.left_rook_moved {
            return;
        } else {
            // To check for tiles is ok, as we are only interestet in the last move anyway
            match self.player {
                PlayerColor::Black => {
                    if *moved == BLACK_KING_SP {
                        self.king_moved = true;
                    } else if *moved == (7, 0) {
                        self.right_rook_moved = true;
                    } else if *moved == (7, 7) {
                        self.left_rook_moved = true;
                    }
                }
                PlayerColor::White => {
                    if *moved == WHITE_KING_SP {
                        self.king_moved = true;
                    } else if *moved == (0, 7) {
                        self.right_rook_moved = true;
                    } else if *moved == (0, 0) {
                        self.left_rook_moved = true;
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Board([[Option<Figure>; 8]; 8], RochadeTracker, RochadeTracker);

impl Index<(u8, u8)> for Board {
    type Output = Option<Figure>;
    fn index(&self, index: (u8, u8)) -> &Self::Output {
        &self.0[index.0 as usize][index.1 as usize]
    }
}

impl IndexMut<(u8, u8)> for Board {
    fn index_mut(&mut self, index: (u8, u8)) -> &mut Self::Output {
        &mut self.0[index.0 as usize][index.1 as usize]
    }
}

impl Board {
    /// Updates the board according to the chess move in-place!
    pub fn update(&mut self, chess_move: &ChessMove,color:&PlayerColor) {

        match color {
            PlayerColor::White => self.1._update_tracker(&chess_move.from_tile),
            PlayerColor::Black => self.2._update_tracker(&chess_move.from_tile),
        };

        match chess_move.move_type {
            MoveType::Norm => {
                self[chess_move.to_tile] = self[chess_move.from_tile].take();
            }
            MoveType::Passing => {
                panic!("Not implemented error");
            }
            MoveType::RochadeLeft => {
                self[chess_move.to_tile] = self[chess_move.from_tile].take();
                if chess_move.from_tile == WHITE_KING_SP {
                    self[(0, 2)] = self[(0, 0)].take()
                } else {
                    self[(7, 2)] = self[(7, 0)].take()
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
    }

    pub fn get_king_position(&self, fig_color: PlayerColor) -> (u8, u8) {
        iproduct!(0..8, 0..8)
            .find(|p| match self[*p] {
                Some(fig) => fig.player_color == fig_color && fig.fig_type == FigType::King,
                None => false,
            })
            .expect("There will always be a king, so this should never panic.")
    }
    // TODO: Delete this and replace by proper index method :D
    pub fn get_fig_on_tile(&self, row: u8, col: u8) -> Option<Figure> {
        self[(row, col)]
    }

    pub fn get_busy_tiles(&self, player_color: PlayerColor) -> HashSet<(u8, u8)> {
        iproduct!(0..8, 0..8)
            .filter(|p| match self[*p] {
                Some(fig) if fig.player_color == player_color => true,
                _ => false,
            })
            .collect()
    }

    pub fn guarding_figures(
        &self,
        king_color: PlayerColor,
        king_pos: (u8, u8),
    ) -> HashMap<(u8, u8), (Direction, FigType)> {
        let mut out: HashMap<(u8, u8), (Direction, FigType)> = HashMap::new();
        let dirs = [
            Direction::R,
            Direction::AR,
            Direction::A,
            Direction::AL,
            Direction::L,
            Direction::BL,
            Direction::B,
            Direction::BR,
        ];
        dirs.into_iter().for_each(|dir| {
            match self.get_first_fig_in_direction(king_pos, dir, (1, 7)) {
                Some((f, r, c)) => {
                    if f.player_color == king_color {
                        out.insert((r, c), (dir, f.fig_type));
                    }
                }
                None => (),
            }
        });

        out
    }

    pub fn king_enemy_circle(
        &self,
        king_color: PlayerColor,
        king_pos: (u8, u8),
    ) -> HashMap<(u8, u8), (Direction, FigType)> {
        let mut out: HashMap<(u8, u8), (Direction, FigType)> = HashMap::new();
        let dirs = [
            Direction::R,
            Direction::AR,
            Direction::A,
            Direction::AL,
            Direction::L,
            Direction::BL,
            Direction::B,
            Direction::BR,
        ];
        dirs.into_iter().for_each(|dir| {
            match self.get_first_fig_in_direction(king_pos, dir, (0, 8)) {
                Some((f, r, c)) => {
                    if f.player_color == king_color.other_player() {
                        out.insert((r, c), (dir, f.fig_type));
                    }
                }
                None => (),
            }
        });
        utils::knights_reach(king_pos)
            .into_iter()
            .for_each(|p| match self[p] {
                Some(fig)
                    if fig.fig_type == FigType::Knight
                        && fig.player_color == king_color.other_player() =>
                {
                    out.insert(p, (Direction::Unrelated, FigType::Knight));
                }
                Some(_) => (),
                None => (),
            });

        out
    }

    /// Checks if the player is in check, if yes it returns the enemy-figure causing the check.
    /// (This method can only handle legal board positions,e.g. adjacent kings can not be checked)
    pub fn player_in_check(&self, player: PlayerColor) -> Option<(u8, u8, FigType)> {
        let my_king_pos = self.get_king_position(player);
        let rank_threats = [FigType::Rook, FigType::Queen];
        let diag_threats = [FigType::Bishop, FigType::Queen];

        self.king_enemy_circle(player, my_king_pos)
            .into_iter()
            .find_map(|((r, c), (dir, fig_type))| match dir {
                Direction::Unrelated => Some((r, c, fig_type)), // In this case we have a knight!
                Direction::R | Direction::A | Direction::L | Direction::B => {
                    if rank_threats.contains(&fig_type) {
                        Some((r, c, fig_type))
                    } else {
                        None
                    }
                }
                Direction::AR | Direction::AL | Direction::BL | Direction::BR => {
                    if diag_threats.contains(&fig_type)
                        || match fig_type {
                            FigType::Pawn => movement_logic::MoveBuilder::new((r, c), &self)
                                .calculate_naive_moves(&self)
                                .extract()
                                .to
                                .iter()
                                .any(|x| x.to_tile == my_king_pos),
                            // .contains(&my_king_pos),
                            _ => false,
                        }
                    {
                        Some((r, c, fig_type))
                    } else {
                        None
                    }
                }
            })
    }

    /// Get tiles in direction, starting from the source position which is exclusive and ending at the bounds of the board
    /// In friendly mode
    pub fn get_tiles_in_direction(
        &self,
        source_pos: (u8, u8),
        direction: Direction,
        bounds: (u8, u8), // low inclusive,High is exlusive,
    ) -> Box<dyn Iterator<Item = (u8, u8)>> {
        let (source_row, source_col) = source_pos;
        let (b_low, b_high) = bounds;
        // NOTE: For the diagonals we rely on the fact, that one of the last elements in the diagonal is always either 0 or 7.
        // Then the zipped other iterator will always be stopped early
        match direction {
            Direction::R => Box::new((source_col + 1..b_high).map(move |c| (source_row, c))),
            Direction::AR => Box::new((source_row + 1..b_high).zip(source_col + 1..b_high)),
            Direction::A => Box::new((source_row + 1..b_high).map(move |r| (r, source_col))),
            Direction::AL => Box::new((source_row + 1..b_high).zip((b_low..source_col).rev())),
            Direction::L => Box::new((b_low..source_col).rev().map(move |c| (source_row, c))),
            Direction::BL => Box::new((b_low..source_row).rev().zip((b_low..source_col).rev())),
            Direction::B => Box::new((b_low..source_row).rev().map(move |r| (r, source_col))), //change to rev?
            Direction::BR => Box::new((b_low..source_row).rev().zip(source_col + 1..b_high)),
            Direction::Unrelated => Box::new([].into_iter()),
        }
    }
    /// Gets tiles until a figure is hit; inlcudes the figure here
    ///TODO: Rook???
    pub fn get_tiles_until_block(
        &self,
        source_pos: (u8, u8),
        direction: Direction,
    ) -> HashSet<(u8, u8)> {
        self.get_tiles_in_direction(source_pos, direction, (0, 8))
            .take_while_inclusive(|&p| self[p].is_none())
            .collect()
    }

    pub fn get_first_fig_in_direction(
        &self,
        source_pos: (u8, u8),
        direction: Direction,
        bounds: (u8, u8),
    ) -> Option<(Figure, u8, u8)> {
        self.get_tiles_in_direction(source_pos, direction, bounds)
            .find_map(|(r, c)| self[(r, c)].map(|f| (f, r, c)))
    }
}

#[derive(Resource)]
pub struct GameState {
    pub board: Board,
    pub player_turn: PlayerColor,
    pub chosen_figure: Option<(Figure, u8, u8)>,
    pub possible_moves: Option<PossibleMoves>,
    pub move_number: usize,
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

    /// Executes the chosen move, if it is valid. In case the move is invalid, nothing will happen.
    /// Also handles the asset moving stuff.
    pub fn execute_move(
        &mut self,
        commands: &mut Commands,
        to_be_moved: &str,
        from_tile: (u8, u8),
        to_tile: (u8, u8),
        query: &mut Query<(Entity, &Name, &mut Transform)>,
    ) {
        if self.move_is_valid(from_tile, to_tile) {
            move_asset(to_be_moved, query, to_tile);

            if let Some(target) = self.board[to_tile].take() {
                self.despawn_target(commands, target.ass_name, query);
            }
            self.board[to_tile] = self.board[from_tile].take();
            self.player_turn = self.player_turn.other_player();
        }
        self.chosen_figure = None;
        self.possible_moves = None;
        self.move_number += 1;
    }

    // Checks if one of the picked moves of the PLAYER is valid
    pub fn move_is_valid(&self, from_tile: (u8, u8), to_tile: (u8, u8)) -> bool {
        if let Some(moves) = &self.possible_moves {
            from_tile == moves.from_tile && moves.to.iter().any(|x| x.to_tile == to_tile)
        } else {
            false
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
            board: Board(
                raw_board,
                RochadeTracker::new(PlayerColor::White),
                RochadeTracker::new(PlayerColor::Black),
            ),
            player_turn: PlayerColor::White,
            chosen_figure: None,
            possible_moves: None,
            move_number: 0,
        }
    }
}

fn move_asset(
    asset_name: &str,
    query: &mut Query<'_, '_, (Entity, &Name, &mut Transform)>,
    to_tile: (u8, u8),
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
