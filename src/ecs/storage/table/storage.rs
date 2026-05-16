use crate::ecs::{component::ComponentId, entity::Entity};

use super::{Archetype, TableComponentValue, TableRowLocation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableEntityLocation {
    pub(crate) archetype: usize,
    pub(crate) row: TableRowLocation,
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
    ) -> TableEntityLocation {
        let component_ids = sorted_component_ids(&components);
        let archetype = self.find_or_create_archetype(component_ids);

        let row = self.archetypes[archetype].push_row(
            entity,
            components
                .into_iter()
                .map(TableComponentValue::into_parts)
                .collect(),
        );

        TableEntityLocation { archetype, row }
    }

    pub(crate) fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }
    pub(crate) fn len(&self) -> usize {
        self.archetypes.iter().map(Archetype::len).sum()
    }

    fn find_or_create_archetype(&mut self, component_ids: Vec<ComponentId>) -> usize {
        if let Some(index) = self
            .archetypes
            .iter()
            .position(|archetype| archetype.component_ids() == component_ids)
        {
            return index;
        }

        let index = self.archetypes.len();
        self.archetypes
            .push(Archetype::new(component_ids, self.chunk_capacity));
        index
    }
}

fn sorted_component_ids(components: &[TableComponentValue]) -> Vec<ComponentId> {
    let mut ids = components
        .iter()
        .map(TableComponentValue::id)
        .collect::<Vec<_>>();

    // TODO! Temporary until ComponentRegistry owns stable component ordering.
    ids.sort_by_key(|id| format!("{id:?}"));
    ids
}
