use crate::game_logic::*;
use bevy_egui::egui::menu::bar;
use rayon::prelude::*;

pub struct MoveBuilder {
    pub fig_pos: (u8, u8),
    pub fig_color: PlayerColor,
    pub fig_type: FigType,
    pub moveset: HashSet<(u8, u8)>,
}

impl MoveBuilder {
    pub fn new(fig_pos: (u8, u8), board: &Board) -> MoveBuilder {
        let fig = board.get_fig_on_tile(fig_pos.0, fig_pos.1).unwrap();
        let fig_color = fig.player_color;
        let fig_type = fig.fig_type;

        Self {
            fig_pos,
            fig_color,
            fig_type,
            moveset: HashSet::new(),
        }
    }
    /// Naively calculates the moves of one figure
    pub fn calculate_naive_moves(mut self, board: &Board) -> MoveBuilder {
        self.moveset = match self.fig_type {
            FigType::Pawn => match self.fig_color {
                PlayerColor::Black => black_pawn_moves(board, self.fig_pos),
                PlayerColor::White => white_pawn_moves(board, self.fig_pos),
            },
            FigType::Rook => rook_moves(board, self.fig_pos),
            FigType::Knight => knight_moves(board, self.fig_pos),
            FigType::Bishop => bishop_moves(board, self.fig_pos),
            FigType::Queen => queen_moves(board, self.fig_pos),
            FigType::King => king_moves(board, self.fig_pos),
        };
        self
    }
    /// TODO: Achtung: König darf natürlich woanders hin
    pub fn filter_not_in_set(mut self, filter_set: &HashSet<(u8, u8)>) -> MoveBuilder {
        self.moveset.retain(|pos| filter_set.contains(pos));

        self
    }

    pub fn calculate_all_smarter(board: &Board, player_color: PlayerColor) -> Vec<PossibleMoves> {
        let king_pos = board.get_king_position(player_color);
        if let Some((threat_r, threat_c, threat_type)) = board.player_in_check(player_color) {
            let dir_to_threat =
                Direction::determine_direction_from_to(king_pos, (threat_r, threat_c));

            let mut stopper_tiles: HashSet<(u8, u8)> =
                board.get_tiles_until_block(king_pos, dir_to_threat);
            // This one is extra for knights as they are not found in direction
            stopper_tiles.insert((threat_r, threat_c));

            board
                .get_busy_tiles(player_color)
                .into_iter()
                // .into_par_iter()
                .map(|(r, c)| {
                    if (r, c) == king_pos {
                        MoveBuilder::new((r, c), board)
                            .calculate_naive_moves(board)
                            ._filter_brute_force_2(board)
                            .extract()
                    } else {
                        // Accelerate a little by using only some
                        MoveBuilder::new((r, c), board)
                            .calculate_naive_moves(board)
                            .filter_not_in_set(&stopper_tiles)
                            ._filter_brute_force_2(board)
                            .extract()
                    }
                })
                .filter(|x| x.to.len() > 0)
                .collect()
        } else {
            let guarding_figures = board.guarding_figures(player_color, king_pos);
            board
                .get_busy_tiles(player_color)
                .into_iter()
                // .into_par_iter()
                .map(|(r, c)| {
                    let naive_moves = MoveBuilder::new((r, c), board).calculate_naive_moves(board);
                    match guarding_figures.get(&(r, c)) {
                        // 1. The chosen tile hosts a guard, check for threats behind our guard.
                        Some((guard_dir, _guard_type)) => {
                            match board.get_first_fig_in_direction((r, c), *guard_dir, (0, 8)) {
                                // 1.1 There is a directional threat behind our guard
                                Some((fig_behind, _br, _bc))
                                    if fig_behind.player_color == player_color.other_player()
                                        && fig_behind.fig_type.pins_in_direction(*guard_dir) =>
                                {
                                    // 1.1.1 Figure behind the to-be-moved figure is a directional threat!
                                    // Do extra function on naive-moves to check for each move whether orientation remains the same and filter other
                                    naive_moves
                                        ._filter_postion_changes_to_king(&king_pos, *guard_dir)
                                        .extract()
                                }
                                // 1.2 Figure behind is not a threat or does not exist
                                _ => naive_moves.extract(),
                            }
                        }
                        // 2. We got the king here, we filter carefully all moves. (For now)
                        None if (r, c) == king_pos => {
                            // naive_moves.filter_brute_force(board).extract()
                            naive_moves._filter_brute_force_2(board).extract()
                        }

                        // 3. fig not in guards, we can move without care
                        None => naive_moves.extract(),
                    }
                })
                .filter(|x| x.to.len() > 0)
                .collect()
        }
    }

    /// Filters all moves from the moveset with position changes to the king, compared to the start direction
    pub fn _filter_postion_changes_to_king(
        mut self,
        king_pos: &(u8, u8),
        start_direction: Direction,
    ) -> Self {
        self.moveset.retain(|&target_pos| {
            start_direction == Direction::determine_direction_from_to(*king_pos, target_pos)
        });

        self
    }

    /// This methods filter also works in check.
    pub fn _filter_brute_force_2(mut self, board: &Board) -> Self {
        match self.fig_type {
            FigType::King => {
                let enemy_king = board.get_king_position(self.fig_color.other_player());
                self.moveset.retain(|&(r, c)| {
                    let mut board_clone = board.clone();
                    board_clone[(r,c)] = board_clone[self.fig_pos].take();
                    board_clone.player_in_check(self.fig_color).is_none()
                        && !utils::figs_adjacent((r,c), enemy_king)
                });
            }
            _ => {
                self.moveset.retain(|&(r, c)| {
                    let mut board_clone = board.clone();
                    board_clone[(r,c)] = board_clone[self.fig_pos].take();
                    board_clone.player_in_check(self.fig_color).is_none()
                });
            }
        }

        self
    }

    // pub fn filter_brute_force(mut self, board: &Board) -> Self {
    //     self.moveset.retain(|&(to_row, to_col)| {
    //         let mut board_clone = board.clone();
    //         board_clone[to_row][to_col] = board_clone[self.fig_pos.0][self.fig_pos.1].take();
    //         let king_pos = board_clone.get_king_position(self.fig_color);
    //         !board_clone
    //             .get_busy_tiles(self.fig_color.other_player())
    //             .into_iter()
    //             .any(|(r, c)| {
    //                 MoveBuilder::new((r, c), &board_clone)
    //                     .calculate_naive_moves(&board_clone)
    //                     .extract()
    //                     .to
    //                     .contains(&king_pos)
    //             })
    //     });

    //     self
    // }

    pub fn extract(self) -> PossibleMoves {
        PossibleMoves {
            from_tile: self.fig_pos,
            to: self.moveset,
        }
    }
}

// ++++++++++++++++++ Each individual figure move ++++++++++++++++++
pub fn white_pawn_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    // 1. Move one up, if there is no other piece (including piece itself, in case of boundary wrap)
    let (r, c) = ((from_row + 1).min(7), from_col);
    if board[(r,c)].is_none() {
        out.insert((r, c));
    }

    // 2. Move two up, if there is no other piece in the way, and we start at row 1
    if from_row == 1 && board[(from_row + 1,c)].is_none() && board[(from_row + 2,c)].is_none() {
        out.insert((from_row + 2, from_col));
    }
    // 3. Move diagonal right /left, in case there is black piece
    let (r, c) = ((from_row + 1).min(7), (from_col + 1).min(7));
    if r != from_row && c != from_col {
        if let Some(f) = board[(r,c)]
            && f.player_color == PlayerColor::Black
        {
            out.insert((r, c));
        }
    }
    // 4. Move diagonally left, in case there is a black piece
    let (r, c) = ((from_row + 1).min(7), (from_col.saturating_sub(1)));
    if r != from_row && c != from_col {
        if let Some(f) = board[(r,c)]
            && f.player_color == PlayerColor::Black
        {
            out.insert((r, c));
        }
    }

    out
}

pub fn black_pawn_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    // 1. Move one down, if there is no other piece (including piece itself, in case of boundary wrap)
    let (r, c) = (from_row.saturating_sub(1), from_col);
    if board[(r,c)].is_none() {
        out.insert((r, c));
    }

    // 2. Move two up, if there is no other piece in the way, and we start at row 1
    if from_row == 6 && board[(from_row - 1,c)].is_none() && board[(from_row - 2,c)].is_none() {
        out.insert((from_row - 2, from_col));
    }
    // 3. Move diagonal right /left, in case there is white piece
    let (r, c) = (from_row.saturating_sub(1), (from_col + 1).min(7));
    if r != from_row && c != from_col {
        if let Some(f) = board[(r,c)]
            && f.player_color == PlayerColor::White
        {
            out.insert((r, c));
        }
    }
    // 4. Move diagonally left, in case there is a white piece
    let (r, c) = (from_row.saturating_sub(1), (from_col.saturating_sub(1)));
    if r != from_row && c != from_col {
        if let Some(f) = board[(r,c)]
            && f.player_color == PlayerColor::White
        {
            out.insert((r, c));
        }
    }

    out
}

pub fn rook_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    if let Some(fig) = board[from_tile] {
        let rook_color = fig.player_color;

        // To the right; stop when encounter
        for r_next in (from_row + 1)..=7 {
            if let Some(block_fig) = board[(r_next,from_col)] {
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
            if let Some(block_fig) = board[(r_next,from_col)] {
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
            if let Some(block_fig) = board[(from_row,c_next)] {
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
            if let Some(block_fig) = board[(from_row,c_next)] {
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

pub fn bishop_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    let (from_row, from_col) = from_tile;
    let mut out = HashSet::new();
    if let Some(fig) = board[(from_row,from_col)] {
        let bishop_color = fig.player_color;

        // To the top right; stop when encounter
        for (r_next, c_next) in ((from_row + 1)..=7).zip((from_col + 1)..=7) {
            if let Some(block_fig) = board[(r_next,c_next)] {
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
            if let Some(block_fig) = board[(r_next,c_next)] {
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
            if let Some(block_fig) = board[(r_next,c_next)] {
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
            if let Some(block_fig) = board[(r_next,c_next)] {
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

pub fn queen_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    rook_moves(board, from_tile)
        .union(&bishop_moves(board, from_tile))
        .cloned()
        .collect()
}

pub fn knight_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    let mut cands = HashSet::new();
    let (from_row, from_col) = from_tile;
    let fig = board[(from_row,from_col)].unwrap();
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
        .filter(|(r, c)| match board[(*r,*c)] {
            Some(block_fig) => knight_color != block_fig.player_color,
            None => true,
        })
        .collect()
}

pub fn king_moves(board: &Board, from_tile: (u8, u8)) -> HashSet<(u8, u8)> {
    let (r, c) = from_tile;
    let mut cands = HashSet::new();
    let fig = board[(r,c)].unwrap();
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
        .filter(|(r, c)| match board[(*r,*c)] {
            Some(block_fig) => king_color != block_fig.player_color,
            None => true,
        })
        .collect()
}
