use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait Event: 'static {}

impl<T: 'static> Event for T {}

trait ErasedEventChannel {
    fn clear(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct EventChannel<T: Event> {
    events: Vec<T>,
}

impl<T: Event> Default for EventChannel<T> {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<T: Event> ErasedEventChannel for EventChannel<T> {
    fn clear(&mut self) {
        self.events.clear();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Events {
    channels: HashMap<TypeId, Box<dyn ErasedEventChannel>>,
}

impl Events {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send<T: Event>(&mut self, event: T) {
        self.channel_mut::<T>().events.push(event);
    }

    #[must_use]
    pub fn read<T: Event>(&self) -> &[T] {
        self.channels
            .get(&TypeId::of::<T>())
            .and_then(|channel| channel.as_any().downcast_ref::<EventChannel<T>>())
            .map(|channel| channel.events.as_slice())
            .unwrap_or(&[])
    }

    #[must_use]
    pub fn len<T: Event>(&self) -> usize {
        self.read::<T>().len()
    }

    #[must_use]
    pub fn is_empty<T: Event>(&self) -> bool {
        self.read::<T>().is_empty()
    }

    pub(crate) fn clear(&mut self) {
        for channel in self.channels.values_mut() {
            channel.clear();
        }
    }

    fn channel_mut<T: Event>(&mut self) -> &mut EventChannel<T> {
        self.channels
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(EventChannel::<T>::default()))
            .as_any_mut()
            .downcast_mut::<EventChannel<T>>()
            .expect("event channel should match registered event type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct DamageEvent(u32);

    #[derive(Debug, PartialEq)]
    struct SpawnEvent;

    #[test]
    fn send_stores_typed_event() {
        let mut events = Events::new();

        events.send(DamageEvent(10));

        assert_eq!(events.read::<DamageEvent>(), &[DamageEvent(10)]);
    }

    #[test]
    fn event_types_use_separate_channels() {
        let mut events = Events::new();

        events.send(DamageEvent(10));
        events.send(SpawnEvent);

        assert_eq!(events.len::<DamageEvent>(), 1);
        assert_eq!(events.len::<SpawnEvent>(), 1);
    }

    #[test]
    fn read_returns_empty_slice_for_unknown_event_type() {
        let events = Events::new();

        assert!(events.read::<DamageEvent>().is_empty());
    }

    #[test]
    fn clear_removes_events_from_all_channels() {
        let mut events = Events::new();

        events.send(DamageEvent(10));
        events.send(SpawnEvent);

        events.clear();

        assert!(events.is_empty::<DamageEvent>());
        assert!(events.is_empty::<SpawnEvent>());
    }
}
