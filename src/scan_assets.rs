use ron::ser::PrettyConfig;
use std::fs;
use std::fs::{DirEntry, File};

use crate::nation_assets::{DynamicNationAssets, NationAssets, UnitAssets};

pub const GENERATED_NATIONS_ASSETS_FILE: &'static str = "generated_nations.assets.ron";

pub fn write_nations_assets() -> ron::Result<()> {
    let dynamic_nation_assets = scan_assets();
    ron::ser::to_writer_pretty(
        File::create(format!("assets/{GENERATED_NATIONS_ASSETS_FILE}")).unwrap(),
        &dynamic_nation_assets,
        PrettyConfig::default(),
    )
}

fn scan_assets() -> DynamicNationAssets {
    let nation_assets_dir = fs::read_dir("assets/nations").unwrap();

    let nation_assets = nation_assets_dir
        .map(|dir_entry| dir_entry.unwrap())
        .map(|dir_entry| NationAssets {
            path: dir_entry.file_name().into_string().unwrap(),
            units: get_unit_assets(dir_entry),
        })
        .collect();

    DynamicNationAssets(nation_assets)
}

fn get_unit_assets(nation_dir: DirEntry) -> Vec<UnitAssets> {
    let unit_assets_dir = fs::read_dir(format!(
        "{}/units",
        nation_dir.path().as_os_str().to_str().unwrap()
    ))
    .unwrap();

    unit_assets_dir
        .map(|dir_entry| dir_entry.unwrap())
        .map(|dir_entry| UnitAssets {
            path: dir_entry.file_name().into_string().unwrap(),
        })
        .collect()
}
