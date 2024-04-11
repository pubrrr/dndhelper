use bevy::ecs::system::SystemId;
use bevy::prelude::{EventWriter, FromWorld, Query, Res, World};
use bevy::utils::HashMap;
use enum_iterator::{all, Sequence};

use game_log::LogEvent;

use crate::game::ingame::combat::{
    CombatConfig, CombatPhase, CombatResource, CombatResult, CombatTrigger,
};
use crate::game::ingame::game_log;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::unit_status::UnitStatus;

#[derive(Debug)]
pub struct RegisteredPassiveCombatAbility {
    pub ability: PassiveCombatAbility,
    pub system_id: SystemId,
    pub combat_phase: CombatPhase,
    pub combat_trigger: CombatTrigger,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Hash, PartialEq, Eq, Sequence)]
pub enum PassiveCombatAbility {
    ArmorBreak,
    HitAndRun,
}

pub struct PassiveCombatAbilitySystemIds(HashMap<PassiveCombatAbility, SystemId>);

impl FromWorld for PassiveCombatAbilitySystemIds {
    fn from_world(world: &mut World) -> Self {
        let mut system_ids = HashMap::default();

        for ability in all::<PassiveCombatAbility>() {
            let system_id = match &ability {
                PassiveCombatAbility::ArmorBreak => world.register_system(armor_break_action),
                PassiveCombatAbility::HitAndRun => world.register_system(hit_and_run_action),
            };
            system_ids.insert(ability, system_id);
        }

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
            PassiveCombatAbility::HitAndRun => (CombatPhase::PostCombat, CombatTrigger::OnAttack),
        };

        RegisteredPassiveCombatAbility {
            ability,
            system_id,
            combat_phase,
            combat_trigger,
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
