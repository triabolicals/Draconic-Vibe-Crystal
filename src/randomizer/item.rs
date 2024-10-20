pub use engage::{
    mess::*,
    menu::{BasicMenuResult, config::*},
    gamevariable::*,
    gameuserdata::*,
    random::*,
    gamedata::{*, unit::*, item::*},
};
use engage::gamedata::skill::SkillData;
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
    for x in 0..item_list.len() {
        let random_item = &mut item_list[x];
        if random_item.get_flag().value == 16 {
            let iid = random_item.iid.get_string().unwrap();
            if iid != "IID_メティオ_G004" { random_item.get_flag().value = 0;  }
            if iid == "IID_メティオ" { 
                random_item.endurance = 1; 
            }
        }
        let item_flag = random_item.get_flag().value;
        if enums::ITEM_BLACK_LIST.lock().unwrap().iter().find(|y| **y == x as i32).is_some() { continue; }
        if crate::utils::str_contains(random_item.name, "MIID_Ring") { continue; }
        if !has_name(random_item, true) { continue; }
        if random_item.is_unknown() { continue; }
        if item_flag & 16777216 != 0 { continue; } //Bless
        if item_flag & 33554432 != 0 { continue; } //Breath
        if item_flag & 67108864 != 0 { continue; }  //Dragon
        if item_flag & 134217728 != 0 { continue; } //Bullet
        if item_flag & 131072 != 0 { continue; } // Bento
        if item_flag & 32768 != 0 { continue; } // AI 
        if item_flag & 510 == 0 { RANDOM_ITEM_POOL.lock().unwrap().push(random_item.parent.index); }
    }
    println!("{} items are in the Random Item Pool", RANDOM_ITEM_POOL.lock().unwrap().len());
    item_rando::WEAPONDATA.lock().unwrap().intitalize();
}

pub fn random_item(item_type: i32, allow_rare: bool) -> &'static Il2CppString {
    let item_list_size = RANDOM_ITEM_POOL.lock().unwrap().len();
    let rng = Random::get_system();
    let base_price = 2500 - 50*GameVariableManager::get_number("G_ItemDropGauge"); 

    loop {
        let item_index = RANDOM_ITEM_POOL.lock().unwrap()[rng.get_value( item_list_size as i32 ) as usize];
        let item = ItemData::try_index_get(item_index);
        if item.is_none() { continue; }
        let random_item = item.unwrap();
        if item_type == 0 || item_type == 4 { //Item Script Replacement
            if random_item.is_inventory() || random_item.is_material() { continue; }
            if item_type == 4 && random_item.price < base_price { continue; } 
        }
        else if item_type == 1 {    // Gift/Reward Items
            if random_item.usetype >= 32 && random_item.usetype <= 39 { continue; }
            if random_item.usetype == 0 && ( random_item.kind != 17 && random_item.kind != 18 ){ continue; }  
        }
        else if item_type == 2 {    // Exploration Drops
            let exploration = GameVariableManager::get_number("G_HubItem");
            let iid = random_item.iid.get_string().unwrap();
            if iid == "IID_スキルの書・離" || iid == "IID_スキルの書・破" {  continue; }    // No Adept/Expert Book
            let kind = random_item.kind;
            if kind == 17 && random_item.price > 5000 { continue; }     // Bond limited to 1000
            if kind == 18 && random_item.price >= 1000 { continue; }    // Limit Money to 1000g
            if kind == 13 || ( kind < 10 && kind != 0 ) { continue; }   // No Key Item or Weapon/Staff Related Items
            if ( kind < 17 && kind > 13 ) || (kind == 10 && random_item.usetype == 21) { continue; } // No Ores or Stat Boosters
            if exploration == 1  && random_item.usetype == 33 { continue; } 
            if exploration == 2 && random_item.usetype == 32 { continue; }
            if exploration == 3 && ( random_item.usetype == 33 || random_item.usetype == 32 ) { continue; }
            if random_item.usetype == 35 { continue; }
        }
        if random_item.get_flag().value & 1 != 0 && !allow_rare { continue; }
        return random_item.iid;
    }
}

// For item replacement
pub fn get_random_item(item: &'static Il2CppString, allow_rare: bool) -> &'static Il2CppString {
    if let Some(item_check) = ItemData::get(&item.get_string().unwrap()) {
        let flag = item_check.get_flag().value;
        if flag & 1 == 1 { return item;  }
        if enums::ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == item_check.parent.index).is_some() { return item; }
    }
    else { return item; }
    let price = ItemData::get(&item.get_string().unwrap()).unwrap().price;
    let mut count = 0;
    loop {
        count += 1;
        let new_iid = random_item(0, allow_rare);
        let new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
        if new_price < price * CONFIG.lock().unwrap().replaced_item_price / 100 { continue; }
        if count >= 150 { return new_iid; }
        return new_iid;
    }
}

pub fn has_name(this: &ItemData, include_money: bool) -> bool {
    unsafe {  if crate::utils::is_null_empty(this.name, None) { return false;  }  }
    let item_name = Mess::get(this.name ).get_string().unwrap();
    if item_name.len() != 0 { return true }
    else if include_money {
        return this.kind == 17 || this.kind == 18  ;    // If Money or bond
    }
    return false; 
}

pub fn randomize_well_rewards() {
    if GameVariableManager::get_number("G_Random_Item") == 0  { return; }
    if CONFIG.lock().unwrap().random_gift_items != 0 {
        let rare_item = CONFIG.lock().unwrap().random_gift_items == 2;
        let rlist = RewardData::get_list_mut().unwrap();
        for x in 0..rlist.len() {
            for y in 0..rlist[x].len() {
                let iid = rlist[x][y].iid;
                let item = ItemData::get(&iid.get_string().expect(format!("Bad Item ID in Reward Data\nItem {} in List {}", y, x).as_str()));
                if item.is_none() { continue; }
                let price = item.unwrap().price;
                let mut new_iid;
                let mut new_price;
                let mut count = 0;
                loop {
                    new_iid = random_item(1, rare_item);
                    new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
                    count += 1;
                    if new_price < 3*price || count >= 50 { break; }
                }
                rlist[x][y].set_iid(new_iid);  
            }
        }
    }
    ["アイテム交換_期待度１", "アイテム交換_期待度２", "アイテム交換_期待度３", "アイテム交換_期待度４", "アイテム交換_期待度５" ].iter().for_each(|&x|{
            if let Some(well_items) = RewardData::try_get_mut(x) {
                let mut in_set: [bool; 1000] = [false; 1000];
                for y in 0..well_items.len() {
                    let iid = well_items[y as usize].iid;
                    let price = ItemData::get(&iid.get_string().expect("Bad Item in Well Item List")).expect(format!("Invalid Item ID in Well Item {}", y).as_str()).price;
                    let mut new_price; 
                    let mut item_index;
                    let mut new_iid; 
                    let curent_reward = &well_items[y as usize];
                    let mut count = 0;
                    loop {
                        new_iid = random_item(1, true);
                        new_price = ItemData::get(&new_iid.get_string().unwrap()).unwrap().price;
                        item_index = ItemData::get(&new_iid.get_string().unwrap()).unwrap().parent.index;
                        if new_price > 3*price { count += 1; continue; }
                        if count < 50 && in_set[item_index as usize] { count += 1; continue; }
                        if count >= 50 { break; }
                        if !in_set[item_index as usize] { break; }
                    }
                    let new_reward = RewardData::instantiate().unwrap();
                    new_reward.ctor();
                    new_reward.set_iid(new_iid);
                    let new_item = ItemData::get(&new_iid.get_string().unwrap()).unwrap();
                    if new_item.get_flag().value & 1 != 0 || ( new_item.kind == 18 || new_item.kind == 17 ) {   // If rare or money / bond
                        new_reward.ratio = 2.5;
                        new_reward.min = 2.5;
                        new_reward.max = 2.5;
                    }
                    else {
                        new_reward.ratio = curent_reward.ratio;
                        new_reward.min = curent_reward.min;
                        new_reward.max = curent_reward.max;
                    }
                    well_items.add(new_reward);
                    in_set[item_index as usize] = true; 
                }
            }
        }
    );
        
    println!("Complete Randomization of Gift/Well Items");
    shop::randomize_hub_random_items();
}

pub fn change_liberation_type() {
    if !GameVariableManager::get_bool("G_Cleared_M002") { return; }
    let liberation = ItemData::get_mut("IID_リベラシオン").unwrap();
    if GameVariableManager::get_number("G_Liberation_Type") != 0 { 
        let l_type = GameVariableManager::get_number("G_Liberation_Type") as u32;
        liberation.kind = l_type;
        if l_type == 4 {
            liberation.range_i = 2;
            liberation.range_o = 3;
            liberation.set_cannon_effect("弓砲台".into());
            liberation.on_complete();
            liberation.get_equip_skills().add_sid("SID_飛行特効",4, 0);
        }
        else if l_type == 5 || l_type == 6 {
            liberation.range_o = 2;
            liberation.range_i = 1;
            if l_type == 6 {
                liberation.set_cannon_effect("魔砲台炎".into());
                liberation.set_hit_effect( "エルファイアー".into());
                liberation.on_complete();
            }
            else { liberation.get_give_skills().add_sid("SID_毒",3, 0); }
        }
        else if l_type == 8 {
            liberation.get_equip_skills().add_sid("SID_気功",4, 0);
            liberation.get_equip_skills().add_sid("SID_２回行動",4,0);
        }
        else {
            liberation.range_i = 1;
            liberation.range_o = 1;
        }
        return; 
    }

    let hero_unit = unsafe { utils::unit_pool_get_hero(false, None) };
    if hero_unit.is_none() { return; }

    let kinds = hero_unit.unwrap().get_job().get_equippable_item_kinds();
    let mut liberation_type = 1; //Sword
    for i in 0..kinds.len() {
        if kinds[i] == 7 || kinds[i] >= 9 { continue; }
        if kinds[i] == 0 { continue; }
        liberation_type = kinds[i];
    }

    liberation.kind = liberation_type as u32;
    if liberation_type == 4 {
        liberation.range_o = 3;
        liberation.range_i = 2;
        liberation.set_cannon_effect("弓砲台".into());
        liberation.on_complete();
        liberation.get_equip_skills().add_skill(SkillData::get("SID_飛行特効").unwrap(),4, 0);
    }
    else if liberation_type == 5 || liberation_type == 6 {
        liberation.range_i = 1;
        liberation.range_o = 2;
        if liberation_type == 6 {
            liberation.set_cannon_effect("魔砲台炎".into());
            liberation.set_hit_effect( "エルファイアー".into());
            liberation.on_complete();
        }
        else { liberation.get_give_skills().add_sid("SID_毒",4, 0); }
    }
    else if liberation_type == 8 {
        liberation.get_equip_skills().add_sid("SID_気功",4, 0);
        liberation.get_equip_skills().add_sid("SID_２回行動",4,0);
    }
    else {
        liberation.range_i = 1;
        liberation.range_o = 1;
    }
    GameVariableManager::make_entry("G_Liberation_Type", liberation_type);
    GameVariableManager::set_number("G_Liberation_Type", liberation_type);
    println!("Liberation changed to weapon type {}", liberation_type);
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

