use std::collections::HashSet;

use crate::game::ingame::action_points::ActionPoints;
use bevy::ecs::system::SystemId;
use bevy::log::warn;
use bevy::prelude::{Component, Entity, EventWriter, FromWorld, In, Parent, Query, With, World};
use bevy::utils::HashMap;
use enum_iterator::{all, Sequence};
use hexx::Hex;

use crate::game::ingame::combat::{Attack, AttackOrDefault, CombatEvent};
use crate::game::ingame::game_log::LogEvent;
use crate::game::ingame::hex::{HexComponent, HexMarker};
use crate::game::ingame::selected_unit::UpdateReachableHexesUnitsQuery;
use crate::game::ingame::terrain::Terrain;
use crate::game::ingame::unit::UnitFilter;
use crate::game::util::find_units_within_range::FindUnitsWithinRange;

#[derive(Component, Debug, Clone)]
pub struct ActivatedAbilityMarker;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Hash, PartialEq, Eq, Sequence)]
pub enum ActiveAbilityType {
    ThrowJavelin,
}

impl ActiveAbilityType {
    fn get_ability(&self, world: &mut World) -> ActiveAbility {
        match self {
            ActiveAbilityType::ThrowJavelin => ActiveAbility::ThrowJavelin {
                throw_javelin_system: world.register_system(throw_javelin_system),
                usages_left: 1,
            },
        }
    }
}

pub struct ActiveAbilityRegistry(HashMap<ActiveAbilityType, ActiveAbility>);

impl ActiveAbilityRegistry {
    pub fn get_registered_ability(&self, ability: &ActiveAbilityType) -> ActiveAbility {
        self.0[ability].clone()
    }
}

impl FromWorld for ActiveAbilityRegistry {
    fn from_world(world: &mut World) -> Self {
        let abilities = all::<ActiveAbilityType>()
            .map(|ability_type| {
                let ability = ability_type.get_ability(world);
                (ability_type, ability)
            })
            .collect();

        Self(abilities)
    }
}

#[derive(Component, Debug, Clone)]
pub enum ActiveAbility {
    ThrowJavelin {
        throw_javelin_system: SystemId<ThrowJavelinInput>,
        usages_left: u8,
    },
}

impl ActiveAbility {
    pub fn get_display_name(&self) -> String {
        match self {
            ActiveAbility::ThrowJavelin { .. } => "Throw Javelin".to_string(),
        }
    }

    pub fn get_reachable_hexes(
        &self,
        units: &UpdateReachableHexesUnitsQuery,
        _: &Query<(&HexComponent, &Terrain), With<HexMarker>>,
        parent: &Parent,
    ) -> Option<HashSet<Hex>> {
        match self {
            ActiveAbility::ThrowJavelin { .. } => {
                let Ok((_, selected_unit_hex, selected_unit_team, _, _)) = units.get(**parent)
                else {
                    warn!("Units query did not contain parent of activated ability {parent:?}");
                    return None;
                };

                Some(
                    units.find_units_within_range(selected_unit_hex.0, 2, |team| {
                        team != selected_unit_team
                    }),
                )
            }
        }
    }

    pub fn can_be_used(&self, action_points: &ActionPoints) -> bool {
        match self {
            ActiveAbility::ThrowJavelin { usages_left, .. } => {
                usages_left > &0 && action_points.can_still_attack_this_turn()
            }
        }
    }
}

pub struct ThrowJavelinInput {
    pub attacker: Entity,
    pub target_hex: Hex,
}

fn throw_javelin_system(
    input: In<ThrowJavelinInput>,
    units: Query<(Entity, &HexComponent), UnitFilter>,
    mut combat_event: EventWriter<CombatEvent>,
    mut log_event: EventWriter<LogEvent>,
) {
    let Some((defender, _)) = units.iter().find(|(_, hex)| hex.0 == input.target_hex) else {
        return;
    };
    log_event.send(LogEvent {
        message: format!("{:?} throws a javelin at {defender:?}", input.attacker),
    });
    combat_event.send(CombatEvent {
        attack: AttackOrDefault::Attack(Attack {
            damage: 1,
            passive_combat_abilities: vec![],
            range: 2,
        }),
        attacker: input.attacker,
        defender,
    });
}
