mod utils;

/// Maybe extract the render logic here later (Movement, highlighting and such)
pub mod render_logic {
    use bevy::prelude::*;
}

pub mod game_logic {

    use std::cmp::Ordering;

    use bevy::prelude::*;
    use itertools::{Itertools, iproduct};

    use crate::utils::idx_to_coordinates;

    /// Position Relative to the own king
    #[derive(Copy, Clone, PartialEq, Eq)]
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

    #[derive(Resource)]
    pub struct GameState {
        pub board: [[Option<Figure>; 8]; 8],
        pub player_turn: PlayerColor,
        pub chosen_figure: Option<(Figure, usize, usize)>,
        pub possible_moves: Option<PossibleMoves>,
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

                move_asset(to_be_moved, query, to_tile);
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

        /// Calculate the moves of the figure on the tile. The moves are not yet filtered,
        /// on whether they might cause a check.
        pub fn calculate_naive_moves(&self, tile: (usize, usize)) -> Vec<(usize, usize)> {
            let (from_row, from_col) = tile;
            if let Some(fig) = self.board[from_row][from_col] {
                match fig.fig_type {
                    FigType::Pawn => match fig.player_color {
                        PlayerColor::Black => black_pawn_moves(&self.board, tile),
                        PlayerColor::White => white_pawn_moves(&self.board, tile),
                    },
                    FigType::Rook => rook_moves(&self.board, tile),
                    FigType::Knight => knight_moves(&self.board, tile),
                    FigType::Bishop => bishop_moves(&self.board, tile),
                    FigType::Queen => queen_moves(&self.board, tile),
                    FigType::King => king_moves(&self.board, tile), // TODO: Add filter for alle dangerous moves
                }
            } else {
                vec![]
            }
        }

        pub fn pos_rel_to_king(king_pos: (usize, usize), fig_pos: (usize, usize)) -> PosRelToKing {
            let (king_row, king_col) = king_pos;
            let (fig_row, fig_col) = fig_pos;

            todo!()
        }

        /// Pick a figure to be moved on the next click to the position
        /// In case no valid tile is clicked, none will be returned
        pub fn pick_figure_to_move() -> Option<(usize, usize)> {
            todo!()
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
}
