use bevy::prelude::{
    debug, info, Changed, Commands, Component, DespawnRecursiveExt, Entity, Event, EventReader,
    EventWriter, Query,
};

use crate::game::ingame::game_log::LogEvent;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::unit_status::UnitStatus;
use crate::game::util::dice::Dice;

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

#[derive(Event, Debug, Clone)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub defender: Entity,
}

pub(super) fn handle_combat(
    mut units: Query<(&CombatConfig, &mut HealthPoints, &UnitMarker)>,
    mut combat_events: EventReader<CombatEvent>,
    mut log_event: EventWriter<LogEvent>,
) {
    for combat_event in combat_events.read() {
        handle_combat_event(&mut units, &mut log_event, combat_event);
    }
}

pub(super) fn handle_combat_event(
    units: &mut Query<(&CombatConfig, &mut HealthPoints, &UnitMarker)>,
    log_event: &mut EventWriter<LogEvent>,
    combat_event: &CombatEvent,
) {
    let Ok((attacker_config, _, attacker_unit)) = units.get_mut(combat_event.attacker) else {
        return;
    };

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
        defender_health_points.left = if attack_points >= defender_health_points.left {
            0
        } else {
            defender_health_points.left - attack_points
        };

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

pub(super) fn despawn_dead_units(
    mut commands: Commands,
    mut units: Query<(Entity, &HealthPoints, &mut UnitStatus), Changed<HealthPoints>>,
) {
    let despawned_entities: Vec<_> = units
        .iter()
        .filter(|(_, health_points, _)| health_points.left == 0)
        .map(|(entity, _, _)| {
            info!("Despawning {entity:?}, because health points are 0");
            commands.entity(entity).despawn_recursive();
            entity
        })
        .collect();

    for (_, _, mut unit_status) in &mut units {
        for despawned_entity in &despawned_entities {
            if unit_status.is_engaged_with(despawned_entity) {
                unit_status.disengage_with(despawned_entity);
            }
        }
    }
}
