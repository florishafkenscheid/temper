use crate::ecs::{component::Component, storage::table::TableComponentValue};

pub struct ComponentBundle {
    components: Vec<TableComponentValue>,
}

impl ComponentBundle {
    pub(crate) fn into_table_components(self) -> Vec<TableComponentValue> {
        self.components
    }
}

pub trait Bundle {
    fn into_bundle(self) -> ComponentBundle;
}

impl<A: Component> Bundle for (A,) {
    fn into_bundle(self) -> ComponentBundle {
        ComponentBundle {
            components: vec![TableComponentValue::new(self.0)],
        }
    }
}

impl<A: Component, B: Component> Bundle for (A, B) {
    fn into_bundle(self) -> ComponentBundle {
        ComponentBundle {
            components: vec![
                TableComponentValue::new(self.0),
                TableComponentValue::new(self.1),
            ],
        }
    }
}

impl<A: Component, B: Component, C: Component> Bundle for (A, B, C) {
    fn into_bundle(self) -> ComponentBundle {
        ComponentBundle {
            components: vec![
                TableComponentValue::new(self.0),
                TableComponentValue::new(self.1),
                TableComponentValue::new(self.2),
            ],
        }
    }
}
