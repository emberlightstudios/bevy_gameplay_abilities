use bevy::{platform::collections::HashMap, prelude::*};
use bevy_gameplay_effects::prelude::StatTrait;
use smallvec::SmallVec;

#[derive(Clone)]
pub struct ItemCost {
    pub item_id: u16,
    pub amount: u8,
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
pub struct AbilityItems(HashMap<u16, u16>);

impl AbilityItems {
    pub fn new() -> Self { Self::default() }
}