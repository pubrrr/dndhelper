#[cfg(feature = "bevy")]
use bevy::prelude::info;
#[cfg(feature = "bevy")]
use bevy::reflect::{TypePath, TypeUuid};
#[cfg(feature = "bevy")]
use bevy_asset_loader::dynamic_asset::DynamicAssets;
#[cfg(feature = "bevy")]
use bevy_asset_loader::prelude::{DynamicAssetCollection, StandardDynamicAsset};

#[cfg(feature = "bevy")]
use crate::game::asset_loading::nation_assets::UnitKey;

#[cfg(feature = "bevy")]
#[derive(serde::Deserialize, serde::Serialize, TypeUuid, TypePath, Debug, PartialEq)]
#[uuid = "20eb6907-3d6f-4a93-bb03-62b812182055"]
pub struct DynamicNationAssetsDefinition(pub Vec<NationAssetsDefinition>);

#[cfg(not(feature = "bevy"))]
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct DynamicNationAssetsDefinition(pub Vec<NationAssetsDefinition>);

#[cfg(feature = "bevy")]
impl DynamicAssetCollection for DynamicNationAssetsDefinition {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        let image_assets = self
            .0
            .iter()
            .map(|nation_assets| nation_assets.get_units())
            .flatten()
            .map(|unit_key| unit_key.get_image_asset_path())
            .collect();
        let unit_stats_files = self
            .0
            .iter()
            .map(|nation_assets| nation_assets.get_units())
            .flatten()
            .map(|unit_key| unit_key.get_stats_asset_path())
            .collect();

        info!("Registering unit images: {image_assets:?}");
        info!("Registering unit stats files: {unit_stats_files:?}");

        dynamic_assets.register_asset(
            "unit_images",
            Box::new(StandardDynamicAsset::Files {
                paths: image_assets,
            }),
        );
        dynamic_assets.register_asset(
            "unit_stats_files",
            Box::new(StandardDynamicAsset::Files {
                paths: unit_stats_files,
            }),
        );
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub struct NationAssetsDefinition {
    pub path: String,
    pub units: Vec<UnitAssetsDefinition>,
}

#[cfg(feature = "bevy")]
impl NationAssetsDefinition {
    pub fn get_units(&self) -> Vec<UnitKey> {
        self.units
            .iter()
            .map(|unit_assets| UnitKey {
                nation: self.path.to_string(),
                name: unit_assets.path.to_string(),
            })
            .collect()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub struct UnitAssetsDefinition {
    pub path: String,
}
