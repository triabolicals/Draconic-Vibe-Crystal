use engage::app::{
    itemdata::*, jobdata::*, unit::*, unititem::*, unititemlist::*, bitfield32::*,
    IPersonDataMethods, IStructData_1Methods, SkillData_Categorys, Transporter
};
use outfit_core::Randomizer;
use unity::Il2CppString;
use crate::gamedata::GameData;
use super::*;

pub fn adjust_unit_items(unit: Unit) {
    let job = unit.get_job();
    let jid = job.hash();
    let is_enemy = unit.get_person().get_asset_force().value != 0;
    if MONSTER_CLASS.iter().any(|&x| jid == x) {
        unit.item_put_off_all();
        add_monster_weapons(unit);
        return;
    }
    unit.update_weapon_mask();
    let custom_inventory = DVCVariable::UnitInventory.get_value();
    let ran_map = DVCVariables::is_random_map();
    if custom_inventory & 1 != 0 && !is_enemy {
        unit.item_put_off_all();
        add_generic_weapons(unit);
    } 
    else if custom_inventory & 2 != 0 && is_enemy && ((DVCVariables::is_main_chapter_complete(11) && ran_map) || DVCVariables::is_main_chapter_complete(9)) {
        dispose_all_but_drops(unit.get_item_list());
        add_generic_weapons(unit);
    } 
    else { GameData::get_item_pool().weapon_db.do_simple_replacement(unit, false); }
    assign_tomes(unit);
    assign_staffs(unit);
    dispose_unusable(unit, false, false);
    adjust_missing_weapons(unit);
    add_equip_condition(unit);

    unit.item_equip();
    if PersonPool::is_playable(unit.get_person()) {
        let recovery_item =
        if unit.get_capability(0, true) >= 45 { "IID_特効薬" } else { "IID_傷薬" };
        add_item_no_dup_by_iid(unit, recovery_item);
    }
    else if DVCVariable::UnitInventory.get_value() & 2 != 0 && DVCVariables::chapter_number_complete(true) > 10 {
        let rng = Random_2::get_system();
        let playable_gods = GameData::get_playable_emblem_hashes();
        let item = unit.get_item_equipped();
        if !item.is_null() && rng.get_value_2(4) == 0 {
            if let Some(god) = playable_gods.get_random_element(rng).map(|&h| GodData::try_get_from_hash(h)) { item.set_m_engrave(god); }
        }
    }
    dispose_duplicates(unit.get_item_list());
    unit.item_close_up();
    unit.item_equip();
}

pub fn add_item_no_dup_by_iid(unit: Unit, iid: impl Into<Il2CppString>) {
    let item = ItemData::get(iid.into());
    add_item_no_dup(unit, item);
}
pub fn add_item_no_dup(unit: Unit, item: ItemData) {
    if !item.is_null() && !unit.has_item_2(item) {
        unit.item_add_3(item);
        reorder_for_empty(unit.get_item_list());
    }
}
pub fn has_drops(unit: Unit) -> bool { unit.get_item_list().m_unit_items().iter().any(|x| x.get_is_drop()) }
pub fn dispose_item_type(list: UnitItemList, item_kind: i32){ list.m_unit_items().iter().for_each(|i|{ if i.m_item().get_kind().value == item_kind { i.dispose(); } }); }
pub fn dispose_all_but_drops(item_list: UnitItemList){
    item_list.m_unit_items().iter().for_each(|i|{ if !i.get_is_drop() { i.dispose(); } });
    reorder_for_empty(item_list);
}
pub fn dispose_duplicates(item_list: UnitItemList){
    for x in 0..8 {
        let index = item_list.get_item(x).m_item().index();
        for y in x+1..8 {
            let item_y = item_list.get_item(y);
            if item_y.m_item().index() == index { item_y.dispose(); }
        }
    }
    reorder_for_empty(item_list);
}
pub fn dispose_unusable(unit: Unit, to_transporter: bool, include_drop: bool) {
    let transporter = (1 << unit.get_force_type().value) & 9 != 0 && to_transporter && !unit.is_vision() && !unit.is_summon();
    unit.get_item_list().m_unit_items().iter().for_each(|i|{
        let item = i.m_item();
        if item.get_flag().m_value() & 128 == 0 && item.index() > 2{
            if !unit.can_item_equip(i, true, true) {
                if transporter && Transporter::can_add() {
                    Transporter::add_2(i.m_item());
                    i.dispose();
                }
                else {
                    let drop = i.get_is_drop();
                    if (drop && include_drop) || !drop { i.dispose(); }
                }
            }
        }
    });
    unit.item_close_up();
}
pub fn reorder_for_empty(list: UnitItemList) {
    for x in 0..8 {
        let index = list.get_item(x).m_item().index();
        if index == 0 {
            for y in x+1..8 {
                let index2 = list.get_item(x).m_item().index();
                if index2 != 0 {
                    list.r#move(y, x);
                    break;
                }
            }
        }
    }
}
pub fn get_number_of_usable_weapons(unit: Unit) -> i32 {
    let mut count = 0;
    for x in 0..8 {
        let item = unit.get_item(x);
        if item.m_item().get_flag().m_value() & 128 == 0 && item.m_item().is_weapon() {
            if unit.can_item_equip_2(item.m_item(), false, true) { count += 1; }
        }
    }
    count
}

pub fn assign_tomes(unit: Unit) {
    let weapon_db = GameData::get_weapon_db();
    let job = unit.get_job();
    let weapon_mask = job.get_weapon_mask_2().m_value();
    dispose_duplicates(unit.get_item_list());
    let is_vander = PersonPool::get_replacement_2(unit.get_person(), true).is_some_and(|v| v.hash() == PersonPool::VANDRE);
    let is_player = unit.get_person().get_asset_force().value == 0;
    let job_rank = job.get_max_weapon_level(6).value;
    if weapon_mask & 64 != 0  && job_rank > 0 {
        dispose_item_type(unit.get_item_list(), 6);
        let (job_rank, total_level) = if is_vander { (1, 1) } else { (job_rank, unit.get_level() + unit.get_internal_level()) };
        if let Some(item) = weapon_db.get_tome(job_rank, total_level, !is_player) { add_item_no_dup(unit, item); }
    }
}
/*
pub fn assign_unique_items(unit: Unit) {
    /*
    let weapon_db = GameData::get_weapon_db();
    dispose_duplicates(unit.get_item_list());
    let pid = unit.get_person().hash();
    let veyle = pid == PIDS[32];
    let job_hash = unit.job.parent.hash;
    let enemy = unit.person.get_asset_force() != 0;
    if veyle {
        magic_dagger_weapon_change(unit.get_job());
        unit.item_list.add_iid_no_duplicate(MISERCODE);
    }  // Misercode for Veyle
    if job_hash == 185670709 || veyle {
        if let Some(item) = ItemData::get(OBSCURITE) {
            unit.add_item(item);
            if let Some(skill) = item.equip_condition.and_then(|sid| SkillData::get(sid)) {
                unit.add_private_skill(skill);
            }
        }
    } // Obscurite for Veyle / Fell Child Veyle

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
    */
}
*/

pub fn assign_staffs(unit: Unit) {
    let weapon_db = GameData::get_weapon_db();
    let mask = unit.get_weapon_mask().m_value();
    let is_player = unit.get_person().get_asset_force().value == 0;
    if mask & 128 != 0{
        let staff_level = unit.get_job().get_max_weapon_level(7).value;
        let total_level = unit.get_level() + unit.get_internal_level();
        let inventory_flag = DVCVariable::UnitInventory.get_value() & 2 != 0;
        dispose_item_type(unit.get_item_list(), 7);
        if DVCVariables::is_main_chapter_complete(5) {
            for x in 1..3 {
                if let Some(staff_) = weapon_db.get_staff(total_level, x, staff_level, !is_player && inventory_flag) {
                    add_item_no_dup(unit, staff_);
                }
            }
        }
        else {
            if let Some(staff_) = weapon_db.get_staff(5, 1, 2, false) {
                add_item_no_dup(unit, staff_);
            }
        }
    }
}

pub fn adjust_missing_weapons(unit: Unit) {
    unit.update_weapon_mask();
    if add_monster_weapons(unit) { return; }
    let weapons = unit.get_weapon_mask().m_value();
    let mut kinds = 0;
    for x in 1..10 {
        if x == 7 { continue; }
        if (1 << x ) & weapons != 0 { kinds += 1; }
    }
    let count = get_number_of_usable_weapons(unit);
    let weapon_db = GameData::get_weapon_db();
    let job = unit.get_job();
    let special_rank = job.get_max_weapon_level(9).value;
    if count < kinds || count == 0 {
        let job = unit.get_job();
        if !job.get_mask_skill().find("SID_弾丸装備").is_null() && special_rank > 1 {
            let max_rank = job.get_max_weapon_level(9).value;
            if unit.get_person().get_asset_force().value != 0 { dispose_all_but_drops(unit.get_item_list()); }
            else { dispose_unusable(unit, false, false) }
            let mut item_hash = 0;
            loop {
                if let Some(item) = weapon_db.get_random_weapon(Some(unit), 9, max_rank, false).filter(|i| i.hash() != item_hash) {
                    add_item_no_dup(unit, item);
                    if item_hash == 0 { item_hash = item.hash(); }
                    else { break; }
                }
            }
        }
        else {
            assign_staffs(unit);
            assign_tomes(unit);
            add_generic_weapons(unit);
            unit.item_equip();
        }
    }
    if special_rank > 1 && !job.get_mask_skill().find("SID_竜石装備").is_null() {
        if unit.get_item_list().m_unit_items().iter().filter(|v| !v.is_none() && v.is_weapon() && v.m_item().is_dragon()).count() == 0 {
            if unit.get_person().get_asset_force().value != 0 {
                if let Some(item) = weapon_db.get_random_weapon(None, 10, special_rank, false) {
                    add_item_no_dup(unit, item);
                }
            }
            else {
                if let Some(item) = weapon_db.get_generic_weapon(10, special_rank){
                    add_item_no_dup(unit, item);
                }
            }

        }
    }
    add_equip_condition(unit);
    #[cfg(feature="debug")]
    for i in 0..8 {
        if unit.get_item(i).m_item().index() > 2 {
            println!("Adding Missing Weapon {} Item #{} {}: {}", i, unit.get_name(), i + 1, unit.get_item(i).m_item().get_name());
        }
    }
}

pub fn add_generic_weapons(unit: Unit) {
    let weapon_db = &GameData::get_item_pool().weapon_db;
    let job = unit.get_job();
    let hash = job.hash();
    if hash == MAGE_CANNON { add_item_no_dup_by_iid(unit, "IID_銀の弓"); }
    else if hash == ENCHANTER {
        add_item_no_dup_by_iid(unit, "IID_HPの薬");
        add_item_no_dup_by_iid(unit, "IID_力の薬");
    }
    unit.update_weapon_mask();
    let mut combine_mask = unit.get_selected_weapon_mask().m_value();
    for i in 1..10 { if job.weapons().get(i as usize) == 1 { combine_mask |= 1 << i; } }

    let weapon_levels = get_unit_avail_weapon_levels(unit);
    let player = unit.get_person().get_asset_force().value == 0;
    combine_mask &= !128;
    unit.get_item_list().m_unit_items().iter().for_each(|i|{
        let item = i.m_item();
        if item.m_is_weapon() && item.get_flag().m_value() & 131 == 0 {
            if unit.can_item_equip(i, false, false) {
                let kind = item.get_kind().value;
                if kind > 0 && kind < 10 { combine_mask &= !(1 << kind); }
            }
            else if !i.get_is_drop() { i.dispose(); }
        }
    });
    let random = DVCVariable::UnitInventory.get_value() & if player { 1 } else { 2 } != 0;
    for i in 1..9 {
        let rank = weapon_levels[i as usize];
        if combine_mask & (1 << i) == 0 || rank == 0 { continue }
        let mut added_weapon = false;
        if random {
            if let Some(item) = weapon_db.get_random_weapon(Some(unit), i, rank, !player){
                if item.m_weapon_level().value <= rank {
                    add_item_no_dup(unit, item);
                    added_weapon = true;
                }
            }
        }
        if !added_weapon {
            let mut search_rank = rank;
            while search_rank > 0 {
                if let Some(item) = weapon_db.get_generic_weapon(i, search_rank) {
                    add_item_no_dup(unit, item);
                    break;
                }
                search_rank -= 1;
            }
        }
    }
    if DVCFlags::RandomEnable.get_value() && job.get_max_weapon_level(4).value >= 2 && DVCVariables::is_main_chapter_complete(22) && Random_2::get_system().get_value_2(3) == 0 {
        add_item_no_dup_by_iid(unit, "IID_長弓");
    }
    dispose_unusable(unit, false, DVCVariable::UnitInventory.get_value() & 2 != 0);
}

pub fn random_items_drops(unit: Unit){
    let rng = Random_2::get_system();
    let item_count = unit.m_item_list().m_unit_items().iter().filter(|i| i.get_is_drop()).count() as i32;
    let mut rate = if item_count == 0 { 100 } else { 100 >> item_count };
    let pool = GameData::get_item_pool();
    for x in 0..8 {
        let item = unit.get_item(x);
        if item.m_item().index() == 0 {
            if rng.get_value_2(100) < rate {
                item.ctor_3(pool.random_item(RandomItemReplacementType::Drop, false));
                item.set_is_drop(true);
                rate = rate >> 2;
            }
            else { break; }
        }
    }
    reorder_for_empty(unit.get_item_list());
}

pub fn magic_dagger_weapon_change(veyle_job: JobData){
    let hash = veyle_job.hash();
    if (hash == 185670709 ||  hash == -1998645787) || (veyle_job.is_high() && DVCVariables::get_single_class(false, true).is_none()) {
        DVCVariable::MisercodeKind.set_value(5);
    }
    else {
        let mut misercode_type = 5; //Dagger
        let kind = get_max_job_weapon_type(veyle_job);
        if kind != 0 { misercode_type = kind; }
        DVCVariable::MisercodeKind.set_value(misercode_type);
        change_misercode_type();
    }
}

pub fn get_max_job_weapon_type(job: JobData) -> i32 {
    let mut kind = 0;
    let mut rank = 0;
    for x in 1..10 {
        if x == 7 { continue; }
        let max_rank = job.get_max_weapon_level(x as i32).value;
        if job.weapons().get(x) == 1 && rank < max_rank {
            kind = x as i32;
            rank = max_rank;
        }
    }
    if kind == 9 { if !job.get_mask_skill().find("SID_弾丸装備").is_null() { kind = 10; } }
    kind
}

pub fn add_equip_condition(unit: Unit) {
    for x in 0..8 {
        let item = unit.get_item(x);
        let item = item.m_item();
        if item.get_flag().m_value() & 201326720 == 0 && !item.get_equip_condition().is_null() { unit.add_private_skill(item.get_equip_condition()); }
    }
}
pub fn add_monster_weapons(unit: Unit) -> bool {
    let jid = unit.get_job().hash();
    if let Some(pos) = MONSTER_CLASS.iter().position(|&x| jid == x) {
        dispose_unusable(unit, false, false);
        match pos {
            0|1 => {    //Wyrms
                add_item_no_dup_by_iid(unit, "IID_火のブレス");
                add_item_no_dup_by_iid(unit, "IID_炎塊");
            }
            2|3 => {    // Wolves
                add_item_no_dup_by_iid(unit, "IID_牙");
            }
            4 => { // Phantom Wyvern
                add_item_no_dup_by_iid(unit, "IID_氷のブレス");
                add_item_no_dup_by_iid(unit, "IID_氷塊");
            }
            5 => {  // Corrupted Wyvern
                add_item_no_dup_by_iid(unit, "IID_瘴気のブレス");
                add_item_no_dup_by_iid(unit, "IID_瘴気の塊");
            }
            7|8 => {
                add_item_no_dup_by_iid(unit, "IID_ソンブル_物理攻撃");
                add_item_no_dup_by_iid(unit, "IID_ソンブル_魔法攻撃");
            }
            _ => { return false; },
        }
        add_equip_condition(unit);
        if unit.get_person().get_asset_force().value == 0 { add_item_no_dup_by_iid(unit, "IID_特効薬"); }
        true
    }
    else { false }
}

pub fn get_unit_avail_weapon_levels(unit: Unit) -> [i32; 10] {
    let mut levels = [0; 10];
    let level = unit.get_level() + unit.get_internal_level();
    let enemy = unit.get_person().get_asset_force().value == 1;
    let rng = Random_2::get_game().get_value_2(100);
    let inventory = DVCVariable::UnitInventory.get_value();
    let random_add = (inventory & 1 != 0 && !enemy) || (inventory & 2 != 0 && enemy);
    let r = if random_add { Random_2::get_game().get_value_2(2) } else { 0 };
    let story_weapon_level =
        if !DVCVariables::is_main_chapter_complete(4) || level < 10 { 1 + r }
        else {
            match level {
                0..10 => { r + 1 }  // D
                10..15 => { // D or C
                    r + if enemy { if rng < (level - 9) * 10 { 2 } else { 1 } } else { 2 }
                }
                15..20 => { r + 2 } // C
                20..25 => { r + if enemy { if rng < (level - 19) * 10 { 3 } else { 2 } } else { 3 } }
                25..30 => { r + 3 }
                30..40 => {
                    3 + ((Random_2::get_game().get_value_2(30) < (level - 30)) as i32)
                }
                _ => { r + if enemy { 5 } else { 4 } }
            }
        };
    let weapon_mask = unit.m_weapon_mask().m_value();
    for x in 1..10 {
        if weapon_mask & (1 << x) != 0 {
        let job_level = unit.get_job().get_max_weapon_level(x as i32).value;
            let level = if story_weapon_level < job_level { story_weapon_level } else { job_level };
            levels[x] = level;
        }
    }
    levels
}
pub fn change_misercode_type(){
    DVCVariable::MisercodeKind.init_var(5, false);
    let value = DVCVariable::MisercodeKind.get_value();
    let veyle_class = GameVariableManager::get_number("G_JG_PID_ヴェイル");
    if veyle_class == 185670709 || veyle_class == -1998645787 { return; }
    if value == 5 { return; }
    let misercode = ItemData::get("IID_ミセリコルデ".into());
    let equip = misercode.get_equip_skills();
    let give = misercode.get_give_skills();
    equip.clear();
    give.clear();
    misercode.set_range_o(2);
    misercode.set_range_i(1);
    misercode.set_kind(ItemData_Kinds{value});
    let flag = misercode.get_flag();
    flag.set_or_clear(false, 0x4000000);
    flag.set_or_clear(false, 0x8000000);
    match value  {
        4 => {
            equip.add("SID_飛行特効", SkillData_Categorys::equip(), 0);
            misercode.set_range_o(3);
            misercode.set_range_i(2);
        }
        5 => { give.add("SID_毒", SkillData_Categorys::item(), 0); }
        8 => {
            misercode.set_range_o(1);
            equip.add("SID_２回行動", SkillData_Categorys::equip(), 0);
        }
        9 => {
            misercode.set_equip_condition("SID_竜石装備");
            flag.set_or_clear(true, 0x4000000);
            misercode.set_range_o(1);
            misercode.set_range_i(1);
        }
        10 => {
            misercode.set_equip_condition("SID_弾丸装備");
            flag.set_or_clear(true, 0x8000000);
            misercode.set_range_o(10);
            misercode.set_range_i(3);
        }
        _ => {}
    }
}
pub fn change_liberation_type()  {
    DVCVariable::LiberationKind.init_var(1, false);
    let mut l_type = DVCVariable::LiberationKind.get_setting_value();
    if DVCVariables::is_main_chapter_complete(2) {
        if let Some(job) = GrowthData::get_starting_class(UnitPool::get_hero(false)) {
            let kind = get_max_job_weapon_type(job);
            if kind > 0 && kind != 7 && kind < 10 { l_type = kind; }
            DVCVariable::LiberationKind.set_value(l_type);
        }
    }
    if l_type == 0 || l_type > 9 {
        DVCVariable::LiberationKind.init_var(1, true);
        l_type = 1;
    }
    println!("Liberation Kind: {}", l_type);
    for x in ["IID_リベラシオン", "IID_リベラシオン改_ノーマル", "IID_リベラシオン改"]  {
        let liberation = ItemData::get(x.into());
        liberation.set_kind(ItemData_Kinds{value: l_type});
        let equip = liberation.get_equip_skills();
        let give = liberation.get_give_skills();
        let flag = liberation.get_flag();
        equip.clear();
        equip.add("SID_撃破時エンゲージカウント＋１", SkillData_Categorys::equip(), 0);
        give.clear();
        flag.set_or_clear(false, 0x4000000);
        flag.set_or_clear(false, 0x8000000);
        liberation.set_attr(ItemData_Attrs::physical());
        match l_type {
            4 => {
                liberation.set_range_o(3);
                liberation.set_range_i(2);
                equip.add("SID_飛行特効", SkillData_Categorys::equip(), 0);
            }
            5 => {
                liberation.set_range_o(2);
                liberation.set_range_i(1);
                give.add("SID_毒", SkillData_Categorys::item(), 0);
            }
            6 => {
                liberation.set_range_o(2);
                liberation.set_range_i(1);
                liberation.set_attr(ItemData_Attrs::magic());
            }
            8 => {
                liberation.set_range_o(1);
                equip.add("SID_２回行動", SkillData_Categorys::equip(), 0);
            }
            9 => {
                liberation.set_equip_condition("SID_竜石装備");
                flag.set_or_clear(true, 0x4000000);
                liberation.set_range_o(1);
                liberation.set_range_i(1);
            }
            10 => {
                liberation.set_equip_condition("SID_弾丸装備");
                flag.set_or_clear(true, 0x8000000);
                liberation.set_range_o(10);
                liberation.set_range_i(3);
            }
            _ => {
                liberation.set_range_o(1);
                liberation.set_range_i(1);
            }
        }
    }
}