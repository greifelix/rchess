use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .run();
}

fn environment_setup(mut commands: Commands) {
    // Setup Camera

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Setup Lighting
    commands
        .spawn(DirectionalLight {
            shadows_enabled: true,
            illuminance: 5000., // Adjusted for a more reasonable value
            ..Default::default()
        })
        .insert(
            Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(-0.5, -1.0, -0.5), Vec3::Y),
        );
}

fn board_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the scene as a child of this entity at the given transform
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("chess_set.glb"))),
    ));
}
