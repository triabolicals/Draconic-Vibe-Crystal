use engage::unit::UnitItemList;
use super::*;
use crate::{assets::animation::MONSTERS, continuous::get_continious_total_map_complete_count};
use crate::randomizer::item::data::WeaponDataFlag;

pub fn replace_staves(item_list: &UnitItemList){
    for x in 0..8 {
        if let Some(staff) = item_list.get_item(x).filter(|x| x.item.kind == 7){
            if staff.item.kind == 7 {
                let staff_name = staff.item.iid.to_string();
                if staff_name == "IID_ライブ" || staff_name == "IID_リブロー" || staff_name == "IID_リライブ" { staff.ctor_str("IID_傷薬"); }
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
            let kind = item.item.kind as i32;
            if  item.item.flag.value & 128 == 0 && kind == item_kind { item.dispose(); }
        }
    }
}

pub fn dispose_all_but_drops(item_list: &UnitItemList){
    for x in 0..8 {
        if let Some(item) = item_list.get_item(x){
            if item.item.parent.index == 1 || item.is_drop()  { item.dispose(); }
        }
    }
}
pub fn dispose_unusables(unit: &Unit) {
    for x in 0..8 {
        if let Some(item) = unit.item_list.get_item(x){
            if item.item.parent.index == 1 { item.dispose(); }
            else {
                if item.item.kind < 10 && item.item.kind > 0 { // Not Engage Weapon or Not Drop
                    if !unit.can_equip_item(item.item, true, true) && item.item.kind < 10 {
                        if item.item.flag.value & 1 == 0 {
                            if unit.force.is_some_and(|f| f.force_type == 0 || f.force_type == 3) && Transporter::can_add() {
                                Transporter::add_unit_item(item);
                            }
                            item.dispose();
                        }
                    }
                }
            }
        }
    }
    reorder_for_empty(unit.item_list);
}
pub fn reorder_for_empty(item_list: &UnitItemList) {
    for x in 0..8 {
        if let Some(item) = item_list.get_item(x){
            if item.item.parent.index == 1 { item.dispose(); }
            if item.item.parent.index < 3 { continue; } // 10 2 2 2 0 4
            for y in 1..x {
                if item_list.get_item(y).filter(|x| x.item.parent.index == 0).is_some(){
                    item_list.move_item(x, y);
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
    reorder_for_empty(item_list);
}

pub fn adjust_melee_weapons(unit: &Unit) {
    let enemy = unit.person.get_asset_force() == 1;
    let job = unit.get_job();
    let db = GameData::get_weapon_db();
    let inventory_flag = DVCVariables::UnitInventory.get_value();
    let allow_rare = inventory_flag & 2 != 0 && enemy;
    if unit.get_capability(1, true) < unit.get_capability(6, true) {
        for x in 0..8 {
            if let Some(unit_item) = unit.item_list.get_item(x).filter(|x| x.is_weapon() && x.item.flag.value & 65664 == 65536 && x.item.parent.index > 2 && x.item.kind != 6) {
                let kind = unit_item.item.kind as i32;
                let enemy_only = unit_item.flags & 2 == 0 && allow_rare;
                let flag = unit_item.flags;
                if job.weapons[kind as usize] == 1 || job.weapons[kind as usize] > 1 && unit.selected_weapon_mask.value & (1 << kind) != 0 {
                    let max_weapon_level = job.get_max_weapon_level(unit_item.item.kind as i32) as u8;
                    let search_kind = if unit_item.item.kind == 9 && unit_item.item.flag.value & 0x4000000 != 0 { 10 } else { unit_item.item.kind } as u8;
                    let pool: Vec<_> = db.magic_weapons.iter().filter(|x|
                        x.weapon_type == search_kind &&
                            x.rank <= max_weapon_level &&
                            ((x.flag.contains(WeaponDataFlag::EnemyOnly) && enemy_only) || !x.flag.contains(WeaponDataFlag::EnemyOnly)))
                        .map(|x| x.item_index).collect();
                    if let Some(item) = pool.get_random_element(Random::get_system()).and_then(|index| ItemData::try_index_get(*index)) {
                        unit_item.ctor(item);
                        unit_item.flags = flag;
                    }
                }
            }
        }
    }
}
pub fn get_number_of_usable_weapons(unit: &Unit) -> i32 {
    let mut count = 0;
    let weapon_mask = unit.selected_weapon_mask.value;
    let job = &unit.job;
    for x in 0..8 {
        if let Some(item) = unit.item_list.get_item(x).filter(|x| x.item.flag.value & 128 == 0 && x.item.is_weapon()) {
            if unit.can_equip_item(item.item, false, true)  {
                let weapon_type = item.item.kind;
                if job.weapons[weapon_type as usize] == 1 { count += 1; }
                else if (weapon_mask & (1 << weapon_type) != 0)  && job.weapons[weapon_type as usize ] > 1 { count += 1; }
            }
        }
    }
    count
}

pub fn assign_tomes(unit: &Unit) {
    let weapon_db = GameData::get_weapon_db();
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask2();
    remove_duplicates(unit.item_list);
    let is_vander = GameVariableManager::get_string("G_R_PID_ヴァンドレ").to_string() == unit.person.pid.to_string();
    let is_player = unit.person.get_asset_force() == 0;
    let job_rank = job.get_max_weapon_level(6);
    if weapon_mask.value & 64 != 0  && job_rank > 0 {
        dispose_item_type(unit.item_list, 6);
        let total_level = if is_vander { 1 } else { unit.level as i32 + unit.internal_level as i32};
        if let Some(item) = weapon_db.get_tome(job_rank, total_level, !is_player) {
            unit.item_list.add_item_no_duplicate(item);
        }
    }
}

pub fn assign_unique_items(unit: &Unit) {
    let weapon_db = GameData::get_weapon_db();
    remove_duplicates(unit.item_list);
    let pid = unit.person.pid.to_string();
    let veyle = pid == PIDS[32];
    let job_hash = unit.job.parent.hash;
    let enemy = unit.person.get_asset_force() != 0;
    if veyle { 
        magic_dagger_weapon_change(unit.get_job());
        unit.item_list.add_iid_no_duplicate("IID_ミセリコルデ");
    }  // Misercode for Veyle
    if job_hash == 185670709 || veyle {  unit.item_list.add_iid_no_duplicate("IID_オヴスキュリテ");  } // Obscurite for Veyle / Fell Child Veyle

    if unit.job.get_max_weapon_level(9) > 1 &&  unit.job.mask_skills.find_sid("SID_竜石装備").is_some() && (pid != PIDS[36] && pid != PIDS[37])
        && unit.item_list.unit_items.iter().any(|i| i.as_ref().is_some_and(|i| i.item.flag.value &  0x4000000 != 0))
    {
        dispose_item_type(unit.item_list, 9);   // Dragonstone for classes that have the equip dragonstone skill
        if let Some(stone1) = weapon_db.get_dragon_stone(enemy) { unit.item_list.add_item_no_duplicate(stone1);  }
        if let Some(stone2) = weapon_db.get_dragon_stone(enemy) { unit.item_list.add_item_no_duplicate(stone2); }
    }
    // Mage Canon
    if unit.job.get_max_weapon_level(9) > 1 && unit.job.mask_skills.find_sid("SID_弾丸装備").is_some()
        && unit.item_list.unit_items.iter().any(|i| i.as_ref().is_some_and(|i| i.item.flag.value &  0x8000000 != 0))
    {
        dispose_item_type(unit.item_list, 9);
        if let Some(item) = weapon_db.get_generic_weapon(9, 1) { unit.item_list.add_item_no_duplicate(item); }
        if enemy {
            if let Some(item) = weapon_db.get_random_weapon(Some(unit), 9, 6, enemy) {
                unit.item_list.add_item_no_duplicate(item);
            }
        }
    }
    if pid == PIDS[36] {
        dispose_item_type(unit.item_list, 9);
        unit.item_list.add_iid_no_duplicate("IID_邪竜石"); 
        unit.item_list.add_iid_no_duplicate("IID_邪竜石_魔法攻撃");
    }
    else if pid == PIDS[37] {
        dispose_item_type(unit.item_list, 9);
        unit.item_list.add_iid_no_duplicate("IID_邪竜石"); 
        unit.item_list.add_iid_no_duplicate("IID_邪竜石_騎馬特効");
    }
}

pub fn assign_staffs(unit: &Unit) {
    let weapon_db = GameData::get_weapon_db();
    let mask = unit.weapon_mask.value;
    let is_player = unit.person.get_asset_force() == 0;
    let skill_staff =  unit.mask_skill.map(|m| m.weapon_levels.levels[7] as i32).unwrap_or(0);
    if mask & 128 == 0 && skill_staff == 0 { replace_staves(unit.item_list); }
    else {
        let staff_level = max(unit.job.get_max_weapon_level(7), skill_staff);
        let total_level = unit.level as i32 + unit.internal_level as i32;
        let inventory_flag = DVCVariables::UnitInventory.get_value() & 2 != 0;
        dispose_item_type(unit.item_list, 7);
        if DVCVariables::is_main_chapter_complete(5) {
            for x in 1..3 {
                if let Some(staff_) = weapon_db.get_staff(total_level, x, staff_level, !is_player && inventory_flag) {
                    unit.item_list.add_item_no_duplicate(staff_);
                }
            }
        }
        else {
            if let Some(staff_) = weapon_db.get_staff(5, 1, 2, false) {
                unit.item_list.add_item_no_duplicate(staff_);
            }
        }
    }
}

pub fn adjust_missing_weapons(unit: &Unit) {
    unit.update_weapon_mask();
    if add_monster_weapons(unit) { return; }
    let count = get_number_of_usable_weapons(unit);
    if count < 2 {
        let weapon_db = GameData::get_weapon_db();
        if unit.job.mask_skills.find_sid("SID_弾丸装備").is_some() && unit.job.get_max_weapon_level(9) > 1 {  //Mage Canon
            if unit.force.is_some_and(|f| f.force_type != 0 ) { dispose_all_but_drops(unit.item_list); }
            else { dispose_unusables(unit); }
            let max_rank = unit.job.get_max_weapon_level(9);
            let mut item_hash = 0;
            loop {
                if let Some(item) = weapon_db.get_random_weapon(Some(unit), 9, max_rank, false).filter(|i| i.parent.hash != item_hash) {
                    unit.item_list.add_item_no_duplicate(item);
                    if item_hash == 0 { item_hash = item.parent.hash; } else { break }
                }
            }
        }
        else {
            assign_staffs(unit);
            assign_tomes(unit);
            add_generic_weapons(unit);
            unit.auto_equip();
            let total_level = unit.level as i32 + unit.internal_level as i32;
            if get_number_of_usable_weapons(unit) < 2 && total_level > 20 && unit.selected_weapon_mask.value != 0 {
                add_generic_weapons(unit);
            }
        }
    }
    add_equip_condition(unit);
    unit.item_reorder();
    reorder_for_empty(unit.item_list);
}

pub fn adjust_player_weapons(unit: &Unit) {
    let weapon_db = GameData::get_weapon_db();
    weapon_db.do_simple_replacement(unit, unit.force.is_some_and(|f| (1 << f.force_type) & 9 != 0 ));
    if get_number_of_usable_weapons(unit) == 0 { add_generic_weapons(unit); }
    reorder_for_empty(unit.item_list);
}
pub fn add_generic_weapons(unit: &Unit) {
    let weapon_db = &GameData::get_item_pool().weapon_db;
    let job = unit.get_job();
    let jid = job.jid.to_string();
    if jid == "JID_ボウナイト" { unit.item_list.add_item_no_duplicate(ItemData::get("IID_銀の弓").unwrap());  }
    if jid == "JID_エンチャント" {
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_HPの薬").unwrap());  
        unit.item_list.add_item_no_duplicate(ItemData::get("IID_力の薬").unwrap());  
    }
    println!("Add Generic Weapons {}: Weapon mask: {} Selected: {}", Mess::get_name(unit.person.pid), unit.weapon_mask.value, unit.selected_weapon_mask.value);
    unit.update_weapon_mask();
    let combine_mask = unit.weapon_mask.value;
    let weapon_levels = get_unit_avail_weapon_levels(unit);
    let player = unit.person.get_asset_force() == 0;

    let mut has_weapon: [bool; 11] = [false; 11];
    has_weapon[7] = true;
    unit.item_list.unit_items.iter()
        .flatten()
        .filter(|u_item| u_item.item.is_weapon() && u_item.item.flag.value & 128 == 0)
        .for_each(|u_item|{
            let kind = u_item.item.kind as usize;
            if kind < 10 {
                let item_level = u_item.item.get_weapon_level();
                let weapon_level = weapon_levels[kind];
                if kind == 9 {

                }
                else if item_level != 0 && item_level <= weapon_level && combine_mask & ( 1 << kind ) != 0 { has_weapon[kind] = true; }
                else if !u_item.is_drop() && u_item.is_weapon() { u_item.dispose();  }
            }
        }
    );
    for i in 1..9 {
        let rank = weapon_levels[i as usize];
        if has_weapon[i as usize] || rank == 0 { continue; }
        let mut added_weapon = false;
        if combine_mask & (1 << i ) != 0 && rank > 0 {
            if player && DVCVariables::UnitInventory.get_value() & 1 != 0 {
                if let Some(item) = weapon_db.get_random_weapon(Some(unit), i, rank, false) {
                    if item.get_weapon_level() <= rank {
                        unit.item_list.add_item_no_duplicate(item);
                        added_weapon = true;
                    }
                }
            }
            else if DVCVariables::UnitInventory.get_value() & 2 != 0 && !player && get_continious_total_map_complete_count() > 9 {
                if let Some(item) = weapon_db.get_random_weapon(Some(unit), i, rank, true) {
                    if item.get_weapon_level() <= rank {
                        unit.item_list.add_item_no_duplicate(item);
                        added_weapon = true;
                    }
                }
            }
            if !added_weapon {
                let mut search_rank = rank;
                while search_rank > 0 {
                    if let Some(item) = weapon_db.get_generic_weapon(i, search_rank) {
                        unit.item_list.add_item_no_duplicate(item);
                        break;
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
    let mut rate = DVCVariables::EnemyItemDropGauge.get_value();
    if !GameVariableManager::get_bool("G_Cleared_M002") { return; }
    if rng.get_value(100) < (rate / 2) {
        let mut item_count = 0;
        for x in 0..8 {
            if let Some(u_item) = unit.item_list.get_item(x) {
                if u_item.item.flag.value & 128 != 0 { u_item.flags = 0; }
                else if u_item.flags & 2 != 0 { item_count += 1; }
            }
        }
        rate = rate / (item_count + 1);
        let pool = GameData::get_item_pool();
        for x in 0..8 {
            if let Some(unit_item) = unit.item_list.get_item(x).filter(|x| x.is_empty()) {
                if rng.get_value(100) < rate {
                    unit_item.ctor_str(&pool.random_item(4, false).to_string());
                    unit_item.flags |= 2;
                    rate = rate / 2;
                }
            }
        }
    }
    reorder_for_empty(unit.item_list);
}

fn magic_dagger_weapon_change(veyle_job: &JobData){
    let hash = veyle_job.parent.hash;
    if (hash == 185670709 ||  hash == -1998645787) || (veyle_job.is_high() && DVCVariables::get_single_class(false, true).is_none()) {
        GameVariableManager::make_entry(DVCVariables::MISERCODE_TYPE, 5); 
        GameVariableManager::set_number(DVCVariables::MISERCODE_TYPE, 5);
        return; 
    }
    let mut misercode_type = 5; //Dagger
    let kind = get_max_job_weapon_type(veyle_job);
    if kind != 0 { misercode_type = kind; }
    GameVariableManager::make_entry(DVCVariables::MISERCODE_TYPE, misercode_type);
    GameVariableManager::set_number(DVCVariables::MISERCODE_TYPE, misercode_type);
    change_misercode_type();
}

pub fn get_max_job_weapon_type(job: &JobData) -> i32 {
    let mut kind = 0;
    let mut rank = 0;
    for x in 1..10 {
        if x == 7 { continue; }
        if job.weapons[x] == 1 && rank < job.get_max_weapon_level(x as i32){
            kind = x as i32;
            rank = job.get_max_weapon_level(x as i32);
        }
    }
    if kind == 9 {
        if job.mask_skills.find_sid("SID_弾丸装備").is_some() { kind = 10; }
    }       
    kind
}

pub fn add_equip_condition(unit: &Unit) {
    for x in 0..8 {
        if let Some(equip) = unit.item_list.unit_items[x].as_ref()
            .filter(|x| x.item.flag.value & 201326720 == 0).and_then(|x| x.item.equip_condition)    // No Engage/Dragon/Bullet Weapons
        {
            unit.private_skill.add_sid(equip.to_string().as_str(), SkillDataCategorys::Private, 0);
        }
    }
}
pub fn removing_equip_condition(unit: &mut Unit) {
    for x in 0..8 {
        if let Some(equip) = unit.item_list.unit_items[x].as_ref()
            .filter(|x| x.item.flag.value & 201326720 == 0).and_then(|x| x.item.equip_condition)    // No Engage/Dragon/Bullet Weapons
        {
            unit.private_skill.remove_sid(equip);
        }
    }
}


pub fn add_monster_weapons(unit: &Unit) -> bool {
    let jid = unit.get_job().jid.to_string();
    if let Some(pos) = MONSTERS.iter().position(|&x| jid == x) {
        if unit.force.is_some_and(|f| f.force_type != 0 ) {
            unit.item_list.put_off_all_item();
        }
        else { dispose_unusables(unit); }
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
        if unit.person.get_asset_force() == 0 { unit.item_list.add_iid_no_duplicate("IID_特効薬"); }
        true
    }
    else { false }
}