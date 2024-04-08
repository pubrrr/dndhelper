use bevy::app::PostUpdate;
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    debug, in_state, info, not, App, Changed, Commands, Component, Condition, DespawnRecursiveExt,
    Entity, Event, EventReader, EventWriter, FromWorld, IntoSystemConfigs, NextState, Plugin,
    Query, Res, ResMut, Resource, Update, World,
};
use bevy::utils::HashMap;

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
    pub attack: usize,
    /// Chance to defend in a D20 dice roll
    pub defense: usize,
    pub range: u32,
    pub passive_combat_abilities: Vec<RegisteredPassiveCombatAbility>,
}

#[derive(Debug)]
pub struct RegisteredPassiveCombatAbility {
    ability: PassiveCombatAbility,
    system_id: SystemId,
    combat_phase: CombatPhase,
    combat_trigger: CombatTrigger,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CombatPhase {
    PreCombat,
    PostCombat,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CombatTrigger {
    OnAttack,
    OnDefense,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum PassiveCombatAbility {
    ArmorBreak,
}

pub struct PassiveCombatAbilitySystemIds(HashMap<PassiveCombatAbility, SystemId>);

impl FromWorld for PassiveCombatAbilitySystemIds {
    fn from_world(world: &mut World) -> Self {
        let mut system_ids = HashMap::default();

        system_ids.insert(
            PassiveCombatAbility::ArmorBreak,
            world.register_system(armor_break_system),
        );

        PassiveCombatAbilitySystemIds(system_ids)
    }
}

impl PassiveCombatAbilitySystemIds {
    pub fn get_registered_ability(
        &self,
        ability: PassiveCombatAbility,
    ) -> RegisteredPassiveCombatAbility {
        let system_id = self.0[&ability];

        let (combat_phase, combat_trigger) = match ability {
            PassiveCombatAbility::ArmorBreak => (CombatPhase::PostCombat, CombatTrigger::OnAttack),
        };

        RegisteredPassiveCombatAbility {
            ability,
            system_id,
            combat_phase,
            combat_trigger,
        }
    }
}

fn armor_break_system(
    mut units: Query<(&mut CombatConfig, &UnitMarker)>,
    combat_resource: Res<CombatResource>,
    mut log_event: EventWriter<LogEvent>,
) {
    if combat_resource.combat_result == CombatResult::Miss {
        if let Ok((mut combat_config, unit_marker)) = units.get_mut(combat_resource.defender) {
            log_event.send(LogEvent {
                message: format!(
                    "{} has lost 1 defense point due to armor break",
                    unit_marker.0
                ),
            });
            combat_config.defense -= 1;
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub defender: Entity,
}

#[derive(Resource, Debug)]
pub struct CombatResource {
    pub attacker: Entity,
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
) {
    for combat_event in combat_events.read() {
        round_state.set(RoundState::PreCombat);
        commands.insert_resource(CombatResource {
            attacker: combat_event.attacker,
            defender: combat_event.defender,
            combat_result: CombatResult::None,
        });
    }
}

fn handle_pre_combat(
    mut commands: Commands,
    units: Query<(&CombatConfig, &UnitMarker)>,
    mut log_event: EventWriter<LogEvent>,
    combat_resource: Res<CombatResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    debug!("Handling pre combat");

    if let Ok((defender_config, unit_marker)) = units.get(combat_resource.defender) {
        filter_and_run_abilities(
            &mut commands,
            defender_config,
            unit_marker,
            &mut log_event,
            CombatPhase::PreCombat,
            CombatTrigger::OnDefense,
        );
    }

    if let Ok((attacker_config, unit_marker)) = units.get(combat_resource.attacker) {
        filter_and_run_abilities(
            &mut commands,
            attacker_config,
            unit_marker,
            &mut log_event,
            CombatPhase::PreCombat,
            CombatTrigger::OnAttack,
        );
    }

    round_state.set(RoundState::Combat);
}

fn filter_and_run_abilities(
    commands: &mut Commands,
    combat_config: &CombatConfig,
    unit_marker: &UnitMarker,
    log_event: &mut EventWriter<LogEvent>,
    combat_phase: CombatPhase,
    combat_trigger: CombatTrigger,
) {
    let unit_name = &unit_marker.0;

    combat_config
        .passive_combat_abilities
        .iter()
        .filter(|ability| ability.combat_phase == combat_phase)
        .filter(|ability| ability.combat_trigger == combat_trigger)
        .for_each(|ability| {
            let ability_type = &ability.ability;
            log_event.send(LogEvent {
                message: format!("{unit_name} trying {ability_type:?}",),
            });
            commands.run_system(ability.system_id);
        });
}

fn handle_combat(
    mut units: Query<(&CombatConfig, &mut HealthPoints, &UnitMarker)>,
    mut log_event: EventWriter<LogEvent>,
    mut combat_resource: ResMut<CombatResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    let Ok((attacker_config, _, attacker_unit)) = units.get_mut(combat_resource.attacker) else {
        return;
    };

    let attack_points = attacker_config.attack;
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
    mut log_event: EventWriter<LogEvent>,
    combat_resource: Res<CombatResource>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    debug!("Handling post combat");

    if let Ok((defender_config, unit_marker)) = units.get(combat_resource.defender) {
        filter_and_run_abilities(
            &mut commands,
            defender_config,
            unit_marker,
            &mut log_event,
            CombatPhase::PostCombat,
            CombatTrigger::OnDefense,
        );
    }

    if let Ok((attacker_config, unit_marker)) = units.get(combat_resource.attacker) {
        filter_and_run_abilities(
            &mut commands,
            attacker_config,
            unit_marker,
            &mut log_event,
            CombatPhase::PostCombat,
            CombatTrigger::OnAttack,
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
