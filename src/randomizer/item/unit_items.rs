use engage::unitpool::UnitPool;
use utils::clamp_value;

use super::*;
use crate::{assets::animation::MONSTERS, continuous::get_continious_total_map_complete_count, randomizer::person::unit::has_sid};

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
pub fn has_drops(unit: &Unit) -> i32 {
    unit.item_list.unit_items.iter().flatten().find(|w| w.is_drop())
        .map(|f| (f.item.flag.value & 1 != 0) as i32 + 1  ).unwrap_or(0)
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
                if !item.is_drop() { item.dispose(); }
            }
        }
    }
}
pub fn dispose_unusables(unit: &Unit) {
    for x in 0..8 {
        if unit.item_list.unit_items[x].as_ref().unwrap().item.parent.index < 3 { continue; }
        if let Some(item) = unit.item_list.get_item(x as i32 ).filter(|x| !x.is_drop() && x.item.flag.value & 128 == 0 ) {
            if item.item.kind < 10 && item.item.kind > 0 { // Not Engage Weapon or Not Drop
                if !unit.can_equip(x as i32, true, true) && item.item.kind < 10 {
                    item.dispose(); 
                }
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
        if let Some(item) = item_list.get_item(x) {
            let iid1 = item.item.iid.to_string();
            for y in x+1..8 {
                if let Some(item2) = item_list.get_item(y) {
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

            if weapon.is_weapon() && weapon.item.get_weapon_level() <= weapon_level && (
                ( unit.selected_weapon_mask.value & ( 1 << kind ) != 0 || unit.selected_weapon_mask.value == 0 ) )
            {
                let is_drop = weapon.is_drop();
                weapon.ctor_str(mag_weapons[kind as usize -1 ]);
                if is_drop { weapon.flags |= 2; }
            }
        }
        return;
    }
}
pub fn get_number_of_usable_weapons(unit: &Unit) -> i32 {
    let mut count = 0;
    let weapon_mask = unit.selected_weapon_mask.value;
    let job = &unit.job;
    for x in 0..8 {
        if let Some(item) = unit.item_list.get_item(x)
            .filter(|x| x.item.flag.value & 128 == 0 && x.item.is_weapon())
        {
            if unit.can_equip(x, false, true)  {
                let weapon_type = item.item.kind;
                if job.weapons[weapon_type as usize] == 1 { count += 1; }
                else if (weapon_mask & (1 << weapon_type) != 0)  && job.weapons[weapon_type as usize ] > 1
                {
                    count += 1
                }
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
    if melee { kind > 0 && kind < 5  }
    else { kind > 0 && kind < 9 }
}

pub fn assign_tomes(unit: &Unit) {
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").to_string() == unit.person.pid.to_string();
    let is_player = unit.person.get_asset_force() == 0;
    let job_rank = job.get_max_weapon_level(6);
    if weapon_mask.value & 64 != 0  && job_rank > 0 {
        dispose_item_type(unit.item_list, 6);
        let total_level = if is_vander { 1 } else { unit.level as i32 + unit.internal_level as i32};
        if let Some(item) = WEAPONDATA.get().unwrap().get_tome(job_rank, total_level, !is_player) {
            unit.item_list.add_item_no_duplicate(item);
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

    if unit.job.get_max_weapon_level(9) > 1 &&  unit.job.mask_skills.find_sid("SID_竜石装備").is_some() && (pid != PIDS[36] && pid != PIDS[37]) { 
        dispose_item_type(unit.item_list, 9);   // Dragonstone for classes that have the equip dragonstone skill
        if let Some(stone1) = WEAPONDATA.get().unwrap().get_dragon_stone(enemy) { unit.item_list.add_item_no_duplicate(stone1);  }
        if let Some(stone2) = WEAPONDATA.get().unwrap().get_dragon_stone(enemy) { unit.item_list.add_item_no_duplicate(stone2); }
    }
    // Mage Canon
    if unit.job.get_max_weapon_level(9) > 1 && unit.job.mask_skills.find_sid("SID_弾丸装備").is_some() {
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
    let is_player = unit.person.get_asset_force() == 0;

    let staff_level = if  has_sid(unit, "SID_杖使い＋＋") { 4 } 
        else if has_sid(unit, "SID_杖使い＋") { 3 }
        else if has_sid(unit, "SID_杖使い") { 2 } 
        else { job.get_max_weapon_level(7) };
    let total_level = unit.level as i32 + unit.internal_level as i32;
    let inventory_flag = GameVariableManager::get_number(DVCVariables::PLAYER_INVENTORY) & 2 != 0;
    if weapon_mask.value & ( 1 << 7 ) == 0 { replace_staves(unit.item_list); }
    else if weapon_mask.value & ( 1 << 7 ) != 0 && staff_level > 0 {
        dispose_item_type(unit.item_list, 7);
        if DVCVariables::is_main_chapter_complete(5) {
            for x in 1..3 {
                if let Some(staff_) = WEAPONDATA.get().unwrap()
                    .get_staff(total_level, x, staff_level, !is_player && inventory_flag )
                {
                    unit.item_list.add_item_no_duplicate(staff_);
                }
            }
        }
        else {
            if let Some(staff_) = WEAPONDATA.get().unwrap().get_staff(5, 1, 3, true){
                unit.item_list.add_item_no_duplicate(staff_);
            }
        }
    };
}

pub fn adjust_missing_weapons(unit: &Unit) {
    unit.update_weapon_mask();
    let count = get_number_of_usable_weapons(unit);
    println!("{} has {} usable weapon", Mess::get_name(unit.person.pid), count);
    if unit.job.mask_skills.find_sid("SID_弾丸装備").is_some() && count < 2 && unit.job.get_max_weapon_level(9) > 1 {  //Mage Canon
        dispose_all_but_drops(unit.item_list);
        let len = WEAPONDATA.get().unwrap().bullet_list.len();
        let rng = Random::get_game();
        if len > 1 {
            let index1 = WEAPONDATA.get().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
            unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index1).unwrap());
            if count < 2 {
                let index2 = WEAPONDATA.get().unwrap().bullet_list[ rng.get_value(len as i32) as usize].item_index;
                unit.item_list.add_item_no_duplicate(ItemData::try_index_get(index2).unwrap());
            }
        }
        return;
    }
    if count < 1 {
        println!("{} has no usable weapons!", Mess::get_name(unit.person.pid));
        assign_staffs(unit);
        assign_tomes(unit);
        add_generic_weapons(unit);
        unit.auto_equip();
    }
    let total_level = unit.level as i32 + unit.internal_level as i32;
    if get_number_of_usable_weapons(unit) < 2 && total_level > 20 && unit.selected_weapon_mask.value != 0 {
        if let Some(uitem) = unit.item_list.unit_items.iter().flatten()
            .find(|x| x.is_weapon() && ((1 << x.item.kind) & (unit.weapon_mask.value | unit.selected_weapon_mask.value) != 0))
        {
            if let Some(item) = WEAPONDATA.get().unwrap().get_additional_weapon(uitem.item) {
                // println!("Adding {}", Mess::get_name(item.iid));
                unit.item_list.add_item_no_duplicate(item);
            }
        }
    }
    unit_items::add_equip_condition(unit);
}

pub fn simple_replacement(unit: &Unit) {
    let job = unit.get_job();
    let mut combine_mask = unit.selected_weapon_mask.value | unit.weapon_mask.value;
    let mut n_weapons = 0;
    for x in 1..9 { if (combine_mask & (1 << x) != 0 ) && x != 7 { n_weapons += 1; }}
    unit.item_list.unit_items.iter().flatten()
        .filter(|uitem| uitem.item.is_weapon() && uitem.item.flag.value & 128 == 0 && !uitem.is_drop() )
        .for_each(|uitem| {
            if let Some(new_item) = WEAPONDATA.get().unwrap().get_simple_replacement(uitem.item, combine_mask, job.weapon_levels)
            {
                uitem.ctor(new_item);
                if n_weapons > 1 { combine_mask &= !(1 << new_item.kind); }
            }
        }
    );
    remove_duplicates(unit.item_list);
    dispose_unusables(unit);
    if combine_mask & 62 != 0 && get_number_of_usable_weapons(unit) < 2 { add_generic_weapons(unit); }
}

pub fn add_generic_weapons(unit: &Unit) {
    let job = unit.get_job();
    let jid = job.jid.to_string();
    if jid == "JID_ボウナイト" { unit.item_list.add_item_no_duplicate(ItemData::get("IID_銀の弓").unwrap());  }
    if jid == "JID_エンチャント" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_HPの薬").unwrap());  
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_力の薬").unwrap());  
    }
    let combine_mask = unit.selected_weapon_mask.value | unit.weapon_mask.value;
    let unit_level = unit.level as i32 + unit.internal_level as i32;
    let player = unit.person.get_asset_force() == 0;
    let mut weapon_rank = if unit_level < 10 || !GameVariableManager::get_bool("G_Cleared_M002") { 1 }
        else if unit_level < 15 { 2 }
        else if unit_level < 25 { 3 }
        else if unit_level < 30 { 4 }
        else {
            4 + (Random::get_system().get_value(10) < 40 - (40 - unit_level)) as i32
        };

    let mut has_weapon: [bool; 10] = [false; 10];
    has_weapon[7] = true;
    unit.item_list.unit_items.iter()
        .flatten()
        .filter(|u_item| u_item.item.is_weapon() && u_item.item.flag.value & 128 == 0)
        .for_each(|u_item|{
            let kind = u_item.item.kind;
            let item_level = u_item.item.get_weapon_level();
            let weapon_level = job.get_max_weapon_level(kind as i32);
            if item_level <= weapon_level && combine_mask & ( 1 << kind ) != 0 { 
                if weapon_rank < item_level { weapon_rank = item_level; }
                has_weapon[kind as usize] = true; 
            
            }
            else if !u_item.is_drop() && u_item.is_weapon() { u_item.dispose();  }
        }
    );
    for i in 1..9 {
        if has_weapon[i as usize] { continue; }
        let rank = clamp_value(weapon_rank, 0, job.get_max_weapon_level(i));
        let mut added_weapon = false;
        if combine_mask & (1 << i ) != 0 && rank > 0 {
            if player && GameVariableManager::get_number(DVCVariables::PLAYER_INVENTORY) & 1 != 0 {
                if let Some(item) = WEAPONDATA.get().unwrap().get_random_weapon(i, rank, false) {
                    if item.get_weapon_level() <= rank {
                        unit.item_list.add_item_no_duplicate(item);
                        added_weapon = true;
                    }
                }
            }
            else if GameVariableManager::get_number(DVCVariables::PLAYER_INVENTORY) & 2 != 0 && !player && get_continious_total_map_complete_count() > 9 {
                if let Some(item) = WEAPONDATA.get().unwrap().get_random_weapon(i, rank, true) {
                    if item.get_weapon_level() <= rank {
                        unit.item_list.add_item_no_duplicate(item);
                        added_weapon = true;
                    }
                }
            }
            if !added_weapon {
                let mut search_rank = rank;
                while search_rank > 0 {
                    if let Some(item) = WEAPONDATA.get().unwrap().get_generic_weapon(i, search_rank) {
                        if (item.get_equip_skills().find_sid("SID_２回行動").is_none() && i < 6) ||
                            (item.get_equip_skills().find_sid("SID_２回行動").is_some() && i > 6)
                        {
                            unit.item_list.add_item_no_duplicate(item);
                            break;
                        }
                    }
                    search_rank -= 1;
                }
            }
        }
    }
    if job.get_max_weapon_level(4) >= 2 && DVCVariables::is_main_chapter_complete(22) && Random::get_system().get_value(10) < 2 {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_長弓").unwrap());
    }
    dispose_unusables(unit);
}

pub fn random_items_drops(unit: &Unit){
    let rng = Random::get_system();
    let mut none_count = 0;
    let mut rate =
        if GameUserData::get_difficulty(false) == 2 {1 / 2} else { 1 } * GameVariableManager::get_number(DVCVariables::ITEM_DROP_GAUGE_KEY);

    for x in 0..8 {
        if let Some(u_item) = unit.item_list.get_item(x) {
            if u_item.item.flag.value & 130 != 0 {  u_item.flags = 0; }
        }
    }
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_none() { continue; }
        let u_item = &mut item.unwrap();
        if u_item.item.parent.index < 2 && none_count == 0 {
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
            println!("Adding Equip Skill Condition");
            unit.private_skill.add_sid(&sid, 10, 0);
        }
    }
}

pub fn add_monster_weapons(unit: &Unit){
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
