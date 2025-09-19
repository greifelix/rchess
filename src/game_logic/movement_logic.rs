// use itertools::Itertools;

use crate::game_logic::*;
use std::collections::HashSet;

pub struct MoveBuilder {
    pub fig_pos: (usize, usize),
    pub king_pos: (usize, usize),
    pub board: Board,
    pub fig: Figure,
    pub moveset: HashSet<(usize, usize)>,
}

impl MoveBuilder {
    pub fn new(fig_pos: (usize, usize), board: Board) -> MoveBuilder {
        let fig = board.get_fig_on_tile(fig_pos.0, fig_pos.1).unwrap();
        let king_pos = board.get_king_position(fig.player_color);

        Self {
            fig_pos,
            king_pos,
            board,
            fig,
            moveset: HashSet::new(),
        }
    }

    pub fn extract(self) -> PossibleMoves {
        PossibleMoves {
            from_tile: self.fig_pos,
            to: self.moveset,
        }
    }

    pub fn calculate_naive_moves(mut self) -> MoveBuilder {
        self.moveset = match self.fig.fig_type {
            FigType::Pawn => match self.fig.player_color {
                PlayerColor::Black => black_pawn_moves(&self.board, self.fig_pos),
                PlayerColor::White => white_pawn_moves(&self.board, self.fig_pos),
            },
            FigType::Rook => rook_moves(&self.board, self.fig_pos),
            FigType::Knight => knight_moves(&self.board, self.fig_pos),
            FigType::Bishop => bishop_moves(&self.board, self.fig_pos),
            FigType::Queen => queen_moves(&self.board, self.fig_pos),
            FigType::King => king_moves(&self.board, self.fig_pos),
        };

        self
    }

    pub fn filter_moves_in_check(mut self, maybe_attacker: Option<Attacker>) -> MoveBuilder {
        if let Some(attacker) = maybe_attacker
            && self.fig.fig_type != FigType::King
        {
            let attack_angle = movement_logic::pos_rel_to_king(attacker.tile, self.king_pos);

            let stopping_moves: HashSet<(usize, usize)> = match attacker.fig.fig_type {
                FigType::King => panic!("Attacker can never be the enemy king."),
                FigType::Knight | FigType::Pawn => HashSet::from([attacker.tile]), // Pawns and Knights can only be stopped by killing move
                FigType::Rook | FigType::Bishop | FigType::Queen => {
                    movement_logic::get_tiles_between(attack_angle, self.king_pos, attacker.tile)
                        .collect()
                }
            };

            self.moveset = self
                .moveset
                .intersection(&stopping_moves)
                .copied()
                .collect();
        }
        self
    }

    pub fn block_selfchecking_moves(mut self) -> MoveBuilder {
        let mut guilty_moves = HashSet::new();
        let (from_row, from_col) = self.fig_pos;
        match self.fig.fig_type {
            FigType::King => {
                for (to_row, to_col) in self.moveset.iter() {
                    let mut board_clone = self.board.clone();
                    board_clone[*to_row][*to_col] = board_clone[from_row][from_col].take();

                    let enemy_tiles = self.board.clone().get_busy_tiles(
                        self.fig.player_color.other_player(),
                    );

                    if enemy_tiles.into_iter().any(|(enemy_row, enemy_col)| {
                        MoveBuilder::new(
                            (enemy_row, enemy_col),
                            board_clone
                        )
                        .calculate_naive_moves()
                        .extract()
                        .to
                        .contains(&(*to_row, *to_col))
                    }) {
                        guilty_moves.insert((*to_row, *to_col));
                    }
                }
            }
            _ => {
                let possible_threat_direction =
                    movement_logic::pos_rel_to_king(self.fig_pos, self.king_pos);
                for (to_row, to_col) in self.moveset.iter() {
                    let mut test_board = self.board.clone();
                    test_board[*to_row][*to_col] = test_board[from_row][from_col].take();

                    if movement_logic::threats_detected(
                        self.king_pos,
                        test_board,
                        possible_threat_direction,
                        self.fig.player_color.other_player(),
                    ) {
                        guilty_moves.insert((*to_row, *to_col));
                    }
                }
            }
        };

        self.moveset = self.moveset.difference(&guilty_moves).copied().collect();
        self
    }
}

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

/// Returns the tiles between the king and a fig, where the kings tile is not inclusive but the figure is
pub fn get_tiles_between(
    relative_pos: PosRelToKing,
    king_pos: (usize, usize),
    pos: (usize, usize),
) -> Box<dyn Iterator<Item = (usize, usize)>> {
    let (king_row, king_col) = king_pos;
    let (fig_row, fig_col) = pos;

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
    board: Board,
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
    board: &Board,
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




// ++++++++++++++++++ Each individual figure move ++++++++++++++++++
pub fn white_pawn_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    // 1. Move one up, if there is no other piece (including piece itself, in case of boundary wrap)
    let (r, c) = ((from_row + 1).min(7), from_col);
    if board[r][c].is_none() {
        out.insert((r, c));
    }

    // 2. Move two up, if there is no other piece in the way, and we start at row 1
    if from_row == 1 && board[from_row + 1][c].is_none() && board[from_row + 2][c].is_none() {
        out.insert((from_row + 2, from_col));
    }
    // 3. Move diagonal right /left, in case there is black piece
    let (r, c) = ((from_row + 1).min(7), (from_col + 1).min(7));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::Black
        {
            out.insert((r, c));
        }
    }
    // 4. Move diagonally left, in case there is a black piece
    let (r, c) = ((from_row + 1).min(7), (from_col.saturating_sub(1)));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::Black
        {
            out.insert((r, c));
        }
    }

    out
}



pub fn black_pawn_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    // 1. Move one down, if there is no other piece (including piece itself, in case of boundary wrap)
    let (r, c) = (from_row.saturating_sub(1), from_col);
    if board[r][c].is_none() {
        out.insert((r, c));
    }

    // 2. Move two up, if there is no other piece in the way, and we start at row 1
    if from_row == 6 && board[from_row - 1][c].is_none() && board[from_row - 2][c].is_none() {
        out.insert((from_row - 2, from_col));
    }
    // 3. Move diagonal right /left, in case there is white piece
    let (r, c) = (from_row.saturating_sub(1), (from_col + 1).min(7));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::White
        {
            out.insert((r, c));
        }
    }
    // 4. Move diagonally left, in case there is a white piece
    let (r, c) = (from_row.saturating_sub(1), (from_col.saturating_sub(1)));
    if r != from_row && c != from_col {
        if let Some(f) = board[r][c]
            && f.player_color == PlayerColor::White
        {
            out.insert((r, c));
        }
    }

    out
}

pub fn rook_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    if let Some(fig) = board[from_row][from_col] {
        let rook_color = fig.player_color;

        // To the right; stop when encounter
        for r_next in (from_row + 1)..=7 {
            if let Some(block_fig) = board[r_next][from_col] {
                if rook_color != block_fig.player_color {
                    out.insert((r_next, from_col));
                }
                break;
            } else {
                out.insert((r_next, from_col));
            }
        }
        // To the left, stop when encounter
        for r_next in (0..from_row).rev() {
            if let Some(block_fig) = board[r_next][from_col] {
                if rook_color != block_fig.player_color {
                    out.insert((r_next, from_col));
                }
                break;
            } else {
                out.insert((r_next, from_col));
            }
        }

        // To the top; stop when encounter
        for c_next in (from_col + 1)..=7 {
            if let Some(block_fig) = board[from_row][c_next] {
                if rook_color != block_fig.player_color {
                    out.insert((from_row, c_next));
                }
                break;
            } else {
                out.insert((from_row, c_next));
            }
        }

        // To the bottom; stop when encounter
        for c_next in (0..from_col).rev() {
            if let Some(block_fig) = board[from_row][c_next] {
                if rook_color != block_fig.player_color {
                    out.insert((from_row, c_next));
                }
                break;
            } else {
                out.insert((from_row, c_next));
            }
        }
    }

    out
}

pub fn bishop_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    if let Some(fig) = board[from_row][from_col] {
        let bishop_color = fig.player_color;

        // To the top right; stop when encounter
        for (r_next, c_next) in ((from_row + 1)..=7).zip((from_col + 1)..=7) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.insert((r_next, c_next));
                }
                break;
            } else {
                out.insert((r_next, c_next));
            }
        }

        // To the bottom left; stop when encounter
        for (r_next, c_next) in (0..from_row).rev().zip((0..from_col).rev()) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.insert((r_next, c_next));
                }
                break;
            } else {
                out.insert((r_next, c_next));
            }
        }

        // To the top left; stop when encounter
        for (r_next, c_next) in ((from_row + 1)..=7).zip((0..from_col).rev()) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.insert((r_next, c_next));
                }
                break;
            } else {
                out.insert((r_next, c_next));
            }
        }

        // To the bottom right; stop when encounter
        for (r_next, c_next) in ((0..from_row).rev()).zip((from_col + 1)..=7) {
            if let Some(block_fig) = board[r_next][c_next] {
                if bishop_color != block_fig.player_color {
                    out.insert((r_next, c_next));
                }
                break;
            } else {
                out.insert((r_next, c_next));
            }
        }
    }

    out
}

pub fn queen_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    rook_moves(board, from_tile)
        .union(&bishop_moves(board, from_tile))
        .cloned()
        .collect()
}

pub fn knight_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    let mut cands = HashSet::new();
    let (from_row, from_col) = from_tile;
    let fig = board[from_row][from_col].unwrap();
    let knight_color = fig.player_color;

    // 2-hoch 1-links/rechts
    if from_row + 2 < 8 {
        if from_col + 1 < 8 {
            cands.insert((from_row + 2, from_col + 1));
        }
        if from_col.saturating_sub(1) < from_col {
            cands.insert((from_row + 2, from_col - 1));
        }
    }
    // 2-runter 1-links/rechts
    if from_row.saturating_sub(2) + 2 == from_row {
        if from_col + 1 < 8 {
            cands.insert((from_row - 2, from_col + 1));
        }
        if from_col.saturating_sub(1) < from_col {
            cands.insert((from_row - 2, from_col - 1));
        }
    }
    // 2-rechts 1-oben/unte
    if from_col + 2 < 8 {
        if from_row + 1 < 8 {
            cands.insert((from_row + 1, from_col + 2));
        }
        if from_row.saturating_sub(1) < from_row {
            cands.insert((from_row - 1, from_col + 2));
        }
    }

    // 2 links 1-oben/unten
    if from_col.saturating_sub(2) + 2 == from_col {
        if from_row + 1 < 8 {
            cands.insert((from_row + 1, from_col - 2));
        }
        if from_row.saturating_sub(1) < from_row {
            cands.insert((from_row - 1, from_col - 2));
        }
    }
    cands
        .into_iter()
        .filter(|(r, c)| match board[*r][*c] {
            Some(block_fig) => knight_color != block_fig.player_color,
            None => true,
        })
        .collect()
}

pub fn king_moves(board: &Board, from_tile: (usize, usize)) -> HashSet<(usize, usize)> {
    let (r, c) = from_tile;
    let mut cands = HashSet::new();
    let fig = board[r][c].unwrap();
    let king_color = fig.player_color;

    // Rechts
    if c + 1 < 8 {
        cands.insert((r, c + 1));
    }

    // Oben-Rechts
    if r + 1 < 8 && c + 1 < 8 {
        cands.insert((r + 1, c + 1));
    }

    // Oben
    if r + 1 < 8 {
        cands.insert((r + 1, c));
    }

    // Oben links
    if r + 1 < 8 && c.saturating_sub(1) + 1 == c {
        cands.insert((r + 1, c - 1));
    }

    // Links
    if c.saturating_sub(1) + 1 == c {
        cands.insert((r, c - 1));
    }

    // Unten Links
    if c.saturating_sub(1) + 1 == c && r.saturating_sub(1) + 1 == r {
        cands.insert((r - 1, c - 1));
    }
    // Unten
    if r.saturating_sub(1) + 1 == r {
        cands.insert((r - 1, c));
    }
    // Unten rechts

    if r.saturating_sub(1) + 1 == r && c + 1 < 8 {
        cands.insert((r - 1, c + 1));
    }

    cands
        .into_iter()
        .filter(|(r, c)| match board[*r][*c] {
            Some(block_fig) => king_color != block_fig.player_color,
            None => true,
        })
        .collect()
}
