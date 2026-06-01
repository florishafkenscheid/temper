use std::{collections::HashMap, mem};

use crate::ecs::{
    bundle::Bundle,
    command::Commands,
    component::{Component, ComponentId, ComponentRegistry},
    entity::{Entity, EntityAllocator},
    query::{QueryItem, QueryItem2, QueryItemMut},
    resource::{Resource, Resources},
    storage::table::{TableComponentKey, TableComponentValue, TableEntityLocation, TableStorage},
};

const DEFAULT_CHUNK_CAPACITY: usize = 1024;

pub struct World {
    entities: EntityAllocator,
    components: ComponentRegistry,
    table_storage: TableStorage,
    locations: HashMap<Entity, TableEntityLocation>,
    resources: Resources,
    commands: Commands,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            components: ComponentRegistry::default(),
            table_storage: TableStorage::new(DEFAULT_CHUNK_CAPACITY),
            locations: HashMap::new(),
            resources: Resources::new(),
            commands: Commands::new(),
        }
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let location = self.locations.get(&entity).copied()?;

        self.table_storage.get(location, ComponentId::of::<T>())
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let location = self.locations.get(&entity).copied()?;

        self.table_storage.get_mut(location, ComponentId::of::<T>())
    }

    pub fn query<T: Component>(&self) -> Vec<QueryItem<'_, T>> {
        self.table_storage.query(ComponentId::of::<T>())
    }

    // TODO refactor
    pub fn query2<A: Component, B: Component>(&self) -> Vec<QueryItem2<'_, A, B>> {
        self.table_storage
            .query2(ComponentId::of::<A>(), ComponentId::of::<B>())
    }

    pub fn query_mut<T: Component>(&mut self) -> Vec<QueryItemMut<'_, T>> {
        self.table_storage.query_mut(ComponentId::of::<T>())
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.spawn_table(bundle.into_bundle().into_table_components())
    }

    pub(crate) fn spawn_table(&mut self, components: Vec<TableComponentValue>) -> Entity {
        let keys = components
            .iter()
            .map(|component| {
                let id = self
                    .components
                    .register_table_id(component.id(), component.name());

                let order = self
                    .components
                    .order(id)
                    .expect("registered component should have an order");

                TableComponentKey::new(id, order)
            })
            .collect::<Vec<_>>();

        let entity = self.entities.spawn();
        let location = self.table_storage.insert(entity, components, keys);
        self.locations.insert(entity, location);
        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let Some(location) = self.locations.remove(&entity) else {
            return self.entities.despawn(entity);
        };

        if let Some(moved_entity) = self.table_storage.remove(location) {
            self.locations.insert(moved_entity, location);
        }

        self.entities.despawn(entity)
    }

    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let value = TableComponentValue::new(component);
        let id = self.components.register_table_id(value.id(), value.name());

        let order = self
            .components
            .order(id)
            .expect("registered component should have an order");

        let key = TableComponentKey::new(id, order);

        let Some(location) = self.locations.get(&entity).copied() else {
            let location = self.table_storage.insert(entity, vec![value], vec![key]);
            self.locations.insert(entity, location);
            return true;
        };

        let Some(insertion) = self.table_storage.insert_component(location, value, key) else {
            return false;
        };

        if let Some(moved_entity) = insertion.moved_entity {
            self.locations.insert(moved_entity, location);
        }

        self.locations
            .insert(insertion.entity, insertion.new_location);

        true
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let Some(location) = self.locations.get(&entity).copied() else {
            return false;
        };

        let Some(removal) = self
            .table_storage
            .remove_component(location, ComponentId::of::<T>())
        else {
            return false;
        };

        if let Some(moved_entity) = removal.moved_entity {
            self.locations.insert(moved_entity, location);
        }

        match removal.new_location {
            Some(new_location) => {
                self.locations.insert(removal.removed_entity, new_location);
            }
            None => {
                self.locations.remove(&removal.removed_entity);
            }
        }

        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.alive_count()
    }

    #[must_use]
    pub fn table_entity_count(&self) -> usize {
        self.table_storage.len()
    }

    #[must_use]
    pub fn archetype_count(&self) -> usize {
        self.table_storage.archetype_count()
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources.insert(resource)
    }

    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove::<T>()
    }

    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.resources.contains::<T>()
    }

    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    pub fn commands(&mut self) -> &mut Commands {
        &mut self.commands
    }

    pub(crate) fn apply_commands(&mut self) {
        let commands = mem::take(&mut self.commands);
        commands.apply(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::storage::table::TableComponentValue;

    #[derive(Debug, PartialEq)]
    struct Position(i32);

    #[derive(Debug, PartialEq)]
    struct Velocity(i32);

    #[derive(Debug, PartialEq)]
    struct Health(i32);

    #[derive(Debug, PartialEq)]
    struct Tick(u64);

    #[test]
    fn spawn_table_creates_alive_entity() {
        let mut world = World::new();

        let entity = world.spawn_table(vec![
            TableComponentValue::new(Position(10)),
            TableComponentValue::new(Velocity(1)),
        ]);

        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
    }

    #[test]
    fn same_component_set_reuses_archetype() {
        let mut world = World::new();

        world.spawn_table(vec![
            TableComponentValue::new(Position(10)),
            TableComponentValue::new(Velocity(1)),
        ]);

        world.spawn_table(vec![
            TableComponentValue::new(Position(20)),
            TableComponentValue::new(Velocity(2)),
        ]);

        assert_eq!(world.archetype_count(), 1);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn different_component_set_creates_new_archetype() {
        let mut world = World::new();

        world.spawn_table(vec![TableComponentValue::new(Position(10))]);

        world.spawn_table(vec![
            TableComponentValue::new(Position(20)),
            TableComponentValue::new(Velocity(2)),
        ]);

        assert_eq!(world.archetype_count(), 2);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn component_order_does_not_change_archetype_identity() {
        let mut world = World::new();

        world.spawn_table(vec![
            TableComponentValue::new(Position(10)),
            TableComponentValue::new(Velocity(1)),
        ]);

        world.spawn_table(vec![
            TableComponentValue::new(Velocity(2)),
            TableComponentValue::new(Position(20)),
        ]);

        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn spawn_accepts_typed_component_bundle() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn typed_spawn_reuses_archetype_for_same_bundle_types() {
        let mut world = World::new();

        world.spawn((Position(10), Velocity(1)));
        world.spawn((Position(20), Velocity(2)));

        assert_eq!(world.archetype_count(), 1);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn typed_spawn_supports_single_component_tuple() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.is_alive(entity));
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn despawn_removes_alive_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.despawn(entity));
        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.table_entity_count(), 0);
    }

    #[test]
    fn despawn_rejects_stale_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert!(!world.despawn(entity));
    }

    #[test]
    fn despawn_repairs_moved_entity_location() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1)));
        let second = world.spawn((Position(20), Velocity(2)));

        assert!(world.despawn(first));
        assert!(world.is_alive(second));

        let second_location = world
            .locations
            .get(&second)
            .expect("moved entity should still have a table location");

        assert_eq!(second_location.row.chunk, 0);
        assert_eq!(second_location.row.row, 0);

        assert!(world.despawn(second));
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.table_entity_count(), 0);
    }

    #[test]
    fn despawn_unknown_entity_returns_false() {
        let mut world = World::new();

        assert!(!world.despawn(Entity::new(123, 0)));
    }

    #[test]
    fn remove_component_moves_entity_to_smaller_archetype() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
        assert_eq!(world.archetype_count(), 2);
    }

    #[test]
    fn remove_component_rejects_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(!world.remove_component::<Velocity>(entity));
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
    }

    #[test]
    fn remove_component_rejects_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));
        assert!(world.despawn(entity));

        assert!(!world.remove_component::<Velocity>(entity));
    }

    #[test]
    fn remove_last_table_component_keeps_entity_alive_without_table_location() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.remove_component::<Position>(entity));
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 0);
        assert!(!world.locations.contains_key(&entity));
    }

    #[test]
    fn remove_component_repairs_swap_moved_entity_location() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1), Health(100)));
        let second = world.spawn((Position(20), Velocity(2), Health(90)));

        assert!(world.remove_component::<Velocity>(first));

        let second_location = world
            .locations
            .get(&second)
            .expect("swap-moved entity should still have a table location");

        assert_eq!(second_location.row.chunk, 0);
        assert_eq!(second_location.row.row, 0);

        assert!(world.is_alive(first));
        assert!(world.is_alive(second));
        assert_eq!(world.entity_count(), 2);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn get_component_returns_spawned_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(1)));
    }

    #[test]
    fn get_component_returns_none_for_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert_eq!(world.get_component::<Velocity>(entity), None);
    }

    #[test]
    fn get_component_returns_none_for_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert_eq!(world.get_component::<Position>(entity), None);
    }

    #[test]
    fn get_component_reflects_component_removal() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), None);
    }

    #[test]
    fn get_component_survives_swap_moved_location_repair() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1)));
        let second = world.spawn((Position(20), Velocity(2)));

        assert!(world.despawn(first));

        assert_eq!(world.get_component::<Position>(second), Some(&Position(20)));
        assert_eq!(world.get_component::<Velocity>(second), Some(&Velocity(2)));
    }

    #[test]
    fn get_component_mut_updates_component_value() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        let position = world
            .get_component_mut::<Position>(entity)
            .expect("entity should have Position");

        position.0 += 5;

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(15)));
    }

    #[test]
    fn get_component_mut_returns_none_for_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.get_component_mut::<Velocity>(entity).is_none());
    }

    #[test]
    fn get_component_mut_returns_none_for_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert!(world.get_component_mut::<Position>(entity).is_none());
    }

    #[test]
    fn get_component_mut_reflects_component_removal() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert!(world.get_component_mut::<Velocity>(entity).is_none());

        let position = world
            .get_component_mut::<Position>(entity)
            .expect("remaining component should still be mutable");

        position.0 = 20;

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(20)));
    }

    #[test]
    fn query_returns_entities_with_component() {
        let mut world = World::new();

        let first = world.spawn((Position(10),));
        let second = world.spawn((Position(20), Velocity(2)));
        world.spawn((Velocity(3),));

        let results = world.query::<Position>();

        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .any(|item| item.entity == first && item.component == &Position(10))
        );
        assert!(
            results
                .iter()
                .any(|item| item.entity == second && item.component == &Position(20))
        );
    }

    #[test]
    fn query2_returns_entities_with_both_components() {
        let mut world = World::new();

        world.spawn((Position(10),));
        let matching = world.spawn((Position(20), Velocity(2)));
        world.spawn((Velocity(3),));

        let results = world.query2::<Position, Velocity>();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity, matching);
        assert_eq!(results[0].first, &Position(20));
        assert_eq!(results[0].second, &Velocity(2));
    }

    #[test]
    fn query_ignores_removed_components() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert_eq!(world.query::<Position>().len(), 1);
        assert_eq!(world.query::<Velocity>().len(), 0);
        assert_eq!(world.query2::<Position, Velocity>().len(), 0);
    }

    #[test]
    fn query_ignores_despawned_entities() {
        let mut world = World::new();

        let first = world.spawn((Position(10),));
        let second = world.spawn((Position(20),));

        assert!(world.despawn(first));

        let results = world.query::<Position>();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity, second);
        assert_eq!(results[0].component, &Position(20));
    }

    #[test]
    fn query_mut_updates_matching_components() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1)));
        let second = world.spawn((Position(20), Velocity(2)));
        world.spawn((Velocity(3),));

        {
            let mut results = world.query_mut::<Position>();

            assert_eq!(results.len(), 2);

            for item in &mut results {
                item.component.0 += 5;
            }
        }

        assert_eq!(world.get_component::<Position>(first), Some(&Position(15)));
        assert_eq!(world.get_component::<Position>(second), Some(&Position(25)));
    }

    #[test]
    fn query_mut_ignores_removed_components() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert_eq!(world.query_mut::<Velocity>().len(), 0);

        {
            let mut positions = world.query_mut::<Position>();
            assert_eq!(positions.len(), 1);
            positions[0].component.0 = 42;
        }

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(42)));
    }

    #[test]
    fn query_mut_ignores_despawned_entities() {
        let mut world = World::new();

        let first = world.spawn((Position(10),));
        let second = world.spawn((Position(20),));

        assert!(world.despawn(first));

        {
            let mut results = world.query_mut::<Position>();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].entity, second);
            results[0].component.0 = 30;
        }

        assert_eq!(world.get_component::<Position>(second), Some(&Position(30)));
    }

    #[test]
    fn world_stores_typed_resources() {
        let mut world = World::new();

        assert_eq!(world.insert_resource(Tick(10)), None);
        assert_eq!(world.get_resource::<Tick>(), Some(&Tick(10)));

        world
            .get_resource_mut::<Tick>()
            .expect("Tick should exist")
            .0 += 1;

        assert_eq!(world.get_resource::<Tick>(), Some(&Tick(11)));
        assert_eq!(world.resource_count(), 1);
    }

    #[test]
    fn apply_commands_spawns_queued_entity() {
        let mut world = World::new();

        world.commands().spawn((Position(10), Velocity(1)));

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.commands().pending_count(), 1);

        world.apply_commands();

        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.query2::<Position, Velocity>().len(), 1);
        assert!(world.commands().is_empty());
    }

    #[test]
    fn apply_commands_despawns_queued_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        world.commands().despawn(entity);

        assert!(world.is_alive(entity));

        world.apply_commands();

        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn commands_apply_in_queue_order() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        world.commands().despawn(entity);
        world.commands().spawn((Position(20),));

        world.apply_commands();

        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.query::<Position>()[0].component, &Position(20));
    }

    #[test]
    fn apply_commands_removes_queued_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        world.commands().remove_component::<Velocity>(entity);

        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(1)));

        world.apply_commands();

        assert_eq!(world.get_component::<Velocity>(entity), None);
        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert!(world.is_alive(entity));
    }

    #[test]
    fn deferred_component_removal_ignores_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        world.commands().remove_component::<Velocity>(entity);
        world.apply_commands();

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert!(world.is_alive(entity));
    }

    #[test]
    fn deferred_component_removal_ignores_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        world.commands().despawn(entity);
        world.commands().remove_component::<Velocity>(entity);
        world.apply_commands();

        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn insert_component_moves_entity_to_larger_archetype() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.insert_component(entity, Velocity(2)));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(2)));
        assert_eq!(world.archetype_count(), 2);
    }

    #[test]
    fn insert_component_replaces_existing_value() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.insert_component(entity, Velocity(5)));

        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(5)));
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn insert_component_rejects_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert!(!world.insert_component(entity, Velocity(2)));
    }

    #[test]
    fn insert_component_adds_first_table_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.remove_component::<Position>(entity));

        assert!(world.insert_component(entity, Velocity(2)));

        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(2)));
        assert_eq!(world.table_entity_count(), 1);
    }

    #[test]
    fn insert_component_repairs_swap_moved_entity_location() {
        let mut world = World::new();

        let first = world.spawn((Position(10),));
        let second = world.spawn((Position(20),));

        assert!(world.insert_component(first, Velocity(1)));

        assert_eq!(world.get_component::<Position>(second), Some(&Position(20)));
        assert_eq!(world.get_component::<Velocity>(second), None);
    }

    #[test]
    fn apply_commands_inserts_queued_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        world.commands().insert_component(entity, Velocity(2));

        assert_eq!(world.get_component::<Velocity>(entity), None);

        world.apply_commands();

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(2)));
    }

    #[test]
    fn deferred_component_insertion_replaces_existing_value() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        world.commands().insert_component(entity, Velocity(5));
        world.apply_commands();

        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(5)));
    }

    #[test]
    fn deferred_component_insertion_ignores_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        world.commands().despawn(entity);
        world.commands().insert_component(entity, Velocity(2));
        world.apply_commands();

        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn deferred_component_insertion_adds_first_table_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.remove_component::<Position>(entity));

        world.commands().insert_component(entity, Velocity(2));
        world.apply_commands();

        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(2)));
        assert_eq!(world.table_entity_count(), 1);
    }
}
