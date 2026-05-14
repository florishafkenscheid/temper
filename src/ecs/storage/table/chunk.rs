use crate::ecs::{
    component::{Component, ComponentId},
    entity::Entity,
};

use super::{ComponentColumn, StoredComponent};

pub(crate) struct Chunk {
    capacity: usize,
    entities: Vec<Entity>,
    columns: Vec<ComponentColumn>,
}

impl Chunk {
    #[must_use]
    pub(crate) fn new(component_ids: &[ComponentId], capacity: usize) -> Self {
        Self {
            capacity,
            entities: Vec::with_capacity(capacity),
            columns: component_ids
                .iter()
                .copied()
                .map(ComponentColumn::new)
                .collect(),
        }
    }

    #[must_use]
    pub(crate) fn get<T: Component>(&self, component_id: ComponentId, row: usize) -> Option<&T> {
        self.columns
            .iter()
            .find(|column| column.component_id() == component_id)?
            .get(row)
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    #[must_use]
    pub(crate) fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub(crate) fn push_row(
        &mut self,
        entity: Entity,
        mut components: Vec<(ComponentId, Box<StoredComponent>)>,
    ) -> usize {
        assert!(!self.is_full());

        self.entities.push(entity);

        for column in &mut self.columns {
            let position = components
                .iter()
                .position(|(id, _)| *id == column.component_id())
                .expect("row is missing component for archetype column");

            let (_, value) = components.swap_remove(position);
            column.push(value)
        }

        assert!(
            components.is_empty(),
            "row contains components that do not belong to the archetype",
        );

        self.len() - 1
    }
}
