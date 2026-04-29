use std::sync::OnceLock;
use engage::{
    resourcemanager::*, gamedata::{*, skill::*, assettable::*}, 
    random::Random, gamedata::item::ItemData
};
use outfit_core::get_outfit_data;
pub use outfit_core::Mount;
use item::WeaponAssets;
use search::*;
use crate::randomizer::{Randomizer};
use super::AnimSetDB;
pub static SEARCH_LIST: OnceLock<AssetData> = OnceLock::new();
pub mod item;
pub mod search;

pub struct AssetData {
    pub items: WeaponAssets,
    pub bond_face: Vec<String>,
    pub engaging: Vec<i32>,
    pub map_events: Vec<String>,
}
impl AssetData {
    pub fn new() -> Self {
        Self{
            map_events: vec![],
            items: WeaponAssets::new(),
            bond_face: vec![],
            engaging: vec![],
        }
    }
    pub fn get_random_outfit(&self, mode: i32, hash: u32, is_female: bool) -> &'static Il2CppString {
        let rng = Random::new(hash);
        let db = get_outfit_data();
        let set = if !is_female { &db.hashes.male_ou } else { &db.hashes.female_ou };
        set.get_random_element(rng).and_then(|v|{
            if mode == 2 { db.hashes.body.get(&v.0) } else { db.hashes.o_body.get(&v.1) }
        }).map(|v| v.as_str().into() ).unwrap()
    }
}
pub fn initialize_search_list() {
    SEARCH_LIST.get_or_init(||{
        let mut slist = AssetData::new();
        
        slist.bond_face = ResourceManager::class().get_static_fields::<ResourceManagerStaticFields>().files.entries.iter()
            .filter(|x|x.key.is_some_and(|x| x.str_contains("Telop/LevelUp/FaceThumb/")))
            .map(|x| x.key.unwrap().to_string()).collect();

        slist.engaging = AnimSetDB::get_list().unwrap().iter().filter(|x| x.parent.index > 0 && x.get_engage1().is_some_and(|x| x.to_string() == "=")).map(|x| x.parent.index).collect();

        slist.map_events = ResourceManager::class().get_static_fields::<ResourceManagerStaticFields>().files.entries.iter()
            .filter(|x|x.key.is_some_and(|x| x.str_contains("Event/PuppetDemo/Materials/")))
            .map(|x|{
                let t1 = x.key.unwrap().to_string();
                let t2 = t1.trim_start_matches("Event/PuppetDemo/Materials/");
                let t3 = t2.trim_end_matches(".mat");
                t3.to_string()
            }).collect();

        ItemData::get_list().unwrap().iter().for_each(|item| { search_by_iid(item.iid, 2).map(|entry| slist.items.add(item, entry.parent.index)); });
        slist
    });
}