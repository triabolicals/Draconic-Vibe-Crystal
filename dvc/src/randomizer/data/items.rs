use engage::gamedata::item::*;
use engage::gamedata::{Gamedata, GamedataArray};
use engage::gamedata::skill::SkillData;
use engage::mess::Mess;
use engage::random::Random;
use unity::prelude::Il2CppString;
use crate::config::DVCVariables;
use crate::continuous::get_continious_total_map_complete_count;
use crate::randomizer::blacklist::DVCBlackLists;
use crate::randomizer::item::data::WeaponDatabase;
use crate::randomizer::Randomizer;
use crate::utils::{dlc_check, max};

pub struct ItemPool {
    pub engage_items: EngageItems,
    pub has_reward: bool,
    pub has_well: bool,
    pub weapon_db: WeaponDatabase,
    pub pool: Vec<i32>,
    pub evolve_data: Vec<i32>,
    pub refine_iid: Vec<i32>,
}

impl ItemPool {
    pub fn init() -> Self {
        let has_well = ["アイテム交換_期待度１", "アイテム交換_期待度２", "アイテム交換_期待度３", "アイテム交換_期待度４", "アイテム交換_期待度５"].iter().all(|&x| RewardData::try_get_mut(x).is_some());
        let has_reward = ["DLC購入特典0", "DLC購入特典1", "Patch0特典", "Patch3特典"].iter().all(|&x| RewardData::try_get_mut(x).is_some());
        let bl = DVCBlackLists::get_read();
        println!("Getting Item Refined Data");
        let refine_iid =
        ItemRefineData::get_list()
            .map(|v|
                v.iter()
                    .flat_map(|x| ItemData::get(x.array_name.to_string().replace("RID_", "IID_")))
                    .map(|i| i.parent.hash)
                    .collect()
            ).unwrap_or(vec![]);

        Self {
            refine_iid,
            evolve_data: ItemEvolveData::get_list().unwrap().iter()
                .flat_map(|x| x.iter())
                .map(|i| ItemData::get(i.iid).map(|i| i.parent.hash).unwrap_or(0) )
                .collect(),
            has_well, has_reward,
            engage_items: EngageItems::init(),
            weapon_db: WeaponDatabase::init(),
            pool: ItemData::get_list().unwrap().iter()
                .filter(|&x| x.flag.value &  251822590 == 0 &&  bl.item.allowed_index(x.parent.index) && x.price != 100)
                .map(|x| x.parent.hash).collect(),
        }
    }
    pub fn random_item(&self, item_type: i32, allow_rare: bool) -> &'static Il2CppString {
        let rng = Random::get_system();
        let chapters = get_continious_total_map_complete_count();
        let extra_rate = 5 + 2*(chapters / 5);
        if rng.get_value(100) <= extra_rate && item_type != 1 && item_type != 2 {
            if let Some(v) = self.weapon_db.extra_items.get_random_element(rng).and_then(|&h| ItemData::try_get_hash(h)) {
                return v.iid;
            }
        }
        let mut price = if chapters < 5 { 1000 } else { 400 * (chapters + 1) };
        if item_type == 4 { price = max(price >> 2, 501); }
        let mut low_price = price / 10;
        let mut count = 0;
        let exploration = DVCVariables::ExplorationItem.get_value();
        loop {
            if let Some(random_item) = self.pool.get_random_element(rng).and_then(|&h| ItemData::try_get_hash(h)){
                if item_type == 2 {    // Exploration Drops
                    let kind = random_item.kind;
                    let use_type = random_item.use_type;
                    let use_type_flag = 1 << use_type as i64;
                    let mut exclude_use = 0xC88800FFFFi64;
                    if exploration & 1 != 0 { exclude_use |= 1 << 33; }
                    if exploration & 2 != 0 { exclude_use |= (1 << 32)| (1 << 17); }
                    if !dlc_check() { exclude_use |= (1 << 40) | (1 << 41); }
                    if use_type_flag & exclude_use != 0 { continue; }
                    if kind == 13 || (kind < 10 && kind > 0) || (kind >= 17 && random_item.price > 1000) || (use_type == 42 && random_item.power > 100) { continue; }   // No Weapons, Bond/Money Under 1000 or Key Items
                }
                else if item_type & 5 != 0 && random_item.price <= price &&
                    (!allow_rare && (random_item.price >= low_price) || (( random_item.flag.value & 1 == 0) == allow_rare))
                {
                    if (1 << (random_item.use_type as i64)) & 2268152036334i64 != 0 { return random_item.iid; }
                    else { continue; }
                }
                else if item_type == 0 || item_type == 4 { //Item Script Replacement
                    if random_item.is_material() || random_item.use_type >= 31 { continue; }
                }
                else if item_type == 1 {    // Gift/Reward Items
                    if random_item.use_type >= 32 && random_item.use_type <= 39 { continue; }
                    if random_item.use_type == 0 && (random_item.kind != 17 && random_item.kind != 18) { continue; }
                }
                if random_item.flag.value & 130 != 0 { continue; }
                if random_item.flag.value & 1 != 0 && !allow_rare { continue; }
                return random_item.iid;
            }
            if (count % 20) == 19 {
                price = price >> 1;
                low_price = price / 10;
            }
            count += 1;
        }
    }
    pub fn random_item_data(&self, rng: &Random) -> &'static ItemData { 
        self.pool.get_random_element(rng).and_then(|&h| ItemData::try_get_hash(h)).unwrap() 
    }

}
pub struct EngageItems{
    pub weapons: Vec<i32>,
    pub bows: Vec<i32>,
    pub non_weapons: Vec<i32>,
    pub enemy: Vec<(String, i32)>,
}

impl EngageItems {
    pub fn init() -> Self {
        let mut no_normal: Vec<i32> = Vec::new();
        let mut weapons: Vec<i32> = Vec::new();
        let mut bows: Vec<i32> = Vec::new();
        let mut non_weapons: Vec<i32> = Vec::new();
        let mut enemy: Vec<(String, i32)> = Vec::new();
        let engage_items: Vec<_> = 
            SkillData::get_list().unwrap().iter().filter(|s| s.flag & 62 != 0 && s.equip_items.len() == 1).map(|s| s.equip_items[0].parent.hash).collect();
        
        ItemData::get_list().unwrap().iter()
            .filter(|i|
                Mess::get(i.name).to_string().len() > 2 && 
                i.icon.is_some() && 
                i.range_o < 255 && i.flag.value & 128 == 128 && i.parent.index > 3 && i.use_type != 15 && i.use_type < 35 &&
                    !i.iid.str_contains("ディザスター") && !i.iid.str_contains("エンゲージ技") && !i.iid.str_contains("_竜穿") 
                && i.equip_condition.is_none() && !engage_items.iter().any(|s| i.parent.hash == *s)            
            )
            .for_each(|item| {
                let iid = item.iid.to_string();
                if iid.contains("_M0") || iid.contains("_E00") || iid.contains("_G00") { enemy.push((iid, item.parent.hash)); }
                if !item.iid.str_contains("_通常") && ItemData::get(format!("{}_通常", item.iid)).is_none() && item.kind < 10 && item.use_type == 1 {
                    no_normal.push(item.parent.hash);
                }
                match item.kind {
                    1|2|3|5|6|8|9 => { weapons.push(item.parent.hash); }
                    4 => { bows.push(item.parent.hash); }
                    _ => { non_weapons.push(item.parent.hash); }
                }
            });
        /*
        println!("Engage Weapon Pool: {}", weapons.len());
        println!("Enemy Weapons: {}", enemy.len());
        println!("Engage Bow Pool: {}", bows.len());
        println!("Engage Non Weapons: {}", non_weapons.len());
        
         */
        Self { weapons, bows, non_weapons, enemy }
    }
}


