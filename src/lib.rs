mod utils;

pub mod GameLogic {
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

        pub fn despawn_entity_on_field(
            &self,
            commands: &mut Commands,
            query: &mut Query<(Entity, &Name)>,
            row: usize,
            col: usize,
        ) {
            if let Some(to_be_killed) = self.get_fig_on_tile(row, col) {
                query
                    .iter()
                    .filter(|(ent, name)| name.as_str() == to_be_killed.ass_name)
                    .for_each(|(ent, _name)| {
                        commands.entity(ent).despawn();
                    });
            }
        }

        pub fn move_figure_and_asset(
            &mut self,
            to_be_moved: &str,
            row: usize,
            col: usize,
            query: &mut Query<(Entity, &Name, &mut Transform)>,
        ) {
            query
                .iter_mut()
                .filter(|(ent, name, t)| name.as_str() == to_be_moved)
                .for_each(|(ent, name, mut t)| {
                    let (x, z) = idx_to_coordinates(row, col);

                    t.as_mut().translation.x = x;
                    t.as_mut().translation.z = z;
                });

            // TODO: Maybe call kill helper from here?
        }

        pub fn get_fig_on_tile(&self, row: usize, col: usize) -> Option<Figure> {
            self.board[row][col]
        }

        pub fn move_is_valid(&self, from_tile: (usize, usize), to_tile: (usize, usize)) -> bool {
            true
        }

        pub fn pick_is_valid(&self, row: usize, col: usize) -> bool {
            true
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
}
