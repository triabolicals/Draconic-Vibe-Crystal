use super::*;
use data::WeaponDatabase;
pub use engage::{
    mess::*,
    hub::access::*,
    util::get_singleton_proc_instance,
    gamevariable::*, gameuserdata::*, random::*,
    gamedata::{*, item::*},
};
use crate::{continuous::get_continious_total_map_complete_count, utils::*};
use engage::transporter::Transporter;
use engage::unit::{Unit, UnitItem};
use crate::randomizer::item::unit_items::{add_generic_weapons, get_max_job_weapon_type, get_number_of_usable_weapons};
use crate::utils::max;

pub mod unit_items;
pub mod shop;
pub mod data;
pub mod hub;
pub mod well;

pub fn get_random_item(item: &'static Il2CppString, allow_rare: bool) -> &'static Il2CppString {
    if let Some(item_check) = ItemData::get(item) {
        let flag = item_check.flag.value;
        if flag & 1 == 1 { return item;  }
        if let Some(item) = ItemData::get(item).filter(|x| x.use_type == 23 || x.use_type == 24 || x.use_type == 40 || x.use_type == 41){
            if item.use_type == 23 && DVCVariables::ClassMode.get_value()& 1 != 0 && DVCVariables::Reclassing.get_value() == 0 {
                if let Some(second_seal) = ItemData::get("IID_チェンジプルフ") { second_seal.add_inventory(1); }
            }
            item.add_inventory(1);
        }
        let pool = GameData::get_item_pool();
        ItemData::get(pool.random_item(0, allow_rare)).unwrap().iid
    }
    else { item }
}
pub fn change_liberation_type() -> i32 {
    if !DVCVariables::is_main_chapter_complete(1) { return 1; }
    for x in ["IID_リベラシオン", "IID_リベラシオン改_ノーマル", "IID_リベラシオン改"]  {
        if let Some(liberation) = ItemData::get_mut(x){
            let l_type =
                if GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) != 0 { GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) }
                else if let Some(hero_unit) = engage::unit::UnitPool::get_hero(false) {
                    let mut liberation_type = 1;
                    let kind = get_max_job_weapon_type(hero_unit.job);
                    if kind != 0 { liberation_type = kind; }
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
                    liberation.equip_skills.add_sid("SID_飛行特効",SkillDataCategorys::Equip, 0); // Flier Effectiveness
                }
                5 => {
                    liberation.range_i = 1;
                    liberation.range_o = 2;
                    liberation.give_skills.add_sid("SID_毒",SkillDataCategorys::Item, 0);    // Poison for Dagger
                }
                6 => {
                    liberation.range_i = 1;
                    liberation.range_o = 2;
                }
                8 => { liberation.equip_skills.add_sid("SID_２回行動",SkillDataCategorys::Equip,0); }
                9 => {
                    liberation.equip_condition = Some("SID_竜石装備".into());
                    liberation.flag.value |= 0x4000000;
                    liberation.range_o = 1;
                    liberation.range_i = 1;
                }
                10 => {
                    liberation.equip_condition = Some("SID_弾丸装備".into());
                    liberation.flag.value |= 134217728;
                    liberation.range_i = 2;
                    liberation.range_o = 10;
                }
                _ => {
                    liberation.range_i = 1;
                    liberation.range_o = 1;
                }
            }
            liberation.on_completed();
        }
    }
    GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE)
}

pub fn change_misercode_type(){
    let value = GameVariableManager::get_number(DVCVariables::MISERCODE_TYPE);
    let veyle_class = GameVariableManager::get_number("G_JG_PID_ヴェイル");
    if veyle_class == 185670709 || veyle_class == -1998645787 { return; }
    let misercode_type = if value == 0 ||  value == 7 || value > 9 {
        GameVariableManager::set_number(DVCVariables::MISERCODE_TYPE, 5);  
        5
    }
    else { value };
    if let Some(misercode) = ItemData::get_mut("IID_ミセリコルデ"){
        misercode.equip_skills.clear();
        misercode.give_skills.clear();
        misercode.range_o = 2;
        misercode.range_i = 1;
        misercode.kind = misercode_type as u32;

        match misercode_type  {
            4 => {
                misercode.equip_skills.add_sid("SID_飛行特効",SkillDataCategorys::Equip, 0);
                misercode.range_o = 3;
                misercode.range_i = 2;
            }
            5 => { misercode.give_skills.add_sid("SID_毒",SkillDataCategorys::Item, 0); }
            8 => {
                misercode.range_o = 1;
                misercode.equip_skills.add_sid("SID_２回行動",SkillDataCategorys::Equip,0);
            }
            9 => {
                misercode.equip_condition = Some("SID_竜石装備".into());
                misercode.flag.value |= 0x4000000;
                misercode.range_i = 2;
                misercode.range_o = 1;
            }
            10 => {
                misercode.equip_condition = Some("SID_弾丸装備".into());
                misercode.flag.value |= 134217728;
                misercode.range_i = 2;
                misercode.range_o = 10;
            }
            _ => {}
        }
        misercode.on_completed();
    }
}
pub fn get_unit_avail_weapon_levels(unit: &Unit) -> [i32; 10] {
    let mut levels = [0; 10];
    let level = unit.level as i32 + unit.internal_level as i32;
    let enemy = unit.person.get_asset_force() == 1;
    let story_weapon_level =
        if !DVCVariables::is_main_chapter_complete(4) { 1 }
        else {
            match level {
                0..10 => { 1 }  // D
                10..15 => { // D or C
                    if enemy { if Random::get_game().get_value(100) < (level - 9) * 10 { 2 } else { 1 } } else { 2 }
                }
                15..20 => { 2 } // C
                20..25 => { // C or B
                    if enemy { if Random::get_game().get_value(100) < (level - 19) * 10 { 3 } else { 2 } } else { 3 }
                }
                25..30 => { 3 }    // B
                30..40 => { // B or A
                    3 + ((Random::get_game().get_value(30) < (level - 30)) as i32)
                }
                _ => { if enemy { 5 } else { 4 } }
            }
        };

    for x in 1..10 {
        if unit.weapon_mask.value & (1<<x) != 0 {
            let job_level = unit.job.get_max_weapon_level(x as i32);
            let level = if story_weapon_level < job_level { story_weapon_level } else { job_level };
            levels[x] = level;
        }
    }
    levels
}
#[skyline::hook(offset=0x20193e0)]
pub fn calc_reward(name: &Il2CppString, optional_method: OptionalMethod) -> Option<&'static mut List<ItemData>> {
    let mut list = call_original!(name, optional_method);
    let mode = DVCVariables::RandomGifts.get_value();
    if mode != 0 {
        let pool = GameData::get_item_pool();
        let is_rare = mode == 2;
        if let Some(item_list) = list.as_mut() {
            item_list.iter_mut().filter(|item| item.kind < 13 && item.use_type != 34)
                .for_each(|item| {
                    let mut count = 0;
                    let weapon_level = item.get_weapon_level();
                    loop {
                        if let Some(new_item) = ItemData::get_mut(pool.random_item(1, is_rare))
                            .filter(|x|
                                (x.get_weapon_level() == 0 || x.get_weapon_level() <= weapon_level)
                                    && x.price < 10000 && x.kind != 18 && x.kind != 19 && x.equip_condition.is_none())
                        {
                            *item = new_item;
                        }
                        count += 1;
                        if count > 50 { break; }
                    }
                });
        }
    }
    list
}
pub fn process_non_unit_inventory_item(unit_item: &UnitItem) {
    match unit_item.item.kind {
        14 => { GameUserData::add_iron(1); },
        15 => { GameUserData::add_steel(1); },
        16 => { GameUserData::add_silver(1); },
        17 => { GameUserData::add_bond(unit_item.item.price); },
        18 => { GameUserData::set_gold(GameUserData::get_gold() + unit_item.item.price); }
        _ => { unit_item.item.add_inventory(1); }
    }
}
pub fn adjust_non_unit_items_inventory() {
    let mut count = 0;
    for x in 0..999 {
        if let Some(convoy_item) = Transporter::get(x).filter(|i| i.item.item.kind > 13 || i.item.item.is_inventory()){
            process_non_unit_inventory_item(convoy_item.item);
            Transporter::delete(x);
            count += 1;
            // println!("Convoy Item #{}: {} was removed", x+1, Mess::get_name(convoy_item.item.item.iid));
        }
    }
    for x in 1..250 {
        if let Some(unit) = engage::unit::UnitPool::get(x).filter(|unit| unit.force.is_some_and(|f| (1 << f.force_type) & 25 != 0)) {
            let mut changed = false;
            unit.item_list.unit_items.iter().filter_map(|item| item.as_ref().filter(|i| i.item.is_inventory() || i.item.kind > 13))
                .for_each(|item| {
                    process_non_unit_inventory_item(item);
                    // println!("Unit Item: {} was removed from {}", Mess::get_name(item.item.iid), unit.get_name());
                    item.dispose();
                    count += 1;
                    changed = true;
                });
            if changed { unit.item_list.put_engage_item(unit.god_link.or(unit.god_unit), unit.status.value & 8388608 != 0); }
        }
    }
    // if count > 0 { println!("Total of {} items removed.", count); }

}

#[unity::hook("App", "ItemRefineData", "TryGetFromItem")]
pub fn item_refine_data_try_get(item: Option<&'static ItemData>, method_info: OptionalMethod) -> Option<&'static List<ItemRefineData>>{
    if GameUserData::get_sequence() != 0 {
        if let Some(item) = item {
            if DVCFlags::RefineItem.get_value() && item.parent.index > 3 {
                let new_item = RandomizedGameData::get_read().refine
                    .get(item.parent.index as usize)
                    .and_then(|v| ItemData::try_get_hash(*v));

                return call_original!(new_item, method_info);
            }
        }
    }
    call_original!(item, method_info)
}