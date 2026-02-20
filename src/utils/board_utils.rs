use super::idx_to_coordinates;
use super::type_utils::{CameraPosition, ChessScene, PlayerColor, SurfaceTile, WoodenPiece};

use crate::menu::settings::{GameMode, GameSettings};
use bevy::gltf::{Gltf, GltfExtras, GltfMesh};
use bevy::{camera::Viewport, prelude::*};
pub fn highlight_tiles(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_query: Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
    move_list_str: Vec<String>,
    tile_mat: &MeshMaterial3d<StandardMaterial>,
) {
    //1. Highlight fields
    if let Some(material) = materials.get_mut(&tile_mat.0) {
        material.base_color = Color::srgba(1.0, 0.0, 0.0, 0.6);
    }
    for (_e, n, m) in tile_query {
        if move_list_str.contains(&n.as_str().to_string()) {
            if let Some(material) = materials.get_mut(&m.0) {
                material.base_color = Color::srgba(0.0, 1.0, 0.0, 0.6); // modifies existing
            }
        }
    }
}

pub fn reset_tile_highlights(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_query: &Query<(Entity, &Name, &MeshMaterial3d<StandardMaterial>), With<SurfaceTile>>,
) {
    for (_e, _n, mat) in tile_query {
        if let Some(material) = materials.get_mut(&mat.0) {
            material.base_color = Color::NONE;
        }
    }
}

pub fn queen_spawner(
    commands: &mut Commands,
    chess_scene: &Res<ChessScene>,
    gltf_assets: &Res<Assets<Gltf>>,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    color: PlayerColor,
    queen_name: &str,
    (row, col): (u8, u8),
) {
    // Wait until the scene is loaded
    let Some(gltf) = gltf_assets.get(&chess_scene.0) else {
        return;
    };
    let (mesh_handle, mat_handle) = if color == PlayerColor::White {
        (
            gltf.meshes[30].clone(),
            gltf.named_materials["white pieces"].clone(),
        )
    } else {
        (
            gltf.meshes[31].clone(),
            gltf.named_materials["black pieces"].clone(),
        )
    };

    let (row_offset, col_offset) = idx_to_coordinates(row, col);
    let gltf_mesh = gltf_meshes.get(mesh_handle.id()).unwrap();
    commands.spawn((
        Mesh3d(gltf_mesh.primitives[0].mesh.clone()),
        MeshMaterial3d(mat_handle.clone()),
        Transform::from_xyz(col_offset, 0.047, row_offset),
        gltf_mesh.primitives[0]
            .extras
            .clone()
            .unwrap_or(GltfExtras::default()),
        Name::from(queen_name),
        WoodenPiece,
    ));
}

/// In case of resizing checks number of current of viewports
pub fn set_camera_viewports(
    windows: Query<&Window>,
    mut query: Query<(&CameraPosition, &mut Camera)>,
    settings: Res<GameSettings>,
) {
    for window in windows {
        let size = if settings.game_mode == GameMode::PVP {
            UVec2::from((window.physical_size().x / 2, window.physical_size().y))
        } else {
            window.physical_size()
        };

        for (camera_position, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}
