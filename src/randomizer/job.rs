use super::*;
use crate::randomizer::person::unit::fixed_unit_weapon_mask;
use assets::animation::MONSTERS;
use engage::menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItem}};
use utils::can_rand;
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

pub struct RandomCC;
impl ConfigBasicMenuItemSwitchMethods for RandomCC {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let previous = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_reclass }
            else { GameVariableManager::get_bool("G_RandomCC")};

        let result = ConfigBasicMenuItem::change_key_value_b(previous);
        if previous != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_reclass = result; }
            else { GameVariableManager::set_bool("G_RandomCC", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let state = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_reclass }
            else { GameVariableManager::get_bool("G_RandomCC")};
        this.help_text = if state { "When reclassing, the new class will be determined randomly." }
            else { "Default reclassing behavior."}.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let state = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_reclass }
        else { GameVariableManager::get_bool("G_RandomCC")};
        this.command_text = if state { "Random" } else { "Default" }.into();
    }
}

fn unit_random_can_reclass(job: &JobData, is_female: bool, high_class: bool, player: bool) -> bool {
    let jid = job.jid.to_string();
    if let Some(pos) = MONSTERS.iter().position(|&j| j == jid) { 
        if player { return true; }  // Allow all
        return  ( pos == 2 || pos == 3 || pos == 5 || pos == 6 );   // No Wyverns + Fell Dragon
    }
    if jid.contains("_紋章士_") { return true; }
    let job_flags = job.flag.value;
    if ( player && ( job_flags & 3 == 0 || job_flags == 0 ) ) || (job_flags & 16 != 0 && is_female ) || (job_flags & 4 != 0 && !is_female) { return false; }    // Wrong Gender / Player Can't reclass
    if high_class && (job.is_low() && job.max_level == 20 ) || ( !high_class && job.is_high() ) { return false; }   // Wrong Job Tier

    // Prevent Emblem Classes

    if jid == "JID_村人" { return false; }  // No Villager
    if jid == "JID_マージカノン" { return GameVariableManager::get_bool("G_CC_マージカノン"); } // FX Checks for Mage Cannoner / Enchanter
    if jid == "JID_エンチャント" { return GameVariableManager::get_bool("G_CC_エンチャント"); } 
    return true;
}

pub fn unit_change_to_random_class(unit: &mut Unit){
    let rng = Random::get_game();
    let job_count = JobData::get_count();
    let mut is_female = 
        if unit.person.pid.to_string() == "PID_リュール" && GameVariableManager::exist("G_Lueur_Gender2") { GameVariableManager::get_number("G_Lueur_Gender2") == 2 }
        else if unit.edit.is_enabled() { unit.edit.gender == 2 }
        else { unit.person.get_gender() == 2 };
    if unit.person.get_flag().value & 32 != 0 { is_female = !is_female };   // Reverse gender

    let job_list = JobData::get_list().unwrap();
    let current_job = unit.get_job();
    let is_high = if current_job.is_low() { unit.level > 20 }
        else { current_job.is_high() };

    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let mut count = 0;
    println!("Unit Level {} / Internal {} (Current Job: {})", unit_level, internal_level, Mess::get(current_job.name));

    let class_list: Vec<_> = job_list.iter().filter(|&job| unit_random_can_reclass(job, is_female, is_high, true) ).collect();
    if class_list.len() == 0 { return; }

    let new_job = class_list[ rng.get_value( class_list.len() as i32) as usize ];
    unit.class_change(new_job);

    // Keep original level and internal level
    if unit_level > 20 {
        if new_job.is_high() {
            unit.set_level(unit_level - 20); 
            unit.set_internal_level(internal_level+20);
        }
        else {
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
    }
    else if unit_level == 20 && new_job.is_high() {
        unit.set_level(1); 
        unit.set_internal_level(internal_level+19);
    }
    else {
        unit.set_level(unit_level); 
        unit.set_internal_level(internal_level);
    }

    unit.set_hp(unit.get_capability(0, true));  // fix HP
    unit.set_weapon_mask_from_person(); 
    fixed_unit_weapon_mask(unit);
    randomize_selected_weapon_mask(unit);

    println!("{} changed to {} (Lv {}/{})", Mess::get_name(unit.person.pid), Mess::get(new_job.name), unit.level, unit.internal_level);
    
}

pub fn enemy_unit_change_to_random_class(unit: &mut Unit) -> bool {
    let current_job = unit.get_job();
    //let current_flags = current_job.get_flag().value;
    //if current_job.parent.index < 10 { return false; }  // If 
    if current_job.name.to_string() == "MJID_Emblem" { return false; }
    if unit.person.get_gender() > 2 || unit.person.get_gender() == 0 { return false; }  // Monster Class
    let rng = Random::get_game();

    let mut is_female = 
        if unit.person.pid.to_string() == "PID_リュール" && GameVariableManager::exist("G_Lueur_Gender2") { GameVariableManager::get_number("G_Lueur_Gender2") == 2 }
        else if unit.edit.is_enabled() { unit.edit.gender == 2 }
        else { unit.person.get_gender() == 2 };
    if unit.person.get_flag().value & 32 != 0 { is_female = !is_female };   // Reverse gender

    let is_high = if current_job.is_low() { unit.level > 20 }
        else { current_job.is_high() };

    let is_flying = unit.get_job().move_type == 3;
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let has_emblem = unit.get_god_unit().is_some() || ( GameUserData::get_chapter().cid.to_string() != "CID_M011" );

    let class_list: Vec<_> = JobData::get_list().unwrap().iter().filter(|&job| unit_random_can_reclass(job, is_female, is_high, true) ).collect();
    if class_list.len() == 0 { return false; }

    let new_job = class_list[ rng.get_value( class_list.len() as i32) as usize ];
    unit.class_change(new_job);

    // Keep original level and internal level
    if unit_level > 20 {
        if new_job.is_high() {
            unit.set_level(unit_level - 20); 
            unit.set_internal_level(internal_level+20);
        }
        else {
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
    }
    else if unit_level == 20 && new_job.is_high() {
        unit.set_level(1); 
        unit.set_internal_level(internal_level+19);
    }
    else {
        unit.set_level(unit_level); 
        unit.set_internal_level(internal_level);
    }

    // Allow Previous Fliers to Fly
    if new_job.move_type != 3 && is_flying {
        unit.private_skill.add_sid("SID_天駆", 10, 0);
        if new_job.move_type == 2 {
            unit.private_skill.add_sid("SID_移動－１", 10, 0); 
            unit.private_skill.add_sid("SID_移動－３", 10, 0);
        }
        else { unit.private_skill.add_sid("SID_移動－２", 10, 0); }
    }

    unit.set_hp(unit.get_capability(0, true));
    fixed_unit_weapon_mask(unit);
    randomize_selected_weapon_mask(unit);
    crate::autolevel::unit_update_learn_skill(unit);
    return true;
}

// Alow Class Change for Exclusive Classes based on Asset Gender
#[skyline::hook(offset=0x019c6700)]
pub fn add_job_list_unit(this: &mut ChangeJobData, unit: &Unit, method_info: OptionalMethod) -> bool {
    let result = call_original!(this, unit, method_info);
    this.is_gender = true;
    this.is_default_job = true;
    return true;
    if !can_rand() { return result; }
    if CONFIG.lock().unwrap().debug {
        this.is_gender = true;
        this.is_default_job = true;
        return true;
    }
    // Dancer-lock
    if this.job.jid.contains("ダンサー") { 
        if unit.get_job().jid.contains("ダンサー") || unit.person.get_job().unwrap().jid.contains("ダンサー") {
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

pub fn is_magic_class(job: &JobData) -> bool {
    let mut weapon_type = 0;
    let mut weapon_level = 0;
    for x in 1..9 {
        if x == 7 { continue; }
        if weapon_level < job.get_max_weapon_level(x) {
            weapon_type = x;
            weapon_level = job.get_max_weapon_level(x);
        }
    }
    weapon_type == 6 || weapon_type == 7
}

pub fn get_weapon_for_asset_table(job: &JobData) -> Option<&'static ItemData> {
    let mut weapon_type = 0;
    let mut weapon_level = 0;
    for x in 1..9 {
        if x == 7 { continue; }
        if weapon_level < job.get_max_weapon_level(x) {
            weapon_type = x;
            weapon_level = job.get_max_weapon_level(x);
        }
    }
    return crate::randomizer::item::item_rando::WEAPONDATA.lock().unwrap().get_random_weapon(weapon_type);
}

pub fn correct_job_base_stats() {
    let job_list = JobData::get_list_mut().unwrap();
    job_list.iter_mut()
        .filter(|job| job.is_low() )
        .for_each(|job|{
            let bases = job.get_base();
            let cap = job.get_limit();
            let high_jobs = job.get_high_jobs();
            //job.get_flag().value |= 3;
            high_jobs.iter()
                .for_each(|hjob| {
                    let h_job = JobData::get_mut(hjob.jid).unwrap();
                    let h_bases = h_job.get_base();
                    let h_cap = h_job.get_limit();
                    for x in 0..10 { 
                        if h_cap[x] < cap[x] { h_cap[x] = cap[x] + 2; }
                        if h_bases[x] < bases[x] { h_bases[x] = bases[x] + 1; } 
                    }
                    if h_bases[10] < bases[10] { h_bases[10] = bases[10]; }
                    if h_cap[10] < cap[10] { h_cap[10] = cap[10]; }
                }
            );
        }
    );
}

pub fn randomize_selected_weapon_mask(unit: &mut Unit) {
    let job = unit.get_job();
    let mut weapon_select_count = 0;
    let selectable_job_weapons = job.get_selectable_weapon_mask(&mut weapon_select_count);
    if weapon_select_count == 0 { return; }
    let rng = Random::get_system();
    // adjusting equippable weapons

    let mut select_kinds: Vec<i32> = Vec::new();
    for x in 1..10 {
        if selectable_job_weapons.value & ( 1 << x ) != 0 { select_kinds.push(x); }
    }
    let mut selected = 0;
    let mut mask = 0;

    while selected < weapon_select_count {
        let index = rng.get_value(select_kinds.len() as i32);
        let wp = 1 << select_kinds[index as usize];
        if mask & wp == 0 {
            mask |= wp;
            selected += 1;
        }
    }
    unit.selected_weapon_mask.value = mask;

}
#[skyline::from_offset(0x01ea2050)]
pub fn get_job_list(method_info: OptionalMethod) -> &'static mut List<ChangeJobData>;

#[unity::class("App", "UnitGrowSequence")]
pub struct UnitGrowSequence {
    proc: [u8; 0x64],
    pub unit: &'static mut Unit,
    pub exp: i32,
    pub old_level: i32,
    pub is_talk: bool,
    pub sp: i32,
    pub cc_job: Option<&'static JobData>,
    pub cc_item:  Option<&'static ItemData>,
    pub cc_weapon_mask:  Option<&'static WeaponMask>,
    pub cc_weapon:  Option<&'static ItemData>,

}
#[skyline::from_offset(0x019c6700)]
fn cc_check(this: &ChangeJobData, unit: &Unit, method_info: OptionalMethod) -> bool;

fn add_to_list(this: &ChangeJobData, unit: &Unit, cc_type: i32) -> bool {
    let job = unit.get_job();
    if this.proof_type == 0 && cc_type == 0 {
        return true;
    }
    if this.proof_type > 0 && cc_type > 0 {
        if this.job.is_low() && job.is_high() { return this.job.max_level == 40;  }
        return true;
    }
    return false;
}

#[unity::hook("App", "UnitGrowSequence", "ClassChange")]
pub fn unit_grow_class_change(this: &mut UnitGrowSequence, method_info: OptionalMethod) {
    if !GameVariableManager::get_bool("G_RandomCC") || !can_rand() || CONFIG.lock().unwrap().debug {
        call_original!(this, method_info);
        return;
    }
    let change_job_list = unsafe { get_job_list(None) };
     // for CCCheck to get classes.

    if let Some(item) = this.cc_item {
        let proof_type = match item.usetype {
            23 => { 0 },
            24|40|41 => { 1 },
            _ => { call_original!(this, None); return; },
        };
        let original_apt = this.unit.aptitude.value;
        this.unit.aptitude.value = -1; 
        let pool: Vec<_> = change_job_list.iter().filter(|&cc_job|{
            if unsafe { cc_check(cc_job, this.unit, None) } { add_to_list(cc_job, this.unit, proof_type) }
            else { false } 

        }).collect();

        this.unit.aptitude.value = original_apt;
        let rng = Random::instantiate().unwrap();
        let seed = this.unit.grow_ssed; 
        rng.ctor(seed as u32);
        this.unit.grow_ssed = rng.value();

        if pool.len() > 1 { 
            let pool_size = pool.len() as i32;
            let class = rng.get_value(pool_size) as usize;
            this.cc_job = Some( pool[class].job );
            this.cc_weapon_mask = Some( pool[class].equippable_weapon_mask );
        }
    }
    call_original!(this, method_info);
}
