use crate::{
    core::{
        plugin::Plugin,
        schedule::{Schedule, Stage},
        time::FixedTime,
    },
    ecs::{resource::Resource, world::World},
};

/// Temporary application shell.
///
/// This exists to establish the library boundary before the real app/plugin
/// runtime is implemented.
pub struct App {
    world: World,
    schedule: Schedule,
    startup_complete: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut world = World::new();
        world.insert_resource(FixedTime::default());

        Self {
            world,
            schedule: Schedule::new(),
            startup_complete: false,
        }
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    #[must_use]
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    #[must_use]
    pub fn schedule_mut(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_system<S>(&mut self, stage: Stage, system: S) -> &mut Self
    where
        S: FnMut(&mut World) + 'static,
    {
        self.schedule.add_system(stage, system);
        self
    }

    pub fn run_stage(&mut self, stage: Stage) {
        self.schedule.run_stage(stage, &mut self.world);
        self.world.apply_commands();
    }

    fn run_startup_once(&mut self) {
        if self.startup_complete {
            return;
        }

        self.run_stage(Stage::Startup);
        self.startup_complete = true;
    }

    pub fn run_fixed_ticks(&mut self, ticks: u64) {
        self.run_startup_once();

        for _ in 0..ticks {
            self.run_stage(Stage::FixedUpdate);

            self.world
                .get_resource_mut::<FixedTime>()
                .expect("FixedTime should exist")
                .advance();

            self.run_stage(Stage::Cleanup);
            self.world.clear_events();
        }
    }

    pub fn run(&mut self) {
        self.run_fixed_ticks(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Tick(u64);

    struct TickPlugin {
        initial_tick: u64,
    }

    impl Plugin for TickPlugin {
        fn build(&self, app: &mut App) {
            app.world_mut().insert_resource(Tick(self.initial_tick));
        }
    }

    #[test]
    fn app_owns_world() {
        let mut app = App::new();

        let entity = app.world_mut().spawn((Tick(1),));

        assert!(app.world().is_alive(entity));
    }

    #[test]
    fn insert_resource_configures_world() {
        let mut app = App::new();
        app.insert_resource(Tick(10));

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(10)));
    }

    #[test]
    fn plugin_configures_app_world() {
        let mut app = App::new();
        app.add_plugin(TickPlugin { initial_tick: 42 });

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(42)));
    }

    #[test]
    fn plugins_apply_in_registration_order() {
        struct IncrementTickPlugin;

        impl Plugin for IncrementTickPlugin {
            fn build(&self, app: &mut App) {
                app.world_mut()
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            }
        }

        let mut app = App::new();
        app.add_plugin(TickPlugin { initial_tick: 10 })
            .add_plugin(IncrementTickPlugin);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }

    #[test]
    fn app_runs_registered_system_stage() {
        let mut app = App::new();

        app.insert_resource(Tick(10))
            .add_system(Stage::FixedUpdate, |world| {
                world
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            });

        app.run_stage(Stage::FixedUpdate);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }

    #[test]
    fn plugin_registers_system() {
        struct IncrementTickPlugin;

        impl Plugin for IncrementTickPlugin {
            fn build(&self, app: &mut App) {
                app.add_system(Stage::FixedUpdate, |world| {
                    world
                        .get_resource_mut::<Tick>()
                        .expect("Tick should exist")
                        .0 += 1;
                });
            }
        }

        let mut app = App::new();

        app.insert_resource(Tick(10))
            .add_plugin(IncrementTickPlugin);

        app.run_stage(Stage::FixedUpdate);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }

    #[test]
    fn fixed_tick_runner_runs_exact_tick_count() {
        let mut app = App::new();

        app.insert_resource(Tick(0))
            .add_system(Stage::FixedUpdate, |world| {
                world
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            });

        app.run_fixed_ticks(3);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(3)));
        assert_eq!(
            app.world()
                .get_resource::<FixedTime>()
                .expect("FixedTime should exist")
                .completed_ticks(),
            3
        );
    }

    #[test]
    fn startup_runs_once_across_multiple_tick_batches() {
        let mut app = App::new();

        app.insert_resource(Tick(0))
            .add_system(Stage::Startup, |world| {
                world
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 10;
            })
            .add_system(Stage::FixedUpdate, |world| {
                world
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            });

        app.run_fixed_ticks(2);
        app.run_fixed_ticks(3);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(15)));
    }

    #[test]
    fn cleanup_runs_after_each_fixed_tick() {
        #[derive(Debug, Default, PartialEq)]
        struct Log(Vec<&'static str>);

        let mut app = App::new();

        app.insert_resource(Log::default())
            .add_system(Stage::FixedUpdate, |world| {
                world
                    .get_resource_mut::<Log>()
                    .expect("Log should exist")
                    .0
                    .push("fixed");
            })
            .add_system(Stage::Cleanup, |world| {
                world
                    .get_resource_mut::<Log>()
                    .expect("Log should exist")
                    .0
                    .push("cleanup");
            });

        app.run_fixed_ticks(2);

        assert_eq!(
            app.world().get_resource::<Log>(),
            Some(&Log(vec!["fixed", "cleanup", "fixed", "cleanup"]))
        );
    }

    #[test]
    fn zero_ticks_still_runs_startup_once() {
        let mut app = App::new();

        app.insert_resource(Tick(0))
            .add_system(Stage::Startup, |world| {
                world
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            });

        app.run_fixed_ticks(0);
        app.run_fixed_ticks(0);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(1)));
    }

    #[test]
    fn fixed_update_applies_spawn_commands_at_stage_boundary() {
        #[derive(Debug, PartialEq)]
        struct Position(i32);

        let mut app = App::new();

        app.add_system(Stage::FixedUpdate, |world| {
            world.commands().spawn((Position(10),));
        });

        app.run_fixed_ticks(1);

        assert_eq!(app.world().entity_count(), 1);
        assert_eq!(app.world().query::<Position>()[0].component, &Position(10));
    }

    #[test]
    fn cleanup_observes_entities_spawned_during_fixed_update() {
        #[derive(Debug, PartialEq)]
        struct Position(i32);

        #[derive(Debug, Default, PartialEq)]
        struct ObservedCount(usize);

        let mut app = App::new();

        app.insert_resource(ObservedCount::default())
            .add_system(Stage::FixedUpdate, |world| {
                world.commands().spawn((Position(10),));
            })
            .add_system(Stage::Cleanup, |world| {
                let count = world.query::<Position>().len();

                world
                    .get_resource_mut::<ObservedCount>()
                    .expect("ObservedCount should exist")
                    .0 = count;
            });

        app.run_fixed_ticks(1);

        assert_eq!(
            app.world().get_resource::<ObservedCount>(),
            Some(&ObservedCount(1))
        );
    }

    #[test]
    fn fixed_update_applies_component_removal_after_stage() {
        #[derive(Debug, PartialEq)]
        struct Position(i32);

        #[derive(Debug, PartialEq)]
        struct Velocity(i32);

        #[derive(Debug, Default, PartialEq)]
        struct ObservedDuringSystem(bool);

        let mut app = App::new();

        let entity = app.world_mut().spawn((Position(10), Velocity(1)));

        app.insert_resource(ObservedDuringSystem::default())
            .add_system(Stage::FixedUpdate, move |world| {
                world.commands().remove_component::<Velocity>(entity);

                world
                    .get_resource_mut::<ObservedDuringSystem>()
                    .expect("ObservedDuringSystem should exist")
                    .0 = world.get_component::<Velocity>(entity).is_some();
            });

        app.run_fixed_ticks(1);

        assert_eq!(
            app.world().get_resource::<ObservedDuringSystem>(),
            Some(&ObservedDuringSystem(true))
        );

        assert_eq!(app.world().get_component::<Velocity>(entity), None);
        assert_eq!(
            app.world().get_component::<Position>(entity),
            Some(&Position(10))
        );
    }

    #[test]
    fn fixed_update_applies_component_insertion_after_stage() {
        #[derive(Debug, PartialEq)]
        struct Position(i32);

        #[derive(Debug, PartialEq)]
        struct Velocity(i32);

        #[derive(Debug, Default, PartialEq)]
        struct ObservedDuringSystem(bool);

        let mut app = App::new();

        let entity = app.world_mut().spawn((Position(10),));

        app.insert_resource(ObservedDuringSystem::default())
            .add_system(Stage::FixedUpdate, move |world| {
                world.commands().insert_component(entity, Velocity(2));

                let velocity_existed = world.get_component::<Velocity>(entity).is_some();

                world
                    .get_resource_mut::<ObservedDuringSystem>()
                    .expect("ObservedDuringSystem should exist")
                    .0 = velocity_existed;
            });

        app.run_fixed_ticks(1);

        assert_eq!(
            app.world().get_resource::<ObservedDuringSystem>(),
            Some(&ObservedDuringSystem(false))
        );

        assert_eq!(
            app.world().get_component::<Velocity>(entity),
            Some(&Velocity(2))
        );
    }

    #[test]
    fn cleanup_observes_events_sent_during_fixed_update() {
        #[derive(Debug, PartialEq)]
        struct DamageEvent(u32);

        #[derive(Debug, Default, PartialEq)]
        struct ObservedDamage(Vec<u32>);

        let mut app = App::new();

        app.insert_resource(ObservedDamage::default())
            .add_system(Stage::FixedUpdate, |world| {
                world.send_event(DamageEvent(10));
            })
            .add_system(Stage::Cleanup, |world| {
                let damage = world
                    .events::<DamageEvent>()
                    .iter()
                    .map(|event| event.0)
                    .collect::<Vec<_>>();

                world
                    .get_resource_mut::<ObservedDamage>()
                    .expect("ObservedDamage should exist")
                    .0
                    .extend(damage);
            });

        app.run_fixed_ticks(1);

        assert_eq!(
            app.world().get_resource::<ObservedDamage>(),
            Some(&ObservedDamage(vec![10]))
        );

        assert!(app.world().events::<DamageEvent>().is_empty());
    }

    #[test]
    fn events_do_not_survive_into_next_tick() {
        #[derive(Debug, PartialEq)]
        struct DamageEvent;

        #[derive(Debug, Default, PartialEq)]
        struct ObservedCounts(Vec<usize>);

        let mut app = App::new();

        app.insert_resource(ObservedCounts::default())
            .add_system(Stage::Cleanup, |world| {
                let count = world.events::<DamageEvent>().len();

                world
                    .get_resource_mut::<ObservedCounts>()
                    .expect("ObservedCounts should exist")
                    .0
                    .push(count);
            });

        app.world_mut().send_event(DamageEvent);
        app.run_fixed_ticks(2);

        assert_eq!(
            app.world().get_resource::<ObservedCounts>(),
            Some(&ObservedCounts(vec![1, 0]))
        );
    }
}
