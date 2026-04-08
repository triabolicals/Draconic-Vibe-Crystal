use unity::prelude::*;
use super::*;
use engage::{
    unit::Unit,
    god::{GodPool, GodUnit, GodBondHolder},
    force::{ForceType, *},
    gamedata::skill::SkillData,
    gameuserdata::GameUserData,
};
use engage::dialog::yesno::TwoChoiceDialogMethods;
use engage::god::GodBond;
use engage::stream::Stream;
use engage::unit::UnitPool;
use crate::ironman::vtable_edit;
use super::person::pid_to_index;
use crate::utils::*;

pub mod engrave;
pub mod menu;
pub static ENEMY_EMBLEM_LIST: OnceLock<Vec<i32>> = OnceLock::new();
pub static RECOMMENED_LVL: OnceLock<Vec<u8>> = OnceLock::new();

pub fn initialize_emblem_list() {
    RECOMMENED_LVL.get_or_init(||{
        let mut list: Vec<u8> = Vec::new();
        for x in 0..12 {
            let cid = format!("CID_{}", EMBELM_PARA[x]);
            list.push(ChapterData::get(&cid).unwrap().recommended_level);
        }
        list
    });
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

pub fn emblem_gmap_spot_adjust(){
    if GameUserData::get_sequence() != 6 || !DVCVariables::random_enabled() { return; }
    let edelgard_obtain = GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み");
    if edelgard_obtain  {
        for x in 1..7 {
            let gmap_flag = format!("G_GmapSpot_G00{}", x);
            let flag_value = GameVariableManager::get_number(gmap_flag.as_str());
            if flag_value == 1 || flag_value == 2 {  GameVariableManager::set_number(gmap_flag.as_str(), 3);  }
        }
    }
    if DVCVariables::EmblemRecruitment.get_value() == 0 || DVCFlags::CustomEmblemsRecruit.get_value() { return; }
    for x in 0..19 {
        let e_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        let cid = EMBELM_PARA[ e_index as usize ];
        let unlock_cid = UNLOCK_PARA[ x as usize]; 
        if cid == "G007" { continue; }  //There's no Edelgard paralogue to unlock
        if unlock_cid == "" {  // open tiki's divine paralogue if edelgard ring is obtained and unlock the emblem paralogue that replaces edelgard
            let gmap_spot_flag = format!("G_GmapSpot_{}", cid);
            if edelgard_obtain {
                if GameVariableManager::get_number("G_GmapSpot_G001") & 3 != 0 { GameVariableManager::set_number("G_GmapSpot_G001", 3); }
                if GameVariableManager::get_number(&gmap_spot_flag) == 1 {  GameVariableManager::set_number(&gmap_spot_flag, 3); }
            }
            else { 
                GameVariableManager::set_number("G_GmapSpot_G001", 1); 
                GameVariableManager::set_number(&gmap_spot_flag, 1); 
            }
            continue;
        }
        if cid.starts_with("G") {
            if edelgard_obtain {
                let gmap_spot_flag = format!("G_GmapSpot_{}", cid);
                if GameVariableManager::get_number(&gmap_spot_flag) != 3 {  GameVariableManager::set_number(&gmap_spot_flag, 3);  }
            }
        }
        else {
            let unlock_flag = format!("G_Cleared_{}", unlock_cid);
            let gmap_spot_flag = format!("G_GmapSpot_{}", cid);
            if GameVariableManager::get_bool(&unlock_flag) {
                if GameVariableManager::get_number(&gmap_spot_flag) <= 2 { GameVariableManager::set_number(&gmap_spot_flag, 3); }
            }
            else { GameVariableManager::set_number(&gmap_spot_flag, 1); }
        }
    }
    //Calculating Recommended Level
    let rec_level = RECOMMENED_LVL.get().unwrap();
    for x in 0..12 {
        let cid_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if let Some(chapter) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize])) {
            if cid_index < 12 { chapter.recommended_level = rec_level[x as usize]; }
            else if let Some(chapter2) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[x as usize])){
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if average >= rec_level[x as usize] { chapter2.recommended_level = rec_level[x as usize]; }
                else {  chapter2.recommended_level = average; }
            }
        }
    }
    for x in 12..19 {
        let cid_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if let Some(chapter) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize])) {
            if cid_index < 12 {
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if average >= rec_level[cid_index as usize] { chapter.recommended_level = rec_level[cid_index as usize]; }
                else { chapter.recommended_level = average; }
            }
        }
    }
}
pub fn randomize_emblems() {
    if !DVCVariables::random_enabled() { DVCVariables::create_recruitment_variables(true); }
    if DVCVariables::is_recruitment_set(true) {
        set_emblem_paralogue_unlock();
        set_m022_emblem_assets();
    }
    else {
        let rng = get_rng();
        let no_dlc = !dlc_check() ||crate::DeploymentConfig::get().dlc & 2 != 0;
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
                let gen_list: Vec<(i32, i32)> = list.iter().zip(list.iter().flat_map(|x| GodData::try_get_hash(*x)).map(|g| g.female + 1))
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
                            if GameVariableManager::exist(key.as_str()) { GameVariableManager::set_string(key.as_str(), v.gid.to_string().as_str()); }
                            else { GameVariableManager::make_entry_str(key.as_str(), v.gid.to_string().as_str()); }
                            println!("{} -> {}", Mess::get(god.mid), Mess::get(v.mid));
                        } else { println!("No Emblem Swaps for {}", Mess::get(god.mid)); }
                    });
            },
            2 => {  // Reverse
                for i in 0..12 { DVCVariables::set_emblem_recruitment(i, 11 - i); }
            },
            3 => {  // Custom
                let order = crate::DeploymentConfig::get().get_custom_recruitment(true);
                order.iter().for_each(|&x| { DVCVariables::set_emblem_recruitment(x.0, x.1); });
            },
            _ => { DVCVariables::create_recruitment_variables(true); },
        }
        set_m022_emblem_assets();
        set_emblem_paralogue_unlock();
    }
}
fn set_emblem_paralogue_unlock() {
    if DVCFlags::CustomEmblemsRecruit.get_value() { return; }
    for x in 0..19 {
        let index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if index >= 0 {
            let string2 = format!("CID_{}",EMBELM_PARA[index as usize]);
            if let Some(emblem_chapter) = ChapterData::get_mut(&string2){
                emblem_chapter.gmap_spot_open_condition = Some(UNLOCK_PARA[x as usize].into());
                if UNLOCK_PARA[index as usize] == "" {
                   // emblem_chapter.
                }
            }
        }
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
                        // println!("Lueur Bond with {} [{}]", unit.get_name(), g_bond.level);
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
pub fn god_bond_holder_get(this: &GodBondHolder, unit: Option<&mut Unit>, method_info: OptionalMethod) -> Option<&'static mut GodBond> {
    if this.data.is_some_and(|f| f.force_type != 0) { call_original!(this, unit, method_info) }
    else {
        if unit.as_ref()
            .is_some_and(|unit|{
                let pid = unit.person.pid.to_string();
                PIDS.contains(&pid.as_str()) || pid.contains("_E00")
            })
        {
            call_original!(this, unit, method_info)
        }
        else { call_original!(this, UnitPool::get_hero(false), method_info) }
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
            if hash == 2044088482 || hash == 1120993642 {   // Chrom or Robin, use Chrom
                bond.data = GodData::try_get_hash(1120993642);
            }
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