# Bevy Gameplay Abilities
This project was inspired by Unreal Engine's Gameplay Ability Systems (GAS).
It depends on two other crates that I wrote: [bevy_hierarchical_tags](https://github.com/emberlightstudios/bevy_hierarchical_tags) and [bevy_gameplay_effects](https://github.com/emberlightstudios/bevy_gameplay_effects).
It also depends on the [bevy_behave](https://github.com/RJ/bevy_behave) crate.

The purpose of this crate is to provide an abstraction layer over the flow control of abiities for characters in games. 
It does not write abilities for you.
It allows you to define rules that control ability execution in a data-driven way so you can avoid having to write complex state machines or keeping track of a ton of marker structs.
Instead it relies on the tag system which is more robust.

Internally there is a plugin, an AbilityRegistry resource, and structs for Ability and AbilityDefinition.  Since this crate is tightly coupled to the other crates, it is generic over the same parameters that many types in those crates are.
Other relevant types from bevy_gameplay_effects are ActiveTags, ActiveEffects, and GameplayStats.  

## AbilityDefinition
You define abilities with this type and then register in the registry. It has several fields worth mentioning.
### AbilityTags
This has several sub-fields
 - ability => Basically the name of the ability.  It is a TagId, and is also the key to this definition in the HashMap inside the AbilityRegistry.
 - required => Derefs to TagList.  It is a list of tags which **must** be present in order for the ability to execute.
 - blocked_by => Derefs to TagList.  It is a list of tags which **must not** be present in order for the ability to execute.
 - canceled_by => Derefs to TagList.  ActiveTags is polled every Update.  If any tags from this list show up there, ability execution will end.
 - add => Derefs to TagList.  A list of tags that will be added to Activetags when ability execution begins.

### Costs
There are 2 types of costs, stat costs and item costs.
One important thing to note is that costs are not actually paid by the abilities systems. 
You only define the conditions that will allow the ability to begin executing.  You must implement cost payment in your own systems/behavior trees.

Stat costs are self explanatory.  I would suggest looking into the GameplayEffect struct from bevy_gameplay_effects to implement them.
For implementing item costs there is a small Inventory component where you can store relevant items indexed by an ItemType: u16.
Note that this crate does not intend to provide a robust character inventory system.
I only need to keep track of items that are relevant for ability execution, e.g. ammo or grenades.

### Targeting Tree
This is an Option<Tree<Behave>>.  Some abilities require targeting behavior, e.g. throwing a grenade.
If this field is_some() then when the ability executes, an instance of the tree will be added as a child to the executing entity.
Implement your targeting systems and logic in that tree.
When targeting is complete, trigger the execution phase.
NOTE: This is still a WIP.  I will get an example of this up as soon as possible.

### Execution Tree
This is also an Option<Tree<Behave>>.  If there is no targeting tree defined, then this will get spawned instead when an ability is executed.
Implement your tree to add gameplay effects, animations, sounds, particles, pay ability costs, etc.

## Ability
Ability is the runtime version of the AbilityDefinition.  Use Ability::from<&AbilityDefinition> to create one.  It will store some relevant state for the lifetime of the ability.
