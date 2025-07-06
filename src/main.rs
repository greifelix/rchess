use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (environment_setup, board_setup).chain())
        .add_systems(Update, (add_markers_to_pieces, move_white_pieces))
        .run();
}

#[derive(Component)]
struct ChessBoard;

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
        ChessBoard,
    ));
}

// Marker components for piece color
#[derive(Component)]
pub struct BlackPiece;
#[derive(Component)]
pub struct WhitePiece;
// Marker components for piece type
#[derive(Component)]
pub struct King;
#[derive(Component)]
pub struct Queen;
#[derive(Component)]
pub struct Rook;
#[derive(Component)]
pub struct Bishop;
#[derive(Component)]
pub struct Knight;
#[derive(Component)]
pub struct Pawn;

fn add_markers_to_pieces(
    mut commands: Commands,
    // Query for all entities that are meshes and have a name
    query: Query<(Entity, &Name)>,
   mut initialized:Local<bool>
) {
    if *initialized {
        return;
    }

    for (entity, name) in query.iter() {
        let name_str = name.as_str();
        let mut entity_commands = commands.entity(entity);
        *initialized = true;

        if ["King", "Queen", "Rook", "Bishop", "Knight", "Pawn"]
            .iter()
            .any(|a| name_str.contains(a))
        {
            // Check colors
            if name_str.contains("1") || name_str.contains("2") {
                entity_commands.insert(WhitePiece);
            } else if name_str.contains("7") || name_str.contains("8") {
                entity_commands.insert(BlackPiece);
            }
            // Check type
            if name_str.contains("King") {
                entity_commands.insert(King);
            } else if name_str.contains("Queen") {
                entity_commands.insert(Queen);
            } else if name_str.contains("Rook") {
                entity_commands.insert(Rook);
            } else if name_str.contains("Bishop") {
                entity_commands.insert(Bishop);
            } else if name_str.contains("Knight") {
                entity_commands.insert(Knight);
            } else if name_str.contains("Pawn") {
                entity_commands.insert(Pawn);
            }
        }
    }
}

fn move_white_pieces(
    mut commands: Commands,
    mut query: Query<(Entity, &Name, &WhitePiece, &mut Transform)>,
) {
    for (entity, name, _, mut transform) in query.iter_mut() {
        println!("{}", name.as_str());
        transform.translation.x += 0.01;
    }
}
