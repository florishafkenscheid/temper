use crate::ecs::entity::Entity;

// TODO tuple-style API
pub struct QueryItem<'a, T> {
    pub entity: Entity,
    pub component: &'a T,
}

pub struct QueryItem2<'a, A, B> {
    pub entity: Entity,
    pub first: &'a A,
    pub second: &'a B,
}

pub struct QueryItemMut<'a, T> {
    pub entity: Entity,
    pub component: &'a mut T,
}
