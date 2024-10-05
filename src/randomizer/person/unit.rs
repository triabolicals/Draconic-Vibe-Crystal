use super::{*, ai};
use crate::randomizer::{job, item::unit_items, assets, skill};
use crate::utils;

#[unity::hook("App", "Unit", "CreateImpl2")]
pub fn unit_create_impl_2_hook(this: &mut Unit, method_info: OptionalMethod){
    let can_lueur_change = unsafe { LUEUR_CHANGE };
    call_original!(this, method_info);
    println!("Create Impl 2 on {} - #{}", this.person.get_name().unwrap().get_string().unwrap(),  this.person.parent.index);
    if !can_lueur_change {
        if this.person.pid.get_string().unwrap() == "PID_リュール" {  unsafe { LUEUR_CHANGE = true; } }
        return;
    }
    if !can_rand() { return; }
    if !is_player_unit(this.person) {
        if this.person.get_sp() > 0 {
            if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
                job::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                println!("CreateFromDispos Adjust Unit Items for {}", Mess::get(this.person.get_name().unwrap()).get_string().unwrap());
            }
            if GameVariableManager::get_number("G_Random_Recruitment") != 0 ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {  adjust_unit_items(this);  }
        }
        if this.person.get_asset_force() != 0 {
            assets::accessory::accesorize_enemy_unit(this);
        }
        return;
    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        if GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == this.person.pid.get_string().unwrap() && GameUserData::get_sequence() == 0 {
            change_unit_autolevel(this, true);
            this.item_list.put_off_all_item();
            this.item_list.add_item_no_duplicate(ItemData::get("IID_鉄の剣").unwrap()); 
            this.item_list.add_item_no_duplicate(ItemData::get("IID_傷薬").unwrap());
        }
        else if GameVariableManager::get_bool("DDFanClub") {
            let old_person = switch_person_reverse(this.person).pid.get_string().unwrap();
            change_unit_autolevel(this, true);

            // If Chapter 1 and Vander -> Switch
            if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M001" && ( old_person == "PID_クラン" || old_person == "PID_フラン" ) {
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
            if this.person.pid.get_string().unwrap() == "PID_ヴェイル" {
                this.item_list.put_off_all_item();
                this.item_list.add_item_no_duplicate(ItemData::get("IID_オヴスキュリテ").unwrap()); 
                this.item_list.add_item_no_duplicate(ItemData::get("IID_ミセリコルデ").unwrap());
            }
        }
        // if randomized to the same person
        else if switch_person(this.person).pid.get_string().unwrap() ==  this.person.pid.get_string().unwrap() {
            println!("Same Person");
            if GameVariableManager::get_number("G_Random_Job") == 1 || GameVariableManager::get_number("G_Random_Job") == 3  {
                job::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
            }
            return;
        }
        else { change_unit_autolevel(this, false);  }
        if this.person.pid.get_string().unwrap() == "PID_リュール" && GameUserData::get_sequence() != 0{
            this.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") );
            if GameVariableManager::exist("G_Lueur_Name") { this.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") ); }
        }
    }
    if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
        job::unit_change_to_random_class(this);
        fixed_unit_weapon_mask(this);

    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 ||  ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {  adjust_unit_items(this);  }
    unit_items::remove_duplicates(this.item_list);
    println!("Finish with Create2Impl for {}", this.person.get_name().unwrap().get_string().unwrap());
    set_unit_edit_name(this);
}

#[unity::hook("App", "Unit", "CreateFromDispos")]
pub fn create_from_dispos_hook(this: &mut Unit, data: &mut DisposData, method_info: OptionalMethod) {
// Changing Emblems
   if data.gid.is_some() && GameVariableManager::get_number("G_Emblem_Mode") != 0 {
       let string = data.gid.unwrap().get_string().unwrap();
       if EMBLEM_GIDS.iter().position(|x| *x == string).is_some() {
           let new_string = format!("G_R_{}", string);
           let new_gid = GameVariableManager::get_string(&new_string);
           data.set_gid(new_gid);
       }
   }
   // Changing Person AI
   ai::adjust_dispos_person_ai(data);
   call_original!(this, data, method_info);
   if !utils::can_rand() { return; }
   let rng = Random::get_game();
   if ( GameVariableManager::get_bool("G_Random_Skills") && GameVariableManager::get_number("G_EnemySkillGauge") > 10 ) && ( utils::can_rand() && this.person.get_asset_force() != 0 ) {
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
    call_original!(this, data, method_info);
   set_unit_edit_name(this);
   if this.person.get_flag().value & 512 == 512 {  // Person was change 
       fixed_unit_weapon_mask(this);
       adjust_unit_items(this); 
       ai::adjust_unit_ai(this, data);
   }
   if this.person.get_asset_force() == 0 && ( GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3 ) {
       ai::adjust_unit_ai(this, data); // change green unit recruitable ai on random classes
   }
   if this.person.get_asset_force() != 0 && !GameUserData::is_evil_map() {
       *CONFIG.lock().unwrap() = crate::DeploymentConfig::new();
       if GameVariableManager::get_bool("G_DVC_Autolevel") { auto_level_unit(this); }
       if GameVariableManager::get_number("G_Random_Job") >= 2 && GameVariableManager::get_bool("G_Cleared_M004"){
           if this.person.get_asset_force() == 2 || ( unsafe { get_bmap_size(this.person, None) } == 1 && rng.get_value(100) < GameVariableManager::get_number("G_EnemyJobGauge") ) {
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
       if str_contains(GameUserData::get_chapter().cid, "CID_S0") && GameVariableManager::get_number("G_Emblem_Mode") != 0 { emblem_paralogue_level_adjustment(this); } 
       if GameVariableManager::get_number("G_Random_Item") >= 2 { unit_items::random_items_drops(this); }
       if !GameVariableManager::get_bool("G_Cleared_M004") { return; }
       if GameVariableManager::get_bool("G_Random_Skills") && rng.get_value(100) < GameVariableManager::get_number("G_EnemySkillGauge") {
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
           let current_chapter = GameUserData::get_chapter().cid.get_string().unwrap();
           if current_chapter != "CID_M022" && current_chapter != "CID_M011"  {
               let emblem = rng.get_value(EMBLEMS.len() as i32) as usize;
               if try_equip_emblem(this, emblem) { 
                ai::adjust_unit_ai(this, data);
               }
           }
       } 
   }
   // Prevent Green Emblems from dying in Chapter 22 if AI is changed
   if str_contains(this.person.pid, "PID_M022_紋章士") { this.private_skill.add_sid("SID_死亡回避", 10, 0);  }  
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

pub fn adjust_unit_items(unit: &Unit) {
    let job = unit.get_job();
    let mut weapon_mask = job.get_weapon_mask().value | unit.selected_weapon_mask.value;
    if weapon_mask == 0 {  weapon_mask = unit.selected_weapon_mask.value; }
    let list_count = unit.item_list.get_count();
    let name =  unit.person.get_name().unwrap().get_string().unwrap();
    if list_count == 0 { return; }
    let mut slot = 0;
    let mut weapon_mask_array: [i32; 4] = [0; 4];
    let mut weapon_level: [i32; 4] = [0; 4];
    for x in 1..9 {
        if x == 7 { continue; }
        if weapon_mask & (1 << x) != 0 {
            weapon_mask_array[slot as usize] =  weapon_mask & (1 << x);
            weapon_level[slot as usize] = job.get_max_weapon_level(x);
            println!("Job has weapon type: {}, max level: {}", x, weapon_level[slot as usize]);
            slot += 1;
        }
        if slot == 4 { break; }
    }
    let n_weapons = slot;
    slot = 0;
    let jid = unit.get_job().jid.get_string().unwrap();
    for x in 0..8 {
        let item = unit.item_list.get_item(x);
        if item.is_some() {
            let weapon = &item.unwrap();
            //if weapon.is_drop() { continue; }
            let kind = weapon.item.get_kind(); 
            if kind > 8 || kind == 0 { continue; }
            if kind == 7 { continue; }
            if weapon.item.get_flag().value & 128 != 0 || weapon.item.get_flag().value & 2 != 0 { continue;  }
            //let rank = weapon.item.get_weapon_level();
            //println!("{}: Weapon Mask {} & {} (kind = {}, rank {} ) = {} for {} ", name, weapon_mask, 1 << kind, kind, rank, weapon_mask & ( 1 <<  kind ), weapon.item.name.get_string().unwrap());
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
                    if slot < n_weapons {
                        unit_items::replace_weapon(weapon, weapon_mask_array[slot as usize], weapon_level[slot as usize]);
                        if n_weapons > 1 { slot += 1; }
                    }
                    else if slot < 4 && slot >= 1 {
                        unit_items::replace_weapon(weapon, weapon_mask_array[slot - 1 as usize], weapon_level[slot - 1 as usize]);
                    }
                }
            }
        }
    }
    unit_items::adjust_staffs(unit);
    unsafe { unit_update_auto_equip(unit, None); }
    unit_items::remove_duplicates(unit.item_list);
}

pub fn set_unit_edit_name(unit: &Unit) {
    if unit.person.pid.get_string().unwrap() == "PID_リュール" || unit.person.get_flag().value & 1024 != 0 {
        if GameVariableManager::get_number("G_Lueur_Gender2") != 0 { unit.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") ); }
        else {unit.edit.set_gender( 1 );  }
        if GameVariableManager::exist("G_Lueur_Name") { unit.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") ); }
        unit.person.set_gender( unit.edit.gender );
    }
    if unit.person.get_flag().value & 128 != 0 {
        println!("Person #{} Name is set to be {}", unit.person.parent.index, GameVariableManager::get_string("G_Lueur_Name").get_string().unwrap());
        unit.edit.set_name( GameVariableManager::get_string("G_Lueur_Name") );
        if GameVariableManager::get_number("G_Lueur_Gender2") != 0 { unit.edit.set_gender( GameVariableManager::get_number("G_Lueur_Gender2") ); }
        else {unit.edit.set_gender( 1 );  }
    } 
}

pub fn emblem_paralogue_level_adjustment(this: &Unit){
    if !crate::utils::can_rand() { return; }
    let level_difference = GameVariableManager::get_number("G_Paralogue_Level");
    if level_difference == 0 { return; }
    if level_difference < 0 {
        let old_level = this.level as i32;
        let level;
        if this.level == 1 { level = 2; }
        else { level = this.level as i32 ;  }
        let mut count = 0;
        let count_max = -1*(level_difference + level_difference / 3 );
        loop {
            this.level_down();
            this.set_level(level);
            count += 1;
            if count == count_max { break; }
        }
        if old_level + level_difference <= 0 { this.set_level(1); }
        else { this.set_level( (old_level + level_difference) as i32)}
    }
    else {
        let mut count = 0;
        loop {
            this.level_up(3);
            count += 1;
            if count == level_difference { break; }
        }
        let job_max_level = this.job.max_level as i32; 
        let level = this.level as i32; 
        if level > job_max_level {
            let new_internal_level = this.internal_level as i32 + job_max_level - level; 
            this.set_level(job_max_level);
            this.set_internal_level(new_internal_level);
        }
    }
    this.set_hp(this.get_capability(0, true));
}

pub fn change_unit_autolevel(unit: &mut Unit, reverse: bool) {
    let person = if reverse { super::switch_person_reverse(unit.person) } else { &unit.person };
    let new_person = if reverse { &unit.person } else { super::switch_person(unit.person) }; 
    println!("{} -> {}",  person.get_name().unwrap().get_string().unwrap(), new_person.get_name().unwrap().get_string().unwrap());
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
            println!("{} -> {} Base -> Base Level {}",  person.get_name().unwrap().get_string().unwrap(), new_person.get_name().unwrap().get_string().unwrap(), current_level);
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
                if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  { 
                    unit.auto_grow_capability(current_level, current_level + 20);
                    unit.set_level(current_level);
                    unit.set_internal_level( current_internal_level );
                    println!("Promoted Unit -> Special Unit but promoted");
                } 
                else {
                    let total_level = current_internal_level + current_level;
                    unit.class_change(new_person.get_job().unwrap());
                    unit.auto_grow_capability(total_level, 20+current_level);
                    unit.set_level(total_level);
                    unit.set_internal_level(0);
                    unit.set_level( ( person.get_level() + person.get_internal_level() as u8 ).into() );
                    println!("Promoted Unit -> Special Unit");
                }
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
    if CONFIG.lock().unwrap().change_unit_offset {
        for x in 0..11 { 
            unit.set_base_capability(x as i32, bases[x] as i32);
        } 
    }
    unit.set_sp( person.get_sp() );
    unit.set_person(new_person);    // change person
    fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes
    unit.set_hp(unit.get_capability(0, true));
    unsafe {  unit_learn_job_skill(unit, None); }
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
    // Use Vander's outset if vander
    if original.pid.get_string().unwrap() == "PID_ヴァンドレ" {
        for x in 0..11 {
            let value: i32 = 100 * old_offset[x as usize ] as i32 + old_level * new_grow[ x ] as i32 + 50;
            out[x as usize] = (value / 100 ) as i8;
        }
    }
    else {
    // Everyone calculate offset by subtracting class growths to adjust to original unit's level
        let diff_grow = new_job.get_diff_grow();
        let n_autolevels = ( if original_job.is_high() { 20 } else { 0 } + original.get_level() ) as i32; 
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

#[unity::from_offset("App", "Unit", "UpdateStateWithAutoEquip")]
pub fn unit_update_auto_equip(this: &Unit, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "set_Person")]
pub fn unit_set_person(this: &Unit, person: &PersonData, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "SetSelectedWeaponFromOriginalAptitude")]
fn unit_set_select_weapon_from_original_aptitude(this: &Unit, mask: &WeaponMask, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "AddAptitudeFromWeaponMask")]
fn unit_add_apt_from_weapon_mask(this: &Unit, method_info: OptionalMethod);

// done in Unit$$CreateImpl1
#[skyline::from_offset(0x01f25ec0)]
fn get_bmap_size(this: &PersonData, method_info: OptionalMethod) -> u8;

#[skyline::from_offset(0x01a3c290)]
fn unit_learn_job_skill(this: &Unit, method_info: OptionalMethod) -> &'static engage::gamedata::skill::SkillData;
