/// This example demonstrates a simple stun area effect.
/// Use WASD to move and space bar to execute the ability.
/// It costs 25 mana and you start with 100, so you can do it 4 times.
/// There is also a 5 second cooldown.
 
use bevy::prelude::*;
use bevy_abilities::prelude::*;
use bevy_behave::prelude::*;
use bevy_gameplay_effects::prelude::*;
use bevy_hierarchical_tags::prelude::*;

mod shared;
use shared::{SharedPlugin, MoveTarget, Player, Stats, Enemy};

const NTAGS: usize = 128;
const MOVE_SPEED: f32 = 2.;


#[derive(Component, Clone, Deref, DerefMut)]
struct PreStunCue(Timer);

#[derive(Clone)]
struct StunTrigger;

#[derive(Component)]
struct EnemyStunShake;

#[derive(Resource)]
struct StunTags {
    character_movement_blocked_stunned: TagId,
    character_movement_blocked_casting: TagId,
    character_movement_blocked: TagId,
    ability_stun: TagId,
    ability_stun_cooldown: TagId,
}

fn main() {
    let mut app = App::new();

    /*------------------------------------------+
     | Set up the tags needed for this example  |
     +------------------------------------------*/
    let mut tag_registry = TagRegistry::<NTAGS>::new();
    let character_movement_blocked = tag_registry.register("Character.Movement.Blocked");
    let character_movement_blocked_stunned = tag_registry.register("Character.Movement.Blocked.Stunned");
    let character_movement_blocked_casting = tag_registry.register("Character.Movement.Blocked.Casting");
    let ability_stun = tag_registry.register("Ability.Stun");
    let ability_stun_cooldown = tag_registry.register("Ability.Stun.Cooldown");

    let tags = StunTags{
        ability_stun, ability_stun_cooldown, character_movement_blocked_stunned,
        character_movement_blocked, character_movement_blocked_casting
    };
    app.insert_resource(tags);

    /*--------------------+
     | Build stun ability |
     +--------------------*/
    let stun_tree = tree!{
        Behave::Sequence => {
            // Do a little animation cue
            Behave::spawn(PreStunCue(Timer::from_seconds(1., TimerMode::Once))),
            // Trigger the effect on enemies
            Behave::trigger(StunTrigger)
        }
    };
    let stun_abililty = AbilityDefinition::<Stats>::new(ability_stun)
        .adds_tags([character_movement_blocked_casting])
        .blocked_by([ability_stun_cooldown, character_movement_blocked_casting])
        .with_execution_tree(stun_tree)
        .with_stat_cost(StatCost::<Stats>{ stat: Stats::Mana, amount: 25. });

        
    /*-----------------------+
     | Register stun ability |
     +-----------------------*/
    let mut abilities = AbilitiesPlugin::<Stats, NTAGS>::new(tag_registry);
    abilities.register(stun_abililty);

    /*-------------+
     | Run the app |
     +-------------*/
    app
        .add_plugins((
            abilities,
            GameplayEffectsPlugin::<Stats>::default(),
            BehavePlugin::default(),
            DefaultPlugins,
            SharedPlugin,
        ))
        .add_systems(Startup, (
            setup_player,
        ))
        .add_systems(Update, (
            move_enemies_towards_targets,
            player_movement,
            execute_stun_ability,
            pre_stun_cue,
            enemy_stun_shake,
        ))
        .add_observer(trigger_stun)
        .run();
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    abilities: Res<AbilityRegistry<Stats>>,
    tags: Res<StunTags>,
) {
    let capsule = meshes.add(Capsule3d::default().mesh());
    commands.spawn((
        Mesh3d(capsule.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba { red: 1., green: 0.3, blue: 0.6, alpha: 1. }),
            ..default()
        })),
        Transform::default(),
        Player,
        ActiveTags::new(),
        ActiveEffects::<Stats>::new(None),
        GrantedAbilities::<Stats>::from_tags(
            [tags.ability_stun], &abilities
        ),
        CurrentAbility::<Stats>::default(),
        GameplayStats::<Stats>::new(
            |s| {
                match s {
                    Stats::Mana => 100.,
                    Stats::None => 0.
                }
            },
        ),
    ));
}

/*----------------+
 | Movement Input |
 +----------------*/
fn player_movement(
    mut q: Query<(&mut Transform, &ActiveTags), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    tag_registry: Res<TagRegistry<NTAGS>>,
    tags: Res<StunTags>,
) {
    let (mut player, active_tags) = q.single_mut().unwrap();
    if active_tags.any_match(tags.character_movement_blocked, &tag_registry) {
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
fn execute_stun_ability(
    player: Query<(Entity, &GrantedAbilities<Stats>), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    tags: Res<StunTags>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Space) {
        let (entity, abilities) = player.single().unwrap();
        let stun_ability_def = abilities.get_from_tag(tags.ability_stun).unwrap();
        let ability = Ability::from(&stun_ability_def);
        commands.trigger(TryExecuteAbility { entity, ability });
    }
}

/*----------------+
 | Enemy Movement |
 +----------------*/
fn move_enemies_towards_targets(
    mut q: Query<(&ActiveTags, &MoveTarget, &mut Transform)>,
    tags: Res<StunTags>,
    tag_registry: Res<TagRegistry<NTAGS>>,
    time: Res<Time>,
) {
    for (active_tags, target, mut transform) in q.iter_mut() {
        // Don't move if stunned
        if active_tags.any_match(tags.character_movement_blocked, &tag_registry) {
            continue
        }

        let Some(target) = **target else { continue };
        let d = target - transform.translation;
        let d = d.normalize();
        transform.translation += time.delta_secs() * MOVE_SPEED * d;
    }
}

/*--------------------------------+
 | Ability Step 1 (animation cue) |
 +--------------------------------*/
fn pre_stun_cue(
    mut cue: Query<(&mut PreStunCue, &BehaveCtx)>,
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    // Simple bounce cue for 1 second before casting
    let mut player = player.single_mut().unwrap();
    if let Ok((mut cue, ctx)) = cue.single_mut() {
        cue.tick(time.delta());
        player.translation.y = (cue.fraction() * 10.).sin().abs();

        if cue.finished() {
            player.translation.y = 0.;
            commands.trigger(ctx.success());
        }
    }
}

/*---------------------------------------+
 | Trigger the stun effect within radius |
 +---------------------------------------*/
fn trigger_stun(
    trigger: Trigger<BehaveTrigger<StunTrigger>>,
    mut player: Query<(Entity, &mut ActiveTags, &Transform), With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    tags: Res<StunTags>,
    mut commands: Commands,
) {
    // Handle effects when abiity actually executes
    
    // Unblock player movement
    let (player, mut player_tags, player_transform) = player.single_mut().unwrap();
    player_tags.remove(tags.character_movement_blocked_casting);

    // Stun enemies in range
    let range = 4.;
    let stun_effect: GameplayEffect<Stats> = GameplayEffect::tag_effect(
        tags.character_movement_blocked_stunned,
        Some(5.0)
    );
    for (enemy, enemy_transform) in enemies.iter() {
        let d = (enemy_transform.translation - player_transform.translation).length();
        if d <= range {
            commands.trigger(AddEffect(AddEffectData {
                target_entity: enemy,
                effect: stun_effect.clone(),
                source_entity: Some(player),
            }));
        }
    }

    // Add cooldown tag to prevent re-casting for 5 seconds
    commands.trigger(
        AddEffect(AddEffectData::<Stats>::new(
            player,
            GameplayEffect::tag_effect(
                tags.ability_stun_cooldown,
                Some(5.0),
            ),
            None,
        ))
    );

    // Pay mana cost
    // We defined this when we register.  All that really
    // does is ensure we have enough to pay the cost before
    // we can execute it.  When it is time to pay we still
    // have to do that manually.
    commands.trigger(
        AddEffect(AddEffectData::<Stats>::new(
            player,
            GameplayEffect::new(
                None,
                Stats::Mana,
                EffectMagnitude::Fixed(-25.),
                EffectCalculation::Additive,
                EffectDuration::Immediate,
            ),
            None,
        ))
    );

    // Finalize
    let ctx = trigger.event().ctx();
    commands.trigger(ctx.success());
    commands.entity(ctx.behave_entity()).despawn();
}

/*------------------------+
 | Enemy effect animation |
 +------------------------*/
fn enemy_stun_shake(
    mut added: EventReader<OnEffectAdded>,
    mut removed: EventReader<OnEffectRemoved>,
    mut shakers: Query<&mut Transform, With<EnemyStunShake>>,
    time: Res<Time>,
    tags: Res<StunTags>,
    mut commands: Commands,
) {
    // You could implement this as an ability also with a tree, but i'll just keep it simple

    // Add new shake components
    for ev in added.read() {
        let EffectMetadata {target_entity, tag, source_entity} = ev.0;
        if let Some(tag) = tag {
            if tag == tags.character_movement_blocked_stunned {
                commands.entity(target_entity).insert(EnemyStunShake);
            }
        }
    }

    // Do the shake!  (really it's just rotating but you get the point)
    for mut transform in shakers.iter_mut() {
        //println!("I'M LITERALLY SHAKING");
        transform.rotate_local_x(time.delta_secs() * 10.);
    }

    // Remove expired shake components
    for ev in removed.read() {
        let EffectMetadata {target_entity, tag, source_entity} = ev.0;
        if let Some(tag) = tag {
            if tag == tags.character_movement_blocked_stunned {
                commands.entity(target_entity).remove::<EnemyStunShake>();
                let mut transform = shakers.get_mut(target_entity).unwrap();
                transform.rotation = Quat::IDENTITY;
            }
        }
    }

}