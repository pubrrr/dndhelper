use bevy::log::warn;
use std::collections::HashSet;

use bevy::prelude::{Component, FromWorld, Parent, Query, With, World};
use bevy::utils::HashMap;
use enum_iterator::{all, Sequence};
use hexx::Hex;

use crate::game::ingame::hex::{HexComponent, HexMarker};
use crate::game::ingame::selected_unit::UpdateReachableHexesUnitsQuery;
use crate::game::ingame::terrain::Terrain;
use crate::game::util::find_units_within_range::FindUnitsWithinRange;
#[derive(Component, Debug, Clone)]
pub struct ActivatedAbilityMarker;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Hash, PartialEq, Eq, Sequence)]
pub enum ActiveAbilityType {
    ThrowJavelin,
}

impl ActiveAbilityType {
    fn get_ability(&self) -> ActiveAbility {
        match self {
            ActiveAbilityType::ThrowJavelin => ActiveAbility::ThrowJavelin,
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
    fn from_world(_: &mut World) -> Self {
        let abilities = all::<ActiveAbilityType>()
            .map(|ability_type| {
                let ability = ability_type.get_ability();
                (ability_type, ability)
            })
            .collect();

        Self(abilities)
    }
}

#[derive(Component, Debug, Clone)]
pub enum ActiveAbility {
    ThrowJavelin,
}

impl ActiveAbility {
    pub fn get_display_name(&self) -> String {
        match self {
            ActiveAbility::ThrowJavelin => "Throw Javelin".to_string(),
        }
    }

    pub fn get_reachable_hexes(
        &self,
        units: &UpdateReachableHexesUnitsQuery,
        _: &Query<(&HexComponent, &Terrain), With<HexMarker>>,
        parent: &Parent,
    ) -> Option<HashSet<Hex>> {
        match self {
            ActiveAbility::ThrowJavelin => {
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
}
