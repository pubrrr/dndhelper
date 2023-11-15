use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};
use bevy_asset_loader::prelude::LoadingStateAppExt;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::common::DynamicNationAssetsDefinition;
use crate::game::asset_loading::nation_asset_resource::{
    insert_nation_assets_resource, NationAssetsResourceHelperAssets,
};
use crate::game::asset_loading::nation_assets::{LoadingState, NationAssetCollection, UnitStats};
use crate::scan_assets::GENERATED_NATIONS_ASSETS_FILE;

pub mod nation_asset_resource;
pub mod nation_assets;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<DynamicNationAssetsDefinition>::new(&["assets.ron"]),
            RonAssetPlugin::<UnitStats>::new(&["stats.ron"]),
        ))
        .add_state::<LoadingState>()
        .add_loading_state(
            bevy_asset_loader::loading_state::LoadingState::new(LoadingState::LoadingDynamicAssets)
                .continue_to_state(LoadingState::LoadingNationAssetsDefinition)
                .set_standard_dynamic_asset_collection_file_endings(vec![]),
        )
        .add_loading_state(
            bevy_asset_loader::loading_state::LoadingState::new(
                LoadingState::LoadingNationAssetsDefinition,
            )
            .continue_to_state(LoadingState::Done)
            .set_standard_dynamic_asset_collection_file_endings(vec![]),
        )
        .register_dynamic_asset_collection::<_, DynamicNationAssetsDefinition>(
            LoadingState::LoadingDynamicAssets,
        )
        .add_dynamic_collection_to_loading_state::<_, DynamicNationAssetsDefinition>(
            LoadingState::LoadingDynamicAssets,
            GENERATED_NATIONS_ASSETS_FILE,
        )
        .add_collection_to_loading_state::<_, NationAssetCollection>(
            LoadingState::LoadingDynamicAssets,
        )
        .add_collection_to_loading_state::<_, NationAssetsResourceHelperAssets>(
            LoadingState::LoadingNationAssetsDefinition,
        )
        .add_systems(OnEnter(LoadingState::Done), insert_nation_assets_resource);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::asset_loading::nation_asset_resource::NationAssetsResource;
    use crate::scan_assets::write_nations_assets;
    use bevy::log::{Level, LogPlugin};
    use bevy::prelude::{default, AssetPlugin, ImagePlugin, State};
    use bevy::tasks::{IoTaskPool, TaskPoolBuilder};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn all_assets_can_be_loaded() {
        write_nations_assets().unwrap();

        let mut app = App::new();

        app.add_plugins((
            AssetPlugin::default(),
            ImagePlugin::default(),
            AssetLoadingPlugin,
            LogPlugin {
                level: Level::TRACE,
                ..default()
            },
        ));
        IoTaskPool::init(|| {
            TaskPoolBuilder::default()
                .thread_name("IO Task Pool".to_string())
                .build()
        });

        app.update();

        sleep(Duration::from_millis(1000));

        app.update();
        app.update();
        app.update();

        sleep(Duration::from_millis(1000));

        app.update();
        app.update();
        app.update();

        sleep(Duration::from_millis(1000));

        app.update();
        app.update();
        app.update();

        sleep(Duration::from_millis(1000));

        app.update();
        app.update();
        app.update();

        sleep(Duration::from_millis(1000));

        app.update();
        app.update();
        app.update();

        assert_eq!(
            app.world.resource::<State<LoadingState>>().get(),
            &LoadingState::Done
        );
        let nation_assets_resource = app.world.resource::<NationAssetsResource>();
        assert!(nation_assets_resource.nation_assets_definition.len() > 0);
    }
}
