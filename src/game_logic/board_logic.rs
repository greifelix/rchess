use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use itertools::{Itertools, iproduct};

use std::ops::{Index, IndexMut};

use crate::game_logic::movement_logic::{self, ChessMove, MoveType};

use crate::utils::{
    core_types::{
        BLACK_KING_SP, BLACK_PIECES, DIAG_THREATS, Direction, FigType, Figure, PlayerColor,
        RANK_THREATS, STRAIGHT_DIRECTIONS, WHITE_KING_SP, WHITE_PIECES,
    },
    figs_adjacent, knights_reach,
};

/// Board contains all information for calculation of all possible moves (for both players)
#[derive(Clone, PartialEq, Eq)]
pub struct Board(
    pub [[Option<Figure>; 8]; 8],
    pub RochadeTracker,
    pub RochadeTracker,
    pub Option<ChessMove>,
);

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
    pub fn new() -> Self {
        let white_pawns = std::array::repeat(Some(Figure {
            fig_type: FigType::Pawn,
            player_color: PlayerColor::White,
        }));

        let black_pawns = std::array::repeat(Some(Figure {
            fig_type: FigType::Pawn,
            player_color: PlayerColor::Black,
        }));

        let empty_rank: [Option<Figure>; 8] = [None; 8];

        let raw_board = [
            WHITE_PIECES,
            white_pawns,
            empty_rank,
            empty_rank,
            empty_rank,
            empty_rank,
            black_pawns,
            BLACK_PIECES,
        ];

        Self(
            raw_board,
            RochadeTracker::new(PlayerColor::White),
            RochadeTracker::new(PlayerColor::Black),
            None,
        )
    }

    /// Updates the board according to the chess move in-place!
    pub fn update(&mut self, chess_move: &ChessMove, color: &PlayerColor) {
        match color {
            // First tracker update is to check whether moved tile was own rook or king
            // Second tracker update is to check whether other rook was captured
            PlayerColor::White => {
                self.1._update_tracker(&chess_move.from_tile);
                self.2._update_tracker(&chess_move.to_tile)
            }
            PlayerColor::Black => {
                self.2._update_tracker(&chess_move.from_tile);
                self.1._update_tracker(&chess_move.to_tile)
            }
        };

        match chess_move.move_type {
            MoveType::Norm | MoveType::DoublePawn => {
                self[chess_move.to_tile] = self[chess_move.from_tile].take();
            }
            MoveType::Promoting => {
                // Remove from original position, potentially kill blocker
                self[chess_move.from_tile] = None;
                let new_queen = Some(Figure {
                    fig_type: FigType::Queen,
                    player_color: *color,
                });
                self[chess_move.to_tile] = new_queen;
            }
            MoveType::Passing => {
                if let Some(last_move) = self.3.clone() {
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
        // Update last taken move
        self.3 = Some(chess_move.clone());
    }

    pub fn get_king_position(&self, fig_color: PlayerColor) -> (u8, u8) {
        iproduct!(0..8, 0..8)
            .find(|p| match self[*p] {
                Some(fig) => fig.player_color == fig_color && fig.fig_type == FigType::King,
                None => false,
            })
            .expect("There will always be a king, so this should never panic.")
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
    ) -> HashMap<(u8, u8), Direction> {
        STRAIGHT_DIRECTIONS
            .into_iter()
            .filter_map(
                |dir| match self.get_first_fig_in_direction(king_pos, dir, (1, 7)) {
                    Some((fig, r, c)) if fig.player_color == king_color => Some(((r, c), dir)),
                    _ => None,
                },
            )
            .collect()
    }

    /// Checks whether player is in check
    /// 1. Add pieces of enemy color in straight directions and in knights reach
    /// 2. Check for each direction whether piece is a threat.
    /// 3. If at least one piece is found, early stop of player in check.
    pub fn player_in_check(&self, king_color: PlayerColor) -> bool {
        let king_pos = self.get_king_position(king_color);
        // Check the first enemy figure in straight directions
        STRAIGHT_DIRECTIONS
            .into_iter()
            .filter_map(
                |dir| match self.get_first_fig_in_direction(king_pos, dir, (0, 8)) {
                    Some((f, r, c)) => {
                        if f.player_color == king_color.other_player() {
                            Some(((r, c), (dir, f.fig_type)))
                        } else {
                            None
                        }
                    }
                    None => None,
                },
            )
            .chain(
                knights_reach(king_pos)
                    .into_iter()
                    .filter_map(|(r, c)| match self[(r, c)] {
                        Some(fig) => {
                            if fig.player_color == king_color.other_player() {
                                Some(((r, c), (Direction::S1D1, fig.fig_type)))
                            } else {
                                None
                            }
                        }
                        None => None,
                    }),
            )
            .find_map(|((r, c), (dir, fig_type))| match dir {
                Direction::Unrelated => None,
                Direction::S1D1 => {
                    if fig_type == FigType::Knight {
                        Some(())
                    } else {
                        None
                    }
                }
                Direction::R | Direction::A | Direction::L | Direction::B => {
                    if RANK_THREATS.contains(&fig_type) {
                        Some(())
                    } else {
                        None
                    }
                }
                Direction::AR | Direction::AL | Direction::BL | Direction::BR => {
                    if DIAG_THREATS.contains(&fig_type)
                        || match fig_type {
                            FigType::Pawn => movement_logic::MoveBuilder::new((r, c), &self)
                                .calculate_naive_moves(&self)
                                .moveset
                                .iter()
                                .any(|x| x.to_tile == king_pos),
                            _ => false,
                        }
                    {
                        Some(())
                    } else {
                        None
                    }
                }
            })
            .is_some()
    }

    /// Get tiles in direction starting at source pos (exclusive) in the given straight direction.
    /// Lower bound is inclusive, higher bound is exclusive!
    /// Low bound is used for the left and lower borders, right bound for right and upper!
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
            _ => Box::new([].into_iter()),
        }
    }
    /// Gets tiles in a straight direction until a figure is hit; inlcudes the figure pos here
    pub fn _get_tiles_until_block(
        &self,
        source_pos: (u8, u8),
        direction: Direction,
    ) -> HashSet<(u8, u8)> {
        self.get_tiles_in_direction(source_pos, direction, (0, 8))
            .take_while_inclusive(|&p| self[p].is_none())
            .collect()
    }

    /// Get the first fig starting at source pos (exclusive) in the given direction.
    /// Lower bound is inclusive, higher bound is exclusive!
    /// Low bound is used for the left and lower borders, right bound for right and upper!
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
    /// Checks if a rochade is possible in the direction.
    /// The direction (left, right) is from the perspektive of the white player;
    /// i.e. the left rochade is also the long one for the black player
    pub fn rochade_possible(&self, board: &Board, dir: Direction) -> bool {
        let king_pos = if self.player == PlayerColor::White {
            WHITE_KING_SP
        } else {
            BLACK_KING_SP
        };

        match dir {
            // Left==Long Rochade
            Direction::L => {
                !self.king_moved
                    && !self.left_rook_moved
                    && board
                        .get_first_fig_in_direction(king_pos, Direction::L, (0, 8))
                        .map(|x| x.0.fig_type)
                        == Some(FigType::Rook)
                    && self._rochade_path_guarded(board, &dir) // Check ob wir auf dem Weg und im Ziel jemals im Schach wären
            }
            // TODO: Fix Sonderfall, dass eigener rook geschlagen wird, anderer rook dessen position einnimmt
            // Right == Short Rochade
            Direction::R => {
                !self.king_moved
                    && !self.right_rook_moved
                    && board
                        .get_first_fig_in_direction(king_pos, Direction::R, (0, 8))
                        .map(|x| x.0.fig_type)
                        == Some(FigType::Rook)
                    && self._rochade_path_guarded(board, &dir)
            }
            _ => panic!("Rochade only possible left and right!"), // Macht halt keinen Sinn
        }
    }

    /// This method assumes the rochade path is free. Checks whether the path is not "bedroht" by the other player.
    /// (We check the enemy king extra, cause player_in_check handles only "legal" positions.)
    fn _rochade_path_guarded(&self, board: &Board, dir: &Direction) -> bool {
        let mut board = board.clone();
        let k_start = if self.player == PlayerColor::White {
            WHITE_KING_SP
        } else {
            BLACK_KING_SP
        };

        let other_king = board.get_king_position(self.player.other_player());
        match *dir {
            Direction::L => {
                !figs_adjacent((k_start.0, k_start.1 - 1), other_king)
                    && !figs_adjacent((k_start.0, k_start.1 - 2), other_king)
                    && !board.player_in_check(self.player)
                    && {
                        board[(k_start.0, k_start.1 - 1)] = board[k_start].take();
                        !board.player_in_check(self.player)
                    }
                    && {
                        board[(k_start.0, k_start.1 - 2)] =
                            board[(k_start.0, k_start.1 - 1)].take();
                        !board.player_in_check(self.player)
                    }
            }
            Direction::R => {
                !figs_adjacent((k_start.0, k_start.1 + 1), other_king)
                    && !figs_adjacent((k_start.0, k_start.1 + 2), other_king)
                    && !board.player_in_check(self.player)
                    && {
                        board[(k_start.0, k_start.1 + 1)] = board[k_start].take();
                        !board.player_in_check(self.player)
                    }
                    && {
                        board[(k_start.0, k_start.1 + 2)] =
                            board[(k_start.0, k_start.1 + 1)].take();
                        !board.player_in_check(self.player)
                    }
            }
            _ => panic!("Rochade path called with unexpected direction"),
        }
    }

    // Updates rochade tracker.
    pub fn _update_tracker(&mut self, moved: &(u8, u8)) {
        if self.king_moved {
            return;
        }
        // To check for tiles is ok, as we are only interestet in the last move anyway
        match self.player {
            PlayerColor::White => {
                if *moved == WHITE_KING_SP {
                    self.king_moved = true;
                } else if *moved == (0, 0) {
                    self.left_rook_moved = true;
                } else if *moved == (0, 7) {
                    self.right_rook_moved = true;
                }
            }
            PlayerColor::Black => {
                if *moved == BLACK_KING_SP {
                    self.king_moved = true;
                } else if *moved == (7, 0) {
                    self.left_rook_moved = true;
                } else if *moved == (7, 7) {
                    self.right_rook_moved = true;
                }
            }
        };
    }
}
