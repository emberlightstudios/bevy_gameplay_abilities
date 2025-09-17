use bevy::prelude::*;
use bevy_hierarchical_tags::prelude::*;
use bevy_abilities::prelude::*;
use bevy_gameplay_effects::prelude::*;


// GameplayStats is tightly integrated into GameplayAbilities.
// Abilities can have costs that can be expressed in stats.
// For this reason the plugin takes a generic argument which is
// the enum you use to define your stats.

// In this simple example we don't actually need any stats but
// I still have to create a dummy enum to use the abilities plugin. 
stats!(Stats {});

type MyAbilityRegistry = AbilityRegistry<Stats>;


fn main() {
    let mut app = App::new();

    let mut tags = TagRegistry::new();
    let death = tags.register("Ability.Death");
    let dying_state = tags.register("Character.State.Dying");
    app.insert_resource(tags);

    let tag_container = MyAbilityTags { death };

    let death_ability = AbilityDefinition::<Stats>::new(death)
        // Mark the entity as dying when the ability executes
        .adds_tags([dying_state])
        // If character is already dying, don't let the ability trigger
        .blocked_by([dying_state]);

    let mut abilities: AbilitiesPlugin<Stats> = AbilitiesPlugin::<Stats>::new();
    abilities.register(death_ability);

    app
        .add_plugins((MinimalPlugins, abilities))
        .insert_resource(tag_container)
        .add_systems(Startup, setup)
        .add_systems(Update, trigger_die_ability)
        .add_observer(handle_death)
        .run();
}

#[derive(Resource)]
struct MyAbilityTags {
    death: TagId,
}

fn setup(
    mut commands: Commands,
    my_tags: Res<MyAbilityTags>,
    abilities: Res<MyAbilityRegistry>,
) {
    let death_ability = my_tags.death;
    commands.spawn((
        ActiveTags::default(),
        CurrentAbility::<Stats>::new(),
        GrantedAbilities::<Stats>::from_tags([death_ability], &abilities),
    ));
}

fn trigger_die_ability(
    mut commands: Commands,
    my_tags: Res<MyAbilityTags>,
    q: Query<(Entity, &GrantedAbilities<Stats>)>,
) {
    if let Ok((entity, abilities)) = q.single() {
        let death_tag = my_tags.death;
        let death = abilities.get_from_tag(death_tag).unwrap();
        commands.trigger(TryExecuteAbility { entity, ability: Ability::from(&death) });
    }
}

fn handle_death(
    trigger: Trigger<ExecuteAbility<Stats>>,
    mut commands: Commands,
) {
    // Notice this only prints once even though the trigger system is in Update
    // This is because the Character.State.Dying tag blocks the ability from applying again
    println!("\nSUCCESS: Character death ability activated");
    let entity = trigger.event().entity;
    let mut entity = commands.get_entity(entity).unwrap();
    //entity.remove::<GameplayStats<Stats>>();
    entity.remove::<GrantedAbilities<Stats>>();
    entity.remove::<CurrentAbility<Stats>>();

    // Play death animation, sound cues, etc.
}