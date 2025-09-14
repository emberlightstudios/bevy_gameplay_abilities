use bevy::prelude::*;
use bevy_abilities::prelude::GrantedAbilities;
use bevy_behave::prelude::*;
use bevy_gameplay_effects::prelude::*;
use rand::prelude::*;
use rand::Rng;


pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_env);
        app.add_systems(Update, (
            check_destination_reached,
        ));
        app.add_observer(set_random_destination);
    }
}

const AREASIZE: f32 = 10.;

stats!(Stats {
    Mana
});


#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Player;

#[derive(Component, Deref, DerefMut)]
pub struct MoveTarget(Option<Vec3>);

#[derive(Component, Clone)]
pub struct MoveUntilDestinationReached;

#[derive(Clone)]
pub struct FindRandomMoveTarget;


pub fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(22., 22.,))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba { red: 0.3, green: 0.6, blue: 0.3, alpha: 1. }),
            ..default()
        })),
        Transform::default(),
    ));

    // A light:
    commands.spawn((
        PointLight {
            intensity: 15_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 0.0),
    ));

    // A camera:
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Some enemies
    let tree = tree! {
        Behave::Forever => {
            Behave::Sequence => {
                Behave::trigger(FindRandomMoveTarget),
                Behave::spawn(MoveUntilDestinationReached),
                Behave::Wait(3.0)
            }
        }
    };

    let capsule = meshes.add(Capsule3d::new(0.2, 1.0).mesh());
    for x in -10..10 {
        for y in -10..10 {
            commands.spawn((
                Mesh3d(capsule.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::LinearRgba(LinearRgba { red: 0.6, green: 0.6, blue: 0.6, alpha: 1. }),
                    ..default()
                })),
                Transform::default().with_translation(Vec3::new(x as f32, 0., y as f32)),
                Enemy,
                ActiveTags::new(),
                ActiveEffects::<Stats>::new(None),
            ))
            .with_child(BehaveTree::new(tree.clone()));
        }
    }

}

fn set_random_destination(
    trigger: Trigger<BehaveTrigger<FindRandomMoveTarget>>,
    mut commands: Commands,
    mut rng: Local<Option<SmallRng>>,
) {
    let ctx = trigger.event().ctx();
    if rng.is_none() {
        *rng = Some(rand::rngs::SmallRng::from_os_rng());
    }
    let entity = ctx.target_entity();
    let random = rng.as_mut().unwrap();
    let target = Vec3::new(
        random.random_range(-AREASIZE..AREASIZE),
        0.,
        random.random_range(-AREASIZE..AREASIZE),
    );
    commands.entity(entity).insert(MoveTarget(Some(target)));
    commands.trigger(ctx.success());
}

fn check_destination_reached(
    mut entities: Query<(&Transform, &mut MoveTarget)>,
    nodes: Query<&BehaveCtx, With<MoveUntilDestinationReached>>,
    mut commands: Commands,
) {
    for ctx in nodes.iter() {
        let entity = ctx.target_entity();
        if let Ok((transform, mut move_target)) = entities.get_mut(entity) {
            if let Some(target) = **move_target {
                let d = transform.translation - target;
                if d.length() < 0.1 {
                    **move_target = None;
                    commands.trigger(ctx.success());
                }
            }
        }
    }
}
