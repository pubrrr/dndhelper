use bevy::app::PostUpdate;
use bevy::prelude::{
    debug, in_state, info, not, App, Changed, Commands, Component, Condition, DespawnRecursiveExt,
    Entity, Event, EventReader, EventWriter, IntoSystemConfigs, NextState, Plugin, Query, Res,
    ResMut, Resource, Update,
};

use crate::game::abilities::passive_combat_abilities::{
    AbilityTrigger, CombatPhase, RegisteredPassiveCombatAbility,
};
use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::game_log::LogEvent;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::unit_status::UnitStatus;
use crate::game::states::in_game_state::InGameState;
use crate::game::states::round_state::RoundState;
use crate::game::util::dice::Dice;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>()
            .add_systems(
                PostUpdate,
                (
                    handle_combat_event,
                    despawn_dead_units.run_if(
                        not(in_state(RoundState::Combat))
                            .and_then(not(in_state(RoundState::PostCombat))),
                    ),
                )
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    handle_pre_combat.run_if(in_state(RoundState::PreCombat)),
                    handle_combat.run_if(in_state(RoundState::Combat)),
                    handle_post_combat.run_if(in_state(RoundState::PostCombat)),
                )
                    .run_if(in_state(InGameState::Playing)),
            );
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
    pub damage: usize,
    /// Chance to defend in a D20 dice roll
    pub defense: usize,
    pub range: u32,
    pub passive_combat_abilities: Vec<RegisteredPassiveCombatAbility>,
}

impl CombatConfig {
    pub fn get_default_attack(&self) -> Attack {
        Attack {
            damage: self.damage,
            range: self.range,
            passive_combat_abilities: self.passive_combat_abilities.clone(),
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub attack: AttackOrDefault,
    pub defender: Entity,
}

#[derive(Debug, Clone)]
pub enum AttackOrDefault {
    Attack(Attack),
    Default,
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub damage: usize,
    pub range: u32,
    pub passive_combat_abilities: Vec<RegisteredPassiveCombatAbility>,
}

#[derive(Resource, Debug)]
pub struct CombatResource {
    pub attacker: Entity,
    pub attack: Attack,
    pub defender: Entity,
    pub combat_result: CombatResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatResult {
    Hit,
    Miss,
    None,
}

fn handle_combat_event(
    mut round_state: ResMut<NextState<RoundState>>,
    mut commands: Commands,
    mut combat_events: EventReader<CombatEvent>,
    units: Query<&CombatConfig>,
) {
    for combat_event in combat_events.read() {
        let attack = match &combat_event.attack {
            AttackOrDefault::Attack(attack) => attack.clone(),
            AttackOrDefault::Default => match units.get(combat_event.attacker) {
                Ok(combat_config) => combat_config.get_default_attack(),
                Err(_) => continue,
            },
        };
        round_state.set(RoundState::PreCombat);
        commands.insert_resource(CombatResource {
            attacker: combat_event.attacker,
            attack,
            defender: combat_event.defender,
            combat_result: CombatResult::None,
        });
    }
}

fn handle_pre_combat(
    mut commands: Commands,
    mut units: Query<(&CombatConfig, &UnitMarker, &mut ActionPoints)>,
    combat_resource: Res<CombatResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    debug!("Handling pre combat");

    if let Ok((defender_config, unit_marker, _)) = units.get(combat_resource.defender) {
        filter_and_run_abilities(
            &mut commands,
            &defender_config.passive_combat_abilities,
            unit_marker,
            AbilityTrigger::OnDefense(CombatPhase::PreCombat),
        );
    }

    if let Ok((_, unit_marker, mut action_points)) = units.get_mut(combat_resource.attacker) {
        action_points.left -= action_points.attack_action_point_cost();
        action_points.attacks_this_round += 1;
        filter_and_run_abilities(
            &mut commands,
            &combat_resource.attack.passive_combat_abilities,
            unit_marker,
            AbilityTrigger::OnAttack(CombatPhase::PreCombat),
        );
    }

    round_state.set(RoundState::Combat);
}

fn filter_and_run_abilities(
    commands: &mut Commands,
    passive_combat_abilities: &[RegisteredPassiveCombatAbility],
    unit_marker: &UnitMarker,
    ability_trigger: AbilityTrigger,
) {
    let unit_name = &unit_marker.0;

    passive_combat_abilities
        .iter()
        .filter(|ability| ability.ability_trigger == ability_trigger)
        .for_each(|ability| {
            debug!("{unit_name} trying {:?}", ability.ability);
            commands.run_system(ability.system_id);
        });
}

fn handle_combat(
    mut units: Query<(&CombatConfig, &mut HealthPoints, &UnitMarker)>,
    mut log_event: EventWriter<LogEvent>,
    mut combat_resource: ResMut<CombatResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    let Ok((_, _, attacker_unit)) = units.get_mut(combat_resource.attacker) else {
        return;
    };

    let damage = combat_resource.attack.damage;
    let attacker_name = attacker_unit.0.clone();

    let (defender_config, mut defender_health_points, defender_unit) =
        units.get_mut(combat_resource.defender).unwrap();

    let dice_roll = Dice::<20>::roll();

    let defender_name = &defender_unit.0;

    let defense = defender_config.defense;
    if (dice_roll as usize) >= defense {
        combat_resource.combat_result = CombatResult::Hit;
        debug!(
            "Successful combat dice roll: {dice_roll} against {}",
            defender_config.defense
        );
        defender_health_points.left = if damage >= defender_health_points.left {
            0
        } else {
            defender_health_points.left - damage
        };

        log_event.send(LogEvent {
            message: format!(
                "{attacker_name} caused {damage} damage to {defender_name} ({dice_roll}/{defense})",
            ),
        });
    } else {
        combat_resource.combat_result = CombatResult::Miss;
        log_event.send(LogEvent {
            message: format!(
                "{attacker_name} has failed to cause significant damage to {defender_name} ({dice_roll}/{defense})",
            ),
        });
    }

    round_state.set(RoundState::PostCombat);
}

fn handle_post_combat(
    mut commands: Commands,
    units: Query<(&CombatConfig, &UnitMarker)>,
    combat_resource: Res<CombatResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    debug!("Handling post combat");

    if let Ok((defender_config, unit_marker)) = units.get(combat_resource.defender) {
        filter_and_run_abilities(
            &mut commands,
            &defender_config.passive_combat_abilities,
            unit_marker,
            AbilityTrigger::OnDefense(CombatPhase::PostCombat),
        );
    }

    if let Ok((_, unit_marker)) = units.get(combat_resource.attacker) {
        filter_and_run_abilities(
            &mut commands,
            &combat_resource.attack.passive_combat_abilities,
            unit_marker,
            AbilityTrigger::OnAttack(CombatPhase::PostCombat),
        );
    }

    round_state.set(RoundState::Input);
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
