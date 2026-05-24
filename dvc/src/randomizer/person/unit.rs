use engage::{
    gamedata::skill::SkillDataCategorys,
    unit::{Unit, UnitPool},
    gamedata::terrain::TerrainData,
    map::terrain::MapTerrain
};
use crate::{
    assets::animation::MONSTERS,
    config::DVCVariables,
    randomizer::{
        grow, item::unit_items, job,
        data::{GameData, RandomizedGameData},
        item::{unit_items::add_generic_weapons, change_liberation_type},
        job::{is_magic_class, randomize_selected_weapon_mask},
        job::reclass::ReclassType,
        status::RandomizerStatus
    }
};
use crate::assets::dress::M002_LUMERA;
use crate::randomizer::item::{MISERCODE, OBSCURITE};
use super::{*, ai};

const VANDRE: i32 = 152765422;
const CLANNE: i32 = 1875144918;
const FRAMME: i32 = 654010808;
const YUNAKA: i32 = 1172357650;
const SEADALL: i32 = -266109647;
const VEYLE: i32 = 356559395;
const NEL: i32 = 2023447537;
const RAFALE: i32 = 1696364213;
const ILLUSORY_DOUBLE: i32 = 1879825845;
const VANDER_MAX: [i8; 11] = [45, 12, 14, 11, 40, 12, 13, 12, 10, 5, 7];

fn calc_max_recruit_stat(total_level: i32) -> [i32; 11] {
    let mut max: [i32; 11] = [0; 11];
    max[0] = 125 * total_level + 2600;
    max[8] = 20 * total_level + 1000;
    for x in [1, 2, 3, 6] { max[x] = 75*total_level + 1000; }
    for x in [5, 7] { max[x] = 60*total_level + 1100; }
    max[4] = 10000;
    max[9] = 1000;
    max[10] = 2000;
    max
}

#[unity::hook("App", "Unit", "CreateImpl2")]
pub fn unit_create_impl_2_hook(this: &mut Unit, method_info: OptionalMethod){
    call_original!(this, method_info);
    if !RandomizerStatus::get().init { return; }
    if !can_rand() || this.person.parent.hash == 1879825845 || this.status.value & 134217728 != 0 { return; }  // Doubles

    let changed_recruit_order = DVCVariables::UnitRecruitment.get_value() != 0;
    let class_mode = DVCVariables::ClassMode.get_value();
    let reclass_mode = ReclassType::get_from_settings(true);
    let random_class = class_mode > 0;
    let random_inventory = DVCVariables::UnitInventory.get_value() & 1 != 0;
    let adjust_items = changed_recruit_order || random_class;

    ai::adjust_person_unit_ai(this);
    let sequence = GameUserData::get_sequence();
    if !DVCVariables::is_main_chapter_complete(2) && changed_recruit_order {
        let old_person = switch_person_reverse(this.person).unwrap_or(this.get_person());
        if old_person.parent.index < 5 && old_person.parent.index > 0 {
            change_unit_autolevel(this, true);
            this.item_list.put_off_all_item();
            if random_class { job::reclass::unit_reclass(this, reclass_mode);  }
            post_unit_creation_adjustment(this);
            return;
        }
    }
    if !is_player_unit(this.person) {
        if is_playable_person(this.person) {
            if random_class { job::reclass::unit_reclass(this, reclass_mode);  }
            if adjust_items {  adjust_unit_items(this);  }
            if random_inventory { unit_items::adjust_missing_weapons(this); }
            if this.person.unit_icon_id.is_some_and(|x| x.str_contains("Lueur")) && this.person.parent.index > 1 {
                let gender = this.person.gender;
                this.edit.set_gender(gender);
            }
        }
        else {
            if let Some(v) = GameData::get().enemy.iter()
                .find(|v| v.hash == this.person.parent.hash)
                .and_then(|p| p.playable_slot)
                .and_then(|p| DVCVariables::get_dvc_person_data(p as i32, false))
            {
                this.edit.set_name(Mess::get_name(v.pid));
                this.edit.set_gender(v.gender);
            }
            enemy_unit_randomization(this);
            let rng = Random::get_game();
            if rng.get_value(100) < DVCVariables::EnemyItemDropGauge.get_value() { unit_items::random_items_drops(this);  }
        }
        post_unit_creation_adjustment(this);
        return;
    }
    if changed_recruit_order {
        if ( sequence == 4 || sequence == 5 ) ||
            (sequence == 3 && ( GameVariableManager::get_bool("MapRecruit") || ( DVCVariables::UnitDeployment.get_value() != 3 && !lueur_on_map() ) ) )
        {
            change_unit_autolevel(this, true);
        }
        else if switch_person(this.person).is_none_or(|v| v.parent.hash == this.person.parent.hash){
            if random_class {
                job::reclass::unit_reclass(this, reclass_mode);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
                if DVCVariables::UnitInventory.get_value() & 1 != 0 { unit_items::adjust_missing_weapons(this); }
            }
            post_unit_creation_adjustment(this);
            return;
        }
        else { change_unit_autolevel(this, false);  }
    }
    if random_class {  job::reclass::unit_reclass(this, reclass_mode);  }
    if adjust_items { adjust_unit_items(this); }
    post_unit_creation_adjustment(this);
}

fn unit_set_drop_seals(this: &mut Unit) {
    for x in 0..8 {
        if let Some(item) = this.item_list.get_item(x) {
            if item.item.iid.str_contains("プルフ") { item.set_flags(2); }
        }
    }
}
pub fn fixed_unit_weapon_mask(this: &mut Unit){
    this.original_aptitude.value = this.person.aptitude.value;
    this.aptitude.value = this.original_aptitude.value | this.person.sub_aptitude.value;
    this.selected_weapon_mask.value = 0;
    this.update_weapon_mask();
    this.set_selected_weapon_from_original_aptitude(this.original_aptitude);
    this.update_weapon_mask();
    this.add_aptitude_from_weapon_mask();
}

pub fn adjust_unit_items(unit: &mut Unit) {
    let job = unit.get_job();
    let jid = job.jid.to_string();
    let is_enemy = unit.person.get_asset_force() != 0;
    if MONSTERS.iter().any(|&x| jid == x) {
        unit.item_list.put_off_all_item();
        unit_items::add_monster_weapons(unit);
        return;
    }
    unit.update_weapon_mask();
    let mut has_drops = unit_items::has_drops(unit);
    let custom_inventory = DVCVariables::UnitInventory.get_value() & 255;
    let ran_map = DVCVariables::is_random_map();
    if custom_inventory & 1 != 0 && !is_enemy {
        unit.item_list.put_off_all_item();
        add_generic_weapons(unit);
    }
    else if custom_inventory & 2 != 0 && is_enemy && ((DVCVariables::is_main_chapter_complete(11) && ran_map) ||
        DVCVariables::is_main_chapter_complete(9))
    {
        unit.item_list.put_off_all_item();
        add_generic_weapons(unit);
    }
    else {
        if !is_enemy { has_drops = 0; }
        GameData::get_item_pool().weapon_db.do_simple_replacement(unit, false);
    }
    unit_items::assign_tomes(unit);
    unit_items::assign_unique_items(unit);
    unit_items::assign_staffs(unit);

    unit_items::adjust_melee_weapons(unit);
    unit_items::adjust_missing_weapons(unit);
    unit_items::add_equip_condition(unit);
    unit_items::remove_duplicates(unit.item_list);
    unit.auto_equip();

    if unit.person.get_asset_force() == 0 {
        if unit.get_capability(0, true) >= 45 { unit.item_list.add_iid_no_duplicate("IID_特効薬") }
        else { unit.item_list.add_iid_no_duplicate("IID_傷薬") }
    }
    else if DVCVariables::UnitInventory.get_value() & 2 != 0 && DVCVariables::chapter_number_complete(true) > 10 {
        let rng = Random::get_system();
        let playable_gods = GameData::get_playable_emblem_hashes();
        unit.item_list.unit_items.iter().flat_map(|x| x.as_ref().filter(|x| x.is_weapon() && x.item.parent.index > 2 && rng.get_value(10) < 2 ))
            .for_each(|item| {
                if let Some(god) = playable_gods.get_random_element(rng).and_then(|v| GodData::try_get_hash(*v)){
                    item.set_engrave(god);
                }
            });
    }
}

pub fn set_unit_edit_name(unit: &Unit) {
    let lueur_gender = DVCVariables::LueurGender.get_value();
    if unit.person.parent.index == 1 || unit.person.flag.value & 1024 != 0 {
        unit.edit.set_gender(if lueur_gender != 0 { lueur_gender } else { 1 });
        if GameVariableManager::exist(DVCVariables::LUEUR_NAME) {
            unit.edit.set_name( GameVariableManager::get_string(DVCVariables::LUEUR_NAME) );
        }
    }
    if !is_player_unit(unit.person) {
        if let Some(appearance) = RandomizedGameData::get_read().person_appearance.get_unit_appearance(unit) {
            unit.edit.set_name(Mess::get(appearance.mpid.as_str()));
            if DVCFlags::RandomBossesNPCs.get_value() { unit.edit.set_gender(if appearance.is_female { 2 } else { 1 }) }
            return;
        }
    }
    if unit.person.flag.value & 128 != 0 && unit.person.get_job().is_some_and(|j| j.jid.to_string().contains("JID_邪竜ノ子")){
        if let Some(pid) = GameVariableManager::try_get_string("G_R_PID_ヴェイル").and_then(|pid| PersonData::get(pid)).filter(|p| p.parent.index > 1) {
            unit.edit.set_name(pid.name.unwrap());
            if pid.gender == 1 {
                if pid.flag.value & 32 != 0 { unit.edit.set_gender( 2 ); }
                else { unit.edit.set_gender( 1 ); }
            }
            else {
                if pid.flag.value & 32 != 0 { unit.edit.set_gender( 1 ); }
                else { unit.edit.set_gender( 2 ); }
            }
        }
        else {
            unit.edit.set_name(GameVariableManager::get_string(DVCVariables::LUEUR_NAME));
            unit.edit.set_gender(if lueur_gender  != 0 { lueur_gender } else { 1 });
        }
    }
}

pub fn change_unit_autolevel(unit: &mut Unit, reverse: bool) {
    let person = if reverse { switch_person_reverse(unit.person).unwrap_or(unit.get_person()) }
    else { unit.get_person() };

    if DVCVariables::is_random_map() && person.parent.hash == -266109647 {
        unit.extra_hp_stock_count = 1;
        unit.extra_hp_stock_count_max = 1;
    }
    let new_person = if reverse { unit.get_person() } else { switch_person(unit.person).unwrap_or(unit.get_person()) };
    if new_person.parent.hash == person.parent.hash { return; }
    println!("{} -> {}",  Mess::get_name(person.pid), Mess::get_name(new_person.pid));
    unit.set_sp( person.get_sp() );
    unit.set_person(new_person);
    job::reclass::unit_reclass(unit, ReclassType::get_from_settings(true));
    let bases = calculate_new_offset(person, new_person);
    for x in 0..11 {  unit.set_base_capability(x as i32, bases[x] as i32);  }
    unit.set_sp( person.get_sp() );
    fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes  // Random map order level adjustment
}

fn calculate_new_offset(original: &PersonData, new: &PersonData) -> [i8; 11] {
    let original_job = original.get_job().unwrap();
    let new_job = new.get_job().unwrap();
    let mut out: [i8; 11] = [0; 11];
    let old_level = if original_job.is_high() { 20  + original.get_level() as i32 }  else { original.get_level() as i32 };
    let new_level = if new_job.is_high() { 20 + new.get_level() as i32 } else { new.get_level() as i32 };
    
    let factor = new_level - old_level;
    let new_offset = new.get_offset_n();
    let new_grow = new.get_grow();
    let diff_grow = new_job.get_diff_grow();

    let n_autolevels =
        if DVCVariables::is_random_map() && DVCVariables::is_main_chapter_complete(4) {
            crate::continuous::random::random_map_mode_level()
        }
        else { ( if original_job.is_high() { 20 } else { 0 } + original.get_level() ) as i32 };

    if original.pid.contains(PIDS[1]) {
        let new_class_bases = new_job.get_base();
        for x in 0..9 {
            let base = new_class_bases[x] as i8;
            let round = if diff_grow[x] as i32 + new_grow[x] as i32 > 0 { 50 } else { 0 };
            let value = (( 100 * new_offset[x] as i32 - factor * diff_grow[x] as i32 + round +  ( n_autolevels  * new_grow[x] as i32) + 100*base as i32 ) / 100) as i8;
            out[x] = if value >= (VANDER_MAX[x]) { VANDER_MAX[x] - base } else { value - base };
        }
    }
    else {
        let new_base = new_job.base;
        let max = calc_max_recruit_stat(n_autolevels);
        for x in 0..11 {
            let class_base = new_base[x] as i32 * 100;
            let round = if diff_grow[x] as i32 + new_grow[x] as i32 > 0 { 50 } else { 0 };
            let value: i32 = 100 * new_offset[x] as i32 - factor * diff_grow[x] as i32;
            let offset = if value < 0 { 0 } else { round + value };
            let new_base = offset + (n_autolevels  * new_grow[x] as i32);
            let total = if (new_base + class_base ) >= max[x] {  max[x] - class_base } else { new_base };
            out[x] = (total/100) as i8;
        }
    }
    out
}

pub fn has_skill(this: &Unit, skill: &SkillData) -> bool {
    if let Some(learn) = this.learned_job_skill {
        if skill.parent.hash == learn.parent.hash { return true; }
    }
    this.mask_skill.unwrap().find_sid(skill.sid).is_some() |
        this.private_skill.find_sid(skill.sid).is_some()|
        this.equip_skill.find_sid(skill.sid).is_some()
}
pub fn has_sid(this: &Unit, sid: &str) -> bool {
    if let Some(learn) = this.learned_job_skill {
        if sid == learn.sid.to_string() { return true; }
    }
   this.mask_skill.unwrap().find_sid(sid).is_some() |
       this.private_skill.find_sid(sid).is_some() |
       this.equip_skill.find_sid(sid).is_some()
}

pub fn reload_all_actors() {
    for_each_unit(15, |unit|{
        unit.reload_actor();
        unit.auto_equip();
    });
}
fn enemy_unit_randomization(unit: &mut Unit) {
    let x = unit.dispos_y as i8;
    let z = unit.dispos_z as i8;
    let diff = 1 << GameUserData::get_difficulty(false);
    let chapter_idx = DVCVariables::get_chapter_index();
    if let Some(data) = DisposData::get_list().unwrap().iter()
        .flat_map(|array| array.iter())
        .find(|data| 
            data.flag.value & diff != 0 &&
            data.get_person().is_some_and(|dispos_person| dispos_person.parent.hash == unit.person.parent.hash) && data.dispos_x == x && data.dispos_y == z )
    {
        let rng = Random::get_game();
        let is_boss = data.flag.value & 16 != 0 ;
        let mut changed_class = false;
        let random_map = DVCVariables::is_random_map();
        let difficulty = GameUserData::get_difficulty(false);
        let m004_complete = DVCVariables::is_main_chapter_complete(4);
        if let Some(person) = data.get_person() {
            if person.engage_sid.is_none() {
                 if ( DVCVariables::random_enabled() && person.get_asset_force() != 0 ) &&
                     ((is_boss && DVCVariables::EnemySkillGauge.get_value() == 10 ) || (DVCVariables::EnemySkillGauge.get_value() > 10))
                  {
                    if let Some(dispos_skill) = data.sid {
                        unit.private_skill
                            .replace_sid(dispos_skill, GameData::get_random_skill_dispos(diff, rng));
                    }
                    else if rng.get_value(20) < 2* difficulty && m004_complete {
                        unit.private_skill.add_skill(GameData::get_random_skill_dispos(diff, rng), SkillDataCategorys::Private, 0);
                    }
                }
            }
        }
        if unit.person.parent.hash == 1879825845 || unit.status.value & 134217728 != 0 { return; }
        let job = unit.get_job();
        if MONSTERS.iter().any(|str| job.jid.contains(str)) &&  m004_complete {
            if random_map { auto_level_unit_for_random_map(unit, is_boss); } else { auto_level_unit(unit, is_boss); }
            return;  
        }
        let mut has_master = unit.item_list.has_item_iid("IID_マスタープルフ");
        set_unit_edit_name(unit);
        if unit.person.flag.value & 512 == 512 && DVCVariables::UnitRecruitment.get_value() != 0 {  // Person was change
            fixed_unit_weapon_mask(unit);
            adjust_unit_items(unit); 
            ai::adjust_unitai(unit);
        }
        if (unit.person.get_asset_force() | 2 == 2) && DVCVariables::ClassMode.get_value()== 1{  ai::adjust_unitai(unit);  }
        if unit.person.get_asset_force() != 0 {
            if random_map && m004_complete && chapter_idx < 60 { // Continuous Mode Random Map
                fixed_unit_weapon_mask(unit);
                let maps_completed = DVCVariables::chapter_number_complete(true);
                if maps_completed < 16 {
                    unit.item_list.put_off_all_item();
                    adjust_unit_items(unit); 
                    unit.auto_equip();
                }
                else { adjust_unit_items(unit);  }
                if data.flag.value & 16 != 0 {  // If leader then add seal
                    match maps_completed {  
                        7|10|11 => { 
                            has_master = false;
                            unit.item_list.add_iid_no_duplicate("IID_マスタープルフ");
                        }
                        8|12|15 => { unit.item_list.add_iid_no_duplicate("IID_チェンジプルフ");   }
                        _ => {},
                    }
                }
                changed_class = true;
                unit_items::adjust_missing_weapons(unit);
            }
            if m004_complete {
                let gauge = DVCVariables::EnemyJobGauge.get_value();
                if unit.person.get_bmap_size()  == 1 && ( rng.get_value(100) < gauge && gauge > 11 )  || ( gauge > 0 && gauge <= 11  && is_boss ) {
                    if job::reclass::unit_reclass(unit, ReclassType::Enemy){
                        changed_class = true;
                        fixed_unit_weapon_mask(unit);
                        adjust_unit_items(unit); 
                        if unit.person.get_asset_force() == 2 { add_generic_weapons(unit);  }
                        ai::adjust_unitai(unit);
                        if !unit.get_job().diff_grow_lunatic.is_zero() {
                            let level = if unit.get_job().get_max_level() == 40 { unit.level as i32 + unit.internal_level as i32 } else {  unit.level as i32 };
                            let target_level = if unit.get_job().is_high() { level + 20 } else { level };
                            unit.auto_grow_capability(level, target_level);
                            if unit.get_job().get_max_level() == 40 { unit.internal_level = 0; }
                        }
                    }
                }
            }
            if DVCVariables::EnemyItemDropGauge.get_value() > 0 { unit_items::random_items_drops(unit); }
            if !m004_complete { 
                unit.auto_equip();
                return;
            }
            let gauge = DVCVariables::EnemySkillGauge.get_value();
            if ( ( rng.get_value(100) < gauge && gauge > 11 ) || ( gauge > 0 && gauge <= 11 && is_boss ) )  && unit.person.engage_sid.is_none() {
                if let Some(skill) = GameData::get_random_skill_job(GameUserData::get_difficulty(false), rng, unit){
                    unit.private_skill.add_skill(skill, SkillDataCategorys::Private, 0);
                }
            }
            let stone_rate = DVCVariables::EnemyRevivalStone.get_value();
            if stone_rate > 0 && DVCVariables::is_main_chapter_complete(5) {
                if (stone_rate <= 10 && is_boss) || (rng.get_value(100) <  stone_rate) {
                    unit.hp_stock_count += 1;
                    unit.hp_stock_count_max += 1;
                }
            }
            if unit.person.get_asset_force() == 1 && rng.get_value(100) < DVCVariables::EnemyEmblemGauge.get_value() &&
                (  unit.person.engage_sid.is_none() && unit.get_god_unit().is_none())
            {
                if chapter_idx != 22 && chapter_idx != 11 {
                    if enemy::try_equip_emblem(unit) {
                        ai::adjust_enemy_emblem_unit_ai_flags(unit);
                    }
                }
            } 
            else if unit.person.engage_sid.is_some() || unit.get_engage_attack().is_some()  { ai::adjust_ai_for_engage_attack(unit);  }
            auto_level_unit(unit, is_boss);
        }
        if has_master {  unit.item_list.add_iid_no_duplicate("IID_マスタープルフ"); }    // Add Seal if lost seal
        unit_set_drop_seals(unit);    // Drop Seals
        
        if changed_class {
            unit_items::adjust_missing_weapons(unit);
            enemy_check_soar(unit);
            ai::adjust_unitai(unit);
        }
        if GameUserData::is_evil_map() { auto_level_unit(unit, is_boss); }
        unit.auto_equip();
        unit.set_hp(unit.get_capability(0, true));
    }
}
pub fn enemy_check_soar(unit: &Unit) {
    if unit.person.get_asset_force() == 0 { return; }
    let dispos_x = unit.dispos_y as i32;
    let dispos_z = unit.dispos_z as i32;
    if dispos_x >= 32 || dispos_z >= 32 { return; }
    if let Some(map_terrain) = MapTerrain::get_instance() {
        if let Some(terrain) = map_terrain.get_tid(dispos_x, dispos_z).and_then(|tid| TerrainData::get(tid)) {
            if terrain.is_flight_only() && unit.job.move_type != 3 {
                unit.add_private_skill(SkillData::get("SID_天駆_飛行").unwrap());
                unit.set_base_capability(10, -3);
            }
        }
    }
}
/// Unit is assumed to be the correct person after recruitment swaps, etc...
pub fn post_unit_creation_adjustment(unit: &mut Unit) {
    if unit.person.parent.hash == VEYLE {
        unit.item_list.add_iid_no_duplicate(MISERCODE);
        unit.item_list.add_iid_no_duplicate(OBSCURITE);
    }
    else if let Some(old_person) = switch_person_reverse(unit.person) {
        let person_hash = old_person.parent.hash;
        match person_hash{
            CLANNE|FRAMME => {
                if DVCVariables::get_chapter_index() == 1 {
                    unit.item_list.put_off_all_item();
                    if DVCVariables::UnitInventory.get_value() & 1 != 0 { adjust_unit_items(unit); }
                }
                else { adjust_unit_items(unit); }
            }
            YUNAKA => { unit.item_list.add_iid_no_duplicate("IID_リライブ"); }
            NEL => {

            }
            RAFALE => {

            }
            SEADALL => {
                if DVCVariables::is_random_map() {
                    unit.extra_hp_stock_count = 1;
                    unit.extra_hp_stock_count_max = 1;
                }
            }
            _ => {}
        }
    }
    else if unit.person.parent.hash == M002_LUMERA && DVCVariables::EnemyJobGauge.get_value() >= 10 {
        let mut stats: [i32; 11] = [0; 11];
        for x in 0..11 { stats[x] = unit.job.base[x] as i32; }
        if let Some(unit) = UnitPool::get_hero(false){
            let kind = change_liberation_type();
            if unit.job.has_high_jobs() {
                if let Some(new_job) = unit.job.get_high_jobs().iter().find(|job| job.weapons[kind as usize] >= 1) {
                    unit.set_job(&new_job);
                    randomize_selected_weapon_mask(unit, Some(kind));
                }
                else {
                    unit.set_job(&unit.job);
                    unit.selected_weapon_mask.value = unit.selected_weapon_mask.value;
                }
            }
            else {
                unit.set_job(&unit.job);
                unit.selected_weapon_mask.value = unit.selected_weapon_mask.value;
            }
            if is_magic_class(unit.job) {
                let base = unit.base_capability[1];
                let str = stats[1];
                stats[1] = stats[6];
                stats[6] = str;
                unit.base_capability[1] = unit.base_capability[6];
                unit.base_capability[6] = base;
            }
            for x in 0..11 {
                let diff = stats[x] - unit.job.base[x] as i32;
                let base = (unit.base_capability[x] as i32) + diff;
                unit.base_capability[x] = base as i8;
            }
        }
    }
    unit_items::remove_duplicates(unit.item_list);
    set_unit_edit_name(unit);
    unit.auto_equip();
    if unit.person.asset_force == 0 && unit.person.parent.hash != ILLUSORY_DOUBLE {
        grow::adaptive_growths(unit, true);
    }
    auto_level_unit_for_random_map(unit, false);
    unit.set_hp(unit.get_capability(0, true));
    let sequence = GameUserData::get_sequence();
    if sequence == 2 || sequence == 3 {
        ai::adjust_unitai(unit);
    }
}