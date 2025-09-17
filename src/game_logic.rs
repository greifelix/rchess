
pub mod movement_logic;

use bevy::prelude::*;
use itertools::iproduct;
use std::cmp::Ordering;

use crate::utils::idx_to_coordinates;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PosRelToKing {
    Above,
    Below,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
    Unrelated,
}

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
    pub from: (usize, usize),
    pub to: Vec<(usize, usize)>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Attacker {
    fig: Figure,
    tile: (usize, usize),
}

#[derive(Resource)]
pub struct GameState {
    pub board: [[Option<Figure>; 8]; 8],
    pub player_turn: PlayerColor,
    pub chosen_figure: Option<(Figure, usize, usize)>,
    pub possible_moves: Option<PossibleMoves>,
    pub under_attack: Option<Attacker>,
}

/// Calculate the moves of the figure on the tile. The moves are not yet filtered,
/// on whether they might cause a check.
pub fn calculate_naive_moves(
    board: &[[Option<Figure>; 8]; 8],
    tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let (from_row, from_col) = tile;
    if let Some(fig) = board[from_row][from_col] {
        match fig.fig_type {
            FigType::Pawn => match fig.player_color {
                PlayerColor::Black => black_pawn_moves(board, tile),
                PlayerColor::White => white_pawn_moves(board, tile),
            },
            FigType::Rook => rook_moves(board, tile),
            FigType::Knight => knight_moves(board, tile),
            FigType::Bishop => bishop_moves(board, tile),
            FigType::Queen => queen_moves(board, tile),
            FigType::King => king_moves(board, tile), // TODO: Add filter for alle dangerous moves
        }
    } else {
        vec![]
    }
}

/// Returns the tiles between the king and a fig, where the kings tile is not inclusive but the figure is
pub fn get_tiles_between(
    relative_pos: PosRelToKing,
    king_pos: (usize, usize),
    pos: (usize, usize),
) -> Box<dyn Iterator<Item = (usize, usize)>> {
    let (king_row, king_col) = king_pos;
    let (fig_row, fig_col) = pos;

    println!("Relative Position to king: {:?}", relative_pos);

    // NOTE: if fig_pos is unknown,
    match relative_pos {
        PosRelToKing::Unrelated => Box::new([].into_iter()),
        PosRelToKing::Above => Box::new((king_row + 1..=fig_row).map(move |r| (r, king_col))),
        PosRelToKing::Below => Box::new((fig_row..king_row).map(move |r| (r, king_col))),
        PosRelToKing::Left => Box::new((fig_col..king_col).rev().map(move |c| (king_row, c))),
        PosRelToKing::Right => Box::new((king_col + 1..=fig_col).map(move |c| (king_row, c))),
        PosRelToKing::UpRight => Box::new((king_row + 1..=fig_row).zip(king_col + 1..=fig_col)),
        PosRelToKing::DownLeft => {
            Box::new((fig_row..king_row).rev().zip((fig_col..king_col).rev()))
        }
        PosRelToKing::DownRight => Box::new((fig_row..king_row).rev().zip(king_col + 1..=fig_col)),
        PosRelToKing::UpLeft => Box::new((king_row + 1..=fig_row).zip((fig_col..king_col).rev())),
    }
}

pub fn threats_detected(
    king_pos: (usize, usize),
    board: [[Option<Figure>; 8]; 8],
    threat_direction: PosRelToKing,
    enemy_color: PlayerColor,
) -> bool {
    let (king_row, king_col) = king_pos;
    let rank_threats = [FigType::Queen, FigType::Rook];
    let diag_threats = [FigType::Queen, FigType::Bishop];

    let (threat_tiles, threat_type): (Box<dyn Iterator<Item = (usize, usize)>>, [FigType; 2]) =
        match threat_direction {
            PosRelToKing::Unrelated => return false,
            PosRelToKing::Above => (
                Box::new((king_row + 1..8).map(|r| (r, king_col))),
                rank_threats,
            ),
            PosRelToKing::Below => {
                let threat_tiles = Box::new((0..king_row).map(|r| (r, king_col)));
                (threat_tiles, rank_threats)
            }
            PosRelToKing::Left => (
                Box::new((0..king_col).rev().map(|c| (king_row, c))),
                rank_threats,
            ),
            PosRelToKing::Right => (
                Box::new((king_col + 1..8).map(|c| (king_row, c))),
                rank_threats,
            ),
            PosRelToKing::UpRight => (
                Box::new((king_row + 1..8).zip(king_col + 1..8)),
                diag_threats,
            ),
            PosRelToKing::UpLeft => (
                Box::new((king_row + 1..8).zip((0..king_col).rev())),
                diag_threats,
            ),
            PosRelToKing::DownLeft => (
                Box::new((0..king_row).rev().zip((0..king_col).rev())),
                diag_threats,
            ),
            PosRelToKing::DownRight => (
                Box::new((0..king_row).rev().zip(king_col + 1..8)),
                diag_threats,
            ),
        };
    _check_threat_vector(&board, enemy_color, threat_type, threat_tiles)
}

fn _check_threat_vector<I>(
    board: &[[Option<Figure>; 8]; 8],
    enemy_color: PlayerColor,
    threat_types: [FigType; 2],
    threat_tiles: I,
) -> bool
where
    I: IntoIterator<Item = (usize, usize)>,
{
    for (r, c) in threat_tiles {
        if let Some(fig) = board[r][c] {
            if (fig.player_color == enemy_color) && threat_types.contains(&fig.fig_type) {
                return true;
            }
            return false;
        }
    }
    false
}

/// Get the position of the figure, relative to the (own) king
pub fn pos_rel_to_king(fig_pos: (usize, usize), king_pos: (usize, usize)) -> PosRelToKing {
    let (king_row, king_col) = king_pos;
    let (fig_row, fig_col) = fig_pos;
    match (fig_row.cmp(&king_row), fig_col.cmp(&king_col)) {
        (Ordering::Equal, Ordering::Greater) => PosRelToKing::Right,
        (Ordering::Equal, Ordering::Less) => PosRelToKing::Left,
        (Ordering::Greater, Ordering::Equal) => PosRelToKing::Above,
        (Ordering::Less, Ordering::Equal) => PosRelToKing::Below,
        (Ordering::Greater, Ordering::Greater) => {
            if fig_row - king_row == fig_col - king_col {
                PosRelToKing::UpRight
            } else {
                PosRelToKing::Unrelated
            }
        }
        (Ordering::Less, Ordering::Less) => {
            if king_row - fig_row == king_col - fig_col {
                PosRelToKing::DownLeft
            } else {
                PosRelToKing::Unrelated
            }
        }
        (Ordering::Greater, Ordering::Less) => {
            if fig_row - king_row == king_col - fig_col {
                PosRelToKing::UpLeft
            } else {
                PosRelToKing::Unrelated
            }
        }
        (Ordering::Less, Ordering::Greater) => {
            if king_row - fig_row == fig_col - king_col {
                PosRelToKing::DownRight
            } else {
                PosRelToKing::Unrelated
            }
        }
        _ => PosRelToKing::Unrelated,
    }
}

pub fn get_busy_tiles(
    board: &[[Option<Figure>; 8]; 8],
    player_color: PlayerColor,
) -> Vec<(usize, usize)> {
    iproduct!(0..8, 0..8)
        .filter(|(r, c)| match board[*r][*c] {
            Some(fig) if fig.player_color == player_color => true,
            _ => false,
        })
        .collect()
}
impl GameState {
    pub fn get_figure_name(&self, row: usize, col: usize) -> Option<&'static str> {
        match self.board[row][col] {
            None => None,
            Some(fig) => Some(fig.ass_name),
        }
    }

    pub fn get_king_position(&self, fig_color: PlayerColor) -> (usize, usize) {
        iproduct!(0..8, 0..8)
            .find(|(r, c)| match self.board[*r][*c] {
                Some(fig) => fig.player_color == fig_color && fig.fig_type == FigType::King,
                None => false,
            })
            .unwrap()
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
            from_tile == moves.from && moves.to.contains(&to_tile)
        } else {
            false
        }
    }

    /// Meant to be called after an own move to see whether enemy is in own naive possible moves
    pub fn enemy_under_attack(&self, attacker_tile: (usize, usize)) -> Option<Attacker> {
        let king_pos = self.get_king_position(self.player_turn.other_player());

        // Here we assume that the move was already executed!
        if calculate_naive_moves(&self.board, attacker_tile).contains(&king_pos) {
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
        let king_pos = self.get_king_position(self.player_turn);

        if king_pos == fig_pos {
            for (to_row, to_col) in fig_moves.into_iter() {
                let mut board_clone = self.board.clone();
                board_clone[to_row][to_col] = board_clone[from_row][from_col].take();

                let enemy_tiles = get_busy_tiles(&board_clone, self.player_turn.other_player());

                if enemy_tiles.into_iter().any(|enemy_move| {
                    calculate_naive_moves(&board_clone, enemy_move).contains(&(to_row, to_col))
                }) {
                    continue;
                } else {
                    out_moves.push((to_row, to_col));
                }
            }
        } else {
            let possible_threat_direction = pos_rel_to_king(fig_pos, king_pos);

            for (to_row, to_col) in self.filter_moves_under_attack(fig_moves).into_iter() {
                let mut test_board = self.board.clone();
                test_board[to_row][to_col] = test_board[from_row][from_col].take();

                if threats_detected(
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
    pub fn filter_moves_under_attack(&self, moves: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        let Some(attacker) = self.under_attack else {
            return moves;
        };

        let king_pos = self.get_king_position(self.player_turn);
        let attack_angle = pos_rel_to_king(attacker.tile, king_pos);

        let stopping_moves: Vec<(usize, usize)> = match attacker.fig.fig_type {
            FigType::King => panic!("Attacker can never be the enemy king."),
            FigType::Knight | FigType::Pawn => Vec::from([attacker.tile]), // Pawns and Knights can only be stopped by killing move
            FigType::Rook | FigType::Bishop | FigType::Queen => {
                get_tiles_between(attack_angle, king_pos, attacker.tile).collect()
            }
        };

        let filter_set: HashSet<(usize, usize)> = stopping_moves.into_iter().collect();
        let out: HashSet<(usize, usize)> = moves.into_iter().collect();

        println!("{:?}", filter_set);

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
            board: board,
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

/// Calculates the pawn moves. Note: Does not check whether the move might be illegal by setting thyself checkmate.
/// This is handled for every move after the fact.
pub fn white_pawn_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out = Vec::<(usize, usize)>::new();
    // 1. Move one up, if there is no other piece (including piece itself, in case of boundary wrap)
    let (r, c) = ((from_row + 1).min(7), from_col);
    if board[r][c].is_none() {
        out.push((r, c));
    }

    // 2. Move two up, if there is no other piece in the way, and we start at row 1
    if from_row == 1 && board[from_row + 1][c].is_none() && board[from_row + 2][c].is_none() {
        out.push((from_row + 2, from_col));
    }
    // 3. Move diagonal right /left, in case there is black piece
    let (r, c) = ((from_row + 1).min(7), (from_col + 1).min(7));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::Black
        {
            out.push((r, c));
        }
    }
    // 4. Move diagonally left, in case there is a black piece
    let (r, c) = ((from_row + 1).min(7), (from_col.saturating_sub(1)));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::Black
        {
            out.push((r, c));
        }
    }

    out
}

pub fn black_pawn_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out = Vec::<(usize, usize)>::new();
    // 1. Move one down, if there is no other piece (including piece itself, in case of boundary wrap)
    let (r, c) = (from_row.saturating_sub(1), from_col);
    if board[r][c].is_none() {
        out.push((r, c));
    }

    // 2. Move two up, if there is no other piece in the way, and we start at row 1
    if from_row == 6 && board[from_row - 1][c].is_none() && board[from_row - 2][c].is_none() {
        out.push((from_row - 2, from_col));
    }
    // 3. Move diagonal right /left, in case there is white piece
    let (r, c) = (from_row.saturating_sub(1), (from_col + 1).min(7));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::White
        {
            out.push((r, c));
        }
    }
    // 4. Move diagonally left, in case there is a white piece
    let (r, c) = (from_row.saturating_sub(1), (from_col.saturating_sub(1)));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::White
        {
            out.push((r, c));
        }
    }

    out
}

pub fn rook_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out: Vec<(usize, usize)> = Vec::new();
    if let Some(fig) = board[from_row][from_col] {
        let rook_color = fig.player_color;

        // To the right; stop when encounter
        for r_next in (from_row + 1)..=7 {
            if let Some(block_fig) = board[r_next][from_col] {
                if rook_color != block_fig.player_color {
                    out.push((r_next, from_col));
                }
                break;
            } else {
                out.push((r_next, from_col));
            }
        }
        // To the left, stop when encounter
        for r_next in (0..from_row).rev() {
            if let Some(block_fig) = board[r_next][from_col] {
                if rook_color != block_fig.player_color {
                    out.push((r_next, from_col));
                }
                break;
            } else {
                out.push((r_next, from_col));
            }
        }

        // To the top; stop when encounter
        for c_next in (from_col + 1)..=7 {
            if let Some(block_fig) = board[from_row][c_next] {
                if rook_color != block_fig.player_color {
                    out.push((from_row, c_next));
                }
                break;
            } else {
                out.push((from_row, c_next));
            }
        }

        // To the bottom; stop when encounter
        for c_next in (0..from_col).rev() {
            if let Some(block_fig) = board[from_row][c_next] {
                if rook_color != block_fig.player_color {
                    out.push((from_row, c_next));
                }
                break;
            } else {
                out.push((from_row, c_next));
            }
        }
    }

    out
}

pub fn bishop_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out: Vec<(usize, usize)> = Vec::new();
    if let Some(fig) = board[from_row][from_col] {
        let bishop_color = fig.player_color;

        // To the top right; stop when encounter
        for (r_next, c_next) in ((from_row + 1)..=7).zip((from_col + 1)..=7) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.push((r_next, c_next));
                }
                break;
            } else {
                out.push((r_next, c_next));
            }
        }

        // To the bottom left; stop when encounter
        for (r_next, c_next) in (0..from_row).rev().zip((0..from_col).rev()) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.push((r_next, c_next));
                }
                break;
            } else {
                out.push((r_next, c_next));
            }
        }

        // To the top left; stop when encounter
        for (r_next, c_next) in ((from_row + 1)..=7).zip((0..from_col).rev()) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.push((r_next, c_next));
                }
                break;
            } else {
                out.push((r_next, c_next));
            }
        }

        // To the bottom right; stop when encounter
        for (r_next, c_next) in ((0..from_row).rev()).zip((from_col + 1)..=7) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.push((r_next, c_next));
                }
                break;
            } else {
                out.push((r_next, c_next));
            }
        }
    }

    out
}

pub fn queen_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    [rook_moves(board, from_tile), bishop_moves(board, from_tile)].concat()
}

pub fn knight_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut cands: Vec<(usize, usize)> = Vec::new();
    let (from_row, from_col) = from_tile;
    if let Some(fig) = board[from_row][from_col] {
        let knight_color = fig.player_color;

        // 2-hoch 1-links/rechts
        if from_row + 2 < 8 {
            if from_col + 1 < 8 {
                cands.push((from_row + 2, from_col + 1));
            }
            if from_col.saturating_sub(1) < from_col {
                cands.push((from_row + 2, from_col - 1));
            }
        }
        // 2-runter 1-links/rechts
        if from_row.saturating_sub(2) + 2 == from_row {
            if from_col + 1 < 8 {
                cands.push((from_row - 2, from_col + 1));
            }
            if from_col.saturating_sub(1) < from_col {
                cands.push((from_row - 2, from_col - 1));
            }
        }
        // 2-rechts 1-oben/unte
        if from_col + 2 < 8 {
            if from_row + 1 < 8 {
                cands.push((from_row + 1, from_col + 2));
            }
            if from_row.saturating_sub(1) < from_row {
                cands.push((from_row - 1, from_col + 2));
            }
        }

        // 2 links 1-oben/unten
        if from_col.saturating_sub(2) + 2 == from_col {
            if from_row + 1 < 8 {
                cands.push((from_row + 1, from_col - 2));
            }
            if from_row.saturating_sub(1) < from_row {
                cands.push((from_row - 1, from_col - 2));
            }
        }
        cands
            .into_iter()
            .filter(|(r, c)| match board[*r][*c] {
                Some(block_fig) => knight_color != block_fig.player_color,
                None => true,
            })
            .collect()
    } else {
        vec![]
    }
}

pub fn king_moves(
    board: &[[Option<Figure>; 8]; 8],
    from_tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let (r, c) = from_tile;
    let mut cands: Vec<(usize, usize)> = Vec::new();
    if let Some(fig) = board[r][c] {
        let king_color = fig.player_color;

        // Rechts
        if c + 1 < 8 {
            cands.push((r, c + 1));
        }

        // Oben-Rechts
        if r + 1 < 8 && c + 1 < 8 {
            cands.push((r + 1, c + 1));
        }

        // Oben
        if r + 1 < 8 {
            cands.push((r + 1, c));
        }

        // Oben links
        if r + 1 < 8 && c.saturating_sub(1) + 1 == c {
            cands.push((r + 1, c - 1));
        }

        // Links
        if c.saturating_sub(1) + 1 == c {
            cands.push((r, c - 1));
        }

        // Unten Links
        if c.saturating_sub(1) + 1 == c && r.saturating_sub(1) + 1 == r {
            cands.push((r - 1, c - 1));
        }
        // Unten
        if r.saturating_sub(1) + 1 == r {
            cands.push((r - 1, c));
        }
        // Unten rechts

        if r.saturating_sub(1) + 1 == r && c + 1 < 8 {
            cands.push((r - 1, c + 1));
        }

        cands
            .into_iter()
            .filter(|(r, c)| match board[*r][*c] {
                Some(block_fig) => king_color != block_fig.player_color,
                None => true,
            })
            .collect()
    } else {
        vec![]
    }
}
