use std::sync::OnceLock;

pub static UNIQUE_JOB_DATA: OnceLock<Vec<job::UniqueJobAssets>> = OnceLock::new();
pub static NAME_DATA: OnceLock<person::NameData> = OnceLock::new();
pub static WEAPON_ASSET: OnceLock<item::WeaponAssets> = OnceLock::new();
pub static HEAD_DATA: OnceLock<person::HeadData> = OnceLock::new();
pub static CUSTOM_CLASS_ASSETS: OnceLock<Vec<job::CustomJobAssets>> = OnceLock::new();
pub static ASSET_DATA: OnceLock<accessory::AccAssetData> = OnceLock::new();

pub mod item;
pub mod job;
pub mod person;
pub mod accessory;

pub fn initalize_asset_data(){
    UNIQUE_JOB_DATA.get_or_init(|| job::get_animation_unique_classes() );
    NAME_DATA.get_or_init(|| person::get_names_data() );
    WEAPON_ASSET.get_or_init(|| item::get_weapon_assets() );
    CUSTOM_CLASS_ASSETS.get_or_init(|| job::get_custom_class_assets() );
    ASSET_DATA.get_or_init(|| accessory::get_all_accesories() );
}
