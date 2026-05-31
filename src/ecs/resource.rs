use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait Resource: 'static {}

impl<T: 'static> Resource for T {}

#[derive(Default)]
pub struct Resources {
    values: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.values
            .insert(TypeId::of::<T>(), Box::new(resource))
            .and_then(|previous| previous.downcast::<T>().ok())
            .map(|previous| *previous)
    }

    #[must_use]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())?.downcast_ref()
    }

    #[must_use]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.values.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        self.values
            .remove(&TypeId::of::<T>())?
            .downcast::<T>()
            .ok()
            .map(|resource| *resource)
    }

    #[must_use]
    pub fn contains<T: Resource>(&self) -> bool {
        self.values.contains_key(&TypeId::of::<T>())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Time {
        tick: u64,
    }

    #[derive(Debug, PartialEq)]
    struct Settings {
        speed: u32,
    }

    #[test]
    fn insert_and_get_resource() {
        let mut resources = Resources::new();

        assert_eq!(resources.insert(Time { tick: 10 }), None);
        assert_eq!(resources.get::<Time>(), Some(&Time { tick: 10 }));
    }

    #[test]
    fn insert_replaces_existing_resource() {
        let mut resources = Resources::new();

        resources.insert(Time { tick: 10 });

        assert_eq!(resources.insert(Time { tick: 20 }), Some(Time { tick: 10 }));

        assert_eq!(resources.get::<Time>(), Some(&Time { tick: 20 }));
    }

    #[test]
    fn get_mut_updates_resource() {
        let mut resources = Resources::new();

        resources.insert(Time { tick: 10 });

        resources.get_mut::<Time>().expect("Time should exist").tick += 1;

        assert_eq!(resources.get::<Time>(), Some(&Time { tick: 11 }));
    }

    #[test]
    fn remove_returns_resource() {
        let mut resources = Resources::new();

        resources.insert(Settings { speed: 2 });

        assert_eq!(resources.remove::<Settings>(), Some(Settings { speed: 2 }));

        assert!(!resources.contains::<Settings>());
        assert!(resources.is_empty());
    }

    #[test]
    fn resources_are_stored_by_type() {
        let mut resources = Resources::new();

        resources.insert(Time { tick: 10 });
        resources.insert(Settings { speed: 2 });

        assert_eq!(resources.len(), 2);
        assert_eq!(resources.get::<Time>(), Some(&Time { tick: 10 }));
        assert_eq!(resources.get::<Settings>(), Some(&Settings { speed: 2 }));
    }
}
