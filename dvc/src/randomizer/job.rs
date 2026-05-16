use engage::{
    unit::*, force::*,
    menu::{menus::class_change::ClassChangeJobData, BasicMenuItemAttribute},
};
use super::*;
use crate::{
    randomizer::{
        person::unit::fixed_unit_weapon_mask, grow::adaptive_growths, job::reclass::ReclassType
    }
};

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
pub const MONSTER_CLASS: [i32; 8] = [-1116401495, 1156629411, -1116399479, 1156631427, -290620106, 704942064, -1274089237, 746034087];
pub const VILLAGER: i32 = 499211320;
pub const ENCHANTER: i32 = -631685449;
pub const MAGE_CANNON: i32 = 1398652429;
pub const LUEUR_CLASS: [i32; 3] = [-1369632991, -1369626630, 185671037];
pub const ENEMY_ONLY: [i32; 5] = [1367578960, 689554073, 1463244270, 2021853286, 1445776173];

pub const FEMALE_CLASS: [i32; 18] = [
    -1165634998, 1133576057, 1848240617,
    748907755, -1955176032, 5510888,
    -910371360, 1977191031, 1982073595,
    7727229, -1536732910, -1406772370,
    1499787884, 185670709, -1998645787,
    30291080, -1028496122, 229381663,
];
pub const MALE_CLASS: [i32; 15] = [
    730392094, -121798307, -149285011,
    -362221162, 511515477, -1241492235,
    1766556981, -1517502902, 352107958,
    -1706110952, 1462565615, -974713853,
    185677222, 1942863689, 30291392,
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
    if (sequence != 3 && sequence != 2) || DVCVariables::ClassMode.get_value() != 1 { BasicMenuItemAttribute::Hide }
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
            reclass::unit_reclass(unit, ReclassType::Recruitment(true));
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
    LUEUR_CLASS.iter().flat_map(|j| JobData::try_get_hash_mut(*j))
        .for_each(|job|{ job.flag.value == 3; });
    ENEMY_ONLY.iter().chain(MONSTER_CLASS.iter()).flat_map(|j| JobData::try_get_hash_mut(*j))
        .for_each(|job|{ job.flag.value == 0; });
    FEMALE_CLASS.iter().flat_map(|j| JobData::try_get_hash_mut(*j))
        .for_each(|job|{ job.flag.value |= 7; });
    MALE_CLASS.iter().flat_map(|j| JobData::try_get_hash_mut(*j))
        .for_each(|job|{ job.flag.value |= 19; });
    /*
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
    */
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
            }
            unit = unit1;
        }
    }
}
pub fn assign_selected_weapon_mask_by_apt(unit: &mut Unit, with_kind: Option<i32>) {
    let job = unit.get_job();
    let n_selects = job.weapons.iter().max().map(|x| (*x)-1).unwrap_or(0);
    if n_selects == 0 { return; }
    let mut possible_kinds: Vec<_> = job.weapons.iter().enumerate()
        .filter(|(k, v)| **v > 1 && *k > 0).map(|(kind, _)| (kind) as i32).collect();
    let mut new_mask = 0;
    let mut count = 0;
    if let Some(kind) = with_kind.filter(|k| possible_kinds.contains(k)){
        new_mask |= 1 << kind;
        count += 1;
    }
    if count < n_selects {
        for apt in [unit.person.aptitude.value, unit.person.sub_aptitude.value] {
            if count == n_selects { break; }
            if let Some(kind) = possible_kinds.iter().find(|v| apt & (1 << *v) != 0 && new_mask & (1 << *v) == 0) {
                new_mask |= 1 << *kind;
                count += 1;
            }
        }
    }
    if count == n_selects { unit.selected_weapon_mask.value = new_mask; }
    else { randomize_selected_weapon_mask(unit, with_kind); }
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