mod utils;

/// Maybe extract the render logic here later (Movement, highlighting and such)
pub mod render_logic {
    use bevy::prelude::*;
}

pub mod game_logic {

    use bevy::prelude::*;

    use crate::utils::idx_to_coordinates;

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum FigType {
        Pawn,
        Rook,
        Knight,
        Bishop,
        Queen,
        King,
    }
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum PlayerColor {
        Black,
        White,
    }
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct Figure {
        fig_type: FigType,
        pub ass_name: &'static str,
        player_color: PlayerColor,
    }

    #[derive(Resource)]
    pub struct GameState {
        pub board: [[Option<Figure>; 8]; 8],
    }

    impl GameState {
        pub fn get_figure_name(&self, row: usize, col: usize) -> Option<&'static str> {
            match self.board[row][col] {
                None => None,
                Some(fig) => Some(fig.ass_name),
            }
        }

        /// Kills target asset
        pub fn kill_helper(
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

        /// Move figure potentially despawn and update game state
        pub fn move_figure_and_asset(
            &mut self,
            commands: &mut Commands,
            to_be_moved: &str,
            from_tile: (usize, usize),
            to_tile: (usize, usize),
            query: &mut Query<(Entity, &Name, &mut Transform)>,
        ) {
            if !self.move_is_valid(from_tile, to_tile) {
                return;
            }

            let (from_row, from_col) = from_tile;
            let (to_row, to_col) = to_tile;

            if let Some(target) = self.board[to_row][to_col].take() {
                self.kill_helper(commands, target.ass_name, query);
            }

            self.board[to_row][to_col] = self.board[from_row][from_col].take();

            // Todo: Replace filter with find or something
            query
                .iter_mut()
                .filter(|(ent, name, t)| name.as_str() == to_be_moved)
                .for_each(|(ent, name, mut t)| {
                    let (z, x) = idx_to_coordinates(to_row, to_col);

                    t.as_mut().translation.x = x;
                    t.as_mut().translation.z = z;
                });
        }

        pub fn get_fig_on_tile(&self, row: usize, col: usize) -> Option<Figure> {
            self.board[row][col]
        }

        pub fn move_is_valid(&self, from_tile: (usize, usize), to_tile: (usize, usize)) -> bool {
            self.calculate_valid_moves(from_tile).contains(&to_tile)
        }

        pub fn calculate_valid_moves(&self, from_tile: (usize, usize)) -> Vec<(usize, usize)> {
            let (from_row, from_col) = from_tile;

            if let Some(fig) = self.board[from_row][from_col] {
                match fig.player_color {
                    PlayerColor::White => match fig.fig_type {
                        FigType::Pawn => white_pawn_moves(&self.board, from_tile),
                        FigType::Rook => rook_moves(&self.board, from_tile),
                        FigType::Knight => {
                            vec![]
                        }
                        FigType::Bishop => {
                            vec![]
                        }
                        FigType::Queen => {
                            // Just chain bishop and rook
                            vec![]
                        }
                        FigType::King => {
                            vec![]
                        }
                    },
                    PlayerColor::Black => match fig.fig_type {
                        FigType::Pawn => {
                            black_pawn_moves(&self.board, from_tile)
                        }
                        FigType::Rook => {
                            rook_moves(&self.board, from_tile)
                        }
                        FigType::Knight => {
                            vec![]
                        }
                        FigType::Bishop => {
                            vec![]
                        }
                        FigType::Queen => {
                            vec![]
                        }
                        FigType::King => {
                            vec![]
                        }
                    },
                }
            } else {
                vec![]
            }
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

            Self { board: board }
        }
    }

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
}
