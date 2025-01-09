use super::{*, ai};
use crate::randomizer::assets::animation::MONSTERS;
use crate::randomizer::emblem;
use crate::randomizer::{grow, job, item::unit_items, assets, skill};
use crate::utils;
#[unity::hook("App", "Unit", "CreateImpl2")]
pub fn unit_create_impl_2_hook(this: &mut Unit, method_info: OptionalMethod){
    let can_lueur_change = unsafe { LUEUR_CHANGE };
    call_original!(this, method_info);
    println!("Create Impl 2 on {} - #{}", this.person.get_name().unwrap().to_string(),  this.person.parent.index);
    if !can_lueur_change {
        if this.person.pid.to_string() == "PID_リュール" {  unsafe { LUEUR_CHANGE = true; } }
        return;
    }
    if !can_rand() { return; }
    if !is_player_unit(this.person) {
        if is_playable_person(this.person) {
            if GameVariableManager::get_number("G_Random_Job") & 1 != 0 {
                job::unit_change_to_random_class(this);
                println!("CreateFromDispos Adjust Unit Items for {}", Mess::get(this.person.get_name().unwrap()).to_string());
            }
            if GameVariableManager::get_number("G_Random_Recruitment") != 0 ||  ( GameVariableManager::get_number("G_Random_Job") & 1 != 0 ) {  adjust_unit_items(this);  }
            grow::adaptive_growths(this);
        }
        if this.person.get_asset_force() != 0 { assets::accessory::accesorize_enemy_unit(this); }
        unsafe { unit_update_auto_equip(this, None); }
        return;
    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        if GameVariableManager::get_string("G_R_PID_リュール") == this.person.pid && GameUserData::get_sequence() == 0 {
            change_unit_autolevel(this, true);
            this.item_list.put_off_all_item();
            this.item_list.add_item_no_duplicate(ItemData::get("IID_鉄の剣").unwrap()); 
            this.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());
        }
        else if GameVariableManager::get_bool("DDFanClub") {
            let old_person = switch_person_reverse(this.person).pid.to_string();
            change_unit_autolevel(this, true);
        // If Chapter 1 and Vander -> Switch
            if GameUserData::get_chapter().cid.contains("M001") && ( old_person == "PID_クラン" || old_person == "PID_フラン" ) {
                this.item_list.put_off_all_item();
            }
            else {
                this.item_list.put_off_all_item();
                this.item_list.add_item_no_duplicate(ItemData::get("IID_鉄の剣").unwrap()); 
                this.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());
            }
        }
         // Hub & Kizuna: person is already the correct person or MapSequence and Alear is not on the Map (Chapter 11)
        else if ( GameUserData::get_sequence() == 5  ||  GameUserData::get_sequence() == 4 ) || 
            (GameUserData::get_sequence() == 3 && ( GameVariableManager::get_bool("MapRecruit") || ( GameVariableManager::get_number("G_DeploymentMode") != 3 && !crate::utils::lueur_on_map() ) ) )  { 
            println!("Hub/Kizuna Recruitment");
            change_unit_autolevel(this, true);
            if this.person.pid.contains("PID_ヴェイル") {
                this.item_list.put_off_all_item();
                this.item_list.add_iid_no_duplicate("IID_オヴスキュリテ");
                this.item_list.add_iid_no_duplicate("IID_ミセリコルデ");
            }
        }
        // if randomized to the same person
        else if switch_person(this.person).pid ==  this.person.pid {
            if GameVariableManager::get_number("G_Random_Job") & 1 != 0  {
                job::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
            }
            grow::adaptive_growths(this);
            random_map_unit_level(this);
            this.set_hp(this.get_capability(0, true));
            return;
        }
        else { change_unit_autolevel(this, false);  }
        if this.person.pid.to_string() == "PID_リュール" && GameUserData::get_sequence() != 0{
            this.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") );
            if GameVariableManager::exist("G_Lueur_Name") { this.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") ); }
        }
    }
    if GameVariableManager::get_number("G_Random_Job") & 1 != 0   { 
        job::unit_change_to_random_class(this); 
    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 || GameVariableManager::get_number("G_Random_Job") & 1 != 0 {  adjust_unit_items(this);  }
    unit_items::remove_duplicates(this.item_list);
    println!("Finish with Create2Impl for {}", this.person.get_name().unwrap().to_string());
    set_unit_edit_name(this);
    this.auto_equip();
    grow::adaptive_growths(this);
}

#[unity::hook("App", "Unit", "CreateFromDispos")]
pub fn create_from_dispos_hook(this: &mut Unit, data: &mut DisposData, method_info: OptionalMethod) {
    if !utils::can_rand() { call_original!(this, data, method_info); return; }
    if GameVariableManager::get_number("G_Emblem_Mode") != 0 { // Changing Emblems
        if let Some(gid) = data.gid {
            let string = gid.to_string();
            if EMBLEM_GIDS.iter().any(|x| *x == string) {
                data.set_gid(GameVariableManager::get_string(format!("G_R_{}", string).as_str()));
            }
        }
        adjust_emblem_paralogue_items(data);
    }
    ai::adjust_dispos_person_ai(data);    // Changing Person AI
    let rng = Random::get_game();
    if let Some(person) = data.get_person() {   // adding random skill if person does not have an engage attack
        if person.get_engage_sid().is_none()  {
            if ( GameVariableManager::get_bool("G_Random_Skills") && GameVariableManager::get_number("G_EnemySkillGauge") > 10 ) && ( utils::can_rand() && person.get_asset_force() != 0 ) {
                let diffculty = GameUserData::get_difficulty(false);
                if data.sid.is_some() {
                    let skill = skill::get_random_skill( diffculty, rng);
                    data.sid = Some(skill.sid); 
                }
                else if rng.get_value(20) < 2*diffculty && GameVariableManager::get_bool("G_Cleared_M004") {
                    let skill = skill::get_random_skill( diffculty, rng);
                    data.sid = Some(skill.sid);
                }
            }
        }
    }
    call_original!(this, data, method_info);
    let job = this.get_job();
    if super::super::assets::animation::MONSTERS.iter().any(|str| job.jid.contains(str)) { 
        if GameVariableManager::get_number("G_Continuous") == 3 && GameVariableManager::get_bool("G_Cleared_M004") {
            auto_level_unit_for_random_map(this, data.get_flag().value & 16 != 0); 
        }
        return;  
    }
    let mut has_master = this.item_list.has_item_iid("IID_マスタープルフ");
    ai::engage_attack_ai(this, data); 
    set_unit_edit_name(this);
    if this.person.get_flag().value & 512 == 512 {  // Person was change 
        fixed_unit_weapon_mask(this);
        adjust_unit_items(this); 
        ai::adjust_unit_ai(this, data);
    }
    if this.person.get_asset_force() == 0 && ( GameVariableManager::get_number("G_Random_Job") & 1 != 0 ){
        ai::adjust_unit_ai(this, data); // change green unit recruitable ai on random classes
    }
    if this.person.get_asset_force() != 0 {
        if GameVariableManager::get_number("G_Continuous") == 3 && GameVariableManager::get_bool("G_Cleared_M004") { // Continuous Mode Random Map
            auto_level_unit_for_random_map(this, data.get_flag().value & 16 != 0); 
            fixed_unit_weapon_mask(this);
            let maps_completed = crate::continuous::get_number_main_chapters_completed2();
            if maps_completed < 16 {
                this.item_list.put_off_all_item();
                unit_items::adjust_staffs(this);
                unit_items::add_generic_weapons(this);
                unit_items::adjust_melee_weapons(this);
                unit_items::remove_duplicates(this.item_list);
                this.auto_equip();
            }
            else { adjust_unit_items(this);  }
            if data.flag.value & 16 != 0 {  // If leader then add seal
                match maps_completed {  
                    7|10|11 => { 
                        has_master = false;
                        this.item_list.add_iid_no_duplicate("IID_マスタープルフ");
                    }
                    8|12|15 => { 
                        has_master = false;
                        this.item_list.add_iid_no_duplicate("IID_チェンジプルフ"); 
                    }
                    _ => {},
                }
            }
        }
        else if GameVariableManager::get_bool("G_DVC_Autolevel") { auto_level_unit(this); }
        if ( GameVariableManager::get_number("G_Random_Job") & 2 != 0 )  && GameVariableManager::get_bool("G_Cleared_M004"){
            if ( this.person.get_bmap_size()  == 1 && rng.get_value(100) < GameVariableManager::get_number("G_EnemyJobGauge") ) {
                if job::enemy_unit_change_to_random_class(this){ 
                    fixed_unit_weapon_mask(this);
                    adjust_unit_items(this); 
                    if this.person.get_asset_force() == 2 { unit_items::add_generic_weapons(this);  }
                    ai::adjust_unit_ai(this, data);
                    if !this.get_job().get_diff_grow_l().is_zero() {
                        let level = if this.get_job().get_max_level() == 40 { this.level as i32 + this.internal_level as i32 } else {  this.level as i32 };
                        let target_level = if this.get_job().is_high() { level + 20 } else { level };
                        this.auto_grow_capability(level, target_level);
                        if this.get_job().get_max_level() == 40 { this.internal_level = 0; }
                   }
               }
           }
       }
       if GameUserData::get_chapter().cid.contains("CID_S0") && GameVariableManager::get_number("G_Emblem_Mode") != 0 { emblem_paralogue_level_adjustment(this); } 
       if GameVariableManager::get_number("G_Random_Item") >= 2 { unit_items::random_items_drops(this); }
       if !GameVariableManager::get_bool("G_Cleared_M004") { 
            this.auto_equip();
            return; 
        }
        if GameVariableManager::get_bool("G_Random_Skills") && rng.get_value(100) < GameVariableManager::get_number("G_EnemySkillGauge") && this.person.get_engage_sid().is_none() {
           let mut valid_skill = false;
           let mut count = 0;
           while !valid_skill && count < 5 {
               let skill = skill::get_random_skill( GameUserData::get_difficulty(false), rng);
               if !has_skill(this, skill) {
                   valid_skill = this.private_skill.add_skill(skill, 10, 0); 
                   count += 1;
               }
           }
       }
       if ( rng.get_value(100) < GameVariableManager::get_number("G_EnemyRevivalStone") ) && GameVariableManager::get_bool("G_Cleared_M005") {
           this.hp_stock_count += 1;
           this.hp_stock_count_max += 1;
       }
       if rng.get_value(200) < GameVariableManager::get_number("G_EnemyEmblemGauge") && this.get_god_unit().is_none() {
           let current_chapter = GameUserData::get_chapter().cid.to_string();
           if current_chapter != "CID_M022" && current_chapter != "CID_M011"  {
               let emblem = rng.get_value(EMBLEMS.len() as i32) as usize;
               if try_equip_emblem(this, emblem) { 
                    ai::engage_attack_ai(this, data); 
                    ai::adjust_unit_ai(this, data);
               }
           }
       } 
   }
    unit_items::adjust_enemy_meteor(this);
    if has_master {  this.item_list.add_iid_no_duplicate("IID_マスタープルフ"); }    // Add Seal if lost seal
    unit_set_drop_seals(this);    // Drop Seals

   // Prevent Green Emblems from dying in Chapter 22 if AI is changed
    if this.person.pid.contains("PID_M022_紋章士") {
        this.private_skill.add_sid("SID_死亡回避", 10, 0);  
        this.put_off_all_item();
    } 
    else { unit_items::adjust_missing_weapons(this); }
   this.auto_equip();
}

fn unit_set_drop_seals(this: &mut Unit) {
    for x in 0..8 {
        if let Some(item) = this.item_list.get_item(x) {
            if item.item.iid.contains("プルフ"){ item.set_flags(2); }
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
    let weapon_mask = job.get_weapon_mask_with_selected(unit.weapon_mask, unit.selected_weapon_mask).value;
    let jid = job.jid.to_string();
    if MONSTERS.iter().any(|&x| jid == x) {
        unit_items::add_monster_weapons(unit);
        return;
    }
    let list_count = unit.item_list.get_count();
    let name =  unit.person.get_name().unwrap().to_string();
    if list_count == 0 { return; }
    let mut slot = 0;
    let mut weapon_mask_array: [i32; 4] = [0; 4];
    let mut weapon_level: [i32; 4] = [0; 4];
    let enemy = unit.person.get_asset_force() != 0;
    for x in 1..9 {
        if x == 7 { continue; }
        if weapon_mask & (1 << x) != 0 {
            weapon_mask_array[slot as usize] =  weapon_mask & (1 << x);
            weapon_level[slot as usize] = job.get_max_weapon_level(x);
            // println!("Job has weapon type: {}, max level: {}", x, weapon_level[slot as usize]);
            slot += 1;
        }
        if slot == 4 { break; }
    }
    let n_weapons = slot;
    slot = 0;
    let jid = unit.get_job().jid.to_string();
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_some() {
            let weapon = &item.unwrap();
            let kind = weapon.item.get_kind(); 
            if kind > 8 || kind == 0 || kind == 7 { continue; }
            if weapon.item.get_flag().value & 128 != 0 || ( weapon.item.get_flag().value & 2 != 0 && !enemy) { continue;  }
            //let rank = weapon.item.get_weapon_level();
            //println!("{}: Weapon Mask {} & {} (kind = {}, rank {} ) = {} for {} ", name, weapon_mask, 1 << kind, kind, rank, weapon_mask & ( 1 <<  kind ), weapon.item.name.to_string());
            if weapon_mask & ( 1 <<  kind ) == 0 {
                // For Veyle
                if name == "MPID_Veyre" && weapon_mask_array[slot] == 64 {
                    if slot == 0 { weapon.ctor(ItemData::get("IID_オヴスキュリテ").unwrap());  }
                    else if slot == 1 { weapon.ctor(ItemData::get("IID_ミセリコルデ").unwrap());  }
                }
                else if jid == "JID_マージカノン" { // mage cannon
                    if slot == 0 { 
                        weapon.ctor_str("IID_弾_物理"); 
                        slot +=1;
                    }
                    if slot == 1 {
                        weapon.ctor_str("IID_弾_魔法");
                        slot += 1;
                    }
                }
                else if jid == "JID_異形狼" || jid == "JID_幻影狼" {
                    if slot == 0 {
                        weapon.ctor_str("IID_牙");
                        slot +=1;
                    }
                    else if slot == 1 {
                        weapon.ctor_str("IID_HPの薬");
                        slot +=1;
                    }
                }
                else {
                   // println!("{} has Weapon {} to be replaced", name, weapon.item.name.to_string());
                    if slot < n_weapons {
                        unit_items::replace_weapon(weapon, weapon_mask_array[slot as usize], weapon_level[slot as usize], enemy);
                        if n_weapons > 1 { slot += 1; }
                    }
                    else if slot < 4 && slot >= 1 {
                        unit_items::replace_weapon(weapon, weapon_mask_array[slot - 1 as usize], weapon_level[slot - 1 as usize], enemy);
                    }
                }
            }
        }
    }
    unit_items::adjust_staffs(unit);
    unit_items::adjust_melee_weapons(unit);
    unit_items::remove_duplicates(unit.item_list);
    unit.auto_equip();
   // println!("Finished adjusting items");
}

pub fn set_unit_edit_name(unit: &Unit) {
    if unit.person.pid.to_string() == "PID_リュール" || unit.person.get_flag().value & 1024 != 0 {
        if GameVariableManager::get_number("G_Lueur_Gender2") != 0 { unit.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") ); }
        else {unit.edit.set_gender( 1 );  }
        if GameVariableManager::exist("G_Lueur_Name") { unit.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") ); }
        unit.person.set_gender( unit.edit.gender );
    }
    if unit.person.get_flag().value & 128 != 0 {
        println!("Person #{} Name is set to be {}", unit.person.parent.index, GameVariableManager::get_string("G_Lueur_Name").to_string());
        unit.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") );
        if GameVariableManager::get_number("G_Lueur_Gender2") != 0 { unit.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") ); }
        else {unit.edit.set_gender( 1 );  }
    } 
}

pub fn emblem_paralogue_level_adjustment(this: &Unit){
    if !crate::utils::can_rand() { return; }
    let level_difference = GameVariableManager::get_number("G_Paralogue_Level");
    if level_difference == 0 { return; }
    let total_level = this.level as i32 + this.internal_level as i32;
    let new_level = total_level + level_difference;
    this.auto_grow_capability(new_level, new_level);
    if this.job.is_high() {
        if new_level < 20 {
            this.set_level(1);
            this.set_internal_level(20);
        }
        else if new_level > 40 {
            this.set_level(20);
            this.set_internal_level(new_level - 20);
        }
        else {
            this.set_level(20 - new_level);
            this.set_internal_level(20);
        }
    }
    else {
        if this.job.max_level == 40 {
            if new_level > 40 {
                this.set_level(40);
                this.set_internal_level(new_level - 40);
            }
            else {
                this.set_level(new_level);
                this.set_internal_level(0);
            }
        }
        else {
            this.set_level(20);
            this.set_internal_level(new_level - 20);
        }
    }
    this.set_hp(this.get_capability(0, true));
}

pub fn change_unit_autolevel(unit: &mut Unit, reverse: bool) {
    let person = if reverse { super::switch_person_reverse(unit.person) } else { &unit.person };
    let new_person = if reverse { &unit.person } else { super::switch_person(unit.person) }; 
    println!("{} -> {}",  person.get_name().unwrap().to_string(), new_person.get_name().unwrap().to_string());
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
            println!("{} -> {} Base -> Base Level {}",  person.get_name().unwrap().to_string(), new_person.get_name().unwrap().to_string(), current_level);
        }
        else {
            let new_job_list = new_person.get_job().unwrap().get_low_jobs();
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
                if GameVariableManager::get_number("G_Random_Job") & 1 == 0  { 
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
    let bases = calculate_new_offset(person, new_person);
     // Change back to original growth rate
    for x in 0..11 { 
        original_gr[x as usize] = original_growth_rates[x as usize]; 
    }
    for x in 0..11 {  unit.set_base_capability(x as i32, bases[x] as i32);  }

    unit.set_sp( person.get_sp() );
    unit.set_person(new_person);    // change person
    fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes
    random_map_unit_level(unit);    // Random map order level adjustment
    unit.set_hp(unit.get_capability(0, true));
    unsafe {  unit_learn_job_skill(unit, None); }
}


pub fn random_map_unit_level(unit: &mut Unit) {
    if GameVariableManager::get_number("G_Continuous") < 3 || !GameVariableManager::get_bool("G_Cleared_M004") { return; }
    let story = crate::utils::max( (crate::continuous::get_story_chapters_completed()-6)*2, crate::continuous::get_story_chapters_completed() + 4); 
    let level =  crate::utils::max( unsafe{ calculate_average_level(get_sortie_unit_count(None)) }, story );
    let average_level =  unsafe{ calculate_average_level(crate::utils::max(8, get_sortie_unit_count(None) )) };
    let total_level = crate::utils::max(level, average_level) - 1;
    unit.set_sp(total_level * 100 + 300 );

    if GameVariableManager::get_number("G_Player_Rating_Average") == 0 { crate::autolevel::calculate_player_cap(); }
    let player_cap = GameVariableManager::get_number("G_Player_Rating_Average") - 10 * GameUserData::get_difficulty(false);
    let mut count = 0;
    while crate::autolevel::unit_cap_total(unit, true) < player_cap && count < 20 {
        unit.set_level(1);
        unit.level_up(3);
        count += 1;
    }
    println!("Random Map Adjusted Rating: {} for {}", crate::autolevel::unit_cap_total(unit, true), Mess::get_name(unit.person.pid));
    let job = unit.get_job();
    let job_max_level = job.max_level as i32;
    if job.is_low() {
        if total_level <= 20 {
            unit.set_level(total_level);
            unit.set_internal_level(0);
        }
        else if job.max_level == 40 {
            if total_level <= 40 {
                unit.set_level(total_level);
                unit.set_internal_level(0);
            }
            else {
                unit.set_level(40);
                unit.set_internal_level(total_level - 40);
            }
        }
        else {
            let high_jobs = job.get_high_jobs();
            if high_jobs.len() > 0 {
                let new_job = &high_jobs[0];
                unit.class_change(new_job);
                unit.set_level(level-job_max_level);
                unit.set_weapon_mask_from_person();
                unit.set_internal_level(job_max_level);
            }
            else {
                unit.set_level(20);
                unit.set_internal_level(total_level-20);
            }
        }
    }
    else {
        if total_level <= 20 {
            unit.set_internal_level(0);
            unit.set_level(total_level);
        }
        else if total_level <= 40 {
            unit.set_internal_level(20);
            unit.set_level(total_level - 20);
        }
        else {
            unit.set_internal_level(total_level - 20);
            unit.set_level(20);
        }
    }
}


fn calculate_new_offset(original: &PersonData, new: &PersonData) -> [i8; 11] {
    let original_job = original.get_job().expect("Original Person does not have a valid default class in Person.xml");
    let new_job = new.get_job().expect("Replacement Person does not have a valid default class in Person.xml");
    let mut out: [i8; 11] = [0; 11];
    let old_level = if original_job.is_high() { 20  + original.get_level() as i32 }  else { original.get_level() as i32 };
    let new_level = if new_job.is_high() { 20 + new.get_level() as i32 } else { new.get_level() as i32 };

    let factor = (new_level as i32 - old_level as i32) as i32;
    let new_offset = new.get_offset_n();
    let old_offset = original.get_offset_n();
    let new_grow = new.get_grow();
    // Use Vander's offset if vander
    if original.pid.to_string() == "PID_ヴァンドレ" {
        for x in 0..11 {
            let value: i32 = 100 * old_offset[x as usize ] as i32 + old_level * new_grow[ x ] as i32 + 50;
            out[x as usize] = (value / 100 ) as i8;
        }
    }
    else {
    // Everyone calculate offset by subtracting class growths to adjust to original unit's level
        let diff_grow = new_job.get_diff_grow();
        let n_autolevels = 
            if GameVariableManager::get_number("G_Continuous") == 3 && GameVariableManager::get_bool("G_Cleared_M004") {
                crate::utils::max( (crate::continuous::get_story_chapters_completed()-6)*2, crate::continuous::get_story_chapters_completed() + 4) + 1
            }  
            else { ( if original_job.is_high() { 20 } else { 0 } + original.get_level() ) as i32 };

        for x in 0..10 {
            let value: i32 = 100 * new_offset[x ] as i32 - factor as i32 * diff_grow[x] as i32 + 50;
            let base: i32 = 
            if value > ( 100* (old_offset[x] + 5 ) as i32 ) as i32 && old_offset[x] != 0 { 100*( old_offset[x] as i32 + 5 ) }
            else if value < 0 && old_offset[x] != 0 { 100 * ( old_offset[x] as i32 ) }
            else if value < 100* ( old_offset[x] ) as i32  {  60*( old_offset[x] as i32 ) + value /2  }
            else { value }  + ( n_autolevels  * new_grow[x] as i32) as i32 + 50;
            out[x] = (base / 100 ) as i8;
        }
    }
    out
}

fn has_skill(this: &Unit, skill: &SkillData) -> bool {
    return this.mask_skill.unwrap().find_sid(skill.sid).is_some() | this.private_skill.find_sid(skill.sid).is_some()| this.equip_skill.find_sid(skill.sid).is_some();
}
pub fn has_sid(this: &Unit, sid: &str) -> bool {
    return this.mask_skill.unwrap().find_sid(sid.into()).is_some() | this.private_skill.find_sid(sid.into()).is_some()| this.equip_skill.find_sid(sid.into()).is_some();
}

fn adjust_emblem_paralogue_items(data: &mut DisposData) {
    let person = data.get_person();
    if person.is_none() { return; }
    let emblem = person.unwrap();
    let job = emblem.get_job().unwrap();
    if let Some(emblem_index) = EMBLEM_ASSET.iter().position(|x| job.jid.contains(x)) {
        if let Some(god) = emblem::get_god_from_index(emblem_index as i32, false) {
            let level_data = god.get_level_data().unwrap();
            let style_items = &level_data[20].style_items[2];
            let mut item_count: usize = 0;

            for x in 0..style_items.len() {
                let item = &style_items[x];
                let new_iid = 
                    if item.iid.contains("アイムール"){ "IID_三級長_アイムール_通常".into() }
                    else if item.iid.contains("アラドヴァル"){  "IID_三級長_アラドヴァル_通常".into()   }
                    else if item.iid.contains("フェイルノート"){  "IID_三級長_フェイルノート_通常".into()  }
                    else if item.get_flag().value & 128 != 0 { format!("{}_通常", item.iid.to_string()) }
                    else { item.iid.to_string() };
                if ItemData::get(new_iid.as_str()).is_some() {
                    data.items[item_count].set_iid(new_iid.clone().into());
                    match item_count {
                        0 => { data.item1.set_iid(new_iid.into()) },
                        1 => { data.item2.set_iid(new_iid.into()) },
                        2 => { data.item3.set_iid(new_iid.into()) },
                        _ => {},
                    }
                    item_count += 1;
                }
                else { println!("Cannot find: {} for emblem {}", Mess::get(item.name), Mess::get_name(emblem.pid)); }
            }
        }
    }
}


#[unity::from_offset("App", "Unit", "SetSelectedWeaponFromOriginalAptitude")]
fn unit_set_select_weapon_from_original_aptitude(this: &Unit, mask: &WeaponMask, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "AddAptitudeFromWeaponMask")]
fn unit_add_apt_from_weapon_mask(this: &Unit, method_info: OptionalMethod);

// done in Unit$$CreateImpl1


#[skyline::from_offset(0x01a3c290)]
fn unit_learn_job_skill(this: &Unit, method_info: OptionalMethod) -> &'static engage::gamedata::skill::SkillData;

pub fn reload_all_actors() {
    engage::unitpool::UnitPool::class().get_static_fields_mut::<crate::randomizer::job::UnitPoolStaticFieldsMut>().s_unit
    .iter_mut().filter(|unit| unit.force.filter(|f| f.force_type < 3  ).is_some()).for_each(|unit|{
        unit.reload_actor();
        unit.auto_equip();
    });
}
