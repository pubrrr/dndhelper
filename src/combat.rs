use std::cmp::max;

use crate::action_points::ActionPoints;
use bevy::prelude::{
    warn, Changed, Commands, Component, DespawnRecursiveExt, Entity, NextState, Query, ResMut,
    Resource,
};

use crate::game_state::RoundState;

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
pub struct AttackConfig {
    pub attack: usize,
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
    mut units: Query<(&AttackConfig, &mut HealthPoints, &mut ActionPoints)>,
    mut combatants_resource: ResMut<CombatantsResource>,
) {
    next_round_state.set(RoundState::Moving);

    let CombatantsResource::Combatants { attacker, defender } = *combatants_resource else {
        warn!(
            "Expected combatants, found {:?} - skipping combat",
            *combatants_resource
        );
        return;
    };

    let (attacker_config, _, mut action_points) = units.get_mut(attacker).unwrap();
    if action_points.left == 0 {
        *combatants_resource = CombatantsResource::NoCombat;
        return;
    }
    action_points.left -= 1;
    let attack_points = attacker_config.attack;
    let (_, mut defender_health_points, _) = units.get_mut(defender).unwrap();
    defender_health_points.left = max(defender_health_points.left - attack_points, 0);

    *combatants_resource = CombatantsResource::NoCombat;
}

pub fn despawn_dead_units(
    mut commands: Commands,
    units: Query<(Entity, &HealthPoints), Changed<HealthPoints>>,
) {
    for (entity, health_points) in units.iter() {
        if health_points.left == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
