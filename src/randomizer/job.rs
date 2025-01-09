use std::f32::consts::E;

use super::*;
use crate::randomizer::person::unit::fixed_unit_weapon_mask;
use assets::animation::MONSTERS;
use engage::force::*;
use engage::unitpool::*;
use engage::util::get_instance;
use engage::menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItem}};
use utils::can_rand;

pub const JOB_HASH: [i32; 111] = [ 
    1367578960, -1369632991, 689554073, -1369626630, 185671037, 1499787884, 185670709, -1998645787, 
    185677222, 1463244270, -1274089237, 730392094, -121798307, -1165634998, 1133576057, -362221162, 
    511515477, 1766556981, -1517502902, 748907755, -1955176032, -910371360, 1977191031, 7727229, 
    -1536732910, -1706110952, 1462565615, 229381663, 1443627162, 624511329, 167430752, -1257479603, 
    -1097089283, 1784856681, -274348868, 2019899510, -1700828674, -2114661299, 1694508683, -116808589, 
    -1521107381, -463737354, -999305939, -1864726869, -932121231, -673485317, -1768549784, -412226531, 
    -1008556160, 49655521, -897692291, 215311336, -828118047, -765055509, -1377297555, -2055769334, 
    -1820313133, -1308218306, 742712801, 1549244198, -2126214230, -1001243599, 1942863689, 1881805028, 
    -1116401495, 1156629411, 499211320, 1455055960, -22101593, 1057745236, 1992190012, 578347357, 
    1196308869, 475113468, 595125119, 1851723759, 1503228531, 1749933065, 1860731459, 30291080, 
    30291392, 2021853286, 1445776173, 746034087, -1028496122, -631685449, 1398652429, -1116399479, 
    1156631427, -290620106, 704942064, -149285011, 1848240617, -1241492235, 352107958, 5510888, 
    1982073595, -1406772370, -974713853, 266659697, -158341635, 577339931, 1316562832, -573263642, 
    355160656, 842455118, 692959593, 1534528826, 877759506, -1361615043, -1095679653
];

pub struct UnitPoolStaticFieldsMut {
    pub s_unit: &'static mut Array<&'static mut Unit>,
    pub forces: &'static mut Array<&'static Force>,
}

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
pub struct CustomJobs;
impl ConfigBasicMenuItemSwitchMethods for CustomJobs {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().get_custom_jobs();
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().set_custom_jobs(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().get_custom_jobs() { "Allows customs classes in the randomization pool." }
        else { "Only vanilla classes in the randomization pool" }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().get_custom_jobs() { "Include"} else { "Exclude"}.into();
    }
}

fn custom_job_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if JobData::get_count() <= 111 { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if GameVariableManager::get_number("G_Random_Job") > 0 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_custom_job() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<CustomJobs>("Custom Classes"); 
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = custom_job_build_attr as _);
    switch
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
        let previous = CONFIG.lock().unwrap().get_random_cc();
        let result = ConfigBasicMenuItem::change_key_value_b(previous);
        if previous != result {
            CONFIG.lock().unwrap().set_random_cc(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().get_random_cc() { "When reclassing, the new class will be determined randomly." }
            else { "Default reclassing behavior."}.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().get_random_cc() { "Random" } else { "Default" }.into();
    }
}

pub struct RerandomizeJobs;
impl ConfigBasicMenuItemCommandMethods for  RerandomizeJobs {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();
        if pad_instance.npad_state.buttons.a() {
            YesNoDialog::bind::<RerandomizeJobsConfirm>(this.menu, "Randomize Classes for Ally Units?\nItems will be replaced.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "Randomize".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.help_text = "Re-randomize ally units' classes.".into();  }
}

pub struct RerandomizeJobsConfirm;
impl TwoChoiceDialogMethods for RerandomizeJobsConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        rerandomize_jobs();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn re_randomize_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_number("G_Random_Job") & 1 == 0 { return  BasicMenuItemAttribute::Hide; }
    if GameVariableManager::get_bool("G_Cleared_M003") {
        let count = if Force::get(ForceType::Ally).is_some() { Force::get(ForceType::Ally).unwrap().get_count() } else { 0 };
        if GameUserData::get_sequence() == 2 && count > 0 { BasicMenuItemAttribute::Enable }
        else {  BasicMenuItemAttribute::Hide  }
    }
    else if GameUserData::get_sequence() == 3 && GameVariableManager::get_number("G_Random_Job") & 1 != 0 { BasicMenuItemAttribute::Enable }
    else {  BasicMenuItemAttribute::Hide  }
}

pub extern "C" fn vibe_job_rerand() -> &'static mut ConfigBasicMenuItem {  
    let class_gauge = ConfigBasicMenuItem::new_command::<RerandomizeJobs>("Re-Randomize Classes"); 
    class_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = re_randomize_build_attr as _);
    class_gauge
}
fn rerandomize_jobs() {
    if GameVariableManager::get_bool("G_Cleared_M003") {
        UnitPool::class().get_static_fields_mut::<UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.filter(|f| f.force_type == 2  ).is_some()).for_each(|unit|{
            if GameVariableManager::get_number("G_Random_Job") & 1 != 0 {
                if unit.person.get_asset_force() == 0 { unit_change_to_random_class(unit); }
                else { enemy_unit_change_to_random_class(unit); }
                if GameVariableManager::get_number("G_Continuous") == 3 { crate::randomizer::person::unit::random_map_unit_level(unit); }
                else if GameVariableManager::get_bool("G_DVC_Autolevel") { crate::autolevel::auto_level_unit(unit); }
                super::person::unit::adjust_unit_items(unit);
                unit.auto_equip();
                unit.reload_actor();
            }
        });
    }
    else {
        UnitPool::class().get_static_fields_mut::<UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.filter(|f| f.force_type == 0 || f.force_type == 2 ).is_some()).for_each(|unit|{
            if GameVariableManager::get_number("G_Random_Job") & 1 != 0 {
                if unit.person.get_asset_force() == 0 { unit_change_to_random_class(unit); }
                else { enemy_unit_change_to_random_class(unit); }
                if GameVariableManager::get_number("G_Continuous") == 3 { crate::randomizer::person::unit::random_map_unit_level(unit); }
                else if GameVariableManager::get_bool("G_DVC_Autolevel") { crate::autolevel::auto_level_unit(unit); }
                super::person::unit::adjust_unit_items(unit);
                unit.auto_equip();
                unit.reload_actor();
            }
        });
    }

}

fn unit_random_can_reclass(job: &JobData, is_female: bool, high_class: bool, player: bool, emblem: bool) -> bool {
    if !CONFIG.lock().unwrap().get_custom_jobs() {
        if !JOB_HASH.iter().any(|&hash| hash == job.parent.hash ) { return false;}
    }
    let jid = job.jid.to_string();
    if let Some(pos) = MONSTERS.iter().position(|&j| j == jid) { 
        if !GameVariableManager::get_bool("G_Cleared_M017") || emblem { return false; }
        if player { 
            if !crate::utils::dlc_check() { return pos == 5 || pos == 6 ; } // Wyrms Only
            else { return pos != 4 && pos != 7; }  // No Fell Dragons
        }  
        return  pos == 2 || pos == 3 || pos == 5 || pos == 6 ;  // Wolfs and Wyrms
    }
    if jid.contains("_紋章士_") { return false; }     // Prevent Emblem Classes
    let job_flags = job.flag.value;
    if job_flags == 0 || ( player && job_flags & 3 == 0  ) || (job_flags & 16 != 0 && is_female ) || (job_flags & 4 != 0 && !is_female) { return false; }    // Wrong Gender / Player Can't reclass
    if high_class && (job.is_low() && job.max_level == 20 ) || ( !high_class && job.is_high() ) { return false; }   // Wrong Job Tier

    if jid == "JID_村人" { return false; }  // No Villager
    if jid == "JID_マージカノン" { return GameVariableManager::get_bool("G_CC_マージカノン"); } // FX Checks for Mage Cannoner / Enchanter
    if jid == "JID_エンチャント" { return GameVariableManager::get_bool("G_CC_エンチャント"); } 
    return true;
}

fn get_old_person_data(unit: &Unit) -> (i32, i32, i32) {
    let key = format!("G_R2_{}", unit.person.pid);
    if GameVariableManager::exist(key.as_str()) {
        if let Some(person) = PersonData::get(GameVariableManager::get_string(key.as_str())) {
            let level = person.get_level() as i32;
            let internal = person.get_internal_level() as i32;
            let job = person.get_job().unwrap();
            if job.is_high() { return (1, level, internal); }
            else if job.max_level == 40 { return (2, level, internal); }
            else { return (0, level, internal); }
        }
    }
    let job = unit.person.get_job().unwrap();
    let level = unit.person.get_level() as i32;
    let internal = unit.person.get_internal_level() as i32;
    if job.is_high() { return (1, level, internal); }
    else if job.max_level == 40 { return (2, level, internal); }
    else { return (0, level, internal); }
}

pub fn unit_change_to_random_class(unit: &mut Unit){
    let rng = Random::get_game();
    let mut is_female = 
        if unit.person.pid.to_string() == "PID_リュール" && GameVariableManager::exist("G_Lueur_Gender2") { GameVariableManager::get_number("G_Lueur_Gender2") == 2 }
        else if unit.edit.is_enabled() { unit.edit.gender == 2 }
        else { unit.person.get_gender() == 2 };
    if unit.person.get_flag().value & 32 != 0 { is_female = !is_female };   // Reverse gender

    let job_list = JobData::get_list().unwrap();
    let current_job = unit.get_job();
    
    let old_data = get_old_person_data(unit);
    let is_high = old_data.0 > 0;

    let unit_level = old_data.1;
    let internal_level = if old_data.0 == 1 && old_data.2 == 0 { 20 } else { old_data.2 };
    println!("Unit Level {} / Internal {} (Current Job: {})", unit_level, internal_level, Mess::get(current_job.name));

    let class_list: Vec<_> = job_list.iter().filter(|&job| unit_random_can_reclass(job, is_female, is_high, true, false) ).collect();
    if class_list.len() == 0 { return; }

    let new_job = class_list[ rng.get_value( class_list.len() as i32) as usize ];
    unit.class_change(new_job);

    // Keep original level and internal level
    if unit_level > 20 {
        if new_job.is_high() {
            if internal_level == 0 {
                if unit_level - internal_level > 20 {
                    unit.set_level(20);
                    unit.set_internal_level(internal_level + unit_level - 20);
                }
                else {
                    unit.set_level(crate::utils::max(1, unit_level - internal_level));
                    unit.set_internal_level(internal_level);
                }
            }
            else {
                unit.set_level(unit_level - 20); 
                unit.set_internal_level(internal_level+20);
            }
        }
        else if new_job.max_level == 40 {
            if unit_level > 40 {
                unit.set_level(40); 
                unit.set_internal_level(0);
            }
            else {
                unit.set_level(unit_level + internal_level); 
                unit.set_internal_level( 40 - unit_level - internal_level);
            }
        }
        else {
            unit.set_level(unit_level); 
            unit.set_internal_level(internal_level);
        }
    }
    else if unit_level == 20 && new_job.is_high() && old_data.0 == 0 {
        unit.set_level(1); 
        unit.set_internal_level(internal_level+19);
    }
    else if new_job.max_level == 40 {
        if unit_level + internal_level > 40 {
            unit.set_level( unit_level + internal_level); 
            unit.set_internal_level( 40 - unit_level - internal_level);
        }
        else {
            unit.set_level(unit_level + internal_level); 
            unit.set_internal_level(0);
        }
    }
    else {
        unit.set_level(unit_level); 
        unit.set_internal_level(internal_level);
    }

    unit.set_hp(unit.get_capability(0, true));  // fix HP
    unit.set_weapon_mask_from_person(); 
    fixed_unit_weapon_mask(unit);
    randomize_selected_weapon_mask(unit);
    crate::autolevel::unit_update_learn_skill(unit);
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

    let class_list: Vec<_> = JobData::get_list().unwrap().iter().filter(|&job| unit_random_can_reclass(job, is_female, is_high, false, has_emblem) ).collect();
    if class_list.len() == 0 { return false; }

    let new_job = class_list[ rng.get_value( class_list.len() as i32) as usize ];
    unit.class_change(new_job);

    // Keep original level and internal level
    if unit_level > 20 {
        if new_job.is_high() {
            if unit_level + internal_level > 40 {
                unit.set_level(40);
                unit.set_internal_level(40 - unit_level - internal_level);
            }
            else {
                unit.set_level(unit_level - 20); 
                unit.set_internal_level(internal_level+20);
            }
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
    else if new_job.max_level == 40 {
        if unit_level + internal_level > 40 {
            unit.set_level( unit_level + internal_level); 
            unit.set_internal_level( 40 - unit_level - internal_level);
        }
        else {
            unit.set_level(unit_level + internal_level); 
            unit.set_internal_level(0);
        }
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
    return crate::randomizer::item::item_rando::WEAPONDATA.lock().unwrap().get_random_weapon(weapon_type - 1, true);
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
    ["JID_邪竜", "JID_不明", "JID_邪竜ノ子", "JID_M002_神竜ノ王", "JID_邪竜ノ王", "JID_M000_邪竜ノ王"].iter().for_each(|jid| 
        if let Some(job) = JobData::get_mut(jid) {job.get_flag().value = 0;}
    );
    ["JID_フロラージュ下級", "JID_フロラージュ", "JID_フロラージュ_E", "JID_リンドブルム下級", "JID_リンドブルム", "JID_リンドブルム_E", "JID_スレイプニル下級", "JID_スレイプニル", "JID_スレイプニル_E", "JID_ピッチフォーク下級", "JID_ピッチフォーク", "JID_ピッチフォーク_E",
    "JID_メリュジーヌ_味方", "JID_メリュジーヌ", "JID_裏邪竜ノ娘"].iter()
    .for_each(|jid| 
        if let Some(job) = JobData::get_mut(jid) {job.get_flag().value |= 4;}
    );

    [ "JID_アヴニール下級", "JID_アヴニール", "JID_アヴニール_E", "JID_スュクセサール下級", "JID_スュクセサール", "JID_スュクセサール_E", "JID_ティラユール下級", "JID_ティラユール", "JID_ティラユール_E", "JID_クピードー下級", "JID_クピードー", "JID_クピードー_E",
    "JID_裏邪竜ノ子", "JID_ダンサー"].iter()
    .for_each(|jid|
        if let Some(job) = JobData::get_mut(jid) {
            job.get_flag().value |= 16;
            job.get_flag().value &= !4; 
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

#[unity::class("App", "ClassChangeJobMenuItem")]
pub struct ClassChangeJobMenuItem {
    pub menu: u64,
    pub junk: [u8; 0x4c],
    pub job_data: &'static mut ChangeJobData,
    pub atr: i32,
}
pub fn class_change_a_call_random_cc(item: &ClassChangeJobMenuItem, _method_info: OptionalMethod) -> i32 {
    if item.atr != 1 { return 0x800; }
    if !GameVariableManager::get_bool("G_RandomCC") || !can_rand() {
        if item.atr == 1 {
            unsafe { class_change_confirm_bind(item.menu, item.job_data, None); }
            return 0x80;
        }
        return 0x800;
    }
    else {
        let unit = unsafe { class_change_get_unit(None) };
        let change_job_list = unsafe { get_job_list(None) };
        // for CCCheck to get classes.
        let proof = if item.job_data.proof_type > 0 { 1 } else { 0 };
        unit.aptitude.value = -1; 
        let pool: Vec<_> = change_job_list.iter().filter(|&cc_job|{
            if unsafe { cc_check(cc_job, unit, None) } { add_to_list(cc_job, unit, proof) }
            else { false } 
           }).collect();
        let rng = Random::instantiate().unwrap();
        let seed = unit.grow_seed; 
        rng.initialize(seed as u32);
        if pool.len() > 1 { 
            let pool_size = pool.len() as i32;
            let class = rng.get_value(pool_size) as usize;
            unsafe { class_change_confirm_bind(item.menu, pool[class], None); }
        }
        else { unsafe { class_change_confirm_bind(item.menu, item.job_data, None); } }
    }
    return 0x80;
}
#[skyline::from_offset(0x019c76c0)]
fn class_change_confirm_bind(this: u64, data: &ChangeJobData, method_info: OptionalMethod);

#[skyline::from_offset(0x01ea4680)]
pub fn class_change_get_unit(method_info: OptionalMethod) -> &'static mut Unit;
