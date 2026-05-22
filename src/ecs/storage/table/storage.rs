use crate::ecs::{component::ComponentId, entity::Entity, storage::table::TableComponentKey};

use super::{Archetype, TableComponentValue, TableRowLocation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableEntityLocation {
    pub(crate) archetype: usize,
    pub(crate) row: TableRowLocation,
}

pub(crate) struct TableComponentRemoval {
    pub(crate) removed_entity: Entity,
    pub(crate) new_location: Option<TableEntityLocation>,
    pub(crate) moved_entity: Option<Entity>,
}

#[derive(Default)]
pub(crate) struct TableStorage {
    archetypes: Vec<Archetype>,
    chunk_capacity: usize,
}

impl TableStorage {
    #[must_use]
    pub(crate) fn new(chunk_capacity: usize) -> Self {
        assert!(
            chunk_capacity > 0,
            "table chunk capacity must be greater than zero"
        );
        Self {
            archetypes: Vec::new(),
            chunk_capacity,
        }
    }

    pub(crate) fn insert(
        &mut self,
        entity: Entity,
        components: Vec<TableComponentValue>,
        keys: Vec<TableComponentKey>,
    ) -> TableEntityLocation {
        let mut keys = keys;
        keys.sort_by_key(|key| key.order());

        let archetype = self.find_or_create_archetype(keys);

        let row = self.archetypes[archetype].push_row(
            entity,
            components
                .into_iter()
                .map(TableComponentValue::into_parts)
                .collect(),
        );

        TableEntityLocation { archetype, row }
    }

    pub(crate) fn remove(&mut self, location: TableEntityLocation) -> Option<Entity> {
        let archetype = self
            .archetypes
            .get_mut(location.archetype)
            .expect("table entity location should reference an existing archetype");

        archetype.remove_row(location.row)
    }

    pub(crate) fn remove_component(
        &mut self,
        location: TableEntityLocation,
        component_id: ComponentId,
    ) -> Option<TableComponentRemoval> {
        let source = self.archetypes.get(location.archetype)?;

        if !source.contains_component(component_id) {
            return None;
        }

        let destination_keys = source.component_keys_without(component_id);

        let removed_row = self.archetypes[location.archetype].take_row(location.row);

        let remaining_components = removed_row
            .components
            .into_iter()
            .filter(|(id, _)| *id != component_id)
            .map(|(id, value)| TableComponentValue::from_erased(id, value))
            .collect::<Vec<_>>();

        let new_location = if destination_keys.is_empty() {
            None
        } else {
            Some(self.insert(removed_row.entity, remaining_components, destination_keys))
        };

        Some(TableComponentRemoval {
            removed_entity: removed_row.entity,
            new_location,
            moved_entity: removed_row.moved_entity,
        })
    }

    pub(crate) fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }
    pub(crate) fn len(&self) -> usize {
        self.archetypes.iter().map(Archetype::len).sum()
    }

    fn find_or_create_archetype(&mut self, keys: Vec<TableComponentKey>) -> usize {
        if let Some(index) = self
            .archetypes
            .iter()
            .position(|archetype| archetype.components() == keys)
        {
            return index;
        }

        let index = self.archetypes.len();
        self.archetypes
            .push(Archetype::new(keys, self.chunk_capacity));
        index
    }
}
