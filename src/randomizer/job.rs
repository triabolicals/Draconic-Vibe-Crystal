use super::*;
use crate::randomizer::person::unit::fixed_unit_weapon_mask;
use engage::force::*;
use engage::unitpool::*;
use engage::util::get_instance;
use engage::menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItem}};
use crate::randomizer::job::reclass::ChangeJobData;

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

pub mod reclass;
pub mod menu;

pub struct UnitPoolStaticFieldsMut {
    pub s_unit: &'static mut Array<&'static mut Unit>,
    pub forces: &'static mut Array<&'static Force>,
}

fn rerandomize_jobs() {
    if DVCVariables::is_main_chapter_complete(3) {
        UnitPool::class().get_static_fields_mut::<UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| (1 << f.force_type) & 6 != 0 )).for_each(|unit|{
            if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 {
                if unit.person.get_asset_force() == 0 { unit_change_to_random_class(unit); }
                else { 
                    super::person::ai::reset_enemy_ai_and_items(unit);
                    enemy_unit_change_to_random_class(unit); 
                }
                crate::autolevel::auto_level_unit_for_random_map(unit, false);
                super::person::unit::adjust_unit_items(unit);
                if unit.force.unwrap().force_type != 0 { crate::randomizer::person::ai::adjust_unitai(unit);  }
                unit.auto_equip();;
                unit.reload_actor();
            }
        });
    }
    else {
        UnitPool::class().get_static_fields_mut::<UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type == 0 )).for_each(|unit|{
            if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 {
                if unit.person.get_asset_force() == 0 { unit_change_to_random_class(unit); }
                else {  enemy_unit_change_to_random_class(unit); }
                crate::autolevel::auto_level_unit_for_random_map(unit, false);
                super::person::unit::adjust_unit_items(unit);
                unit.auto_equip();
                unit.reload_actor();
            }
        });
    }

}

fn unit_random_can_reclass(job: &JobData, is_female: bool, high_class: bool, player: bool, emblem: bool) -> bool {
    if !GameVariableManager::get_bool(DVCVariables::CUSTOM_JOB_KEY) {
        if !JOB_HASH.iter().any(|&hash| hash == job.parent.hash ) { return false;}
    }
    let job_flags = job.flag.value;
    if job_flags & 32 != 0 { return false; }
    let jid = job.jid.to_string();
    if let Some(pos) = crate::assets::animation::MONSTERS.iter().position(|&j| j == jid) { 
        if !DVCVariables::is_main_chapter_complete(17) || emblem { return false; }
        if player { 
            if !crate::utils::dlc_check() { return pos == 5 || pos == 6 ; } // Wyrms Only
            else { return pos != 4 && pos != 7; }  // No Fell Dragons
        }  
        return  pos == 2 || pos == 3 || pos == 5 || pos == 6 ;  // Wolfs and Wyrms
    }
    if jid.contains("_紋章士_") { return false; }     // Prevent Emblem Classes

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
    if ( unit.person.pid.to_string() == PIDS[0] || unit.person.get_flag().value & 128 != 0 ) && 
        GameVariableManager::exist(DVCVariables::LUEUR_GENDER) { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 }
        else if unit.edit.is_enabled() { unit.edit.gender == 2 }
        else { unit.person.get_gender() == 2 };
    if unit.person.get_flag().value & 32 != 0 { is_female = !is_female };   // Reverse gender

    let job_list = JobData::get_list().unwrap();
    let current_job = unit.get_job();
    
    let old_data = get_old_person_data(unit);
    let is_high = old_data.0 == 1 || (old_data.0 == 2 && old_data.1 > 15 );

    let unit_level = old_data.1;
    let internal_level = if old_data.0 == 1 && old_data.2 == 0 { 20 } else { old_data.2 };
    println!("Unit Level {} / Internal {} (Current Job: {})", unit_level, internal_level, Mess::get(current_job.name));

    let class_list: Vec<_> = job_list.iter().filter(|&job| unit_random_can_reclass(job, is_female, is_high, true, false) ).collect();
    if class_list.len() == 0 { return; }

    let new_job = class_list[ rng.get_value( class_list.len() as i32) as usize ];
    unit.class_change(new_job);

    match old_data.0 {
        2 =>  { //Special
            if new_job.is_high() {
                if unit_level > 35 { 
                    unit.set_level(unit_level - 20);
                    unit.set_internal_level(20);
                }
                else if unit_level > 15 {
                    unit.set_level(unit_level - 15);
                    unit.set_internal_level(15);
                }
                else {
                    unit.set_level(1);
                    unit.set_internal_level(unit_level);
                }
            }
            else if new_job.max_level == 40 {
                let new_level = if unit_level > 40 { 40 } else { unit_level };
                unit.set_level(new_level); 
                unit.set_internal_level(0);
            }
            else {
                unit.set_level(unit_level); 
                unit.set_internal_level(0);
            }
        }
        1 => {  //Promoted
            if new_job.is_high() {
                unit.set_level(unit_level); 
                unit.set_internal_level(internal_level);
            }
            else if new_job.max_level == 40 {
                let total = unit_level + internal_level;
                let new_level = if total > 40 { 40 } else { total };
                unit.set_level(new_level); 
                unit.set_internal_level(0);
            }
            else {
                let total = unit_level + internal_level;
                let new_level = if total > 20 { 20 } else { total };
                unit.set_level(new_level);
                unit.set_internal_level(0);
            }
        }
        _ => {  //Base
            if new_job.is_high()   {
                if unit_level > 20 {
                    unit.set_level(unit_level - 20); 
                    unit.set_internal_level(20);
                }
                else {
                    unit.set_level(1); 
                    unit.set_internal_level(unit_level - 1);
                }
            }
            else {
                unit.set_level(unit_level);
                unit.set_internal_level(internal_level);
            }
        }
    }


    unit.set_hp(unit.get_capability(0, true));  // fix HP
    unit.set_weapon_mask_from_person(); 

    fixed_unit_weapon_mask(unit);
    randomize_selected_weapon_mask(unit);
    let mut new_opt = 
        if unit.job.get_weapon_mask().value & (1 << 9) != 0 { 1 << ( rng.get_value(8) + 1 ) }
        else if unit.selected_weapon_mask.value == 0 { unit.job.get_weapon_mask().value }
        else { unit.selected_weapon_mask.value };

    if rng.get_value(20) < 5 {
        new_opt |= 1 << ( rng.get_value(8) + 1 );
        if rng.get_value(50) < 5 {
            new_opt |= 1 << ( rng.get_value(8) + 1 );
        }
    }
    unit.original_aptitude.value = new_opt;
    unit.aptitude.value |= new_opt;
    crate::randomizer::skill::learn::unit_update_learn_skill(unit);
    println!("{} changed to {} (Lv {}/{})", Mess::get_name(unit.person.pid), Mess::get(new_job.name), unit.level, unit.internal_level);
    
}

pub fn enemy_unit_change_to_random_class(unit: &mut Unit) -> bool {
    let current_job = unit.get_job();
    let current_job_name = current_job.name.to_string();
    if current_job_name == "MJID_Emblem" { return false; }
    if unit.person.get_gender() > 2 || unit.person.get_gender() == 0 { return false; }  // Monster Class
    let rng = Random::get_game();

    let mut is_female = 
        if ( unit.person.pid.to_string() == PIDS[0] || unit.person.get_flag().value & 128 != 0 ) && 
            GameVariableManager::exist(DVCVariables::LUEUR_GENDER) { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 }
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
    println!("{} changed to {} (Lv {}/{}) from {}", Mess::get_name(unit.person.pid), Mess::get(new_job.name), unit.level, unit.internal_level, Mess::get(current_job_name));
    unit.set_hp(unit.get_capability(0, true));
    fixed_unit_weapon_mask(unit);
    randomize_selected_weapon_mask(unit);
    crate::randomizer::skill::learn::unit_update_learn_skill(unit);
    return true;
}

// Alow Class Change for Exclusive Classes based on Asset Gender


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
    return crate::randomizer::item::data::WEAPONDATA.get().unwrap().get_random_weapon(weapon_type - 1, true);
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
    ["JID_邪竜", "JID_不明", "JID_邪竜ノ王", "JID_M000_邪竜ノ王"].iter().for_each(|jid| 
        if let Some(job) = JobData::get_mut(jid) {job.get_flag().value = 0;}
    );
    ["JID_フロラージュ下級", "JID_フロラージュ", "JID_フロラージュ_E", "JID_リンドブルム下級", "JID_リンドブルム", "JID_リンドブルム_E", "JID_スレイプニル下級", "JID_スレイプニル", "JID_スレイプニル_E", "JID_ピッチフォーク下級", "JID_ピッチフォーク", "JID_ピッチフォーク_E",
    "JID_メリュジーヌ_味方", "JID_メリュジーヌ", "JID_裏邪竜ノ娘"].iter()
    .for_each(|jid| 
        if let Some(job) = JobData::get_mut(jid) {job.get_flag().value |= 7;}
    );

    [ "JID_アヴニール下級", "JID_アヴニール", "JID_アヴニール_E", "JID_スュクセサール下級", "JID_スュクセサール", "JID_スュクセサール_E", "JID_ティラユール下級", "JID_ティラユール", "JID_ティラユール_E", "JID_クピードー下級", "JID_クピードー", "JID_クピードー_E",
    "JID_裏邪竜ノ子", "JID_ダンサー"].iter()
    .for_each(|jid|
        if let Some(job) = JobData::get_mut(jid) {
            job.get_flag().value |= 16;
            job.get_flag().value &= !4; 
            job.get_flag().value |= 3;
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

