/// Shall have common types
use bevy::prelude::*;

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
