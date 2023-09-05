use std::cmp::max;

use bevy::prelude::{
    debug, info, warn, Changed, Commands, Component, DespawnRecursiveExt, Entity, EventWriter,
    NextState, Query, ResMut, Resource,
};

use crate::game::action_points::ActionPoints;
use crate::game::common_components::UnitMarker;
use crate::game::game_log::LogEvent;
use crate::game::states::round_state::RoundState;
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
}

#[derive(Resource, Debug, Default)]
pub enum CombatantsResource {
    #[default]
    NoCombat,
    Combatants {
        attacker: Entity,
        defender: Entity,
    },
}

pub fn handle_combat(
    mut next_round_state: ResMut<NextState<RoundState>>,
    mut units: Query<(
        &CombatConfig,
        &mut HealthPoints,
        &mut ActionPoints,
        &UnitMarker,
    )>,
    mut combatants_resource: ResMut<CombatantsResource>,
    mut log_event: EventWriter<LogEvent>,
) {
    next_round_state.set(RoundState::Moving);

    let CombatantsResource::Combatants { attacker, defender } = *combatants_resource else {
        warn!(
            "Expected combatants, found {:?} - skipping combat",
            *combatants_resource
        );
        return;
    };

    let (attacker_config, _, mut action_points, attacker_unit) = units.get_mut(attacker).unwrap();
    if !action_points.can_still_attack_this_turn() {
        debug!(
            "Skipping attack, because not possible this turn: {:?}",
            *action_points
        );
        *combatants_resource = CombatantsResource::NoCombat;
        return;
    }
    action_points.left -= action_points.attack_action_point_cost();
    action_points.attacks_this_round += 1;
    let attack_points = attacker_config.attack;
    let attacker_name = attacker_unit.0.clone();

    let (defender_config, mut defender_health_points, _, defender_unit) =
        units.get_mut(defender).unwrap();

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

    *combatants_resource = CombatantsResource::NoCombat;
}

pub fn despawn_dead_units(
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
