use crate::prelude::*;
use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_gameplay_effects::prelude::*;
use bevy_hierarchical_tags::prelude::*;

#[derive(Event)]
pub struct TryExecuteAbility<T: StatTrait> {
    pub entity: Entity,
    pub ability: Ability<T>,
    pub target: Option<Entity>,
}

#[derive(Event)]
pub struct ExecuteAbility<T: StatTrait> {
    pub entity: Entity,
    pub ability: Ability<T>,
}

#[derive(Event)]
pub struct EndAbility<T: StatTrait> {
    pub entity: Entity,
    pub ability: Ability<T>,
}

#[derive(Event)]
pub struct CancelAbility {
    pub entity: Entity,
    pub ability: TagId,
}

#[derive(Clone)]
pub struct Cleanup;

pub(crate) fn cleanup_ability<T: StatTrait>(
    trigger: On<BehaveTrigger<Cleanup>>,
    mut current: Query<&mut CurrentAbility<T>>,
    mut commands: Commands,
) {
    let ctx = trigger.event().ctx();
    let owner = ctx.target_entity();

    if let Ok(mut ability) = current.get_mut(owner) {
        if let Some(ability) = ability.0.take() {
            commands.trigger(EndAbility {
                entity: owner,
                ability,
            });
        }
    }

    commands.trigger(ctx.success());
}
