use std::collections::VecDeque;
use std::marker::PhantomData;
use std::time::Duration;

use bevy::app::App;
use bevy::prelude::{
    in_state, info, trace, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Local,
    NextState, Plugin, PostUpdate, Query, Res, ResMut, Resource, Time, Timer, Update, With,
};
use bevy::time::TimerMode;
use hexx::Hex;

use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::CombatEvent;
use crate::game::ingame::hex::{HexComponent, HexMarker};
use crate::game::ingame::terrain::{MovementCost, Terrain};
use crate::game::ingame::unit::UnitFilter;
use crate::game::ingame::unit_status::UnitStatus;
use crate::game::states::in_game_state::InGameState;
use crate::game::states::round_state::RoundState;

pub struct MoveUnitsPlugin;

impl Plugin for MoveUnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbstractMoveUnitsPlugin::<Time>::default());
    }
}

#[derive(Default)]
struct AbstractMoveUnitsPlugin<Time: TimeInterface> {
    phantom: PhantomData<Time>,
}

impl<Time: TimeInterface> Plugin for AbstractMoveUnitsPlugin<Time> {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveUnitEvent>()
            .init_resource::<MovingUnitsResource>()
            .add_systems(
                Update,
                move_unit_system::<Time>.run_if(in_state(RoundState::MovingUnit)),
            )
            .add_systems(
                PostUpdate,
                check_whether_movement_has_ended.run_if(in_state(RoundState::MovingUnit)),
            )
            .add_systems(
                PostUpdate,
                handle_move_event.run_if(in_state(InGameState::Playing)),
            );
    }
}

trait TimeInterface: Resource {
    fn delta(&self) -> Duration;
}

impl TimeInterface for Time {
    fn delta(&self) -> Duration {
        self.delta()
    }
}

#[derive(Event, Debug)]
pub struct MoveUnitEvent {
    pub(crate) entity: Entity,
    pub(crate) path: Vec<Hex>,
}

#[derive(Resource, Debug, Default)]
struct MovingUnitsResource(Vec<MovingUnit>);

#[derive(Debug)]
struct MovingUnit {
    entity: Entity,
    path: VecDeque<Hex>,
}

fn handle_move_event(
    mut move_events: EventReader<MoveUnitEvent>,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
    mut units: Query<&mut ActionPoints, UnitFilter>,
    mut moving_unit_resource: ResMut<MovingUnitsResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    for move_event in move_events.iter() {
        let cost: usize = move_event
            .path
            .iter()
            .map(|hex| {
                hexes
                    .iter()
                    .find(|(hex_component, _)| &hex_component.0 == hex)
                    .expect("A hex on the path must exist on the map")
            })
            .map(|(_, terrain)| &terrain.movement_cost)
            .map(|movement_cost| match movement_cost {
                MovementCost::Impassable => {
                    unreachable!("An impassable tile must not be on the way")
                }
                MovementCost::Passable(tile_cost) => tile_cost,
            })
            .sum();

        let mut action_points = units
            .get_mut(move_event.entity)
            .expect("The moving entity must exist");

        let action_points_left_before = action_points.left;
        action_points.left = if cost > action_points.left {
            0
        } else {
            action_points.left - cost
        };
        trace!(
            "Updated action points left from {action_points_left_before} to {}",
            action_points.left
        );

        moving_unit_resource.0.push(MovingUnit {
            entity: move_event.entity,
            path: move_event.path.clone().into(),
        });
        round_state.set(RoundState::MovingUnit);
        info!("Accepted move event for {:?}", moving_unit_resource.0);
    }
}

#[derive(Resource, Debug)]
struct MovementTimer(Timer);

const MOVE_TICK_SECONDS: f32 = 0.3;

impl Default for MovementTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(MOVE_TICK_SECONDS, TimerMode::Repeating))
    }
}

fn move_unit_system<Time: TimeInterface>(
    time: Res<Time>,
    mut timer: Local<MovementTimer>,
    mut moving_unit_resource: ResMut<MovingUnitsResource>,
    mut units: Query<(&mut HexComponent, &UnitStatus), UnitFilter>,
    mut combat_event: EventWriter<CombatEvent>,
) {
    if !timer.0.tick(time.delta()).finished() {
        trace!("Movement timer not finished yet: {timer:?}");
        return;
    }

    for moving_unit in &mut moving_unit_resource.0 {
        trace!("Moving unit: {moving_unit:?}");

        let (mut hex_component, unit_status) = units
            .get_mut(moving_unit.entity)
            .expect("The moving unit must exist");

        hex_component.0 = moving_unit
            .path
            .pop_front()
            .expect("If the path is empty, the MovingUnit should be removed");

        for unit_engaged_with in unit_status.get_engaged_with_units() {
            info!(
                "{:?} disengages from {unit_engaged_with:?} triggering attack",
                moving_unit.entity
            );
            combat_event.send(CombatEvent {
                attacker: *unit_engaged_with,
                defender: moving_unit.entity,
            });
        }
    }
}

fn check_whether_movement_has_ended(
    mut moving_unit_resource: ResMut<MovingUnitsResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    let length_before = moving_unit_resource.0.len();
    moving_unit_resource
        .0
        .retain(|moving_unit| !moving_unit.path.is_empty());
    let length_after = moving_unit_resource.0.len();

    let unit_finished_moving_count = length_before - length_after;
    if unit_finished_moving_count > 0 || length_after == 0 {
        info!("Movement ended for {unit_finished_moving_count} units - ending movement phase");
        round_state.set(RoundState::Input);
    }
}

#[cfg(test)]
mod tests {
    use bevy::log::{Level, LogPlugin};
    use bevy::prelude::{default, Handle};
    use bevy::time::TimePlugin;

    use crate::game::ingame::combat::{CombatConfig, HealthPoints};
    use crate::game::ingame::selected_unit::SelectedUnitResource;
    use crate::game::ingame::team_setup::Team;
    use crate::game::ingame::unit::{ProtoUnitBundle, UnitBundle, UnitMarker};
    use crate::game::ingame::unit_status::update_engagement;
    use crate::generate_test_app;
    use crate::tests::AppWrapper;

    use super::*;

    #[derive(Resource, Default, Debug)]
    struct MockTime {
        delta: Duration,
    }

    impl TimeInterface for MockTime {
        fn delta(&self) -> Duration {
            self.delta
        }
    }

    #[test]
    fn move_a_unit() {
        let mut app = TestApp::build_spawning_unit_at(Hex::ZERO);

        app.send_event(MoveUnitEvent {
            entity: app.unit_entity,
            path: vec![Hex::new(1, 0), Hex::new(2, 0), Hex::new(2, 1)],
        });

        app.update();
        app.update();
        assert_eq!(app.get::<RoundState>(), &RoundState::MovingUnit);

        app.update();
        assert_eq!(app.get_unit_hex(), Hex::new(0, 0));

        app.set_passed_time(Duration::from_secs_f32(MOVE_TICK_SECONDS));
        app.update();
        assert_eq!(app.get_unit_hex(), Hex::new(1, 0));

        app.update();
        assert_eq!(app.get_unit_hex(), Hex::new(2, 0));

        app.update();
        assert_eq!(app.get_unit_hex(), Hex::new(2, 1));

        app.update();
        assert_eq!(app.get_unit_action_points().left, MAX_ACTION_POINTS - 3);
        assert_eq!(app.get::<RoundState>(), &RoundState::Input);
    }

    #[test]
    fn moving_a_unit_triggers_enemy_attacks_on_the_way() {
        let mut app = TestApp::build_spawning_unit_at(Hex::ZERO);
        let mut combat_event_reader = app.get_event_reader::<CombatEvent>();

        let enemy_entity = app.spawn_enemy_unit_at(Hex::new(1, 1));

        app.send_event(MoveUnitEvent {
            entity: app.unit_entity,
            /*
             * . . . 3
             *  . . 2 .
             * . 0 1 .
             *  . . E .
             *
             * 1 should trigger the attack, E is the enemy
             */
            path: vec![Hex::new(1, 0), Hex::new(2, -1), Hex::new(3, -2)],
        });

        app.update();
        app.update();
        assert_eq!(app.get::<RoundState>(), &RoundState::MovingUnit);

        app.set_passed_time(Duration::from_secs_f32(MOVE_TICK_SECONDS));
        app.update();
        assert_eq!(app.get_events(&mut combat_event_reader).len(), 0);

        app.update();
        let combat_events = app.get_events(&mut combat_event_reader);
        let combat_event = combat_events
            .iter()
            .next()
            .expect("combat event in second movement step");
        assert_eq!(combat_event.defender, app.unit_entity);
        assert_eq!(combat_event.attacker, enemy_entity);

        app.update();
        assert_eq!(app.get_events(&mut combat_event_reader).len(), 0);
    }

    generate_test_app!(unit_entity: Entity);

    const MAX_ACTION_POINTS: usize = 10;

    impl TestApp {
        fn build_spawning_unit_at(hex: Hex) -> TestApp {
            let mut app = App::new();

            app.add_state::<RoundState>();
            app.add_state::<InGameState>();
            app.world
                .resource_mut::<NextState<InGameState>>()
                .set(InGameState::Playing);

            app.add_event::<CombatEvent>();

            app.init_resource::<MockTime>();

            app.add_plugins((
                AbstractMoveUnitsPlugin::<MockTime>::default(),
                TimePlugin,
                LogPlugin {
                    level: Level::TRACE,
                    ..default()
                },
            ));

            Hex::ZERO.spiral_range(0..=5).for_each(|hex| {
                app.world.spawn((
                    HexMarker,
                    HexComponent(hex),
                    Terrain {
                        name: "test terrain".to_string(),
                        movement_cost: MovementCost::Passable(1),
                    },
                ));
            });

            let unit_entity = app
                .world
                .spawn::<UnitBundle>(
                    ProtoUnitBundle {
                        texture: Handle::default(),
                        transform: Default::default(),
                        unit_marker: UnitMarker("test unit".to_string()),
                        player: Team::Red,
                        action_points: ActionPoints::new(MAX_ACTION_POINTS, 1, 1),
                        health_points: HealthPoints::new(5),
                        combat_config: CombatConfig {
                            attack: 1,
                            defense: 1,
                            range: 1,
                        },
                        hex,
                    }
                    .into(),
                )
                .id();

            let mut selected_unit_resource = SelectedUnitResource::default();
            selected_unit_resource.set_selected_unit(Some(unit_entity));
            app.insert_resource(selected_unit_resource);

            app.add_systems(
                PostUpdate,
                update_engagement.run_if(in_state(InGameState::Playing)),
            );

            app.update();

            TestApp { app, unit_entity }
        }

        fn spawn_enemy_unit_at(&mut self, hex: Hex) -> Entity {
            self.app
                .world
                .spawn::<UnitBundle>(
                    ProtoUnitBundle {
                        texture: Handle::default(),
                        transform: Default::default(),
                        unit_marker: UnitMarker("enemy unit".to_string()),
                        player: Team::Blue,
                        action_points: ActionPoints::new(MAX_ACTION_POINTS, 1, 1),
                        health_points: HealthPoints::new(5),
                        combat_config: CombatConfig {
                            attack: 1,
                            defense: 1,
                            range: 1,
                        },
                        hex,
                    }
                    .into(),
                )
                .id()
        }

        fn set_passed_time(&mut self, passed_time: Duration) {
            self.app.world.resource_mut::<MockTime>().delta = passed_time;
        }

        fn get_unit_hex(&self) -> Hex {
            self.app
                .world
                .entity(self.unit_entity)
                .get::<HexComponent>()
                .unwrap()
                .0
        }

        fn get_unit_action_points(&self) -> &ActionPoints {
            self.app
                .world
                .entity(self.unit_entity)
                .get::<ActionPoints>()
                .unwrap()
        }
    }
}
