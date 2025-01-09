pub use engage::{
    mess::*,
    menu::{BasicMenuResult, config::*},
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::{*, unit::*, item::*},
};
use unity::system::*;
use std::sync::Mutex;
pub mod unit_items;
pub mod shop;
pub mod item_rando;

use super::CONFIG;
use crate::{enums, utils};
pub static RANDOM_ITEM_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());
// Contains methods of random items, and jobs
// standard set 

pub fn create_item_pool() {
    if RANDOM_ITEM_POOL.lock().unwrap().len() != 0 { return; }
    let item_list = ItemData::get_list_mut().unwrap();
    item_list.iter_mut()
        .for_each(|item|{
            if item.flag.value == 16 {
                let iid = item.iid.to_string();
                if iid != "IID_メティオ_G004" { item.flag.value = 0;  }
                if iid == "IID_メティオ" {  item.endurance = 1;  }
            }
            if item.flag.value & 251822590 == 0 && has_name(item, true) && !enums::ITEM_BLACK_LIST.lock().unwrap().iter().any(|y| *y == item.parent.index) 
                && !crate::utils::str_contains(item.name, "MIID_Ring") && !item.is_unknown()
            {
                RANDOM_ITEM_POOL.lock().unwrap().push(item.parent.hash);
            }
        }
    );
    println!("{} items are in the Random Item Pool", RANDOM_ITEM_POOL.lock().unwrap().len());
    item_rando::WEAPONDATA.lock().unwrap().intitalize();
}

pub fn random_item(item_type: i32, allow_rare: bool) -> &'static Il2CppString {
    let item_list_size = RANDOM_ITEM_POOL.lock().unwrap().len();
    let rng = Random::get_system();
    let base_price = 2500 - 50*GameVariableManager::get_number("G_ItemDropGauge"); 
    let exploration = GameVariableManager::get_number("G_HubItem");
    loop {
        let item_hash = RANDOM_ITEM_POOL.lock().unwrap()[rng.get_value( item_list_size as i32 ) as usize];
        if let Some(random_item) = ItemData::try_get_hash(item_hash) {
            if item_type == 0 || item_type == 4 { //Item Script Replacement
                if random_item.is_inventory() || random_item.is_material() { continue; }
                if item_type == 4 && random_item.price < base_price { continue; } 
            }
            else if item_type == 1 {    // Gift/Reward Items
                if random_item.usetype >= 32 && random_item.usetype <= 39 { continue; }
                if random_item.usetype == 0 && ( random_item.kind != 17 && random_item.kind != 18 ){ continue; }  
            }
            else if item_type == 2 {    // Exploration Drops
                let iid = random_item.iid.to_string();
                if iid == "IID_スキルの書・離" || iid == "IID_スキルの書・破" {  continue; }    // No Adept/Expert Book
                let kind = random_item.kind;
                if kind == 17 && random_item.price > 5000 { continue; }     // Bond limited to 1000
                if kind == 18 && random_item.price >= 1000 { continue; }    // Limit Money to 1000g
                if kind == 13 || ( kind < 10 && kind != 0 ) { continue; }   // No Key Item or Weapon/Staff Related Items
                if ( kind < 17 && kind > 13 ) || (kind == 10 && random_item.usetype == 21) { continue; } // No Ores or Stat Boosters
                if exploration & 1 != 0 && random_item.usetype == 33 { continue; } 
                if exploration & 2 != 0 && random_item.usetype == 32 { continue; }
                if random_item.usetype == 35 { continue; }
            }
            if random_item.get_flag().value & 1 != 0 && !allow_rare { continue; }
            return random_item.iid;
        }
    }

}

// For item replacement
pub fn get_random_item(item: &'static Il2CppString, allow_rare: bool) -> &'static Il2CppString {
    if let Some(item_check) = ItemData::get(item) {
        let flag = item_check.get_flag().value;
        if flag & 1 == 1 { return item;  }
        if enums::ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == item_check.parent.index).is_some() { return item; }
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
    else { return item; }

}

pub fn has_name(this: &ItemData, include_money: bool) -> bool {
    unsafe {  if crate::utils::is_null_empty(this.name, None) { return false;  }  }
    let item_name = Mess::get(this.name ).to_string();
    if item_name.len() != 0 { return true }
    else if include_money {
        return this.kind == 17 || this.kind == 18 ;    // If Money or bond
    }
    return false; 
}
#[skyline::hook(offset=0x0293c890)]
pub fn random_well_item(proc: u64, level: i32, random: &Random, method_info: OptionalMethod) -> &'static mut List<ItemData> {
    let list = call_original!(proc, level, random, method_info);
    if GameVariableManager::get_number("G_Random_Item") & 1 == 0  { return list; }  
    let mut sum = 0;
    list.iter().for_each(|item| sum += item.price + (random.get_value(10) + level) * 100);
    let count = list.iter().count() as i32;
    let rare_item = CONFIG.lock().unwrap().random_gift_items == 2;
    list.clear();
    println!("Total Price of Items: {}", sum);
    let min_price = ( 1 + level ) * 5000 + 1000 * (1 + random.get_value(count) );
    if sum < min_price { sum = min_price; }
    let mut total_price = 0;
    while total_price < sum {
        let iid = random_item(1, rare_item);
        if let Some(item) = ItemData::get_mut(iid) {
            if item.price > 20000 { continue; }
            total_price += item.price;
            println!("Add: {}", item.name);
            list.add(item);
        }
    }
    println!("Total Cost: {} / {}", total_price, min_price);
    return list;
}
pub fn randomize_well_rewards() {
    if GameVariableManager::get_number("G_Random_Item") == 0  { return; }
    if unsafe { super::STATUS.well_randomized } { return; }
    if CONFIG.lock().unwrap().random_gift_items != 0 {
        let rare_item = CONFIG.lock().unwrap().random_gift_items == 2;
        let rlist = RewardData::get_list_mut().unwrap();
        rlist.iter_mut().for_each(|reward|{
            reward.iter_mut().for_each(|ritem|{
                if ItemData::get(ritem.iid).is_some() {
                    loop {
                        if let Some(new_item) = ItemData::get(random_item(1, rare_item)){
                            if new_item.equip_condition.is_none() { 
                                ritem.set_iid(new_item.iid);  
                                break;
                            }
                        }
                    }
                }
            });
        });
    }
    println!("Complete Randomization of Gift/Well Items");
    shop::randomize_hub_random_items();
    unsafe { super::STATUS.well_randomized = true };
}

pub fn change_liberation_type() {
    if !GameVariableManager::get_bool("G_Cleared_M002") { return; }
    let liberation = ItemData::get_mut("IID_リベラシオン").unwrap();
    let l_type =  if GameVariableManager::get_number("G_Liberation_Type") != 0 {  GameVariableManager::get_number("G_Liberation_Type") }
        else if let Some(hero_unit) = unsafe { utils::unit_pool_get_hero(false, None) } {
            let mut liberation_type = 1;
            hero_unit.get_job().get_equippable_item_kinds().iter().for_each(|&k| if k != 7 && k < 9 && k > 0 { liberation_type = k });
            GameVariableManager::make_entry("G_Liberation_Type", liberation_type);
            GameVariableManager::set_number("G_Liberation_Type", liberation_type);
            liberation_type
        }
        else { 
            GameVariableManager::make_entry("G_Liberation_Type", 1);
            GameVariableManager::set_number("G_Liberation_Type", 1);
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
            //liberation.get_equip_skills().add_sid("SID_気功",4, 0);
            liberation.get_equip_skills().add_sid("SID_２回行動",4,0);  // Brave for Fist
        }
        _ => {
            liberation.range_i = 1;
            liberation.range_o = 1;
        }
    }
    liberation.on_completed();
    println!("Liberation changed to weapon type {}", l_type);
}

pub fn change_misercode_type(){
    let value = GameVariableManager::get_number("G_Misercode_Type");
    let misercode_type = if value == 0 || value == 7 || value >= 9 {
        GameVariableManager::make_entry("G_Misercode_Type", 5);
        GameVariableManager::set_number("G_Misercode_Type", 5);  
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

pub struct EnemyDropGauge;
impl ConfigBasicMenuItemGaugeMethods  for EnemyDropGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().enemy_drop_rate  as f32 / 100.0 }
            else { GameVariableManager::get_number("G_ItemDropGauge") as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().enemy_drop_rate as f32 / 100.0 }
            else { GameVariableManager::get_number("G_ItemDropGauge") as f32 / 100.0 };

        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.10);
        if gauge != result {
            if GameUserData::get_sequence() == 0 {CONFIG.lock().unwrap().enemy_drop_rate  = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_ItemDropGauge", ( result * 100.0 ) as i32 ); }
            this.gauge_ratio = result;
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().enemy_drop_rate }
            else { GameVariableManager::get_number("G_ItemDropGauge")};

        this.help_text = format!("Percentage of enemy units dropping random items. ({:.2}%)",  value).into();
    }
}
pub extern "C" fn vibe_drops() -> &'static mut ConfigBasicMenuItem {
    let item_gauge = ConfigBasicMenuItem::new_gauge::<EnemyDropGauge>("Enemy Item Drop Rate");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map as _);
    item_gauge
}

pub struct ItemPriceGauge;
impl ConfigBasicMenuItemGaugeMethods  for ItemPriceGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().replaced_item_price as f32 / 100.0 }
            else { GameVariableManager::get_number("G_ItemGauge") as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().replaced_item_price as f32 / 100.0 }
            else { GameVariableManager::get_number("G_ItemGauge") as f32 / 100.0 };

        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.25);
        if gauge != result {
            if GameUserData::get_sequence() == 0 {CONFIG.lock().unwrap().replaced_item_price = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_ItemGauge", ( result * 100.0 ) as i32 ); }
            this.gauge_ratio = result;
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Minimum value of new item as a percentage of original item's value.".into();
    }
}
pub extern "C" fn vibe_item_gauge() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_gauge::<ItemPriceGauge>("Randomized Item Value");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_hub_items as _);
    item_gauge
}
pub struct RandomItemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomItemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_item, 0, 3, 1);
        if CONFIG.lock().unwrap().random_item != result {
            CONFIG.lock().unwrap().random_item  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_item {
            1 => { "Items obtained from chests/villages will be random." },
            2 => { "Item drops from enemies will be random." },
            3 => { "Item obtained from events and enemy drops will be random." },
            _ => { "No changes made to item events or item drops." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_item {
            1 => { "Events"},
            2 => { "Drops"},
            3 => { "Events/Drops"}
            _ => { "None "}
        }.into();
    }
}

pub struct RandomGiftMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGiftMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_gift_items, 0, 2, 1);
        if CONFIG.lock().unwrap().random_gift_items != result {
            CONFIG.lock().unwrap().random_gift_items  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_gift_items;
        if selection == 1 {  this.help_text = "No rare items will be included when randomizing gift item lists.".into(); }
        else if selection == 2 {  this.help_text = "Rare items will be included when randomizing gift item lists.".into(); } 
        else { this.help_text = "No randomization done to gift items.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let selection = CONFIG.lock().unwrap().random_gift_items;
        if selection == 1 { this.command_text = "No Rare Items".into(); }
        else if selection == 2 { this.command_text = "With Rare Items".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

