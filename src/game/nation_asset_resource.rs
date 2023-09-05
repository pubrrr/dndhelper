use bevy::prelude::{AssetServer, Assets, Commands, Handle, Image, Res, Resource};
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::AssetCollection;

use crate::common::DynamicNationAssetsDefinition;
use crate::common::NationAssetsDefinition;
use crate::game::nation_assets::UnitKey;
use crate::game::nation_assets::{NationAssetCollection, UnitStats};

#[derive(AssetCollection, Resource)]
pub struct NationAssetsResourceHelperAssets {
    #[asset(path = "generated_nations.assets.ron")]
    handle: Handle<DynamicNationAssetsDefinition>,
}

#[derive(Debug, Clone, Resource)]
pub struct NationAssetsResource {
    nation_assets_definition: Vec<NationAssetsDefinition>,
    unit_images: HashMap<String, Handle<Image>>,
    unit_stats: HashMap<String, UnitStats>,
}

impl NationAssetsResource {
    pub fn get_nations(&self) -> Vec<Nation> {
        self.nation_assets_definition
            .iter()
            .map(|nation_assets| Nation {
                name: nation_assets.path.to_string(),
                key: NationKey(nation_assets.path.to_string()),
            })
            .collect()
    }

    pub fn get_nation(&self, nation_key: &NationKey) -> Nation {
        self.nation_assets_definition
            .iter()
            .find(|nation_assets| nation_assets.path == nation_key.0)
            .map(|nation_assets| Nation {
                name: nation_assets.path.to_string(),
                key: NationKey(nation_assets.path.to_string()),
            })
            .unwrap()
    }

    pub fn get_units(&self, nation_key: &NationKey) -> Vec<UnitKey> {
        self.nation_assets_definition
            .iter()
            .find(|nation_assets| nation_assets.path == nation_key.0)
            .unwrap()
            .units
            .iter()
            .map(|unit_assets| UnitKey {
                nation: nation_key.0.to_string(),
                name: unit_assets.path.to_string(),
            })
            .collect()
    }

    pub fn get_unit_assets(&self, unit_key: &UnitKey) -> UnitAssets {
        let image = self.unit_images[&unit_key.get_image_asset_path()].clone();
        let stats = self.unit_stats[&unit_key.get_stats_asset_path()].clone();

        UnitAssets { image, stats }
    }
}

pub struct Nation {
    pub name: String,
    pub key: NationKey,
}

#[derive(Debug, Clone)]
pub struct NationKey(String);

pub fn insert_nation_assets_resource(
    mut commands: Commands,
    nation_assets_resource_helper: Res<NationAssetsResourceHelperAssets>,
    nation_assets_collection: Res<NationAssetCollection>,
    dynamic_nation_assets: Res<Assets<DynamicNationAssetsDefinition>>,
    unit_stats_assets: Res<Assets<UnitStats>>,
) {
    let dynamic_nation_assets = dynamic_nation_assets
        .get(&nation_assets_resource_helper.handle)
        .expect("The DynamicNationAssets should be loaded by now");

    let unit_stats = nation_assets_collection
        .unit_stats_files
        .iter()
        .map(|(key, handle)| {
            let stats = unit_stats_assets.get(handle).unwrap().clone();
            (key.clone(), stats)
        })
        .collect();

    commands.insert_resource(NationAssetsResource {
        nation_assets_definition: dynamic_nation_assets.0.clone(),
        unit_images: nation_assets_collection.unit_images.clone(),
        unit_stats,
    });

    commands.remove_resource::<NationAssetCollection>();
}

pub struct UnitAssets {
    pub image: Handle<Image>,
    pub stats: UnitStats,
}
