use bevy::{platform::collections::HashMap, prelude::*};
use bevy_hierarchical_tags::prelude::*;
use bevy_gameplay_effects::prelude::*;

use crate::prelude::*;

mod ability_definition;
mod ability;
mod costs;
mod tags;
mod events;

pub mod prelude {
    pub use crate::{
        AbilitiesPlugin, AbilityRegistry,
        ability_definition::AbilityDefinition,
        ability::{Ability, GrantedAbilities, CurrentAbility},
        tags::{AbilityTags},
        costs::{ItemCost, StatCost, Inventory},
        events::*,
    };
}

pub struct AbilitiesPlugin<T: StatTrait, const N: usize> {
    tags: TagRegistry<N>,
    abilities: AbilityRegistry<T>,
}

impl<T: StatTrait, const N: usize> AbilitiesPlugin<T, N> {
    pub fn new(tag_registry: TagRegistry<N>) -> Self {
        Self { tags: tag_registry, abilities: AbilityRegistry::<T>::new() }
    }

    pub fn register(&mut self, ability: AbilityDefinition<T>) {
        self.abilities.insert(ability.tags.ability, ability);
    }
}

impl<T: StatTrait, const N: usize> Plugin for AbilitiesPlugin<T, N> {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.abilities.clone());
        app.insert_resource(self.tags.clone());
        app.add_observer(ability::check_ability_constraints::<T, N>);
        app.add_observer(ability::execute_ability::<T>);
        app.add_observer(ability::remove_current::<T>);
        app.add_systems(Update, (
            ability::check_ability_canceled::<T, N>,
        ));
    }
}

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct AbilityRegistry<T: StatTrait>(HashMap<TagId, AbilityDefinition<T>>);

impl<T: StatTrait> AbilityRegistry<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ability() {

    }
}