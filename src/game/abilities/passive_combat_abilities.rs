use bevy::ecs::system::SystemId;
use bevy::prelude::{EventWriter, FromWorld, Query, Res, World};
use bevy::utils::HashMap;
use enum_iterator::{all, Sequence};

use game_log::LogEvent;

use crate::game::ingame::combat::{CombatConfig, CombatResource, CombatResult};
use crate::game::ingame::game_log;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::unit_status::UnitStatus;

#[derive(Debug, Clone)]
pub struct RegisteredPassiveCombatAbility {
    pub ability: PassiveCombatAbility,
    pub system_id: SystemId,
    pub ability_trigger: AbilityTrigger,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Hash, PartialEq, Eq, Sequence)]
pub enum PassiveCombatAbility {
    ArmorBreak,
    HitAndRun,
}

impl PassiveCombatAbility {
    pub fn register_system(&self, world: &mut World) -> SystemId {
        match self {
            PassiveCombatAbility::ArmorBreak => world.register_system(armor_break_action),
            PassiveCombatAbility::HitAndRun => world.register_system(hit_and_run_action),
        }
    }

    pub fn get_trigger(&self) -> AbilityTrigger {
        match self {
            PassiveCombatAbility::ArmorBreak => AbilityTrigger::OnAttack(CombatPhase::PostCombat),
            PassiveCombatAbility::HitAndRun => AbilityTrigger::OnAttack(CombatPhase::PostCombat),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AbilityTrigger {
    OnAttack(CombatPhase),
    OnDefense(CombatPhase),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CombatPhase {
    PreCombat,
    PostCombat,
}

pub struct PassiveCombatAbilityRegistry(HashMap<PassiveCombatAbility, SystemId>);

impl FromWorld for PassiveCombatAbilityRegistry {
    fn from_world(world: &mut World) -> Self {
        let system_ids_map = all::<PassiveCombatAbility>()
            .map(|ability| {
                let system_id = ability.register_system(world);
                (ability, system_id)
            })
            .collect();

        PassiveCombatAbilityRegistry(system_ids_map)
    }
}

impl PassiveCombatAbilityRegistry {
    pub fn get_registered_ability(
        &self,
        ability: PassiveCombatAbility,
    ) -> RegisteredPassiveCombatAbility {
        let system_id = self.0[&ability];

        let ability_trigger = ability.get_trigger();

        RegisteredPassiveCombatAbility {
            ability,
            system_id,
            ability_trigger,
        }
    }
}

fn armor_break_action(
    mut units: Query<(&mut CombatConfig, &UnitMarker)>,
    combat_resource: Res<CombatResource>,
    mut log_event: EventWriter<LogEvent>,
) {
    if combat_resource.combat_result != CombatResult::Miss {
        return;
    }

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

fn hit_and_run_action(
    mut units: Query<(&mut UnitStatus, &UnitMarker)>,
    combat_resource: Res<CombatResource>,
    mut log_event: EventWriter<LogEvent>,
) {
    if combat_resource.combat_result != CombatResult::Hit {
        return;
    }

    if let Ok((mut unit_status, unit_marker)) = units.get_mut(combat_resource.attacker) {
        log_event.send(LogEvent {
            message: format!(
                "{} {:?} disengaged due to Hit&Run",
                unit_marker.0, combat_resource.attacker
            ),
        });
        unit_status.disengage_with(&combat_resource.defender);
    }
}
