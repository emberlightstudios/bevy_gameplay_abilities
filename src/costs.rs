use bevy::{platform::collections::HashMap, prelude::*};
use bevy_behave::prelude::*;
use bevy_gameplay_effects::prelude::StatTrait;
use bevy_hierarchical_tags::prelude::*;
use smallvec::SmallVec;

use crate::ability::CurrentAbility;

#[derive(Clone)]
pub struct ItemCost {
    pub item_id: TagId,
    pub amount: u32,
}

#[derive(Clone)]
pub struct StatCost<T: StatTrait> {
    pub stat: T,
    pub amount: f32,
}

#[derive(Clone, Default)]
pub struct AbilityCost<T: StatTrait> {
    pub stat_costs: SmallVec<[StatCost<T>; 1]>,
    pub item_costs: SmallVec<[ItemCost; 1]>,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct AbilityItems(HashMap<TagId, u32>);

impl AbilityItems {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone)]
pub struct PayCosts;

#[derive(Event, Clone)]
pub struct CostsPaid {
    pub entity: Entity,
}

pub(crate) fn pay_item_costs<T: StatTrait>(
    trigger: On<BehaveTrigger<PayCosts>>,
    current: Query<&CurrentAbility<T>>,
    mut items: Query<&mut AbilityItems>,
    mut commands: Commands,
) {
    let ctx = trigger.event().ctx();
    let owner = ctx.target_entity();

    let Ok(Some(ability)) = current.get(owner).map(|c| c.as_ref()) else {
        return;
    };

    if let Ok(mut inventory) = items.get_mut(owner) {
        for cost in ability.costs.item_costs.iter() {
            if let Some(qty) = inventory.get_mut(&cost.item_id) {
                *qty = qty.saturating_sub(cost.amount);
            }
        }
    }

    commands.trigger(CostsPaid { entity: owner });

    commands.trigger(ctx.success());
}
