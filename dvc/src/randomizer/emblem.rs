use unity::prelude::*;
use engage::{
    unit::{Unit, UnitPool},force::{ForceType, *},
    god::*, gamedata::skill::SkillData, stream::Stream,
};
use crate::{utils::*, ironman::vtable_edit};
use super::*;

pub mod engrave;
pub mod menu;
pub static ENEMY_EMBLEM_LIST: OnceLock<Vec<i32>> = OnceLock::new();

pub fn initialize_emblem_list() {
    ENEMY_EMBLEM_LIST.get_or_init(||{
        let mut list: Vec<i32> = Vec::new();
        for x in 0..20 {
            GodData::get(format!("GID_相手{}", EMBLEM_ASSET[x])).map(|god| list.push(god.parent.index));
        }
        GodData::get_list().unwrap().iter()
            .filter(|god| {
                let gid = god.gid.to_string();
                !EMBLEM_ASSET.iter().any(|asset| gid.contains(asset)) && gid.contains("GID_相手")
            }).for_each(|god| list.push(god.parent.index));
        list
    });
    println!("Number of Enemy Emblems: {}", ENEMY_EMBLEM_LIST.get().unwrap().len());
}
pub fn randomize_emblems() {
    if !DVCVariables::random_enabled() { DVCVariables::create_recruitment_variables(true); }
    if DVCVariables::is_recruitment_set(true) {
        set_m022_emblem_assets();
    }
    else {
        let rng = get_rng();
        let no_dlc = !dlc_check() || DVCConfig::get().dlc & 2 != 0;
        let emblem_recruitment = DVCVariables::EmblemRecruitment.get_value();
        DVCVariables::create_recruitment_variables(true);
        match emblem_recruitment {
            1 => {
                let custom_emblems = DVCFlags::CustomEmblemsRecruit.get_value();
                let mut list = GameData::get_playable_emblem_hashes();
                if list.len() > 20 {
                    if custom_emblems { list.remove(19); }
                    else { list.drain(19..); }
                }
                else { list.remove(19); }
                if no_dlc { list.drain(12..19); }
                let gen_list: Vec<(i32, i32)> = list.iter()
                    .zip(list.iter().flat_map(|x| GodData::try_get_hash(*x)).map(|g| g.female + 1))
                    .map(|x| (*x.0, x.1))
                    .collect::<Vec<_>>();
                    
                let mut available = gen_list.clone();
                gen_list.iter()
                    .for_each(|&(hash, gender)| {
                        let god = GodData::try_get_hash(hash).unwrap();
                        let key = format!("G_R_{}", god.gid);
                        if let Some(v) = available.get_remove_filter(rng, |x| x.1 != gender)
                            .or_else(|| available.get_remove(rng))
                            .and_then(|h| GodData::try_get_hash(h.0)) 
                        {
                            let key2 = format!("G_R2_{}", v.gid);
                            DVCVariables::set_variable_key_string(key.as_str(), v.gid);
                            DVCVariables::set_variable_key_string(key2.as_str(), god.gid);
                            println!("{} -> {}", Mess::get(god.mid), Mess::get(v.mid));
                        } else { println!("No Emblem Swaps for {}", Mess::get(god.mid)); }
                    });
            },
            2 => {  // Reverse
                for i in 0..12 { DVCVariables::set_emblem_recruitment(i, 11 - i); }
            },
            3 => {  // Custom
                let order = DVCConfig::get().get_custom_recruitment(true);
                let emblems = GameData::get_playable_god_list();
                let list_size = emblems.len();
                order.iter().for_each(|&x| {
                    let idx = x.0 as usize;
                    let new_idx = x.1 as usize;
                    if idx < list_size && new_idx < list_size {
                        DVCVariables::set_variable_key_string(format!( "G_R_{}",emblems[idx].gid), emblems[new_idx].gid);
                        DVCVariables::set_variable_key_string(format!("G_R2_{}",emblems[new_idx].gid), emblems[idx].gid);
                    }
                });
            },
            _ => { DVCVariables::create_recruitment_variables(true); },
        }
        set_m022_emblem_assets();
    }
}
pub fn set_m022_emblem_assets() {
    for x in 1..12 {
        if let Some(person) = PersonData::get_mut(format!("PID_M022_紋章士_{}", EMBLEM_ASSET[x])) {
            let replacement_gid = GameVariableManager::get_string(&format!("G_R_GID_{}", EMBLEM_ASSET[x])).to_string();
            if let Some(index) = EMBLEM_GIDS.iter().position(|&gid| gid == replacement_gid) {
                let jid = format!("JID_紋章士_{}", EMBLEM_ASSET[index]);
                if let Some(job) = JobData::get(&jid){
                    let gender = if job.unit_icon_id_m.is_some() { 1 }  else { 2 };
                    person.gender = gender;
                    person.name = Some( format!("MGID_{}", RINGS[index]).into());
                    person.jid = Some( jid.into());
                }
            }
            else {
                if let Some(god) = GodData::get(replacement_gid) {
                    person.name = Some(god.mid.clone());
                    person.gender = if god.female == 2 { 2 } else { 1 };
                }
            }
        }
    }
}
pub fn update_lueur_bonds() {
    if DVCVariables::is_main_chapter_complete(22) {
        if let Some(g_unit) = GodPool::try_get_gid("GID_リュール", false) {
            Force::get(ForceType::Absent).unwrap().iter().chain(  Force::get(ForceType::Player).unwrap().iter() )
                .for_each(|unit|{
                    if let Some(g_bond) = g_unit.get_bond(unit){
                        if g_bond.level < 20 {
                            g_bond.set_level(20);
                            unit.inherit_apt(g_unit);
                        }
                    }
                });
        }
    }
}
pub fn get_engage_attack_type(skill: Option<&SkillData>) -> i32 {
    if let Some(engage_type) = skill
        .map(|s| s.sid.to_string())
        .and_then(|engage_str|
            EMBLEM_ASSET.iter().position(|sid|  engage_str.contains(sid))
        ){
        match engage_type {
            0|2|4|5|6|11|12|16|18|19|20|21 => { 0 }, //AI_AT_EngageAttack
            1 => { 1 }, //AI_AT_EngagePierce
            3 => { 9 }, //AI_AT_Versus
            7 => { 2 }, // AI_AT_EngageVision
            8 => { 10 }, // AI_AT_EngageWait
            9 => { 3 }, // AI_AT_EngageDance
            10 => { 4 }, // AI_AT_EngageOverlap
            13 => { 5 }, // AI_AT_EngageBless
            14 => { 6 }, // AI_AT_EngageWaitGaze
            15 => { 7 }, // AI_AT_EngageSummon
            17 => { 8 }, // AI_AT_EngageCamilla
            _ => { -1 },    // None
        }
    }
    else { -1 }
}

#[unity::hook("App", "GodBondHolder", "Get")]
pub fn god_bond_holder_get(this: &GodBondHolder, mut unit: Option<&mut Unit>, method_info: OptionalMethod) -> Option<&'static mut GodBond> {
    if this.data.is_some_and(|f| f.force_type == 1 && !f.gid.str_contains("M0")) { call_original!(this, unit, method_info) }
    else if DVCVariables::is_random_map() || DVCVariables::Continuous.get_value() == 1 {
        let bond: Option<&'static mut GodBond> = call_original!(this, UnitPool::get_hero(false), method_info);
        if let Some((bond, unit)) = bond.as_ref().zip(unit.as_mut()) {
            unit.aptitude.value |= bond.level_data.aptitude.value;
        }
        bond
    }
    else if unit.as_ref().is_some_and(|unit|{
        let pid = unit.person.pid.to_string();
        PIDS.contains(&pid.as_str()) || pid.contains("E00")
    }) { call_original!(this, unit, method_info) }
    else {
        let bond: Option<&'static mut GodBond> = call_original!(this, UnitPool::get_hero(false), method_info);
        if let Some((bond, unit)) = bond.as_ref().zip(unit.as_mut()) {
            unit.aptitude.value |= bond.level_data.aptitude.value;
        }
        bond
    }
}
pub fn god_unit_on_serialize(this: &mut GodUnit, stream: &Stream, _method_info: OptionalMethod){
    check_fix_god_bonds(this);
    this.on_serialize(stream);
}
pub fn god_unit_on_deserialize(this: &mut GodUnit, stream: &Stream, version: i32, _method_info: OptionalMethod) {
    this.on_deserialize(stream, version);
    check_fix_god_bonds(this);
}
fn check_fix_god_bonds(this: &mut GodUnit) {
    let hash = this.data.main_data.parent.hash;
    if let Some(bond) = this.get_god_bonds() {
        if bond.data.is_some_and(|g| g.main_data.parent.hash != hash && g.force_type == 0) {
            if hash == 2044088482 || hash == 1120993642 { bond.data = GodData::try_get_hash(1120993642); }
            else { bond.data = GodData::try_get_hash(hash); }
        }
        else if bond.bonds.is_none(){
            println!("GodBonds are missing. Deleting GodBondHolder.");
            this.delete();
            if let Some(god) = GodData::try_get_hash(hash) {
                println!("GodUnit::Build for {}", Mess::get(god.mid));
                this.build(god);
                set_bond_levels(this);
            }
        }
    }
    else {
        println!("GodBondHolder is missing.");
        if let Some(god) = GodData::try_get_hash(hash){
            println!("GodUnit::Build for {}", Mess::get(god.mid));
            this.build(god);
            set_bond_levels(this);
        }
    }
}
fn set_bond_levels(this: &mut GodUnit) {
    if this.data.force_type != 0 { return; }
    let level = if DVCVariables::is_main_chapter_complete(22) {
        if let Some(lock) = this.data.main_data.unlock_level_cap_flag {
            if GameVariableManager::get_bool(lock) { 20 }
            else { 10 }
        }
        else { 20 }
    }
    else if DVCVariables::is_main_chapter_complete(20) {
        if let Some(lock) = this.data.main_data.unlock_level_cap_flag {
            if GameVariableManager::get_bool(lock) { 15 } else { 10 }
        }
        else { 15 }
    }
    else if DVCVariables::is_main_chapter_complete(5) { 10 } else { 5 };
    PIDS.iter().for_each(|pid| {
        if let Some(unit) = UnitPool::get_from_pid(pid.into(), false) {
            if let Some(g_bond) = this.get_bond(unit) {
                g_bond.set_level(level);
                unit.inherit_apt(this);
            }
        }
    });
}
pub fn god_pool() {
    if let Some(klass) = Il2CppClass::from_name("App", "GodUnit").ok() {
        vtable_edit(klass, "OnDeserialize", god_unit_on_deserialize as _);
        vtable_edit(klass, "OnSerialize",  god_unit_on_serialize as _);
    }
}