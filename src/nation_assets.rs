use bevy::asset::Error;
use bevy::prelude::{info, AssetServer, Handle, HandleUntyped, Image, Resource, States, World};
use bevy::reflect::{TypePath, TypeUuid};
use bevy::utils::HashMap;
use bevy_asset_loader::dynamic_asset::DynamicAssets;
use bevy_asset_loader::prelude::{
    AssetCollection, DynamicAsset, DynamicAssetCollection, DynamicAssetType, StandardDynamicAsset,
};

#[derive(AssetCollection, Resource, Debug)]
pub struct NationAssetCollection {
    #[asset(key = "unit_images", collection(typed, mapped))]
    pub unit_images: HashMap<String, Handle<Image>>,
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
    fn get_standard_assets(&self) -> Vec<StandardDynamicAsset> {
        vec![StandardDynamicAsset::File {
            path: self.get_image_asset(),
        }]
    }

    fn get_image_asset(&self) -> String {
        format!("nations/{}/units/{}/sprite.png", self.nation, self.name).to_string()
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

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, TypePath, Debug, PartialEq)]
#[uuid = "20eb6907-3d6f-4a93-bb03-62b812182055"]
pub struct DynamicNationAssets(pub Vec<NationAssets>);

impl DynamicAssetCollection for DynamicNationAssets {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        let image_assets = self
            .0
            .iter()
            .map(|nation_assets| nation_assets.get_units())
            .flatten()
            .map(|unit_key| unit_key.get_image_asset())
            .collect();

        info!("Registering unit images: {image_assets:?}");

        dynamic_assets.register_asset(
            "unit_images",
            Box::new(StandardDynamicAsset::Files {
                paths: image_assets,
            }),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct NationAssets {
    pub path: String,
    pub units: Vec<UnitAssets>,
}

impl NationAssets {
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

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct UnitAssets {
    pub path: String,
}

#[derive(Debug, Default, Clone, States, PartialEq, Eq, Hash)]
pub enum LoadingState {
    #[default]
    Loading,
    Done,
}

#[cfg(test)]
mod tests {
    use ron::de::from_bytes;
    use ron::ser::PrettyConfig;

    use crate::nation_assets::{DynamicNationAssets, UnitKey};

    #[test]
    fn asd() {
        let collection = DynamicNationAssets(vec![
            UnitKey {
                nation: "Kurzland".to_string(),
                name: "Tree".to_string(),
            }
            .into(),
            UnitKey {
                nation: "Mansreich".to_string(),
                name: "Manf".to_string(),
            }
            .into(),
        ]);

        let string = ron::ser::to_string_pretty(&collection, PrettyConfig::default()).unwrap();
        println!("{}", string);

        let deserialized: DynamicNationAssets = ron::from_str(&string).unwrap();

        let deserialized2: DynamicNationAssets = from_bytes(string.as_bytes()).unwrap();

        assert_eq!(deserialized, collection);
        println!("{:?}", deserialized2);
        assert_eq!(deserialized2, collection);
    }

    #[test]
    fn asdfasdf() {
        let bytes = [
            40u8, 91, 13, 10, 32, 32, 32, 32, 40, 13, 10, 32, 32, 32, 32, 32, 32, 32, 32, 110, 97,
            116, 105, 111, 110,
        ];

        let map = bytes.as_slice().into_iter().map(|b| char::from(*b));
        let s = String::from_iter(map);

        println!("{s}");
    }
}
