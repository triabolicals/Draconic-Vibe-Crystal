use engage::{
    unit::*, force::*,
    menu::{menus::class_change::ClassChangeJobData, BasicMenuItemAttribute},
};
use super::*;
use crate::randomizer::{person::unit::fixed_unit_weapon_mask, grow::adaptive_growths};

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
pub mod single;
pub mod chaos;
pub mod lockout;

pub struct UnitPoolStaticFieldsMut {
    pub s_unit: &'static mut Array<&'static mut Unit>,
    pub forces: &'static mut Array<&'static Force>,
}
const WEP_KEY: [&str; 4] = ["Sword", "Lance", "Ax", "Bow"];

pub fn re_rand_jobs_build_attr() -> BasicMenuItemAttribute {
    let sequence = GameUserData::get_sequence();
    if (sequence != 3 && sequence != 2) || DVCVariables::ClassMode.get_value() != 1 {
        BasicMenuItemAttribute::Hide
    }
    else if !DVCVariables::is_main_chapter_complete(3) { BasicMenuItemAttribute::Enable }
    else if UnitPool::class().get_static_fields_mut::<UnitPoolStaticFieldsMut>().s_unit.iter().filter(|v| can_re_rand_job_check(v)).count() > 0 {
        BasicMenuItemAttribute::Enable
    }
    else { BasicMenuItemAttribute::Hide }
}
fn can_re_rand_job_check(unit: &Unit) -> bool {
    (unit.check_status(UnitStatusField::Summon) && unit.person.summon_rank < 2) ||
        (unit.force.is_some_and(|f| f.force_type == 1 || f.force_type == 2) && unit.person.asset_force == 0)
}
pub fn rerandomize_jobs() -> Vec<String> {
    let mut out = vec![];
    let m003 = DVCVariables::is_main_chapter_complete(3);
    UnitPool::class().get_static_fields_mut::<UnitPoolStaticFieldsMut>().s_unit
        .iter_mut()
        .filter(|unit| unit.force.is_some_and(|f| f.force_type < 3) &&  unit.person.asset_force == 0 && (!m003 || can_re_rand_job_check(unit)))
        .for_each(|unit|{
            unit_change_to_random_class(unit, false);
            crate::autolevel::auto_level_unit_for_random_map(unit, false);
            person::unit::adjust_unit_items(unit);
            let mut spr = String::new();
            for x in 1..5 {
                if unit.selected_weapon_mask.value & (1 << x) != 0 {
                    spr += Mess::create_sprite_tag_str(2, WEP_KEY[x-1]).to_string().as_str();
                }
            }
            out.push(format!("{}: {} {}", unit.get_name(), Mess::get_name(unit.job.jid), spr));
            unit.auto_equip();
            unit.reload_actor();
        });
    out
}

fn unit_random_can_reclass(job: &JobData, is_female: bool, high_class: bool, player: bool, emblem: bool) -> bool {
    if !DVCFlags::CustomClass.get_value() { if !JOB_HASH.iter().any(|&hash| hash == job.parent.hash ) { return false;} }
    let job_flags = job.flag.value;
    if job_flags & 32 != 0 { return false; }
    let jid = job.jid.to_string();
    if let Some(pos) = crate::assets::animation::MONSTERS.iter().position(|&j| j == jid) { 
        if !DVCVariables::is_main_chapter_complete(11) || emblem { return false; }
        if player { 
            if !dlc_check() { return pos == 5 || pos == 6 ; } // Wyrms Only
            else { return pos != 4 && pos != 7; }  // No Fell Dragons
        }  
        return  pos == 2 || pos == 3 || pos == 5 || pos == 6 ;  // Wolves and Wyrms
    }
    if jid.contains("_紋章士_") { return false; }     // Prevent Emblem Classes

    if job_flags == 0 || ( player && job_flags & 3 == 0  ) || (job_flags & 16 != 0 && is_female ) || (job_flags & 4 != 0 && !is_female) { return false; }    // Wrong Gender / Player Can't reclass
    if high_class && (job.is_low() && job.max_level == 20 ) || ( !high_class && job.is_high() ) { return false; }   // Wrong Job Tier

    if jid == "JID_村人" { return false; }  // No Villager
    if jid == "JID_マージカノン" { return GameVariableManager::get_bool("G_CC_マージカノン"); } // FX Checks for Mage Cannoner / Enchanter
    if jid == "JID_エンチャント" { return GameVariableManager::get_bool("G_CC_エンチャント"); }
    true
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
    if job.is_high() { (1, level, internal) }
    else if job.max_level == 40 { (2, level, internal) }
    else { (0, level, internal) }
}

pub fn unit_change_to_random_class(unit: &mut Unit, change_level: bool) {
    let old_data = if change_level { get_old_person_data(unit) }
    else {
        let current_tier = if unit.job.is_high() { 1 } else if unit.job.max_level >= 40 { 2 } else { 0 };
        (current_tier, unit.level as i32, unit.internal_level as i32)
    };
    let is_high = old_data.0 == 1 || (old_data.0 == 2 && old_data.1 > 15);
    let rng = Random::get_game();
    let mode = DVCVariables::ClassMode.get_value();
    let unit_level = old_data.1;
    let internal_level = if old_data.0 == 1 && old_data.2 == 0 { 20 } else { old_data.2 };
    let mut is_female = if unit.edit.is_enabled() { unit.edit.gender == 2 } else { unit.person.get_dress_gender() == Gender::Female };
    let lockout = mode >= 3;
    if let Some(class) = DVCVariables::get_single_class(!is_high, is_female).filter(|p| DVCFlags::SingleJobEnabled.get_value() && !lockout){
        if class.parent.hash != unit.job.parent.hash {
            let level = unit.person.get_level() as i32;
            let internal = unit.person.get_internal_level() as i32;
            unit.class_change(class);
            if class.max_level > 20 {
                if (level + internal_level) <= 40 {
                    unit.level = level as u8 + internal as u8;
                    unit.internal_level = 0;
                }
                else {
                    unit.level = 21;
                    unit.internal_level = (level + internal_level) as i8 - 21;
                }
            }
            else {
                unit.level = level as u8;
                unit.internal_level = internal as i8;
            }
        }
        else { return; }
    }
    else{
        if unit.person.flag.value & 32 != 0 { is_female = !is_female };   // Reverse gender
        let mut class_list: Vec<_> = vec![];
        let mut new_class = None;
        if lockout {
            let current_classes = lockout::get_all_playable_unit_classes(unit.person);
            class_list = JobData::get_list().unwrap().iter().filter(|&job| !(lockout && current_classes.contains(&job.parent.hash))).collect();
            new_class = class_list.get_remove_filter(rng, |j| unit_random_can_reclass(j, is_female, is_high, true, false));
            if new_class.is_none() {
                new_class = class_list.get_remove_filter(rng, |j| unit_random_can_reclass(j, is_female, !is_high, true, false));
            }
            if new_class.is_none() {
                if let Some(v) = JobData::get("JID_村人") {
                    unit.class_change(v);
                    unit.set_hp(unit.get_capability(0, true));
                    randomize_selected_weapon_mask(unit, None);
                    skill::learn::unit_update_learn_skill(unit);
                    return;
                }
            }
        }
        else {
            class_list = JobData::get_list().unwrap().iter().filter(|&job| unit_random_can_reclass(job, is_female, is_high, true, false)).collect();
            new_class = class_list.get_remove_filter(rng, |j| unit_random_can_reclass(j, is_female, is_high, true, false));
        }
        if let Some(new_class) = new_class.filter(|s| s.parent.hash != unit.job.parent.hash){ unit.class_change(new_class); }
        else { return; }

        match old_data.0 {
            2 => { //Special
                if unit.job.is_high() {
                    if unit_level > 25 {
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
                else if unit.job.max_level == 40 {
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
                if unit.job.is_high() {
                    unit.set_level(unit_level);
                    unit.set_internal_level(internal_level);
                } else if unit.job.max_level == 40 {
                    let total = unit_level + internal_level;
                    let new_level = if total > 40 { 40 } else { total };
                    unit.set_level(new_level);
                    unit.set_internal_level(0);
                } else {
                    let total = unit_level + internal_level;
                    let new_level = if total > 20 { 20 } else { total };
                    unit.set_level(new_level);
                    unit.set_internal_level(0);
                }
            }
            _ => {  //Base
                if unit.job.is_high() {
                    if unit_level > 20 {
                        unit.set_level(unit_level - 20);
                        unit.set_internal_level(20);
                    } else {
                        unit.set_level(1);
                        unit.set_internal_level(unit_level - 1);
                    }
                } else {
                    unit.set_level(unit_level);
                    unit.set_internal_level(internal_level);
                }
            }
        }
    }
    unit.set_hp(unit.get_capability(0, true));  // fix HP
    randomize_selected_weapon_mask(unit, None);
    if change_level {
        unit.set_weapon_mask_from_parson();
        fixed_unit_weapon_mask(unit);
        let mut new_opt =
            if unit.job.get_weapon_mask2().value & (1 << 9) != 0 { 1 << ( rng.get_value(8) + 1 ) }
            else if unit.selected_weapon_mask.value == 0 { unit.weapon_mask.value }
            else { unit.selected_weapon_mask.value };

        if rng.get_value(20) < 5 {
            new_opt |= 1 << ( rng.get_value(8) + 1 );
            if rng.get_value(50) < 5 {
                new_opt |= 1 << ( rng.get_value(8) + 1 );
            }
        }
        unit.original_aptitude.value = new_opt;
        unit.aptitude.value |= new_opt;
    }
    adaptive_growths(unit, true);
    skill::learn::unit_update_learn_skill(unit);
}
pub fn enemy_unit_change_to_random_class(unit: &mut Unit) -> bool {
    let current_job = unit.get_job();
    let current_job_name = current_job.name.to_string();
    if current_job_name == "MJID_Emblem" { return false; }
    if unit.person.get_gender() > 2 || unit.person.get_gender() == 0 { return false; }  // Monster Class
    let rng = Random::get_game();

    let mut is_female = 
        if ( unit.person.pid.to_string() == PIDS[0] || unit.person.flag.value & 128 != 0 ) &&
            GameVariableManager::exist(DVCVariables::LUEUR_GENDER) { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 }
        else if unit.edit.is_enabled() { unit.edit.gender == 2 }
        else { unit.person.get_gender() == 2 };
    if unit.person.flag.value & 32 != 0 { is_female = !is_female };   // Reverse gender

    let is_high = if current_job.is_low() { unit.level > 20 }
        else { current_job.is_high() };
    let rank = if current_job.is_low() { if current_job.max_level > 20 { 2 } else { 0 }} else { 1 };
    let is_flying = unit.get_job().move_type == 3;
    let unit_level = unit.level as i32;
    let internal_level = unit.internal_level as i32;
    let has_emblem = unit.get_god_unit().is_some() || ( GameUserData::get_chapter().cid.to_string() != "CID_M011" );

    let class_list: Vec<_> = JobData::get_list().unwrap().iter()
        .filter(|&job| unit_random_can_reclass(job, is_female, is_high, false, has_emblem) )
        .collect();

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
    if new_job.move_type != 3  {
        if is_flying {
            unit.private_skill.add_sid("SID_天駆_飛行", SkillDataCategorys::Private, 0);
            unit.base_capability[10] = -4;
        }
        else { unit.base_capability[10] = 0; }
    }
    else { unit.base_capability[10] = 0; }
    unit.set_hp(unit.get_capability(0, true));
    fixed_unit_weapon_mask(unit);
    randomize_selected_weapon_mask(unit, None);
    skill::learn::unit_update_learn_skill(unit);
    true
}

pub fn is_magic_class(job: &JobData) -> bool {
    let mut weapon_type = 0;
    let mut weapon_level = 0;
    for x in 1..9 {
        if job.weapons[x as usize] == 1 {
            if weapon_level < job.get_max_weapon_level(x) {
                weapon_type = x;
                weapon_level = job.get_max_weapon_level(x);
            }
        }
    }
    weapon_type == 6
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
    GameData::get_item_pool().weapon_db.get_random_weapon(None, weapon_type, -1, true)
}

pub fn correct_job_base_stats() {
    let job_list = JobData::get_list_mut().unwrap();
    job_list.iter_mut()
        .filter(|job| job.is_low())
        .for_each(|job| {
            if job.flag.value & 3 == 1 {
                let hash = job.parent.hash;
                if let Some(gender) = PersonData::get_list().unwrap().iter().find(|x| x.get_job().is_some_and(|x| x.parent.hash == hash)).map(|p| p.gender) {
                    if gender == 1 {
                        job.flag.value ^= !4;
                        job.flag.value |= 16;

                    }
                    else if gender == 2 {
                        job.flag.value ^= !16;
                        job.flag.value |= 4;
                    }
                }
            }
            if job.get_base()[10] == 0 { job.flag.value = 0; }
        });
    ["JID_神竜ノ子", "JID_邪竜ノ子", "JID_神竜ノ王"].iter().for_each(|jid|{
        if let Some(job) = JobData::get_mut("JID_邪竜ノ子"){
            job.flag.value = 3;
        }
    });
    ["JID_邪竜", "JID_不明", "JID_邪竜ノ王", "JID_M000_邪竜ノ王"].iter().for_each(|jid|
        if let Some(job) = JobData::get_mut(jid) { job.flag.value = 0; }
    );
    ["JID_フロラージュ下級", "JID_フロラージュ", "JID_フロラージュ_E", "JID_リンドブルム下級", "JID_リンドブルム", "JID_リンドブルム_E",
        "JID_スレイプニル下級", "JID_スレイプニル", "JID_スレイプニル_E", "JID_ピッチフォーク下級", "JID_ピッチフォーク", "JID_ピッチフォーク_E",
        "JID_メリュジーヌ_味方", "JID_メリュジーヌ", "JID_裏邪竜ノ娘", "JID_邪竜ノ娘", "JID_邪竜ノ娘_敵"].iter()
        .for_each(|jid|
            if let Some(job) = JobData::get_mut(jid) { job.flag.value |= 7; }
        );

    ["JID_アヴニール下級", "JID_アヴニール", "JID_アヴニール_E", "JID_スュクセサール下級", "JID_スュクセサール",
        "JID_スュクセサール_E", "JID_ティラユール下級", "JID_ティラユール", "JID_ティラユール_E", "JID_クピードー下級",
        "JID_クピードー", "JID_クピードー_E", "JID_裏邪竜ノ子", "JID_ダンサー"
    ].iter()
        .for_each(|jid|
            if let Some(job) = JobData::get_mut(jid) {
                job.flag.value |= 19;
                job.flag.value &= !4;
            }
        );
    JobData::get_list_mut().unwrap().iter_mut()
        .filter(|job| job.jid.str_contains("JID_紋章士_") && job.parent.index > 0)
        .for_each(|emblem_job| { emblem_job.flag.value = 0; });
}
pub fn adjust_missing_weapon_mask() {
    if let Some(start) = UnitPool::get_first(9, 0) {
        let mut weapon_select_count = 0;
        start.job.get_selectable_weapon_mask(&mut weapon_select_count);
        let selected = start.selected_weapon_mask.value;
        let count = (1..10).into_iter().filter(|x| (1 << x) & selected != 0).count() as i32;
        if count < weapon_select_count { randomize_selected_weapon_mask(start, None); }
        let mut unit = start;
        while let Some(unit1) = UnitFor::get_next_by_force(unit, 9) {
            unit1.job.get_selectable_weapon_mask(&mut weapon_select_count);
            let selected = unit1.selected_weapon_mask.value;
            let count = (1..10).into_iter().filter(|x| (1 << x) & selected != 0).count() as i32;
            if count < weapon_select_count { 
                randomize_selected_weapon_mask(unit1, None);
                println!("{} corrected weapon mask", unit1.get_name());
            }
            unit = unit1;
        }
    }
}
pub fn randomize_selected_weapon_mask(unit: &mut Unit, with_kind: Option<i32>) {
    let job = unit.get_job();
    unit.selected_weapon_mask.value = 0;
    let mut selectable_weapons = 0;
    let mut selected = 0;
    let mut possible_kinds = vec![];
    for x in 1..10 {
        let v = job.weapons[x];
        if v > 1 {
            if with_kind == Some(x as i32) {
                selected += 1;
                unit.selected_weapon_mask.value |= 1 << x;
            }
            else { possible_kinds.push(x); }
            selectable_weapons = if v != 2 {  2 } else { 1 };
        }
    }
    while selected < selectable_weapons {
        if let Some(kind) = possible_kinds.get_remove(Random::get_game()) {
            unit.selected_weapon_mask.value |= 1 << kind;
            selected += 1;
        }
        else { break; }
    }
    unit.update_weapon_mask();
}
fn can_change_job_to_list(this: &ClassChangeJobData, unit: &Unit, cc_type: i32) -> bool {
    if !this.cc_check(unit) { return false; }
    let job = unit.get_job();
    if !can_reclass(this.job) { return false; }
    if (this.proof_type as i32) == 0 && cc_type == 0 { true }
    else if (this.proof_type as i32) > 0 && cc_type > 0 {
        if this.job.is_low() && job.is_high() { this.job.max_level == 40  }
        else { true }
    }
    else { false }
}

pub fn can_reclass(job: &JobData) -> bool {
    let vanilla_class = JOB_HASH.iter().any(|x| *x == job.parent.hash);
    (vanilla_class && job.flag.value & 32 == 0) || (!vanilla_class && DVCFlags::CustomClass.get_value() )
}