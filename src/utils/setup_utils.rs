use bevy::prelude::*;

use super::core_types::{CameraPosition, ChessScene, SurfaceTile, WhiteCamera, WoodenPiece};

pub fn environment_setup(mut commands: Commands) {
    // I need camera for UI for splitscreen mode
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            ..default()
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            order: 0,
            ..default()
        },
        CameraPosition {
            pos: UVec2::new(0, 0),
        },
        MeshPickingCamera,
        WhiteCamera,
    ));

    commands
        .spawn(DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.,
            ..Default::default()
        })
        .insert(
            Transform::from_xyz(0.0, 0.8, 0.0).looking_to(Vec3::new(-0.5, -1.0, -0.5), Vec3::Y),
        );
}

pub fn board_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gltf_handle: Handle<Gltf> = asset_server.load("chess_set.gltf");
    commands.insert_resource(ChessScene(gltf_handle));

    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("chess_set.gltf"));
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        SceneRoot(scene_handle),
        Name::new("Original Chess Scene"),
        WoodenPiece,
    ));

    let square_size = 0.05;

    for (row, _row_c) in (0..8).zip('1'..='8') {
        for (col, _col_c) in (0..8).zip('a'..='h') {
            let (row_offset, col_offset) = super::idx_to_coordinates(row, col);

            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(square_size, square_size))),
                Transform::from_xyz(col_offset, 0.01, row_offset),
                Name::new(format!("Tile_{}_{}", row, col)),
                MeshMaterial3d(materials.add(Color::NONE)),
                SurfaceTile,
            ));
        }
    }
}
