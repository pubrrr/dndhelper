use bevy::asset::Error;
use bevy::prelude::{AssetServer, Handle, HandleUntyped, Image, Resource, States, World};
use bevy::reflect::{TypePath, TypeUuid};
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::{
    AssetCollection, DynamicAsset, DynamicAssetType, StandardDynamicAsset,
};

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct NationAssetCollection {
    #[asset(key = "unit_images", collection(typed, mapped))]
    pub unit_images: HashMap<String, Handle<Image>>,
    #[asset(key = "unit_stats_files", collection(typed, mapped))]
    pub unit_stats_files: HashMap<String, Handle<UnitStats>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnitKey {
    pub nation: String,
    pub name: String,
}

impl Into<String> for UnitKey {
    fn into(self) -> String {
        format!("{}:{}", self.nation, self.name)
    }
}

impl From<String> for UnitKey {
    fn from(value: String) -> Self {
        let parts: Vec<_> = value.split(':').collect();
        match parts.as_slice() {
            [nation, name] => Self {
                nation: nation.to_string(),
                name: name.to_string(),
            },
            _ => panic!(
                "Could not parse {value} as unit key. Did not match pattern '<nation>:<unit_name>'"
            ),
        }
    }
}

impl UnitKey {
    pub fn get_standard_assets(&self) -> Vec<StandardDynamicAsset> {
        vec![
            StandardDynamicAsset::File {
                path: self.get_image_asset_path(),
            },
            StandardDynamicAsset::File {
                path: self.get_stats_asset_path(),
            },
        ]
    }

    pub fn get_image_asset_path(&self) -> String {
        format!("nations/{}/units/{}/sprite.png", self.nation, self.name).to_string()
    }

    pub fn get_stats_asset_path(&self) -> String {
        format!("nations/{}/units/{}/unit.stats.ron", self.nation, self.name).to_string()
    }
}

impl DynamicAsset for UnitKey {
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped> {
        self.get_standard_assets()
            .iter()
            .map(|asset| asset.load(asset_server))
            .flatten()
            .collect()
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, Error> {
        let handles = self
            .get_standard_assets()
            .iter()
            .map(|asset| asset.build(world))
            .map(|result| result.unwrap())
            .fold(vec![], |mut collected_handles, dynamic_asset_type| {
                match dynamic_asset_type {
                    DynamicAssetType::Single(handle) => collected_handles.push(handle),
                    DynamicAssetType::Collection(mut handles) => {
                        collected_handles.append(&mut handles)
                    }
                }
                collected_handles
            });

        Ok(DynamicAssetType::Collection(handles))
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, TypePath, Clone, Debug, PartialEq)]
#[uuid = "6e753c5b-0158-48ac-a54e-c90cd6022815"]
pub struct UnitStats {
    pub name: String,
    pub max_action_points: usize,
    pub max_health_points: usize,
    pub attack: usize,
    pub defense: usize,
    #[serde(default = "default_attack_action_point_cost")]
    pub attack_action_point_cost: usize,
    #[serde(default = "default_max_attacks_per_round")]
    pub max_attacks_per_round: usize,
    #[serde(default = "default_range")]
    pub range: u32,
}

fn default_attack_action_point_cost() -> usize {
    2
}

fn default_max_attacks_per_round() -> usize {
    1
}

fn default_range() -> u32 {
    1
}

#[derive(Debug, Default, Clone, States, PartialEq, Eq, Hash)]
pub enum LoadingState {
    #[default]
    LoadingDynamicAssets,
    LoadingNationAssetsDefinition,
    Done,
}
