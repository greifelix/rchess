/// Shall have common types and constants.
use bevy::prelude::*;

pub const WHITE_KING_SP: (u8, u8) = (0, 4);
pub const BLACK_KING_SP: (u8, u8) = (7, 4);

#[derive(Component)]
pub struct CameraPosition {
    pub pos: UVec2,
}

#[derive(Component)]
pub struct SurfaceTile;

/// Use this marker for respawning everything
#[derive(Component)]
pub struct WoodenPiece;

#[derive(Resource)]
pub struct ChessScene(pub Handle<Gltf>);

use std::cmp::Ordering;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerColor {
    Black,
    White,
}

#[derive(Component)]
pub struct WhiteCamera;

#[derive(Component)]
pub struct BlackCamera;

impl PlayerColor {
    pub fn other_player(&self) -> PlayerColor {
        match self {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
        }
    }
}

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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Figure {
    pub fig_type: FigType,
    pub player_color: PlayerColor,
}

/// Right, AboveRight,Above,AboveLeft,Left,BelowLeft,Below,BelowRight, Straight1Diag1
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
    S1D1,
    Unrelated,
}

pub const STRAIGHT_DIRECTIONS: [Direction; 8] = [
    Direction::R,
    Direction::AR,
    Direction::A,
    Direction::AL,
    Direction::L,
    Direction::BL,
    Direction::B,
    Direction::BR,
];

pub const RANK_THREATS: [FigType; 2] = [FigType::Rook, FigType::Queen];
pub const DIAG_THREATS: [FigType; 2] = [FigType::Bishop, FigType::Queen];

impl Direction {
    /// Determines the targets_pos position relative to the source_position.
    /// (i.e. if target position is right of source position: Direction::R is returned)
    pub fn determine_relative_position(source_pos: (u8, u8), target_pos: (u8, u8)) -> Self {
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
            _ => {
                if (source_pos.0.abs_diff(target_pos.0) == 2
                    && source_pos.1.abs_diff(target_pos.1) == 1)
                    || (source_pos.1.abs_diff(target_pos.1) == 2
                        && source_pos.0.abs_diff(target_pos.0) == 1)
                {
                    Self::S1D1
                } else {
                    Self::Unrelated
                }
            }
        }
    }
}

pub const WHITE_PIECES: [Option<Figure>; 8] = [
    Some(Figure {
        fig_type: FigType::Rook,
        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::Knight,

        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::Bishop,
        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::Queen,
        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::King,
        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::Bishop,
        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::Knight,
        player_color: PlayerColor::White,
    }),
    Some(Figure {
        fig_type: FigType::Rook,
        player_color: PlayerColor::White,
    }),
];

pub const BLACK_PIECES: [Option<Figure>; 8] = [
    Some(Figure {
        fig_type: FigType::Rook,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::Knight,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::Bishop,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::Queen,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::King,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::Bishop,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::Knight,
        player_color: PlayerColor::Black,
    }),
    Some(Figure {
        fig_type: FigType::Rook,
        player_color: PlayerColor::Black,
    }),
];
