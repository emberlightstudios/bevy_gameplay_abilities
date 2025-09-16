use bevy::prelude::*;
use bevy_gameplay_effects::prelude::*;
use bevy_hierarchical_tags::prelude::*;
use crate::prelude::*;


#[derive(Event)]
pub struct TryExecuteAbility<T: StatTrait> {
    pub entity: Entity,
    pub ability: Ability<T>,
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

