use engage::menu::BasicMenuItemAttribute;
use engage::unitpool::UnitPool;
use utils::clamp_value;

use super::*;
use super::data::*;
use crate::{assets::animation::MONSTERS, continuous::{get_story_chapters_completed, get_continious_total_map_complete_count}, randomizer::person::unit::has_sid};

pub struct PlayerRandomWeapons;
impl ConfigBasicMenuItemSwitchMethods for PlayerRandomWeapons {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        if is_main && CONFIG.lock().unwrap().random_item == 0 {
            this.help_text = "Enable item randomization to enable this setting.".into();
            this.update_text();
            return BasicMenuResult::new();
        }
        let state = if is_main { CONFIG.lock().unwrap().player_inventory } else {  GameVariableManager::get_bool(DVCVariables::PLAYER_INVENTORY) };
        let result = ConfigBasicMenuItem::change_key_value_b(state);
        if state != result {
            if is_main { CONFIG.lock().unwrap().player_inventory = result; }
            else { GameVariableManager::set_bool(DVCVariables::PLAYER_INVENTORY, result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let state = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().player_inventory } else {  GameVariableManager::get_bool(DVCVariables::PLAYER_INVENTORY) };
        this.help_text = if state { "Player units can be recruited with non-standard weapons."}
            else { "Player units are recruited with standard weapons." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let state = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().player_inventory } else {  GameVariableManager::get_bool(DVCVariables::PLAYER_INVENTORY) };
        this.command_text = if state { "Random"} else {"Standard"}.into();
    }
}
fn prw_build_attrs(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_number(DVCVariables::ITEM_KEY) != 0 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_prw() ->  &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<PlayerRandomWeapons>("Player Starting Inventory");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = prw_build_attrs as _);
    switch
}

//Has Healing staff
pub fn replace_staves(item_list: &UnitItemList){
    for x in 0..8 {
        let item = item_list.get_item(x);
        if item.is_some() {
            let staff = &item.unwrap();
            let kind = staff.item.get_kind(); 
            if kind == 7 { 
                let staff_name = staff.item.iid.to_string();
                if staff_name == "IID_ライブ" || staff_name == "IID_リブロー" || staff_name == "IID_リライブ" { 
                    staff.ctor_str("IID_傷薬");
                }
                else { staff.ctor_str("IID_特効薬"); }
            }
        }
    }
}
pub fn dispose_item_type(item_list: &UnitItemList, item_kind: i32){
    for x in 0..8 {
        if let Some(item) = item_list.get_item(x){
            let kind = item.item.get_kind(); 
            if  item.item.flag.value & 128 == 0 && kind ==  item_kind { item.dispose(); }
        }
    }
}

pub fn dispose_all_but_drops(item_list: &UnitItemList){
    for x in 0..8 {
        if !item_list.unit_items[x].as_ref().unwrap().is_drop() {
            if let Some(item) = item_list.get_item(x as i32 ) {
                if  !item.is_drop() { item.dispose(); }
            }
        }
    }
}
pub fn dispose_unusables(unit: &Unit) {
    for x in 0..8 {
        if unit.item_list.unit_items[x].as_ref().unwrap().item.parent.index < 3 { continue; }
        if let Some(item) = unit.item_list.get_item(x as i32 ) {
            if item.item.flag.value & 128 == 0 && item.flags & 2 == 0 && item.item.kind > 0 { // Not Engage Weapon or Not Drop
                if !unit.can_equip(x as i32, true, true){ item.dispose(); }
            }
        }
    }
    for x in 0..8 {
        let item = unit.item_list.unit_items[x].as_ref().unwrap();
        if item.item.parent.index < 3 { continue; }
        if item.is_empty() {
            for y in x+1..unit.item_list.unit_items.len() {
                if !unit.item_list.unit_items[y].as_ref().unwrap().is_empty() {
                    unit.item_list.move_item(y as i32, x as i32);
                    break;
                }
            }
        }
    }
}

pub fn remove_duplicates(item_list: &UnitItemList) {
    for x in 0..8 {
        if let Some(item) = item_list.get_item(x as i32) { 
            let iid1 = item.item.iid.to_string();
            for y in x+1..8 {
                if let Some(item2) = item_list.get_item(y as i32 ) {
                    let iid2 = item2.item.iid.to_string();
                    if item2.item.parent.index == item.item.parent.index { 
                        if item2.is_drop() { item.flags |= 2; }
                        item2.dispose(); 
                    }
                    else if (iid1 == "IID_毒消し" || iid1 == "IID_特効薬") && iid2 == "IID_傷薬" { item2.dispose();  }
                    else if iid1 == "IID_傷薬" && (iid2 == "IID_特効薬" || iid2 == "IID_毒消し")  { item.dispose(); }
                }
            }
        }
    }

    for x in 0..8 {
        let item = item_list.unit_items[x].as_ref().unwrap();
        let iid1 =  item.item.iid.to_string();
        if iid1 == "IID_エンゲージ枠" || iid1 == "IID_不明" { continue; } 
        if item.is_empty() {
            for y in x+1..item_list.unit_items.len() {
                if !item_list.unit_items[y].as_ref().unwrap().is_empty() {
                    item_list.move_item(y as i32, x as i32);
                    break;
                }
            }
        }
    }
}

pub fn adjust_melee_weapons(unit: &Unit) {
    if unit.person.get_asset_force() == 0 { return; }
    let job = unit.get_job();
    if unit.get_capability(1, true) < unit.get_capability(6, true) {
        let mag_weapons = ["IID_いかづちの剣", "IID_ほのおの槍", "IID_かぜの大斧", "IID_光の弓"];
        for x in 0..unit.item_list.unit_items.len() {
            let item = unit.item_list.get_item(x as i32);
            if item.is_none() { continue;}
            let weapon = item.unwrap();
            let kind = weapon.item.kind;
            if !can_change_weapon(weapon, true) { continue; }
            let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
            if weapon.item.get_weapon_level() <= weapon_level && ( ( unit.selected_weapon_mask.value & ( 1 << kind ) != 0 || unit.selected_weapon_mask.value == 0 ) ) {
               weapon.ctor_str(mag_weapons[kind as usize -1 ]);
            }
        }
        return;
    }
}
pub fn get_number_of_usable_weapons(unit: &Unit) -> i32 {
    let mut count = 0;
    for x in 0..8 {
        if let Some(item) = unit.item_list.get_item(x) {
            if item.item.flag.value & 128 == 0 {
                if unit.can_equip(x, false, true)  { count += 1;}
            }
        }
    }
    count
}

pub fn can_change_weapon(weapon: &UnitItem, melee: bool) -> bool {
    if !weapon.item.is_weapon() { return false; }
    let flag = weapon.item.get_flag().value; 
    if flag & 128 != 0 || flag & 2 != 0 { return false; }
    let kind = weapon.item.kind;
    if melee { return kind > 0 && kind < 5;  }
    else { return kind > 0 && kind < 9; }
}

pub fn assign_tomes(unit: &Unit) {
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").to_string() == unit.person.pid.to_string();
    let is_veyle = unit.person.pid.to_string() == PIDS[32];
    let rng = Random::get_system();
    let story_chapter = get_story_chapters_completed();
    let continous = GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 3;
    let is_player = unit.person.get_asset_force() == 0;
    if weapon_mask.value & 64 != 0 && (!is_vander && !is_veyle) {
        dispose_item_type(unit.item_list, 6);
        if GameVariableManager::get_bool(DVCVariables::PLAYER_INVENTORY) && is_player {
            let mut count = 0;
            while count < 10 {
                if let Some(new_item) = data::WEAPONDATA.get().unwrap().get_random_weapon(5, false) {
                    if new_item.get_weapon_level() <= job.weapon_levels[6]  { 
                        unit.item_list.add_item_no_duplicate(new_item); 
                        break;
                    }
                    count += 1;
                }
            }
        }
        else {
            let value = rng.get_value(10);
            let job_level = job.get_max_weapon_level(6);
            if job.is_low() && unit.level <= 20  { 
                if DVCVariables::is_main_chapter_complete(9) || (continous && story_chapter >= 8 ){ 
                    if value < 6 { unit.item_list.add_iid_no_duplicate("IID_エルファイアー"); }
                    else { unit.item_list.add_iid_no_duplicate("IID_エルウィンド"); }
                }
                else { 
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_ファイアー");  }
                    else {unit.item_list.add_iid_no_duplicate("IID_ティラミス魔道書");  }
                }
                if DVCVariables::is_main_chapter_complete(7) || (continous && story_chapter >= 6 ){
                    if value & 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_サージ");  }
                    else if DVCVariables::is_main_chapter_complete(10) || (continous && story_chapter >= 10) { unit.item_list.add_iid_no_duplicate("IID_エルサージ"); }
                    else { unit.item_list.add_iid_no_duplicate("IID_サージ");  }
                }
            }
            else {
                if ( unit.level < 10 && job.is_high() ) || ( job.is_low() && unit.level < 30 ) || job_level < 4 {
                    if value < 6 { unit.item_list.add_iid_no_duplicate("IID_エルファイアー"); } //Elfire
                    else { unit.item_list.add_iid_no_duplicate("IID_エルウィンド"); }   //Elwind
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_エルサンダー"); }  //Elthunder 50%
                }
                else {
                    if job_level > 4 { // S-Rank
                        if value % 4 == 0 { unit.item_list.add_iid_no_duplicate("IID_ノヴァ");  }  // Nova
                        else if value % 4 == 1 { unit.item_list.add_iid_no_duplicate("IID_ボルガノン");   } // Bol
                        else if value % 4 == 2 { unit.item_list.add_iid_no_duplicate("IID_エクスカリバー");  } //Excal
                        else { unit.item_list.add_iid_no_duplicate("IID_トロン"); } // Thoron
                        if ( unit.person.get_asset_force() != 0 && value == 1) && ( DVCVariables::is_main_chapter_complete(21) || GameVariableManager::get_bool("G_Cleared_S009")) {
                            unit.item_list.add_iid_no_duplicate("IID_メティオ_G004");  //Meteor
                        }
                        else { unit.item_list.add_iid_no_duplicate("IID_トロン"); }
                    }
                    else {
                        if value % 3 == 0 { unit.item_list.add_iid_no_duplicate("IID_ボルガノン");   } // Bol
                        if value % 3 == 1 { unit.item_list.add_iid_no_duplicate("IID_エクスカリバー");  } //Excal
                        if value % 3 == 2 { unit.item_list.add_iid_no_duplicate("IID_エルサージ");  } //Excal
                        if value < 5 { unit.item_list.add_iid_no_duplicate("IID_トロン"); } // Thoron
                    }
                }
            }
        }
    }
}

pub fn assign_unique_items(unit: &Unit) {
    unit_items::remove_duplicates(unit.item_list);
    let pid = unit.person.pid.to_string();
    let veyle = pid == PIDS[32];
    let job_hash = unit.job.parent.hash;
    let enemy = unit.person.get_asset_force() != 0;
    if veyle { 
        magic_dagger_weapon_change(unit.get_job());
         unit.item_list.add_iid_no_duplicate("IID_ミセリコルデ");
    }  // Misercode for Veyle
    if job_hash == 185670709 || veyle  {  unit.item_list.add_iid_no_duplicate("IID_オヴスキュリテ");  } // Obscurite for Veyle / Fell Child Veyle

    if unit.job.get_max_weapon_level(9) > 1 &&  unit.job.mask_skills.find_sid("SID_竜石装備".into()).is_some() && (pid != PIDS[36] && pid != PIDS[37]) { 
        dispose_item_type(unit.item_list, 9);   // Dragonstone for classes that have the equip dragonstone skill
        if let Some(stone1) = WEAPONDATA.get().unwrap().get_dragon_stone(enemy) { unit.item_list.add_item_no_duplicate(stone1);  }
        if let Some(stone2) = WEAPONDATA.get().unwrap().get_dragon_stone(enemy) { unit.item_list.add_item_no_duplicate(stone2); }
    }
    // Mage Canon
    if unit.job.get_max_weapon_level(9) > 1 && unit.job.mask_skills.find_sid("SID_弾丸装備".into()).is_some() {
        dispose_item_type(unit.item_list, 9);
        let len = WEAPONDATA.get().unwrap().bullet_list.len();
        if len > 1 {
            let rng = Random::get_game();
            let bullet_1 = WEAPONDATA.get().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
            unit.item_list.add_item_no_duplicate(ItemData::try_index_get(bullet_1).unwrap());
            let bullet_2 = WEAPONDATA.get().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
            unit.item_list.add_item_no_duplicate(ItemData::try_index_get(bullet_2).unwrap());
        }
    }
    if pid == PIDS[36] {
        unit.item_list.add_iid_no_duplicate("IID_邪竜石"); 
        unit.item_list.add_iid_no_duplicate("IID_邪竜石_騎馬特効"); 
    }
    if pid == PIDS[37] {    
        unit.item_list.add_iid_no_duplicate("IID_邪竜石"); 
        unit.item_list.add_iid_no_duplicate("IID_邪竜石_騎馬特効");
    }
}

pub fn assign_staffs(unit: &Unit) {
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask();
    let jid = job.jid.to_string();
    let rng = Random::get_system();
    let is_player = unit.person.get_asset_force() == 0;

    let staff_level = if  has_sid(unit, "SID_杖使い＋＋") { 4 } 
        else if has_sid(unit, "SID_杖使い＋") { 3 }
        else if has_sid(unit, "SID_杖使い") { 2 } 
        else { job.get_max_weapon_level(7) };
    let total_level = unit.level as i32 + unit.internal_level as i32;
    if weapon_mask.value & ( 1 << 7 ) == 0 { replace_staves(unit.item_list); }
    else if weapon_mask.value & ( 1 << 7 ) != 0 && staff_level > 0 {
        dispose_item_type(unit.item_list, 7);
        if is_player && !GameVariableManager::get_bool(DVCVariables::PLAYER_INVENTORY) { //Player Staff users
            if job.is_low() { //Fracture for Wing Tamer Hortensia
                if jid == "JID_スレイプニル下級" { unit.item_list.add_iid_no_duplicate("IID_コラプス"); }
                if unit.level < 10 { unit.item_list.add_iid_no_duplicate("IID_ライブ"); }
                else if unit.level < 15 {  unit.item_list.add_iid_no_duplicate("IID_リライブ") }
                else {  unit.item_list.add_iid_no_duplicate("IID_リブロー");  }
            }
            else {
                if jid == "JID_スレイプニル" { unit.item_list.add_iid_no_duplicate("IID_コラプス");  } 
                if jid == "JID_ハイプリースト" {    // Warp/Fortify for High Priest
                    let rng = Random::get_game();
                    let value = rng.get_value(10);
                    if value % 2 == 0 { unit.item_list.add_iid_no_duplicate("IID_ワープ");  }
                    else {unit.item_list.add_iid_no_duplicate("IID_レスキュー");  }
                    unit.item_list.add_iid_no_duplicate("IID_リザーブ");
                }
                else { unit.item_list.add_iid_no_duplicate("IID_リブロー"); }  // physic for the rest of staffers instead
            }
        }
        else {
            for x in 1..4 {
                if x == 3 && rng.get_value(3) > 1 {  continue; }
                if let Some(staff_) = WEAPONDATA.get().unwrap().get_staff(total_level, x, staff_level){
                    unit.item_list.add_item_no_duplicate(staff_);
                }
            }
        }
    };
}

pub fn adjust_missing_weapons(unit: &Unit) {
    if get_number_of_usable_weapons(unit) < 1 {
        println!("{} has no usable weapons!", Mess::get_name(unit.person.pid));
        dispose_unusables(unit);
        if unit.job.mask_skills.find_sid("SID_弾丸装備".into()).is_some() {  //Mage Canon
            let len = WEAPONDATA.get().unwrap().bullet_list.len();
            let rng = Random::get_game();
            if len > 1 {
                let index1 = WEAPONDATA.get().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
                unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index1).unwrap());
                let index2 = WEAPONDATA.get().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
                unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index2).unwrap());
            }
            return;
        }
        add_generic_weapons(unit);
        unit.auto_equip();
    }
    let total_level = unit.level as i32 + unit.internal_level as i32;
    if get_number_of_usable_weapons(unit) < 2 && total_level > 15 {
        println!("{} has only one weapons!", Mess::get_name(unit.person.pid));
        if let Some(uitem) = unit.item_list.unit_items.iter().flatten().find(|x| x.is_weapon() ) {
            println!("Found Weapon: {}", Mess::get_name(uitem.item.iid));
            if let Some(item) = WEAPONDATA.get().unwrap().get_additional_weapon(uitem.item) {
                unit.item_list.add_item_no_duplicate(item);
            }
        }
    }
}

pub fn simple_replacement(unit: &Unit) {
    let job = unit.get_job();
    let mut combine_mask = unit.selected_weapon_mask.value | job.get_weapon_mask().value;
    unit.item_list.unit_items.iter().flatten()
        .filter(|uitem| uitem.item.is_weapon() && uitem.item.flag.value & 128 == 0)
        .for_each(|uitem|{
            if let Some(new_item) = WEAPONDATA.get().unwrap().get_simple_replacement(uitem.item, combine_mask, job.weapon_levels) {
                println!("Unit {} ({}): Simple Replacing {} -> {} ({} -> {})", Mess::get_name(unit.person.pid), Mess::get_name(unit.job.jid), Mess::get_name(uitem.item.iid), Mess::get_name(new_item.iid), uitem.item.power, new_item.power);
                uitem.ctor(new_item);
                combine_mask &= !(1 << new_item.kind); // remove kind mask
            }
        }
    );
    remove_duplicates(unit.item_list);
    dispose_unusables(unit);
    if combine_mask & 62 != 0 && get_number_of_usable_weapons(unit) < 2 { add_generic_weapons(unit); }
}



pub fn add_generic_weapons(unit: &Unit) {
    //if unit.person.get_asset_force() != 0 { // Making sure enemies have weapon
    let job = unit.get_job();
    let job_mask = job.get_weapon_mask().value;
    println!("Adding Weapons for {} ({}), with Selected Mask {} / {}", Mess::get(unit.person.get_name().unwrap()).to_string(), Mess::get(unit.get_job().name), unit.selected_weapon_mask.value, job_mask);
    let jid = job.jid.to_string();
    if jid == "JID_ボウナイト" { unit.item_list.add_item_no_duplicate(ItemData::get("IID_銀の弓").unwrap());  }
    if jid == "JID_エンチャント" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_HPの薬").unwrap());  
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_力の薬").unwrap());  
    }
    let combine_mask = unit.selected_weapon_mask.value | job_mask;
    let unit_level = unit.level as i32 + unit.internal_level as i32;
    let player = unit.person.get_asset_force() == 0;
    let mut weapon_rank = if unit_level < 10 { 1 }
        else if unit_level < 21 { 2 }
        else { 3 };

    let mut has_weapon: [bool; 10] = [false; 10];
    has_weapon[7] = true;
    unit.item_list.unit_items.iter()
        .flatten()
        .filter(|uitem| uitem.item.is_weapon() && uitem.item.flag.value & 128 == 0)
        .for_each(|uitem|{
            let kind = uitem.item.kind;
            let item_level = uitem.item.get_weapon_level();
            let weapon_level = job.get_max_weapon_level(kind as i32) as i32;
            if item_level <= weapon_level && combine_mask & ( 1 << kind ) != 0 { 
                if weapon_rank < item_level { weapon_rank = item_level; }
                has_weapon[kind as usize] = true; 
            
            }
            else if uitem.flags & 2 != 0 { uitem.dispose();  }
        }
    );
    for i in 1..9 {
        if has_weapon[i as usize] { continue; }
        let rank = clamp_value(weapon_rank, 0, job.get_max_weapon_level(i));
        let mut added_weapon = false;
        if combine_mask & (1 << i ) != 0 && rank > 0 {
            if player {
                if GameVariableManager::get_bool(DVCVariables::PLAYER_INVENTORY) {
                    if let Some(item) = WEAPONDATA.get().unwrap().get_random_weapon(i-1, false) {
                        if item.get_weapon_level() <= rank { 
                            unit.item_list.add_item_no_duplicate(item);  
                            added_weapon = true;
                        }
                    }
                }
            }
            else {
                if GameVariableManager::get_number(DVCVariables::ITEM_KEY) != 0 && get_continious_total_map_complete_count() > 10 {
                    if let Some(item) = WEAPONDATA.get().unwrap().get_random_weapon(i-1, true) {
                        if item.get_weapon_level() <= rank {
                            unit.item_list.add_item_no_duplicate(item); 
                            added_weapon = true;
                        }
                    }
                }
            }
            if !added_weapon {
                let mut search_rank = rank;
                while search_rank > 0 {
                    if let Some(item) = WEAPONDATA.get().unwrap().get_generic_weapon(i-1, search_rank) { 
                        unit.item_list.add_item_no_duplicate(item); 
                        break;
                    }
                    search_rank -= 1;
                }
            }
        }
    }
    if job.get_max_weapon_level(4) >= 2 && DVCVariables::is_main_chapter_complete(22) { unit.item_list.add_item_no_duplicate(ItemData::get("IID_長弓").unwrap());  } 

}

pub fn random_items_drops(unit: &Unit){
    let rng = Random::get_system();
    let mut none_count = 0;
    let mut rate = if GameUserData::get_difficulty(false) == 2 {1 / 2} else { 1 } * GameVariableManager::get_number(DVCVariables::ITEM_DROP_GAUGE_KEY);
        
    for x in 0..8 {
        if let Some(u_item) = unit.item_list.get_item(x) {
            if u_item.item.flag.value & 131 != 0 {  u_item.flags = 0; }
        }
    }
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_none() { continue; }
        let u_item = &mut item.unwrap();
        if u_item.is_drop(){
            if u_item.item.flag.value & 131 != 0  { //Unequip 
                u_item.flags = 0;  
                continue;
            }
            else if !u_item.is_equip() { 
                let new_item = get_random_item(u_item.item.iid, false);
                u_item.ctor_str(&new_item.to_string());
                u_item.flags = 2;
                none_count += 1;
            }
        } 
        let iid1 =  u_item.item.iid.to_string();
        if iid1 == "IID_無し" && none_count == 0 { 
            if rng.get_value(100) < rate {
                u_item.ctor_str(&random_item(4, false).to_string());
                u_item.flags = 2;
                rate = rate / 2;
            }
            else { none_count += 1; }
        } 
    }

}

fn magic_dagger_weapon_change(veyle_job: &JobData){
    // Change Veyle's Dagger to whatever class weapon she has. (low class only)
    if veyle_job.is_high() { 
        GameVariableManager::make_entry(DVCVariables::MISERCODE_TYPE, 5); 
        GameVariableManager::set_number(DVCVariables::MISERCODE_TYPE, 5);
        return; 
    }
    let mut misercode_type = 5; //Dagger
    veyle_job.get_equippable_item_kinds().iter().for_each(|&k| if ( k > 0 && k < 6 ) || k == 8 { misercode_type = k });
    GameVariableManager::make_entry(DVCVariables::MISERCODE_TYPE, misercode_type);
    GameVariableManager::set_number(DVCVariables::MISERCODE_TYPE, misercode_type);
    super::change_misercode_type();
}


// Meteor Adjustment
pub fn adjust_items() {
    let meteor = ItemData::get_mut("IID_メティオ").unwrap();
    meteor.endurance = 1;
    let index = meteor.parent.index;
    let item_list = ItemData::get_list_mut().unwrap();
    let can_trade = true; //CONFIG.lock().unwrap().enable_tradables_item;
    for x in 0..item_list.len() {
        let random_item = &mut item_list[x];
        let flag = random_item.flag.value;
        if can_trade && ( flag & 131 == 3 || flag & 131 == 2 ) { random_item.flag.value &= !2; }
        if random_item.get_flag().value & 16 != 0  {
            let iid = random_item.iid.to_string();
            if iid != "IID_メティオ_G004" {
                random_item.flag.value = 0; 
                random_item.price = 5000;
            }
            else {
                random_item.flag.value |= 3;
                random_item.price = 100;
            }
        }
    }
    // Unit Pool 
    for x in 1..250 {
        let o_unit: Option<&mut Unit> = UnitPool::get_by_index(x);
        if o_unit.is_none() { continue; }
        let unit = o_unit.unwrap();
        let force = if unit.force.is_none() { -1 } else {
            unit.force.unwrap().force_type
        };
        if force == 1 { adjust_missing_weapons(unit); }
        let uses = if unit.force.is_none() { 255 }
            else if unit.force.unwrap().force_type == 1 { 255 }
            else { 1 };

        for y in 0..8 {
            if unit.item_list.unit_items[y].as_ref().unwrap().item.parent.index < 3 { continue; }
            if unit.item_list.unit_items[y].as_ref().unwrap().item.parent.index == index {
                unit.item_list.unit_items[y].as_ref().unwrap().set_endurance(uses);
            }
            if ( force == 0 || force == 3 ) && unit.item_list.unit_items[y].as_ref().unwrap().item.flag.value & 2 != 0 {  //Cant Trade
                unit.item_list.unit_items[y].as_ref().unwrap().dispose();
            }
        }
        for x in 0..8 {
            let item = unit.item_list.unit_items[x].as_ref().unwrap();
            if item.item.parent.index < 3 { continue; } 
            if item.is_empty() {
                for y in x+1..unit.item_list.unit_items.len() {
                    if !unit.item_list.unit_items[y].as_ref().unwrap().is_empty() {
                        unit.item_list.move_item(y as i32, x as i32);
                        break;
                    }
                }
            }
        }
        unit.auto_equip();
    }
    let convoy_index = unsafe { get_empty_index(None) };
    for x in 0..convoy_index {
        let item = unsafe {  get_item_from_index(x, None ) };
        if item.unit_item.item.parent.index == index {
            item.unit_item.endurance = 1;
        }
        if item.unit_item.item.flag.value & 130 != 0 {  //Engage + //Cant Trade
            item.unit_item.dispose();
        }
    }
}

pub fn adjust_enemy_meteor(unit: &mut Unit) {
    if unit.person.get_asset_force() == 0 {  return;  }
    let index = ItemData::get("IID_メティオ").unwrap().parent.index;
    for y in 0..8 {
        if unit.item_list.unit_items[y].as_ref().unwrap().item.parent.index == index {
            unit.item_list.unit_items[y].as_ref().unwrap().ctor_str("IID_メティオ_G004");
        }
    }
}

pub fn add_equip_condition(unit: &Unit) {
    for x in 0..8 {
        if unit.item_list.unit_items[x].as_ref().unwrap().item.equip_condition.is_some() {
            let sid = unit.item_list.unit_items[x].as_ref().unwrap().item.equip_condition.unwrap().to_string();
            unit.private_skill.add_sid(&sid, 10, 0);
        }
    }
}

pub fn add_monster_weapons(unit: &Unit){
    // ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
    let jid = unit.get_job().jid.to_string();
    unit.item_list.put_off_all_item();
    if let Some(pos) = MONSTERS.iter().position(|&x| jid == x) {
        match pos {
            0 => { // Phantom Wyvern
                unit.item_list.add_iid_no_duplicate("IID_氷のブレス");
                unit.item_list.add_iid_no_duplicate("IID_氷塊");
            }
            1 => {  // Corrupted Wyvern
                unit.item_list.add_iid_no_duplicate("IID_瘴気のブレス");
                unit.item_list.add_iid_no_duplicate("IID_瘴気の塊");
            }
            2|3 => {    // Wolves
                unit.item_list.add_iid_no_duplicate("IID_牙");
            }
            4|7 => {
                unit.item_list.add_iid_no_duplicate("IID_邪竜石_魔法攻撃");
                unit.item_list.add_iid_no_duplicate("IID_邪竜石");
            }   
            5|6 => {    //Wyrms
                unit.item_list.add_iid_no_duplicate("IID_火のブレス");
                unit.item_list.add_iid_no_duplicate("IID_炎塊");
            }
            _ => {},
        }
        add_equip_condition(unit);
    }
    if unit.person.get_asset_force() == 0 { unit.item_list.add_iid_no_duplicate("IID_特効薬"); }
}


#[unity::class("App", "Transporter.Data")]
pub struct TransporterData {
    pub unit_item: &'static mut UnitItem,
}

#[skyline::from_offset(0x022a13d0)]
fn get_item_from_index(index: i32, method_info: OptionalMethod) -> &'static mut TransporterData;

#[skyline::from_offset(0x022a1990)]
fn get_empty_index(method_info: OptionalMethod) -> i32;
