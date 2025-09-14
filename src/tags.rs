use bevy_hierarchical_tags::prelude::*;
use bevy::prelude::*;




#[derive(Component, Clone, Deref, DerefMut)]
pub struct CurrentAbility(Option<TagId>);

// Per ability tags
#[derive(Clone)]
pub struct AbilityTags {
    pub ability: TagId,
    pub required: RequiredTags,
    pub blocked_by: BlockingTags,
    pub canceled_by: CancelTags,
    pub add: AbilityAddTags,
}

impl AbilityTags {
    pub fn new(tag: TagId) -> Self {
        Self{
            ability: tag,
            required: RequiredTags::default(),
            blocked_by: BlockingTags::default(),
            canceled_by: CancelTags::default(),
            add: AbilityAddTags::default(),
        }
    }
}

/// These tags are required to be present for an ability to activate
#[derive(Clone, Deref, DerefMut, Default)]
pub struct RequiredTags(TagList<4>);

/// These tags are forbidden from being present for an ability to acitvate
#[derive(Clone, Deref, DerefMut, Default)]
pub struct BlockingTags(TagList<4>);

/// These tags will cancel execution of an active ability
#[derive(Clone, Deref, DerefMut, Default)]
pub struct CancelTags(TagList<4>);

/// These tags are added to an entity when the ability executes
#[derive(Clone, Deref, DerefMut, Default)]
pub struct AbilityAddTags(TagList<2>);