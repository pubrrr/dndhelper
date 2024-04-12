use bevy::prelude::{Component, FromWorld, World};
use bevy::utils::HashMap;
use enum_iterator::{all, Sequence};

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
}
