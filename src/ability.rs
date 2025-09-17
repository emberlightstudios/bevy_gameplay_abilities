use bevy::prelude::*;
use bevy_hierarchical_tags::prelude::*;
use bevy_gameplay_effects::prelude::*;
use smallvec::SmallVec;
use crate::{costs::{AbilityCost, AbilityItems}, prelude::*};
use bevy_behave::prelude::*;


#[derive(Clone)]
pub struct Ability<T: StatTrait> {
    pub tags: AbilityTags,
    pub execution_tree: Option<Tree<Behave>>,
    pub costs: AbilityCost<T>,
    tree_entity: Option<Entity>,
    //pub level: u8,
}

impl<T: StatTrait> From<&AbilityDefinition<T>> for Ability<T> {
    fn from(value: &AbilityDefinition<T>) -> Self {
        let AbilityDefinition::<T> { tags, execution_tree,  costs } = value;
        Self {
            tags: tags.clone(),
            costs: costs.clone(),
            execution_tree: execution_tree.clone(),
            tree_entity: None,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct CurrentAbility<T: StatTrait>(Option<Ability<T>>);

impl<T: StatTrait> Default for CurrentAbility<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: StatTrait> CurrentAbility<T> {
    pub fn new() -> Self { Self::default() }
}

/// This component stores a list of ability definitions the entity is allowed to execute
#[derive(Component, Clone, Deref, DerefMut)]
pub struct GrantedAbilities<T: StatTrait>(SmallVec<[AbilityDefinition<T>; 16]>);

impl<T: StatTrait> GrantedAbilities<T> {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn from_tags(tags: impl IntoIterator<Item = TagId>, registry: &AbilityRegistry<T>) -> Self {
        let mut vec = SmallVec::new();
        tags.into_iter().for_each(|t| {
            if let Some(ability_def) = registry.get(&t) {
                vec.push(ability_def.clone());
            }
        });
        Self(vec)
    }

    pub fn get_from_tag(&self, tag: TagId) -> Option<AbilityDefinition<T>> {
        if let Some(index) = self.iter().position(|x| x.tags.ability == tag) {
            Some(self[index].clone())
        } else { None }
    }
}

pub(crate) fn ability_tags_ok(
    tags: &AbilityTags,
    tag_registry: &TagRegistry,
    active_tags: &ActiveTags,
) -> bool {
    // Must have tags
    active_tags.all_match_from(&tags.required, tag_registry) &&
    // Must NOT have tags
    active_tags.none_match_from(&tags.blocked_by, tag_registry) &&
    active_tags.none_match_from(&tags.canceled_by, tag_registry) 
}

pub(crate) fn check_ability_constraints<T: StatTrait>(
    trigger: Trigger<TryExecuteAbility<T>>,
    tag_registry: Res<TagRegistry>,
    stats: Query<&GameplayStats<T>>,
    active_tags: Query<(&ActiveTags, &GrantedAbilities<T>)>,
    items: Query<&AbilityItems>,
    mut commands: Commands,
) {
    let TryExecuteAbility{ entity, ability } = trigger.event();
    let tags = &ability.tags;
    let costs = &ability.costs;
    let Ok((active_tags, granted)) = active_tags.get(*entity) else { return };

    if !granted.iter().any(|g| g.tags.ability == tags.ability) { return }
    if !ability_tags_ok(tags, &tag_registry, active_tags) { return }

    let mut can_pay = true;

    if costs.stat_costs.iter().len() > 0 {
        let Ok(stats) = stats.get(*entity) else { return };
        for cost in costs.stat_costs.iter() {
            if stats.get(cost.stat).current_value < cost.amount {
                can_pay = false;
            }
        }
    }

    if costs.item_costs.iter().len() > 0 {
        let Ok(items) = items.get(*entity) else { return };
        for cost in costs.item_costs.iter() {
            let Some(&inventory) = items.get(&cost.item_id) else { return };
            if inventory < cost.amount as u16 {
                can_pay = false;
            }
        }
    }

    if can_pay {
        let mut ability = ability.clone();
        if let Some(tree) = &ability.execution_tree {
            let tree = commands.spawn(BehaveTree::new(tree.clone())).id();
            commands.entity(*entity).add_child(tree);
            ability.tree_entity = Some(tree);
        }
        commands.trigger(ExecuteAbility{ entity: *entity, ability: ability });
    }
}


pub(crate) fn end_ability<T: StatTrait>(
    trigger: Trigger<EndAbility<T>>,
    mut commands: Commands,
    mut current: Query<(&mut CurrentAbility<T>, &mut ActiveTags)>,
) {
    let EndAbility{ entity, ability } = trigger.event();
    if let Ok((mut current_ability, mut tags)) = current.get_mut(*entity) {
        for tag in ability.tags.add.iter() {
            tags.remove(*tag);
        }
        current_ability.0 = None;
        if let Some(tree) = &ability.tree_entity {
            commands.entity(*tree).despawn();
        }
    }
}
        
pub(crate) fn execute_ability<T: StatTrait>(
    trigger: Trigger<ExecuteAbility<T>>,
    mut q: Query<(&mut ActiveTags, &mut CurrentAbility<T>)>,
) {
    let ExecuteAbility { entity, ability } = trigger.event();
    if let Ok((mut tags, mut current)) = q.get_mut(*entity) {
        ability.tags.add.iter().for_each(|t| tags.push(*t));
        current.0 = Some(ability.clone());
    }
}

pub(crate) fn check_ability_canceled<T: StatTrait>(
    q: Query<(Entity, &ActiveTags, &CurrentAbility<T>)>,
    registry: Res<TagRegistry>,
   mut commands: Commands,
) {
    q.iter().for_each(|(entity, tags, current)| {
        if let Some(ability) = &current.0 {
            if ability.tags.canceled_by.any_match_from(tags, &registry) {
                commands.trigger(EndAbility{ entity, ability: ability.clone() });
            }
        }
    })
}
