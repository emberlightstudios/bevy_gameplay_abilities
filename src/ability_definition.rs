use bevy_hierarchical_tags::TagId;
use bevy_gameplay_effects::prelude::StatTrait;
use smallvec::SmallVec;
use crate::{costs::{AbilityCost, ItemCost, StatCost}, tags::AbilityTags};
use bevy_behave::prelude::*;


#[derive(Clone)]
pub struct AbilityDefinition<T: StatTrait> {
    pub tags: AbilityTags,
    pub execution_tree: Option<Tree<Behave>>,
    pub costs: AbilityCost<T>,
}

impl<T: StatTrait> AbilityDefinition<T> {
    pub fn new(tag: TagId) -> Self {
        Self {
            tags: AbilityTags::new(tag),
            costs: AbilityCost::<T> {
                stat_costs: SmallVec::new(),
                item_costs: SmallVec::new()
            },
            execution_tree: None,
        }
    }

    pub fn with_execution_tree(mut self, tree: Tree<Behave>) -> Self {
        self.execution_tree = Some(tree);
        self
    }

    pub fn with_stat_cost(mut self, cost: StatCost<T>) -> Self {
        self.costs.stat_costs.push(cost);
        self
    }

    pub fn with_item_cost(mut self, cost: ItemCost) -> Self {
        self.costs.item_costs.push(cost);
        self
    }

    /*
    pub fn level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }
     */
    
    pub fn required(mut self, tags: impl IntoIterator<Item = TagId>) -> Self {
        tags.into_iter().for_each(|tag| self.tags.required.push(tag));
        self
    }

    pub fn blocked_by(mut self, tags: impl IntoIterator<Item = TagId>) -> Self{
        tags.into_iter().for_each(|tag| self.tags.blocked_by.push(tag));
        self
    }

    pub fn canceled_by(mut self, tags: impl IntoIterator<Item = TagId>) -> Self {
        tags.into_iter().for_each(|tag| self.tags.canceled_by.push(tag));
        self
    }
    
    pub fn adds_tags(mut self, tags: impl IntoIterator<Item = TagId>) -> Self {
        tags.into_iter().for_each(|tag| self.tags.add.push(tag));
        self
    }

}