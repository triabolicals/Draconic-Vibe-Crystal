use super::*;
use crate::randomizer::person::unit::fixed_unit_weapon_mask;
use engage::menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItem}};

#[unity::class("App", "ClassChange.ChangeJobData")]
pub struct ChangeJobData {
    pub job: &'static JobData,
    pub job_weapon_mask: &'static WeaponMask,
    pub original_job_weapon_mask: &'static WeaponMask,
    pub proof_type: i32, 
    __: i32,
    pub cost_level: &'static Il2CppString,
    pub is_enough_level: bool,
    pub junk: [u8; 7],
    pub cost_weapon_mask: &'static WeaponMask,
    pub equippable_weapon_mask: &'static WeaponMask,
    pub enough_item: bool,
    pub is_gender: bool,
    pub is_default_job: bool,
}

pub struct RandomJobMod;
impl ConfigBasicMenuItemSwitchMethods for RandomJobMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_job, 0, 3, 1);
        if CONFIG.lock().unwrap().random_job != result {
            CONFIG.lock().unwrap().random_job  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_job {
            1 => { "Playable units will be in random classes." },
            2 => { "Enemy/NPC units will be in random classes."},
            3 => { "All units will be in random classes."},
            _ => { "Units will be in their assigned class"},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match  CONFIG.lock().unwrap().random_job {
            1 => {  "Player" },
            2 => { "Enemy/NPC" },
            3 => { "All Units" },
            _ => { "None"},
        }.into();
    }
}
pub struct EnemyJobGauge;
impl ConfigBasicMenuItemGaugeMethods for EnemyJobGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_enemy_job_rate as f32 / 100.0 }
            else { GameVariableManager::get_number("G_EnemyJobGauge") as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_enemy_job_rate as f32 / 100.0 }
            else { GameVariableManager::get_number("G_EnemyJobGauge") as f32 / 100.0 };

        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.25);
        if gauge != result {
            if GameUserData::get_sequence() == 0 {CONFIG.lock().unwrap().random_enemy_job_rate = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_EnemyJobGauge", ( result * 100.0 ) as i32 ); }
            this.gauge_ratio = result;
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = format!("{:2}% chance of enemy units will be in a random class.", this.gauge_ratio*100.0).into();
    }
}

pub extern "C" fn vibe_job_gauge() -> &'static mut ConfigBasicMenuItem {  
    let class_gauge = ConfigBasicMenuItem::new_gauge::<EnemyJobGauge>("Random Enemy Class Rate"); 
    class_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_job_gauge as _);
    class_gauge
}


pub fn unit_change_to_random_class(unit: &mut Unit){
    let rng = Random::get_game();
    let job_count = JobData::get_count();
    let is_female;
    if unit.person.pid.get_string().unwrap() == "PID_リュール" && GameVariableManager::exist("G_Lueur_Gender2") {
        is_female = GameVariableManager::get_number("G_Lueur_Gender2") == 2;
    }
    else if unit.edit.is_enabled() { is_female = unit.edit.gender == 2; }
    else { is_female = unit.person.get_gender() == 2; }
    let job_list = JobData::get_list().unwrap();
    let mut is_high = false;
    if unit.get_job().is_low() { is_high = false; }
    if unit.level >= 20 || unit.get_job().is_high() { is_high = true; }
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let mut count = 0;
    println!("Unit Level {} / Internal {}", unit_level, internal_level);
    loop {
        let index = rng.get_value(job_count);
        if index >= job_count { continue; }
        let job = &job_list[index as usize];
        if job.jid.get_string().unwrap() == "JID_マージカノン" { continue;}
        let job_flags = job.get_flag();
        if ( job_flags.value & 16 != 0 ) && ( is_female || unit.person.get_flag().value & 32 != 0 ) { continue; }
        if job_flags.value & 1 == 0 && job_flags.value & 2 == 0 { count += 1; continue;}
        if job_flags.value == 0 { continue;}
        if job_flags.value & 1 == 1 && job_flags.value & 2 == 0 { 
            if !is_high {
                if index % 4 == 0 {                 
                    if unit.person.get_job().unwrap().get_flag().value & 2 == 0 && unit.person.get_job().unwrap().is_low() {
                        unit.class_change(unit.person.get_job().unwrap());
                    }
                    else { unit.class_change(JobData::get("JID_マージ").unwrap()); }
                }
                else if index % 4 == 1 { unit.class_change(JobData::get("JID_モンク").unwrap()); }
                else if index % 4 == 2 { unit.class_change(JobData::get("JID_アーチャー").unwrap()); }
                else if index % 4 == 3 { unit.class_change(JobData::get("JID_シーフ").unwrap()); }
                else {
                    count += 1;
                    continue;
                }
                unit.set_level(unit_level); 
                unit.set_internal_level(internal_level);
                unit.set_hp(unit.get_capability(0, true));
                unit.set_weapon_mask_from_person();
                fixed_unit_weapon_mask(unit);
                return;
            }
            else { 
                count += 1;
                continue;
            }
        }
        if job_flags.value & 1 == 1 && job_flags.value & 2 == 0 { count += 1; continue;}
        if (job_flags.value & 4 == 4 ) && !is_female { count+=1; continue; }  // if female only and not a female
        if (!is_high && job.is_high() ) || (is_high && job.is_low() && job.jid.get_string().unwrap() != "JID_ダンサー") {
            count += 1;
            continue; 
        } // if promoted and new class is not promoted
        if unit.get_job().jid.get_string().unwrap() == job.jid.get_string().unwrap() { 
            count += 1;
            continue;
        }
        if job.jid.get_string().unwrap() == "JID_マージカノン" && !GameVariableManager::get_bool("G_CC_マージカノン") { 
            count += 1;
            continue;
        }
        if job.jid.get_string().unwrap() == "JID_エンチャント" && !GameVariableManager::get_bool("G_CC_エンチャント") { 
            count += 1;
            continue;
        }
        unit.class_change(job);
        if unit_level > 20 && job.is_high() { 
            unit.set_level(unit_level - 20); 
            unit.set_internal_level(internal_level+20);
        }
        else if unit_level == 20 && job.is_high() {
            unit.set_level(1); 
            unit.set_internal_level(internal_level+19);
        }
        else { 
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
        println!("{} changed to {} from {} in {} steps (Lv {}/{})", 
            unit.person.get_name().unwrap().get_string().unwrap(), 
            job.name.get_string().unwrap(),  
            unit.get_job().name.get_string().unwrap(), count, unit.level, unit.internal_level);
        
        unit.set_hp(unit.get_capability(0, true));
        fixed_unit_weapon_mask(unit);
        crate::autolevel::unit_update_learn_skill(unit);
        return;
    }
}

pub fn enemy_unit_change_to_random_class(unit: &mut Unit) -> bool {
    let current_job = unit.get_job();
    //let current_flags = current_job.get_flag().value;
    //if current_job.parent.index < 10 { return false; }  // If 
    if current_job.name.get_string().unwrap() == "MJID_Emblem" { return false; }
    let rng = Random::get_game();
    let job_count = JobData::get_count();
    let is_female;
    if unit.person.pid.get_string().unwrap() == "PID_リュール" && GameVariableManager::exist("G_Lueur_Gender2") {
        is_female = GameVariableManager::get_number("G_Lueur_Gender2") == 2;
    }
    else if unit.edit.is_enabled() { is_female = unit.edit.gender == 2; }
    else { is_female = unit.person.get_gender() == 2; }

    let job_list = JobData::get_list().unwrap();
    let mut is_high = false;
    if unit.get_job().is_low() { is_high = false; }
    if unit.level >= 20 || unit.get_job().is_high() { is_high = true; }
    let is_flying = unit.get_job().move_type == 3;
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let has_emblem = unit.get_god_unit().is_some() || ( GameUserData::get_chapter().cid.get_string().unwrap() != "CID_M011" );
    loop {
        let index = rng.get_value(job_count);
        let job = &job_list[index as usize];
        let job_flags = job.get_flag();
        let jid = job.jid.get_string().unwrap();
        if ( job_flags.value & 16 != 0 ) && ( is_female || unit.person.get_flag().value & 32 != 0 ) { continue; }
        if job_flags.value <= 1 { continue; }
        if (job_flags.value & 4 == 4 ) && !is_female {  continue; } 
        if jid == "JID_異形飛竜" || jid == "JID_幻影飛竜" { continue; } //Wyverns
        if jid == "JID_異形竜" || jid == "JID_幻影竜" { continue; } //Wyrms
        if jid == "JID_村人" { continue; }  // Villager

        if (!is_high && job.is_high() ) || (is_high && (job.is_low() && job.max_level == 20 ) ) {
            continue; 
        } // if promoted and new class is not promoted
        if unit.get_job().jid.get_string().unwrap() == job.jid.get_string().unwrap() { 
            return false;
        }
        if jid == "JID_異形狼" || jid == "JID_幻影狼"  {
            if has_emblem { continue; }   // has emblem and is either wolf class
            if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M011" { continue; }
        }
        unit.class_change(job);
        println!("Person #{}: {}:  Class Change to #{} {}", 
            unit.person.parent.index, 
            Mess::get(unit.person.get_name().unwrap()).get_string().unwrap(), 
            job.parent.index, Mess::get(job.name).get_string().unwrap()
        );

        if job.move_type != 3 && is_flying {
            if !unit.private_skill.add_sid("SID_天駆", 10, 0)  { continue; }
            if job.move_type == 2 {
                unit.private_skill.add_sid("SID_移動－１", 10, 0); 
                unit.private_skill.add_sid("SID_移動－３", 10, 0);
            }
            else { unit.private_skill.add_sid("SID_移動－２", 10, 0); }
        }
        if unit_level > 20 && job.is_high() { 
            unit.set_level(unit_level - 20); 
            unit.set_internal_level(internal_level+20);
        }
        else if unit_level == 20 && job.is_high() {
            unit.set_level(1); 
            unit.set_internal_level(internal_level+19);
        }
        else { 
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
        unit.set_hp(unit.get_capability(0, true));
        fixed_unit_weapon_mask(unit);
        crate::autolevel::unit_update_learn_skill(unit);
        return true;
    }
}

// Alow Class Change for Exclusive Classes based on Asset Gender
#[skyline::hook(offset=0x019c6700)]
pub fn add_job_list_unit(this: &mut ChangeJobData, unit: &Unit, method_info: OptionalMethod) -> bool {
    let result = call_original!(this, unit, method_info);
    if !crate::utils::can_rand() { return result; }
    // Dancer-lock
    if this.job.jid.get_string().unwrap() == "JID_ダンサー" { 
        if unit.get_job().jid.get_string().unwrap() == "JID_ダンサー" || unit.person.get_job().unwrap().jid.get_string().unwrap() == "JID_ダンサー" {
            if this.job.get_flag().value & 16 != 0 {
                let gender; 
                if unit.edit.is_enabled() { gender = unit.edit.gender; }  // Alear
                else { gender = unit.person.get_gender(); } // Everyone Else 
                if gender == 2 {  
                    this.is_gender = false;
                    return false; 
                }
                this.is_default_job = true;
                return result 
            }
        }
        else {
            this.is_gender = false;
            return false; 
        }
    }
    if this.job.get_flag().value & 16 != 0 {
        let gender = if unit.edit.is_enabled() { unit.edit.gender }  // Alear
                     else { unit.person.get_gender() }; // Everyone Else 
        if gender == 2 {  
            this.is_gender = false;
            return false; 
        }
        else {
            //Male in male only (with female animations)
            if unit.person.get_flag().value & 32 != 0 { 
                this.is_gender = false;
                return false; 
            }
        }
        return result;
    }
    if unit.person.get_flag().value & 32 != 0 && this.job.get_flag().value & 4 != 0 {
        if unit.person.get_gender() == 1 { 
            this.is_gender = true;
            let job_wm = this.job_weapon_mask.value;
            if unit.aptitude.value & job_wm == job_wm && (this.is_enough_level && this.enough_item ) {
                return true;
            }
            return result; 
        } 
        else { 
            this.is_gender = false; 
            return false; 
        } // Male Crossdressing in female class: true
          // Female Crossdressing in female class: false
    }
    return result;
}

pub fn get_weapon_for_asset_table(job: &JobData) -> Option<&'static ItemData> {
    let mut weapon_type = 0;
    let mut weapon_level = 0;
    for x in 1..10 {
        if x == 7 { continue; }
        if weapon_level < job.get_max_weapon_level(x) {
            weapon_type = x;
            weapon_level = job.get_max_weapon_level(x);
        }
    }
    return crate::randomizer::item::item_rando::WEAPONDATA.lock().unwrap().get_random_weapon(weapon_type);
}