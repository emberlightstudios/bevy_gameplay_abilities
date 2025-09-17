use std::time::Duration;

/// This example demonstrates an ability with targeting and item costs.
/// Use WASD to move and space bar to execute the ability.
/// Use the same keys for targeting.
/// You will have 10 grenades
 
use bevy::prelude::*;
use bevy_abilities::prelude::*;
use bevy_behave::prelude::*;
use bevy_gameplay_effects::prelude::*;
use bevy_hierarchical_tags::prelude::*;

mod shared;
use shared::{SharedPlugin, MoveTarget, Player, Stats, Enemy};

const MOVE_SPEED: f32 = 2.;


#[derive(Component, Clone)]
struct WaitForTargetConfirmation;

#[derive(Component, Clone)]
struct WaitForImpact;

#[derive(Component)]
struct GrenadeTarget(Vec3);

#[derive(Component)]
struct TargetingReticle;

#[derive(Clone)]
struct Explode;

#[derive(Resource)]
struct Tags {
    grenade_ability: TagId,
    throwing: TagId,
}

fn main() {
    let mut app = App::new();
    /*------+
     | Tags |
     +------*/
    // We won't use many tags in this example, but the ability needs one
    let mut tag_registry = TagRegistry::new();
    let grenade_ability = tag_registry.register("Ability.Grenade");
    let throwing = tag_registry.register("Ability.Grenade.Throwing");
    let tags = Tags { grenade_ability, throwing };
    app.insert_resource(tags);
    app.insert_resource(tag_registry);

    /*---------------+
     | Build ability |
     +---------------*/
     app.add_plugins(DefaultPlugins);

    let grenade_tree = tree!{
        Behave::Sequence => {
            Behave::spawn(WaitForTargetConfirmation),
            Behave::spawn(WaitForImpact),
            // Trigger the effect on enemies
            Behave::trigger(Explode)
        }
    };
    let grenade_ability = AbilityDefinition::<Stats>::new(grenade_ability)
        .adds_tags([throwing])
        .blocked_by([throwing])
        .with_execution_tree(grenade_tree)
        .with_item_cost(ItemCost { item_id: 1, amount: 1 });

        
    /*--------------------------+
     | Register grenade ability |
     +--------------------------*/
    let mut abilities = AbilitiesPlugin::<Stats>::new();
    abilities.register(grenade_ability);

    /*-------------+
     | Run the app |
     +-------------*/
    app
        .add_plugins((
            abilities,
            GameplayEffectsPlugin::<Stats>::default(),
            BehavePlugin::default(),
            SharedPlugin,
        ))
        .add_systems(Startup, (
            setup_player,
        ))
        .add_systems(Update, (
            move_enemies_towards_targets,
            player_movement,
            execute_grenade_ability,
            targeting_reticle,
            grenade_in_flight,
        ))
        .add_observer(explode)
        .run();
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    abilities: Res<AbilityRegistry<Stats>>,
    tags: Res<Tags>,
) {
    let mut inventory = AbilityItems::new();
    // Use u16 keys to identify items
    inventory.insert(1, 10);

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::default().mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba { red: 1., green: 0.3, blue: 0.6, alpha: 1. }),
            ..default()
        })),
        Transform::default(),
        GlobalTransform::default(),
        Player,
        ActiveTags::new(),
        ActiveEffects::<Stats>::new(None),
        GrantedAbilities::<Stats>::from_tags(
            [tags.grenade_ability], &abilities
        ),
        CurrentAbility::<Stats>::default(),
        inventory,
    ));
}

/*----------------+
 | Movement Input |
 +----------------*/
fn player_movement(
    mut q: Query<(&mut Transform, &ActiveTags), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    tag_registry: Res<TagRegistry>,
    tags: Res<Tags>,
) {
    let (mut player, active_tags) = q.single_mut().unwrap();
    if active_tags.any_match(tags.throwing, &tag_registry) {
        return;
    }

    let mut vel = Vec3::ZERO;
    if input.pressed(KeyCode::KeyA) {
        vel += Vec3::X;
    }
    if input.pressed(KeyCode::KeyD) {
        vel -= Vec3::X;
    }
    if input.pressed(KeyCode::KeyW) {
        vel += Vec3::Z;
    }
    if input.pressed(KeyCode::KeyS) {
        vel -= Vec3::Z;
    }
    if vel != Vec3::ZERO {
        vel = vel.normalize();
    }
    player.translation += 2. * vel * MOVE_SPEED * time.delta_secs();
}

/*---------------+
 | Ability Input |
 +---------------*/
fn execute_grenade_ability(
    player: Query<(Entity, &GrantedAbilities<Stats>), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    tags: Res<Tags>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Space) {
        let (entity, abilities) = player.single().unwrap();
        let grenade_def = abilities.get_from_tag(tags.grenade_ability).unwrap();
        let ability = Ability::from(&grenade_def);
        commands.trigger(TryExecuteAbility { entity, ability });
    }
}

/*----------------+
 | Enemy Movement |
 +----------------*/
fn move_enemies_towards_targets(
    mut q: Query<(&MoveTarget, &mut Transform)>,
    time: Res<Time>,
) {
    for (target, mut transform) in q.iter_mut() {
        let Some(target) = **target else { continue };
        let d = target - transform.translation;
        transform.translation += time.delta_secs() * MOVE_SPEED * d.normalize();
    }
}

/*---------------------------+
 | Ability Step 1 (targetng) |
 +---------------------------*/
fn targeting_reticle(
    mut targeter: Query<&BehaveCtx, With<WaitForTargetConfirmation>>,
    mut reticle: Query<(Entity, &mut Transform), With<TargetingReticle>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut initialized: Local<bool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
    player: Query<&Transform, (With<Player>, Without<TargetingReticle>)>,
) {
    if let Ok(ctx) = targeter.single_mut() {
        if !*initialized {
            *initialized = true;
            commands.spawn((Transform::default(), TargetingReticle));
        }

        // Did we trigger the throw?
        if input.just_pressed(KeyCode::Space) {
            *initialized = false;
            // Move to next b tree node

            // Spawn grenade object, set target
            if let Ok((reticle, reticle_transform)) = reticle.single() {
                let grenade_mesh = Mesh3d(meshes.add(Sphere::new(0.2).mesh()));
                let grenade_material = MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::LinearRgba(LinearRgba { red: 1., green: 0.3, blue: 0.9, alpha: 1. }),
                    ..default()
                }));
                let player = player.single().unwrap();
                commands.spawn((
                    GrenadeTarget(reticle_transform.translation),
                    Transform::from_translation(player.translation),
                    grenade_mesh, grenade_material,
                ));

                // Despawn reticle
                commands.entity(reticle).despawn();
            }

            commands.trigger(ctx.success());
        }
    }

    // Handle reticle input
    if let Ok((_, mut transform)) = reticle.single_mut() {
        let mut vel = Vec3::ZERO;
        if input.pressed(KeyCode::KeyA) {
            vel += Vec3::X;
        }
        if input.pressed(KeyCode::KeyD) {
            vel -= Vec3::X;
        }
        if input.pressed(KeyCode::KeyW) {
            vel += Vec3::Z;
        }
        if input.pressed(KeyCode::KeyS) {
            vel -= Vec3::Z;
        }
        if vel != Vec3::ZERO {
            vel = vel.normalize();
        }
        transform.translation += 2. * vel * MOVE_SPEED * time.delta_secs();

        let mut isometry = Isometry3d::from_translation(transform.translation);
        isometry.rotation = Quat::from_rotation_x(90_f32.to_radians());
        gizmos.circle(
            isometry,
            4., Color::linear_rgb(1., 0., 1.)
        );
    }
}

fn parabola(start: Vec3, end: Vec3, max_height: f32, t: f32) -> Vec3 {
    let mid = (start + end) / 2.0;
    let control = Vec3::new(mid.x, max_height, mid.z);
    (1.0 - t) * (1.0 - t) * start + 2.0 * (1.0 - t) * t * control + t * t * end
}
/*---------------------------------+
 | Ability Step 2 (launch grenade) |
 +---------------------------------*/
fn grenade_in_flight(
    mut ctx: Query<&BehaveCtx, With<WaitForImpact>>,
    mut player: Query<(&Transform, &mut AbilityItems), With<Player>>,
    mut grenade: Query<(&mut Transform, &GrenadeTarget), Without<Player>>,
    mut commands: Commands,
    time: Res<Time>,
    mut initialized: Local<bool>,
    mut timer: Local<Timer>,
) {
    if let Ok(ctx) = ctx.single_mut() {
        let (player, mut items) = player.single_mut ().unwrap();
        if !*initialized {
            *initialized = true;

            // pay cost
            let grenades = items.get_mut(&1).unwrap();
            // This should never go below 0 because of the cost checking in the BevyAbilities systems
            // But just in case we can use saturating sub 
            *grenades = grenades.saturating_sub(1);

            timer.set_duration(Duration::from_secs(2));
        }
        timer.tick(time.delta());

        if timer.finished() {
            *initialized = false;
            timer.reset();
            commands.trigger(ctx.success());
        }
        if let Ok((mut transform, target)) = grenade.single_mut() {
            transform.translation = parabola(player.translation, target.0, 3., timer.fraction());
        }
    }
}

/*-------------------------------+
 | Ability step 3 (kill enemies) |
 +-------------------------------*/
fn explode(
    trigger: Trigger<BehaveTrigger<Explode>>,
    grenade: Query<(Entity, &GrenadeTarget)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    player: Query<(Entity, &CurrentAbility<Stats>), With<Player>>,
    mut commands: Commands,
) {
    let ctx = trigger.event().ctx();
    let (grenade_entity, target) = grenade.single().unwrap();
    commands.entity(grenade_entity).despawn();

    // Stun enemies in range
    let range = 4.;
    for (enemy, xform) in enemies.iter() {
        let d = (xform.translation - target.0).length();
        if d <= range {
            commands.entity(enemy).despawn();
        }
    }

    // Add cooldown tag to prevent re-casting for 5 seconds
    commands.trigger(ctx.success());
    let (player, current) = player.single().unwrap();
    commands.trigger(EndAbility{ entity: player, ability: current.as_ref().unwrap().clone() });
}
