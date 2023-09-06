use std::cmp::max;

use bevy::app::App;
use bevy::prelude::{
    debug, in_state, info, Changed, Commands, Component, DespawnRecursiveExt, Entity, Event,
    EventReader, EventWriter, IntoSystemConfigs, Plugin, PostUpdate, Query,
};

use crate::game::common_components::UnitMarker;
use crate::game::game_log::LogEvent;
use crate::game::states::in_game_state::InGameState;
use crate::game::util::dice::Dice;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (handle_combat, despawn_dead_units).run_if(in_state(InGameState::Playing)),
        )
        .add_event::<CombatEvent>();
    }
}

#[derive(Component, Debug)]
pub struct HealthPoints {
    max: usize,
    pub left: usize,
}

impl HealthPoints {
    pub fn new(max: usize) -> Self {
        Self { max, left: max }
    }

    pub fn get_max(&self) -> usize {
        self.max
    }
}

#[derive(Component, Debug)]
pub struct CombatConfig {
    /// damage to health points if defense fails
    pub attack: usize,
    /// Chance to defend in a D20 dice roll
    pub defense: usize,
    pub range: u32,
}

#[derive(Event, Debug)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub defender: Entity,
}

fn handle_combat(
    mut units: Query<(&CombatConfig, &mut HealthPoints, &UnitMarker)>,
    mut combat_events: EventReader<CombatEvent>,
    mut log_event: EventWriter<LogEvent>,
) {
    for combat_event in combat_events.iter() {
        handle_combat_event(&mut units, &mut log_event, combat_event);
    }
}

fn handle_combat_event(
    units: &mut Query<(&CombatConfig, &mut HealthPoints, &UnitMarker)>,
    log_event: &mut EventWriter<LogEvent>,
    combat_event: &CombatEvent,
) {
    let (attacker_config, _, attacker_unit) = units.get_mut(combat_event.attacker).unwrap();

    let attack_points = attacker_config.attack;
    let attacker_name = attacker_unit.0.clone();

    let (defender_config, mut defender_health_points, defender_unit) =
        units.get_mut(combat_event.defender).unwrap();

    let dice_roll = Dice::<20>::roll();

    let defender_name = &defender_unit.0;

    let defense = defender_config.defense;
    if (dice_roll as usize) >= defense {
        debug!(
            "Successful combat dice roll: {dice_roll} against {}",
            defender_config.defense
        );
        defender_health_points.left = max(defender_health_points.left - attack_points, 0);

        log_event.send(LogEvent {
            message: format!(
                "{attacker_name} caused {attack_points} damage to {defender_name} ({dice_roll}/{defense})",
            ),
        });
    } else {
        log_event.send(LogEvent {
            message: format!(
                "{attacker_name} has failed to cause significant damage to {defender_name} ({dice_roll}/{defense})",
            ),
        });
    }
}

fn despawn_dead_units(
    mut commands: Commands,
    units: Query<(Entity, &HealthPoints), Changed<HealthPoints>>,
) {
    for (entity, health_points) in units.iter() {
        if health_points.left == 0 {
            info!("Despawning {entity:?}, because health points are 0");
            commands.entity(entity).despawn_recursive();
        }
    }
}
