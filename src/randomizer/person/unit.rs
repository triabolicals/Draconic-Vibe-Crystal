use super::{*, ai};
use crate::{
    assets::animation::MONSTERS, config::DVCVariables, randomizer::{emblem::ENEMY_EMBLEM_LIST, grow, item::unit_items, job, skill}
};
use crate::randomizer::item::random_item;

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
    if this.person.parent.hash == 1879825845 || this.status.value & 134217728 != 0 { return; }  // Doubles
    let single_class = DVCVariables::get_single_class(false).is_some();
    if !can_rand() && single_class && (is_player_unit(this.person) || is_playable_person(this.person)){
        job::unit_change_to_random_class(this, true);
        adjust_unit_items(this);
        this.auto_equip();
        return;
    }
    if !can_rand()  { return; }
    let changed_recruit_order = GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0;
    let random_class = GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0;
    let random_inventory = GameVariableManager::get_number(DVCVariables::PLAYER_INVENTORY) & 1 != 0;
    let adjust_items = changed_recruit_order || random_class || single_class;

    ai::adjust_person_unit_ai(this);
    if !is_player_unit(this.person) {
        if is_playable_person(this.person) {
            if random_class || single_class { job::unit_change_to_random_class(this, true);  }
            if adjust_items {  adjust_unit_items(this);  }
            if random_inventory { unit_items::adjust_missing_weapons(this); }
            grow::adaptive_growths(this, true);
            auto_level_unit_for_random_map(this, false);
            if this.person.unit_icon_id.is_some_and(|x| x.str_contains("Lueur")) {
                this.person.set_gender(GameVariableManager::get_number(DVCVariables::LUEUR_GENDER));
                this.edit.set_gender(GameVariableManager::get_number(DVCVariables::LUEUR_GENDER));
            }
        }
        else {  // Enemy Randomization
            println!("Enemy Randomization of {}", Mess::get_name(this.person.pid));
            crate::assets::accessory::accesorize_enemy_unit(this); 
            enemy_unit_randomization(this);
            let rng = Random::get_game();
            if rng.get_value(100) < 2*GameVariableManager::get_number(DVCVariables::ITEM_DROP_GAUGE_KEY) { unit_items::random_items_drops(this);  }
        }
        this.auto_equip();
        this.set_hp(this.get_capability(0, true));
        println!("Finish creating {}", Mess::get_name(this.person.pid));
        return;
    }
    if changed_recruit_order {
        let old_person = switch_person_reverse(this.person).pid.to_string();
        if DVCVariables::get_dvc_person(0, false) == this.person.pid && DVCVariables::is_main_menu() {    //IsLueur
            change_unit_autolevel(this, true);
            this.item_list.put_off_all_item();
            this.item_list.add_iid_no_duplicate("IID_鉄の剣");
            this.item_list.add_iid_no_duplicate("IID_傷薬");
        }
        else if GameVariableManager::get_bool("DDFanClub") {
            change_unit_autolevel(this, true);
            if old_person == PIDS[2] || old_person == PIDS[3] {
                if GameUserData::get_chapter().cid.str_contains("M001") {
                    if single_class { job::unit_change_to_random_class(this, true); }
                    this.item_list.put_off_all_item();
                    if random_inventory { adjust_unit_items(this); }
                    grow::adaptive_growths(this, true);
                    this.set_hp(this.get_capability(0, true));
                    return;
                }
                else {
                    this.item_list.put_off_all_item();
                    this.item_list.add_item_no_duplicate(ItemData::get("IID_鉄の剣").unwrap());
                    this.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());
                }
            }
        }
         // Hub & Kizuna: person is already the correct person or MapSequence and Alear is not on the Map (Chapter 11)
        else if ( GameUserData::get_sequence() == 5  ||  GameUserData::get_sequence() == 4 ) || 
            (GameUserData::get_sequence() == 3 && ( GameVariableManager::get_bool("MapRecruit") || ( GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) != 3 && !lueur_on_map() ) ) )
        {
            println!("Hub/Kizuna Recruitment");
            change_unit_autolevel(this, true);
            if this.person.pid.str_contains(PIDS[VEYLE]) {
                this.item_list.put_off_all_item();
                this.item_list.add_iid_no_duplicate("IID_オヴスキュリテ");
                this.item_list.add_iid_no_duplicate("IID_ミセリコルデ");
            }
        }
        else if switch_person(this.person).pid ==  this.person.pid {
            if random_class || single_class  {
                job::unit_change_to_random_class(this, true);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
                if GameVariableManager::get_number(DVCVariables::PLAYER_INVENTORY) & 1 != 0 { unit_items::adjust_missing_weapons(this); }
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
        if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 {
            ai::adjust_unitai(this);
        }
    }

    unit_items::remove_duplicates(this.item_list);
    set_unit_edit_name(this);
    this.auto_equip();
    grow::adaptive_growths(this, true);
    auto_level_unit_for_random_map(this, false);
    this.set_hp(this.get_capability(0, true));
    println!("Finish creating {}", Mess::get_name(this.person.pid));
}

fn unit_set_drop_seals(this: &mut Unit) {
    for x in 0..8 {
        if let Some(item) = this.item_list.get_item(x) {
            if item.item.iid.str_contains("プルフ") { item.set_flags(2); }
        }
    }
}
pub fn fixed_unit_weapon_mask(this: &mut Unit){
    this.original_aptitude.value = this.person.get_aptitude().value;
    this.aptitude.value = this.original_aptitude.value | this.person.get_sub_aptitude().value;
    this.selected_weapon_mask.value = 0;
    this.update_weapon_mask();
    this.set_select_weapon_from_original_aptitude(this.original_aptitude);
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
    if CONFIG.lock().unwrap().debug {
        println!("Adjusting Items for {} Job: #{} {}", Mess::get_name(unit.person.pid), job.parent.index, Mess::get_name(job.jid));
        unit.item_list.unit_items.iter().flat_map(|x| x)
            .for_each(|item| { println!("Adjusting item for #{}, {}", item.item.parent.index, Mess::get_name(item.item.iid)); });
    }
    unit.update_weapon_mask();
    let mut has_drops = unit_items::has_drops(unit);
    let custom_inventory = GameVariableManager::get_number(DVCVariables::PLAYER_INVENTORY);
    let ran_map = GameVariableManager::get_number(DVCVariables::CONTINUOUS) == 3;
    if custom_inventory & 1 != 0 && !is_enemy {
        println!("Custom Inventory");
        unit.item_list.put_off_all_item();
        unit_items::add_generic_weapons(unit);
        unit_items::assign_tomes(unit);
    }
    else if custom_inventory & 2 != 0 && is_enemy && ((DVCVariables::is_main_chapter_complete(11) && ran_map) ||
        DVCVariables::is_main_chapter_complete(9))
    {
        unit.item_list.put_off_all_item();
        unit_items::add_generic_weapons(unit);
        unit_items::assign_tomes(unit);
    }
    else {
        println!("Simple Replacement");
        has_drops = 0;
        unit_items::simple_replacement(unit);
    }
    unit_items::assign_unique_items(unit);
    unit_items::assign_staffs(unit);
    unit_items::add_equip_condition(unit);
    unit_items::adjust_melee_weapons(unit);
    unit_items::adjust_missing_weapons(unit);
    unit_items::remove_duplicates(unit.item_list);
    unit.auto_equip();
    if has_drops != 0 {
        let iid = random_item(5, has_drops == 2 ).to_string();
        unit.item_list.add_iid_no_duplicate(&iid);
        if let Some(x) = unit.item_list.unit_items.
            iter_mut().flatten().find(|x| x.item.iid.to_string() == iid)
        {
            x.flags |= 2;
        }
    }

    if unit.person.get_asset_force() == 0 {
        if unit.get_capability(0, true) >= 45 {
            unit.item_list.add_iid_no_duplicate("IID_特効薬")
        }
        else {
            unit.item_list.add_iid_no_duplicate("IID_傷薬")
        }
    }
    unit.item_list.unit_items.iter().flat_map(|x| x)
        .for_each(|item| {
            println!("Final Items #{}, {}", item.item.parent.index, Mess::get_name(item.item.iid));
        });
}

pub fn set_unit_edit_name(unit: &Unit) {
    if unit.person.pid.str_contains(PIDS[0]) || unit.person.get_flag().value & 1024 != 0 {
        if GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) != 0 {
            unit.edit.set_gender( GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) );
        }
        else {unit.edit.set_gender( 1 );  }
        if GameVariableManager::exist(DVCVariables::LUEUR_NAME) { unit.edit.set_name( GameVariableManager::get_string(DVCVariables::LUEUR_NAME) ); }
        unit.person.set_gender( unit.edit.gender );
    }
    if unit.person.get_flag().value & 128 != 0 && unit.person.get_job().is_some_and(|j| j.jid.to_string().contains("JID_邪竜ノ子")){
        if let Some(pid) = GameVariableManager::try_get_string("G_R_PID_ヴェイル").
            and_then(|pid| PersonData::get(pid))
            .filter(|p| p.parent.index > 1)
        {
            unit.edit.set_name(pid.name.unwrap());
            if pid.gender == 1 {
                if pid.get_flag().value & 32 != 0 { unit.edit.set_gender( 2 ); }
                else { unit.edit.set_gender( 1 ); }
            }
            else {
                if pid.get_flag().value & 32 != 0 { unit.edit.set_gender( 1 ); }
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
    let person = if reverse { super::switch_person_reverse(unit.person) } else { unit.get_person() };
    if DVCVariables::is_random_map() && person.parent.hash == -266109647 {
        unit.extra_hp_stock_count = 1;
        unit.extra_hp_stock_count_max = 1;
    }
    let new_person = if reverse { &unit.person } else { super::switch_person(unit.person) }; 
    if new_person.parent.hash == person.parent.hash { return; }
    println!("{} -> {}",  Mess::get_name(person.pid), Mess::get_name(new_person.pid));
    let is_low = person.get_job().unwrap().is_low();
    let is_new_low = new_person.get_job().unwrap().is_low();
    let current_level = person.get_level() as i32;
    let mut current_internal_level = person.get_internal_level() as i32;
    if current_internal_level == 0 && !is_low { current_internal_level = 20; }
    let mut original_growth_rates: [u8; 11] = [0; 11];  // storing growth rates of the original person
    let original_gr = person.get_grow();    // growth rate of the original person
    let new_gr = new_person.get_grow(); // growth rate of the new person
        // Switch Growths rates to calculate stats, store the previous person's growths to restore it at the end
    for x in 0..11 { 
        original_growth_rates[x as usize] = original_gr[x as usize];
        original_gr[x as usize] = new_gr[x as usize];  
    }
    unit.set_person(person);
    unit.class_change(person.get_job().unwrap());
    if is_low {
       if current_level > 20 { //Old Unit is in a special class so new unit needs to be promoted
            if is_new_low {
                if new_person.get_job().unwrap().has_high() {    // new unpromoted unit can promoted 
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
            if new_person.get_job().unwrap().has_high() {   // base class -> 1st promoted class
                let new_high_jobs = new_person.get_job().unwrap().get_high_jobs();
                if new_high_jobs.len() == 0 { unit.class_change(JobData::get("JID_ソードマスター").unwrap());  } // if no high class, change to Swordmaster
                else { unit.class_change(&new_high_jobs[0]); }
                unit.auto_grow_capability(current_level, current_level + 20);
                unit.set_level(current_level);
                unit.set_internal_level(current_internal_level);
                println!("Promoted Unit -> Base Unit");
            }
            else { // Promoted -> Special
                if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 == 0  { 
                    unit.class_change(new_person.get_job().unwrap());
                    println!("Promoted Unit -> Special Unit");
                }
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
    for x in 0..11 { original_gr[x as usize] = original_growth_rates[x as usize]; }
    let bases = calculate_new_offset(person, new_person);
     // Change back to original growth rate
    for x in 0..11 {  unit.set_base_capability(x as i32, bases[x] as i32);  }

    unit.set_sp( person.get_sp() );
    unit.set_person(new_person);    // change person
    fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes  // Random map order level adjustment
}

fn calculate_new_offset(original: &PersonData, new: &PersonData) -> [i8; 11] {
    let original_job = original.get_job().expect("Original Person does not have a valid default class in Person.xml");
    let new_job = new.get_job().expect("Replacement Person does not have a valid default class in Person.xml");
    let mut out: [i8; 11] = [0; 11];
    let old_level = if original_job.is_high() { 20  + original.get_level() as i32 }  else { original.get_level() as i32 };
    let new_level = if new_job.is_high() { 20 + new.get_level() as i32 } else { new.get_level() as i32 };

    let factor = new_level - old_level;
    let new_offset = new.get_offset_n();
    let old_offset = original.get_offset_n();
    let new_grow = new.get_grow();
    let diff_grow = new_job.get_diff_grow();
    let old_grow = original.get_grow();
    let class_base = original_job.get_base();
    let n_autolevels =
        if DVCVariables::is_random_map() && DVCVariables::is_main_chapter_complete(4) {
            max(crate::continuous::random::random_map_mode_level(), engage::util::get_instance::<engage::map::situation::MapSituation>().average_level) +
                crate::continuous::get_continious_total_map_complete_count() / 3
        }
        else { ( if original_job.is_high() { 20 } else { 0 } + original.get_level() ) as i32 };

    if original.pid.contains(PIDS[1]) {
        let new_class_bases = new_job.get_base();
        for x in 0..11 {
            let offset = 100*old_offset[x] as i32 + n_autolevels * ( old_grow[x] as i32) + 100*class_base[x] as i32 + 50;
            let round = if diff_grow[x] as i32 + new_grow[x] as i32 > 0 { 50 } else { 0 };
            let value: i32 = 100 * new_offset[x] as i32 - factor * diff_grow[x] as i32 + round +  ( n_autolevels  * new_grow[x] as i32) + 100*new_class_bases[x] as i32;
            if value >= (offset + 100) {
                out[x] = ((offset- 100*(new_class_bases[x] as i32) + 100) / 100) as i8;
            }
            else if value < (offset - 300) {
                out[x] = ((offset - 100*(new_class_bases[x] as i32) - 300) / 100) as i8;
            }
            else {
                out[x] = ((value - 100*new_class_bases[x] as i32) / 100) as i8;
            }
        }
    }
    else {
    // Everyone calculate offset by subtracting class growths to adjust to original unit's level
        for x in 0..10 {
            let original_stat = old_offset[x] as i32;
            let round = if diff_grow[x] as i32 + new_grow[x] as i32 > 0 { 50 } else { 0 };
            let value: i32 = 100 * new_offset[x] as i32 - factor * diff_grow[x] as i32;
            let base: i32 =
            // Clamp difference between -4 to +4 of the original unit
            if value > ( 100* (original_stat + 4 )) { 100*( 4 + old_offset[x] as i32 ) }
            else if value < 100 * ( original_stat - 5 ) { ( 50*value + 5000 * original_stat ) / 100  }
            else { value } + ( n_autolevels  * new_grow[x] as i32) + round;
            out[x] = (base / 100 ) as i8;
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
        let is_boss = data.get_flag().value & 16 != 0 ;
        let mut changed_class = false;
        let random_map = DVCVariables::is_random_map();
        let difficulty = GameUserData::get_difficulty(false);
        let m004_complete = DVCVariables::is_main_chapter_complete(4);
        if let Some(person) = data.get_person() {
            if person.get_engage_sid().is_none() {
                 if ( DVCVariables::random_enabled() && person.get_asset_force() != 0 ) &&
                     (
                         (is_boss && GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY) == 10 ) ||
                         (GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY) > 10)
                     )
                  {
                    if let Some(dispos_skill) = data.sid {
                        unit.private_skill
                            .replace_sid(dispos_skill, skill::get_random_skill_dispos(difficulty, rng));
                    }
                    else if rng.get_value(20) < 2* difficulty && m004_complete {
                        unit.private_skill
                            .add_skill(skill::get_random_skill_dispos(difficulty, rng), 10, 0);
                    }
                }
            }
        }
        if unit.person.parent.hash == 1879825845 || unit.status.value & 134217728 != 0 { return; }
        let job = unit.get_job();
        if MONSTERS.iter().any(|str| job.jid.contains(str)) {
            if random_map && m004_complete { auto_level_unit_for_random_map(unit, is_boss);  }
            else { emblem_paralogue_level_adjustment(unit); }
            return;  
        }
        let mut has_master = unit.item_list.has_item_iid("IID_マスタープルフ");
        set_unit_edit_name(unit);
        if unit.person.get_flag().value & 512 == 512 && GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {  // Person was change 
            fixed_unit_weapon_mask(unit);
            // println!("Adjust Weapon 1");
            adjust_unit_items(unit); 
            ai::adjust_unitai(unit);
        }
        if (unit.person.get_asset_force() | 2 == 2) && ( GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 ){  ai::adjust_unitai(unit);  }
        if unit.person.get_asset_force() != 0 {
            if random_map && m004_complete && !GameUserData::get_chapter().cid.contains("E00") { // Continuous Mode Random Map
                fixed_unit_weapon_mask(unit);
                let maps_completed = crate::continuous::get_continious_total_map_complete_count();
                if maps_completed < 16 {
                    unit.item_list.put_off_all_item();
                    //println!("Adjust Weapon RNG");
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
                // println!("Mauvier Passed?");
            }
            if ( GameVariableManager::get_number(DVCVariables::JOB_KEY) & 2 != 0 )  && m004_complete {
                let gauge = GameVariableManager::get_number(DVCVariables::ENEMY_JOB_GAUGE_KEY);
                if unit.person.get_bmap_size()  == 1 && ( rng.get_value(100) < gauge && gauge > 11 )  || ( gauge > 0 && gauge <= 11  && is_boss ) {
                    if job::enemy_unit_change_to_random_class(unit){ 
                        changed_class = true;
                        fixed_unit_weapon_mask(unit);
                       //  println!("Adjust Weapon 2");
                        adjust_unit_items(unit); 
                        if unit.person.get_asset_force() == 2 { unit_items::add_generic_weapons(unit);  }
                        ai::adjust_unitai(unit);
                        if !unit.get_job().get_diff_grow_l().is_zero() {
                            let level = if unit.get_job().get_max_level() == 40 { unit.level as i32 + unit.internal_level as i32 } else {  unit.level as i32 };
                            let target_level = if unit.get_job().is_high() { level + 20 } else { level };
                            unit.auto_grow_capability(level, target_level);
                            if unit.get_job().get_max_level() == 40 { unit.internal_level = 0; }
                        }
                    }
                    else {
                        // println!("Random Class: None");
                    }
                }
                else {
                   // println!("Random Class: None");
                }
            }
            if GameUserData::get_chapter().cid.str_contains("CID_S0") && GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                emblem_paralogue_level_adjustment(unit);
            }
            if GameVariableManager::get_number(DVCVariables::ITEM_KEY) & 2 != 0 { unit_items::random_items_drops(unit); }
            if !m004_complete {
                unit.auto_equip();
                return;
            }
            let gauge = GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY);
            if ( ( rng.get_value(100) < gauge && gauge > 11 ) || ( gauge > 0 && gauge <= 11 && is_boss ) )  && unit.person.get_engage_sid().is_none() {
                if let Some(skill) = skill::get_random_skill_job(GameUserData::get_difficulty(false), rng, unit){
                    unit.private_skill.add_skill(skill, 10, 0); 
                }
            }
            println!("Revival Stone/ Emblem Check");
            if ( rng.get_value(100) < GameVariableManager::get_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY) ) && DVCVariables::is_main_chapter_complete(5) {
               unit.hp_stock_count += 1;
               unit.hp_stock_count_max += 1;
            }
            else if unit.person.get_asset_force() == 1 && rng.get_value(100) < GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) &&
                (  unit.person.get_engage_sid().is_none() && unit.get_god_unit().is_none())
            {
                let current_chapter = GameUserData::get_chapter().cid.to_string();
                if current_chapter != "CID_M022" && current_chapter != "CID_M011"  {
                    let emblem = rng.get_value( ENEMY_EMBLEM_LIST.get().unwrap().len() as i32) as usize;
                    println!("Emblem: {}", emblem);
                    if enemy::try_equip_emblem(unit, emblem) {
                        ai::adjust_enemy_emblem_unit_ai_flags(unit);
                    }
                }
            } 
            else if unit.person.get_engage_sid().is_some() || unit.get_engage_attack().is_some()  { ai::adjust_ai_for_engage_attack(unit);  }
            auto_level_unit(unit, is_boss);
        }
        unit_items::adjust_enemy_meteor(unit);
        if has_master {  unit.item_list.add_iid_no_duplicate("IID_マスタープルフ"); }    // Add Seal if lost seal
        unit_set_drop_seals(unit);    // Drop Seals
    
       // Prevent Green Emblems from dying in Chapter 22 if AI is changed
        if unit.person.pid.str_contains("PID_M022_紋章士") {
            unit.private_skill.add_sid("SID_死亡回避", 10, 0);  
            unit.put_off_all_item();
        } 
        else if changed_class {
            unit_items::adjust_missing_weapons(unit);
            ai::adjust_unitai(unit);
        }
        if GameUserData::is_evil_map() { auto_level_unit(unit, is_boss); }
        unit.auto_equip();
        unit.set_hp(unit.get_capability(0, true));
    }
}