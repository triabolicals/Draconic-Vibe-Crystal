use super::*;
use data::WeaponDatabase;
pub use engage::{
    mess::*,
    hub::{access::*, hubsequence::HubSequence}, 
    util::get_singleton_proc_instance,
    menu::{BasicMenuResult, config::*},
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::{*, unit::*, item::*},
};
use crate::{continuous::get_continious_total_map_complete_count, utils::{self, get_list_item_class}};
use std::sync::{OnceLock, Mutex};
use crate::randomizer::item::data::WEAPONDATA;
use crate::utils::max;
use super::CONFIG;

pub static RANDOM_ITEM_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static HAS_REWARD: OnceLock<bool> = OnceLock::new();

pub mod unit_items;
pub mod shop;
pub mod data;
pub mod hub;
pub mod menu;
pub mod well;

// standard set
pub fn create_item_pool() {
    let mut has_well = true;
    ["アイテム交換_期待度１", "アイテム交換_期待度２", "アイテム交換_期待度３", "アイテム交換_期待度４", "アイテム交換_期待度５"].iter().for_each(|&x|{
        if RewardData::try_get_mut(x).is_none() {
            has_well = false;
        }
        else if RewardData::try_get_mut(x).unwrap().len() == 0 { has_well = false; }
    });
    let reward_len = RewardData::get_list().unwrap().len();
    if reward_len < 4 || !has_well {
        Patch::in_text(0x023f3c00).bytes(&[0xc0, 0x03, 0x5f, 0xd6]).unwrap();
    }
    HAS_REWARD.get_or_init(|| has_well);
    if RANDOM_ITEM_POOL.lock().unwrap().len() != 0 { return; }
    let item_list = ItemData::get_list_mut().unwrap();
    item_list.iter_mut()
        .for_each(|item|{
            if item.flag.value == 16 {
                let iid = item.iid.to_string();
                if iid != "IID_メティオ_G004" { item.flag.value = 3;  }
                if iid == "IID_メティオ" {  
                    item.endurance = 1;  
                    item.price = 2500;
                }
            }
            if item.flag.value & 251822590 == 0 && has_name(item, true) && !ITEM_BLACK_LIST.lock().unwrap().iter().any(|&y| y == item.parent.index)
                && !utils::str_contains(item.name, "MIID_Ring") && item.price != 100
            {
                RANDOM_ITEM_POOL.lock().unwrap().push(item.parent.hash);
            }
        }
    );
    println!("{} items are in the Random Item Pool", RANDOM_ITEM_POOL.lock().unwrap().len());
    WEAPONDATA.get_or_init(||{
        let mut db =  WeaponDatabase::new();
        db.initialize();
        db
    });
}

pub fn random_item(item_type: i32, allow_rare: bool) -> &'static Il2CppString {
    let rng = Random::get_system();
    let chapters = get_continious_total_map_complete_count();

    let extra_rate = 5 + 2*(chapters / 5);
    if rng.get_value(100) <= extra_rate && item_type != 1 && item_type != 2 {
        let database = WEAPONDATA.get().unwrap();
        let hash = database.extra_items[rng.get_value(database.extra_items.len() as i32) as usize];
        if let Some(extra_item) = ItemData::try_get_hash(hash){
            return extra_item.iid;
        }
    }
    let item_list_size = RANDOM_ITEM_POOL.lock().unwrap().len();
    let mut price = if chapters < 5 { 1000 } else { 400 * (chapters + 1) };
    if item_type == 4 { price = max(price >> 2, 501); }
    let mut low_price = price / 10;
    let mut count = 0;
    let exploration = GameVariableManager::get_number(DVCVariables::HUB_ITEM_KEY);
    loop {
        let item_hash = RANDOM_ITEM_POOL.lock().unwrap()[rng.get_value(item_list_size as i32) as usize];
        if let Some(random_item) = ItemData::try_get_hash(item_hash) {
            if item_type & 5 != 0 && random_item.price <= price &&
                (!allow_rare && (random_item.price >= low_price) || (( random_item.flag.value & 1 == 0) == allow_rare))
            {
                if (1 << (random_item.usetype as i64)) & 2268152036334i64 != 0 { return random_item.iid; }
                else { continue; }
            } else if item_type == 0 || item_type == 4 { //Item Script Replacement
                if random_item.is_inventory() || random_item.is_material() || random_item.usetype >= 31 { continue; }
            } else if item_type == 1 {    // Gift/Reward Items
                if random_item.usetype >= 32 && random_item.usetype <= 39 { continue; }
                if random_item.usetype == 0 && (random_item.kind != 17 && random_item.kind != 18) { continue; }
            } else if item_type == 2 {    // Exploration Drops
                let kind = random_item.kind;
                if kind == 17 && random_item.price > 1000 { continue; }     // Bond limited to 1000
                if kind == 18 && random_item.price > 1000 { continue; }    // Limit Money to 1000g
                if kind == 13 || (kind < 10 && kind != 0) { continue; }   // No Key Item or Weapon/Staff Related Items
                if (kind < 17 && kind > 13) || (kind == 10 && random_item.usetype == 21) { continue; } // No Ores or Stat Boosters
                if exploration & 1 != 0 && random_item.usetype == 33 { continue; }
                if exploration & 2 != 0 && random_item.usetype == 32 { continue; }
                if random_item.usetype == 35 { continue; }
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

// For item replacement
pub fn get_random_item(item: &'static Il2CppString, allow_rare: bool) -> &'static Il2CppString {
    if let Some(item_check) = ItemData::get(item) {
        let flag = item_check.get_flag().value;
        if flag & 1 == 1 { return item;  }
        if ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == item_check.parent.index).is_some() { return item; }
        let price = item_check.price;
        let mut count = 0;
        loop {
            count += 1;
            let new_iid = random_item(0, allow_rare);
            let new_price = ItemData::get(new_iid).unwrap().price;
            if new_price < price * CONFIG.lock().unwrap().replaced_item_price / 100 { continue; }
            if count >= 150 { return new_iid; }
            return new_iid;
        }
    }
    item
}

pub fn has_name(this: &ItemData, include_money: bool) -> bool {
    unsafe {  if utils::is_null_empty(this.name, None) { return false;  }  }
    let item_name = Mess::get(this.name ).to_string();
    item_name.len() != 0 || ( include_money && ( this.kind == 17 || this.kind == 18 ) )
}
pub fn randomize_well_rewards() {
    let mode = GameVariableManager::get_number(DVCVariables::GIFTS_KEY);
    if mode  == 0  { return; }
    if RANDOMIZER_STATUS.read().unwrap().well_randomized  { return; }
    let is_rare = mode == 2;
    ["DLC購入特典0", "DLC購入特典1", "Patch3特典", "Patch0特典"].iter().for_each(|r| {
        if let Some(reward) = RewardData::try_get_mut(*r) {
            let total = reward.iter().flat_map(|r| ItemData::get(r.iid))
                .filter(|i| (i.kind < 14 ||  i.kind > 18) && i.usetype < 40 && i.usetype != 24 && i.usetype != 34 )
                .map(|item| item.price)
                .sum::<i32>();
            let mut price = 0;
            reward.iter_mut().filter(|i|
                ItemData::get(i.iid).is_some_and(|i| (i.kind < 14 ||  i.kind > 18) && i.usetype < 40 && i.usetype != 24 && i.usetype != 34 )
            )
            .for_each(|r| {
                if price > total { r.set_iid("IID_絆のかけら10".into()); }
                loop {
                    if let Some(new_item) = ItemData::get(random_item(1, is_rare))
                        .filter(|x| x.get_weapon_level() < 4 &&  x.price < 10000 && x.kind != 18 && x.kind != 19 && x.equip_condition.is_none())
                    {
                        r.set_iid(new_item.iid);
                        price += new_item.price;
                        break;
                    }
                }
            });
        }
    });
    println!("Completed Randomization of Gift Items");
    let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.well_randomized = true);
}

pub fn change_liberation_type() {
    if !DVCVariables::is_main_chapter_complete(2) { return; }
    for x in ["IID_リベラシオン", "IID_リベラシオン改_ノーマル", "IID_リベラシオン改"]  {
        if let Some(liberation) = ItemData::get_mut(x){
            let l_type =  if GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) != 0 {
                GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) }
            else if let Some(hero_unit) = unsafe { utils::unit_pool_get_hero(false, None) } {
                let mut liberation_type = 1;
                hero_unit.get_job().get_equippable_item_kinds().iter().for_each(|&k| if k != 7 && k < 9 && k > 0 { liberation_type = k });
                GameVariableManager::set_number(DVCVariables::LIBERATION_TYPE, liberation_type);
                liberation_type
            }
            else {
                GameVariableManager::set_number(DVCVariables::LIBERATION_TYPE, 1);
                1
            } as u32;
            liberation.kind = l_type;
            match l_type {
                4 => {
                    liberation.range_i = 2;
                    liberation.range_o = 3;
                    liberation.set_cannon_effect("弓砲台".into());
                    liberation.get_equip_skills().add_sid("SID_飛行特効",4, 0); // Flier Effectiveness
                }
                5 => {
                    liberation.range_i = 1;
                    liberation.range_o = 2;
                    liberation.get_give_skills().add_sid("SID_毒",3, 0);    // Poison for Dagger
                }
                6 => {
                    liberation.range_i = 1;
                    liberation.range_o = 2;
                    liberation.set_cannon_effect("魔砲台炎".into());
                    liberation.set_hit_effect( "エルファイアー".into());
                }
                8 => {
                    liberation.get_equip_skills().add_sid("SID_２回行動",4,0);  // Brave for Fist
                }
                _ => {
                    liberation.range_i = 1;
                    liberation.range_o = 1;
                }
            }
            liberation.on_completed();
        }
    }
}

pub fn change_misercode_type(){
    let value = GameVariableManager::get_number(DVCVariables::MISERCODE_TYPE);
    let misercode_type = if value == 0 || value == 7 || value >= 9 {
        GameVariableManager::set_number(DVCVariables::MISERCODE_TYPE, 5);  
        5
    } else { value };
    let misercode = ItemData::get_mut("IID_ミセリコルデ").unwrap();
    misercode.get_give_skills().clear();
    misercode.get_equip_skills().clear();
    misercode.range_o = 2; 
    misercode.range_i = 1;
    misercode.kind = misercode_type as u32;
    match misercode_type  {
        4 => {
            misercode.set_cannon_effect("弓砲台".into());
            misercode.get_equip_skills().add_sid("SID_飛行特効",4, 0);
        }
        5 => { misercode.get_give_skills().add_sid("SID_毒",3, 0); }
        6 => {                     
            misercode.set_cannon_effect("魔砲台炎".into());
            misercode.set_hit_effect( "オヴスキュリテ".into());
        }
        8 => {
            misercode.range_o = 1;
            misercode.get_equip_skills().add_sid("SID_２回行動",4,0); 
        }
        _ => {}
    }
    misercode.on_completed();
}
