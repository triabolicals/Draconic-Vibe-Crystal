use engage::gamedata::skill::SkillDataCategorys;
use engage::unit::{Unit, UnitPool};
use super::{*, ai};
use crate::{
    assets::animation::MONSTERS, config::DVCVariables, randomizer::{emblem::ENEMY_EMBLEM_LIST, grow, item::unit_items, job}
};
use crate::continuous::get_continious_total_map_complete_count;
use crate::randomizer::data::{GameData, RandomizedGameData};
use crate::randomizer::item::change_liberation_type;
use crate::randomizer::item::unit_items::add_generic_weapons;
use crate::randomizer::job::{is_magic_class, randomize_selected_weapon_mask};

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
    let can_lueur_change = RANDOMIZER_STATUS.read().unwrap().enabled;
    call_original!(this, method_info);
    if !can_lueur_change {
        if this.person.pid.str_contains(PIDS[ALEAR]) {
            let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.enabled = true);
        }
        return;
    }
    if !can_rand()  || this.person.parent.hash == 1879825845 || this.status.value & 134217728 != 0 { return; }  // Doubles
    let single_class = DVCVariables::get_single_class(false, false).is_some();
    let changed_recruit_order = DVCVariables::UnitRecruitment.get_value() != 0;
    let class_mode = DVCVariables::ClassMode.get_value();
    let random_class =
        if class_mode == 3 { job::lockout::get_all_playable_unit_classes(this.person).contains(&this.job.parent.hash) }
        else { class_mode == 1 || class_mode == 4 };

    let random_inventory = DVCVariables::UnitInventory.get_value() & 1 != 0;
    let adjust_items = changed_recruit_order || random_class || single_class;
    ai::adjust_person_unit_ai(this);
    let sequence = GameUserData::get_sequence();
    if !DVCVariables::is_main_chapter_complete(2) && changed_recruit_order {
        let old_person = switch_person_reverse(this.person).unwrap_or(this.get_person());
        if old_person.parent.index < 5 && old_person.parent.index > 0 {
            change_unit_autolevel(this, true);
            this.item_list.put_off_all_item();
            if random_class || single_class { job::unit_change_to_random_class(this, true);  }
            if (old_person.parent.index == 3 || old_person.parent.index == 4) && GameUserData::get_chapter().cid.str_contains("M001") {
                if random_inventory { adjust_unit_items(this); }
            }
            else { adjust_unit_items(this); }
            if sequence== 3 || sequence == 2 { ai::adjust_unitai(this); }
            unit_items::remove_duplicates(this.item_list);
            set_unit_edit_name(this);
            this.auto_equip();
            grow::adaptive_growths(this, true);
            auto_level_unit_for_random_map(this, false);
            this.set_hp(this.get_capability(0, true));
            return;
        }
    }
    if !is_player_unit(this.person) {
        if is_playable_person(this.person) {
            if random_class || single_class { job::unit_change_to_random_class(this, true);  }
            if adjust_items {  adjust_unit_items(this);  }
            if random_inventory { unit_items::adjust_missing_weapons(this); }
            grow::adaptive_growths(this, true);
            auto_level_unit_for_random_map(this, false);
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
            if this.person.parent.hash == 258677212 && DVCVariables::EnemyJobGauge.get_value() >= 10 {
                let mut stats: [i32; 11] = [0; 11];
                for x in 0..11 { stats[x] = this.job.base[x] as i32; }
                if let Some(unit) = UnitPool::get_hero(false){
                    let kind = change_liberation_type();
                    if unit.job.has_high_jobs() {
                        if let Some(new_job) = unit.job.get_high_jobs().iter().find(|job| job.weapons[kind as usize] >= 1) {
                            this.set_job(&new_job);
                            randomize_selected_weapon_mask(this, Some(kind));
                        }
                        else {
                            this.set_job(&unit.job);
                            this.selected_weapon_mask.value = unit.selected_weapon_mask.value;
                        }
                    }
                    else {
                        this.set_job(&unit.job);
                        this.selected_weapon_mask.value = unit.selected_weapon_mask.value;
                    }
                    if is_magic_class(this.job) {
                        let base = this.base_capability[1];
                        let str = stats[1];
                        stats[1] = stats[6];
                        stats[6] = str;
                        this.base_capability[1] = this.base_capability[6];
                        this.base_capability[6] = base;
                    }
                    for x in 0..11 {
                        let diff = stats[x] - this.job.base[x] as i32;
                        let base = (this.base_capability[x] as i32) + diff;
                        this.base_capability[x] = base as i8;
                    }
                }
            }
            else { enemy_unit_randomization(this); }
            let rng = Random::get_game();
            if rng.get_value(100) < DVCVariables::EnemyItemDropGauge.get_value() { unit_items::random_items_drops(this);  }
        }
        this.auto_equip();
        this.set_hp(this.get_capability(0, true));
        set_unit_edit_name(this);
        println!("Finish creating {} Lvl: {}/{}", this.get_name(), this.level, this.internal_level);
        return;
    }
    if changed_recruit_order {
        if ( sequence == 4 ||  sequence == 5 ) ||
            (sequence == 3 && ( GameVariableManager::get_bool("MapRecruit") || ( DVCVariables::UnitDeployment.get_value() != 3 && !lueur_on_map() ) ) )
        {
            // println!("Hub/Kizuna Recruitment");
            change_unit_autolevel(this, true);
            if this.person.pid.str_contains(PIDS[VEYLE]) {
                this.item_list.put_off_all_item();
                this.item_list.add_iid_no_duplicate("IID_オヴスキュリテ");
                this.item_list.add_iid_no_duplicate("IID_ミセリコルデ");
            }
        }
        else if switch_person(this.person).is_none_or(|v| v.parent.hash == this.person.parent.hash){
            if random_class || single_class  {
                job::unit_change_to_random_class(this, true);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
                if DVCVariables::UnitInventory.get_value() & 1 != 0 { unit_items::adjust_missing_weapons(this); }
            }
            grow::adaptive_growths(this, true);
            auto_level_unit_for_random_map(this, false);
            ai::adjust_unitai(this);
            this.set_hp(this.get_capability(0, true));
            return;
        }
        else { change_unit_autolevel(this, false);  }
        if this.person.pid.str_contains(PIDS[ALEAR]) && GameUserData::get_sequence() != 0 {
            this.edit.set_gender( GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) );
            if GameVariableManager::exist(DVCVariables::LUEUR_NAME) {
                this.edit.set_name( GameVariableManager::get_string(DVCVariables::LUEUR_NAME) );
            }
        }
    }
    if random_class || single_class {  job::unit_change_to_random_class(this, true);  }
    if adjust_items {
        adjust_unit_items(this);
        if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { ai::adjust_unitai(this); }
    }
    unit_items::remove_duplicates(this.item_list);
    set_unit_edit_name(this);
    this.auto_equip();
    grow::adaptive_growths(this, true);
    auto_level_unit_for_random_map(this, false);
    this.set_hp(this.get_capability(0, true));
    println!("Finish creating {} Lvl: {}/{}", this.get_name(), this.level, this.internal_level);
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
        has_drops = 0;
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
    if has_drops != 0 {
        let iid = GameData::get_item_pool().random_item(5, has_drops == 2 ).to_string();
        unit.item_list.add_iid_no_duplicate(&iid);
        if let Some(x) = unit.item_list.unit_items.iter_mut()
            .flatten().find(|x| x.item.iid.to_string() == iid) { x.flags |= 2; }
    }

    if unit.person.get_asset_force() == 0 {
        if unit.get_capability(0, true) >= 45 { unit.item_list.add_iid_no_duplicate("IID_特効薬") }
        else { unit.item_list.add_iid_no_duplicate("IID_傷薬") }
    }
    else if DVCVariables::UnitInventory.get_value() & 2 != 0 && get_continious_total_map_complete_count() > 10 {
        let rng = Random::get_system();
        let playable_gods = GameData::get_playable_god_list();
        unit.item_list.unit_items.iter().flat_map(|x| x.as_ref().filter(|x| x.is_weapon() && x.item.parent.index > 2 && rng.get_value(10) < 2 ))
            .for_each(|item| {
                if let Some(god) = playable_gods.get_random_element(rng) { item.set_engrave(god); }
            });
    }
}

pub fn set_unit_edit_name(unit: &Unit) {
    if unit.person.pid.str_contains(PIDS[0]) || unit.person.flag.value & 1024 != 0 {
        if GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) != 0 {
            unit.edit.set_gender( GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) );
        }
        else {unit.edit.set_gender( 1 );  }
        if GameVariableManager::exist(DVCVariables::LUEUR_NAME) { unit.edit.set_name( GameVariableManager::get_string(DVCVariables::LUEUR_NAME) ); }
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
            if GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) != 0 {
                unit.edit.set_gender(GameVariableManager::get_number(DVCVariables::LUEUR_GENDER));
            } else { unit.edit.set_gender(1); }
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
    let new_person = if reverse { &unit.person } else { switch_person(unit.person).unwrap_or(unit.get_person()) };
    if new_person.parent.hash == person.parent.hash { return; }
    println!("{} -> {}",  Mess::get_name(person.pid), Mess::get_name(new_person.pid));
    let is_low = person.get_job().unwrap().is_low();    
    let is_new_low = new_person.get_job().unwrap().is_low();
    let current_level = person.get_level() as i32;
    let mut current_internal_level = person.get_internal_level() as i32;
    if current_internal_level == 0 && !is_low { current_internal_level = 20; }
    unit.set_person(person);
    unit.class_change(person.get_job().unwrap());
    if is_low {
       if current_level > 20 { //Old Unit is in a special class so new unit needs to be promoted
            if is_new_low {
                if new_person.get_job().unwrap().has_high_jobs() {    // new unpromoted unit can promoted
                    let level = current_level - 20;
                    let new_job = &new_person.get_job().unwrap().get_high_jobs()[0];
                    unit.auto_grow_capability( level, current_level);
                    unit.class_change(new_job);
                    unit.set_level( level );
                    unit.set_internal_level( 20 );
                }
                else {   // special -> special
                    unit.class_change(new_person.get_job().unwrap());
                    unit.auto_grow_capability( current_level, current_level);
                    unit.set_level( current_level );
                    unit.set_internal_level( 0 );
                }
            }
            else {  // special -> high
                unit.class_change(new_person.get_job().unwrap());
                unit.auto_grow_capability( current_level-20, current_level);
                unit.set_level( current_level - 20 );
                unit.set_internal_level( 20 );
            }
        }
        else if is_new_low { // base or special class lvl < 20 -> base class
            unit.class_change(new_person.get_job().unwrap());
            unit.auto_grow_capability( current_level, current_level);
            unit.set_level( current_level );
            unit.set_internal_level( 0 );
          //  println!("{} -> {} Base -> Base Level {}",  person.get_name().unwrap().to_string(), new_person.get_name().unwrap().to_string(), current_level);
        }
        else {
            let new_job_list = get_base_classes(new_person.get_job().unwrap());
            unit.auto_grow_capability(current_level, current_level);
            if new_job_list.len() == 3 {
                let index = super::get_low_class_index(new_person);
                unit.class_change(&new_job_list[index as usize]);
            }
            else if new_job_list.len() == 0 { unit.class_change(JobData::get("JID_ソードファイター").unwrap()); }    // if promoted class doesn't have a low class, change to sword fighter
            else {  unit.class_change(&new_job_list[0]); }
            unit.set_level(current_level);
            unit.set_internal_level(0);
        }
    }
    else {  // Promoted
        if is_new_low { // new unit has a base class
            let total_level = current_internal_level + current_level;
            if new_person.get_job().unwrap().has_high_jobs() {   // base class -> 1st promoted class
                let new_high_jobs = new_person.get_job().unwrap().get_high_jobs();
                if new_high_jobs.len() == 0 { unit.class_change(JobData::get("JID_ソードマスター").unwrap());  } // if no high class, change to Swordmaster
                else { unit.class_change(&new_high_jobs[0]); }
                unit.auto_grow_capability(current_level, current_level + 20);
                unit.set_level(current_level);
                unit.set_internal_level(current_internal_level);
                println!("Promoted Unit -> Base Unit");
            }
            else { // Promoted -> Special
                if DVCVariables::ClassMode.get_value()== 1 {
                    unit.class_change(new_person.get_job().unwrap());
                    println!("Promoted Unit -> Special Unit");
                }
                else { unit.class_change(new_person.get_job().unwrap()); }
                unit.auto_grow_capability(total_level, 20+current_level);
                unit.set_level(total_level);
                unit.set_internal_level(0);
            }
        }
        else {  // Promoted -> Promoted
            unit.class_change(new_person.get_job().unwrap());
            unit.auto_grow_capability(current_level, current_level + 20);
            unit.set_level(current_level);
            unit.set_internal_level( current_internal_level );
        }
    }
    let bases = calculate_new_offset(person, new_person);
    for x in 0..11 {  unit.set_base_capability(x as i32, bases[x] as i32);  }

    unit.set_sp( person.get_sp() );
    unit.set_person(new_person);    // change person
    fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes  // Random map order level adjustment
    println!("{}: Level {} / {} from Old Person: {} | {} ", unit.get_name(), unit.level, unit.internal_level, person.level, person.internal_level);
}

fn calculate_new_offset(original: &PersonData, new: &PersonData) -> [i8; 11] {
    let original_job = original.get_job().expect("Original Person does not have a valid default class in Person.xml");
    let new_job = new.get_job().expect("Replacement Person does not have a valid default class in Person.xml");
    let mut out: [i8; 11] = [0; 11];
    let old_level = if original_job.is_high() { 20  + original.get_level() as i32 }  else { original.get_level() as i32 };
    let new_level = if new_job.is_high() { 20 + new.get_level() as i32 } else { new.get_level() as i32 };
    
    let factor = new_level - old_level;
    let new_offset = new.get_offset_n();
    let new_grow = new.get_grow();
    let diff_grow = new_job.get_diff_grow();

    let n_autolevels =
        if DVCVariables::is_random_map() && DVCVariables::is_main_chapter_complete(4) {
            max(crate::continuous::random::random_map_mode_level(), engage::util::get_instance::<engage::map::situation::MapSituation>().average_level) +
                get_continious_total_map_complete_count() / 3
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
    // Everyone calculate offset by subtracting class growths to adjust to original unit's level
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
    UnitPool::class().get_static_fields_mut::<job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type < 3  )).for_each(|unit|{
            unit.reload_actor();
            unit.auto_equip();
    });
}
fn enemy_unit_randomization(unit: &mut Unit) {
    let x = unit.dispos_y as i8;
    let z = unit.dispos_z as i8;
    let diff = 1 << GameUserData::get_difficulty(false);
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
            if random_map && m004_complete && !GameUserData::get_chapter().cid.contains("E00") { // Continuous Mode Random Map
                fixed_unit_weapon_mask(unit);
                let maps_completed = get_continious_total_map_complete_count();
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
            if DVCVariables::ClassMode.get_value()== 1  && m004_complete {
                let gauge = DVCVariables::EnemyJobGauge.get_value();
                if unit.person.get_bmap_size()  == 1 && ( rng.get_value(100) < gauge && gauge > 11 )  || ( gauge > 0 && gauge <= 11  && is_boss ) {
                    if job::enemy_unit_change_to_random_class(unit){
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
                    println!("Gain Skill: {}", Mess::get(skill.name.unwrap()));
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
                let current_chapter = GameUserData::get_chapter().cid.to_string();
                if current_chapter != "CID_M022" && current_chapter != "CID_M011"  {
                    let emblem = rng.get_value( ENEMY_EMBLEM_LIST.get().unwrap().len() as i32) as usize;
                    if enemy::try_equip_emblem(unit, emblem) {
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
            ai::adjust_unitai(unit);
        }
        if GameUserData::is_evil_map() { auto_level_unit(unit, is_boss); }
        unit.auto_equip();
        unit.set_hp(unit.get_capability(0, true));
    }
}