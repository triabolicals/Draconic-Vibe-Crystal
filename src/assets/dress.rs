// use bitflags::Flags;
use engage::gameuserdata::GameUserData;
use accessory::*;
use conditions::*;
use transform::{has_enemy_tiki, MONSTER_PERSONS};
use crate::randomizer::names::get_emblem_person;
use crate::randomizer::person::ENEMY_PERSONS;
use super::*;

const UNIQUE: &[&str] = &["001", "051", "002", "052", "049", "099", "100", "150", "200", "201", "351", "350", "400", "403", "450", "553", "551"];
pub const IDS: [i32; 43] = [0, 001, 051,
    500, 550, 501, 100, 101, 152, 150, 102, 153, 200, 203, 250, 201, 251, 252, 350, 301,
    302, 303, 351, 352, 450, 453, 452, 400, 401, 402, 552, 103, 253, 403, 304, 254, 551,
    502, 099, 049, 553, 503, 554
];

pub fn commit_for_unit_dress(
    result: &mut AssetTableResult,
    mode: i32,
    unit: &mut Unit,
    equipped: Option<&ItemData>,
    conditions: &mut Array<&'static Il2CppString>)
    -> ConditionFlags
{
    // Ignoring Emblem Classes and More No Engage Accessories
    remove_condition("AID_NoEngage");
    remove_condition("AID_NoEngage2");
    remove_condition("AID_NoEngage3");
    let sf = AssetTableStaticFields::get();
    let mut flags = ConditionFlags::get_from_conditions(conditions);
    if m022_god_dress(result, mode, conditions) || random_emblem_name_asset_switch(result, unit, mode, equipped, conditions) {
        flags.set(ConditionFlags::AllyDarkEmblem, true);
        return flags;  
    }
    if unit.status.value & 0x8000000 != 0 { flags.set(ConditionFlags::Vision, true); }
    if unit.status.value & 0x800000 != 0 { flags.set(ConditionFlags::Engaged, true); }
    let condition_unit = 
        if flags.contains(ConditionFlags::Vision) {  UnitUtil::get_vision_owner(unit).unwrap_or(unit) } // Switch to conditioned unit if Doubles
        else { &unit  };

    // Boss Randomization / Past Alear Edit / Veyle Edit
    set_gender_conditions(condition_unit, &mut flags);
    let pid = condition_unit.person.pid.to_string();
    if unit.person.get_asset_force() != 0 && unit.person.get_job().is_some_and(|j| j.jid.str_contains("JID_邪竜ノ子")) {
        if let Some(lueur_replacement) =  PersonData::get(DVCVariables::get_dvc_person(0, false))
            .filter(|p| p.parent.index > 1)
        {
            remove_condition("女装");
            remove_condition("男装");
            remove_condition(condition_unit.person.pid);
            remove_condition("MPID_Lueur");
            remove_condition(condition_unit.person.name.unwrap());
            if lueur_replacement.gender == 1 && lueur_replacement.get_flag().value & 32 == 0 {
                condition_unit.edit.set_gender(1);
                condition_unit.person.set_gender(1);
                conditions::add_condition("男装");
            }
            else {
                condition_unit.person.set_gender(2);
                condition_unit.edit.set_gender(2);
                conditions::add_condition("女装");
            }
            conditions::add_condition(lueur_replacement.name.unwrap());
            condition_unit.edit.set_name(Mess::get_name(lueur_replacement.name.unwrap()));

            flags.set(ConditionFlags::Generic, false);
        }
    }
    else if pid.contains("ヴェイル") && pid != PIDS[32] {
        if let Some(veyle_replacement) = PersonData::get(DVCVariables::get_dvc_person(32, false))
            .filter(|p| p.pid.to_string() != PIDS[32])
        {
            remove_condition(condition_unit.person.pid);
            remove_condition(condition_unit.person.name.unwrap());
            if veyle_replacement.parent.index == 1 {
                conditions::add_condition("JID_邪竜ノ子");
            }
            remove_condition("女装");
            remove_condition("男装");
            conditions::add_condition(veyle_replacement.name.unwrap());
            if veyle_replacement.gender == 1 && veyle_replacement.get_flag().value & 32 == 0 {
                conditions::add_condition("男装");
            }
            else { conditions::add_condition("女装"); }
            flags.set(ConditionFlags::Generic, false);
        }
    }
    else if GameVariableManager::get_bool(DVCVariables::RANDOM_BOSS_KEY) && condition_unit.person.get_flag().value & 2048 != 0 {
        if let Some(new_person) = ENEMY_PERSONS.get()
            .and_then(|v| v.iter().find(|x| x.1 == unit.person.parent.index && x.0 >= 150))
            .and_then(|p| crate::randomizer::names::get_new_npc_person(p.0 as usize - 150))
        {
            if !GameVariableManager::exist("BossCount") { GameVariableManager::make_entry("BossCount", 0); }
            let new_name = new_person.name.unwrap().to_string().replace("XPID", "MPID");
            if let Some(name) = unit.person.name {
                let old_name = name.to_string().replace("XPID", "MPID");
                let key = format!("Ch_{}", unit.person.pid);
                if !GameVariableManager::exist(&key) {
                    let count = GameVariableManager::get_number("BossCount");
                    GameVariableManager::make_entry_str(format!("Old_{}", unit.person.pid).as_str(), old_name.as_str());
                    GameVariableManager::make_entry_str(format!("Ch_{}", unit.person.pid).as_str(), new_name.as_str());
                    GameVariableManager::make_entry_str(format!("DVC_Boss{}", count).as_str(), unit.person.pid);
                    GameVariableManager::set_number("BossCount", count + 1);
                }
                condition_unit.edit.set_name(Mess::get(new_name.as_str()));
                condition_unit.edit.set_gender(new_person.gender);
                remove_condition(condition_unit.person.pid);
                remove_condition(old_name.as_str());
                conditions::add_condition(new_name);
                conditions::add_condition(new_person.pid);
                remove_condition("女装");
                remove_condition("男装");
                if new_person.gender == 1 { conditions::add_condition("男装"); }
                else { conditions::add_condition("女装"); }
                flags.set(ConditionFlags::Generic, false);
            }
        }
    }
    else if GameVariableManager::exist( format!("Old_{}", unit.person.pid).as_str()) {
        condition_unit.edit.set_gender(0);
    }

    if condition_unit.person.parent.index > 1 && condition_unit.person.get_flag().value & 128 == 0 {
        remove_condition("MPID_Lueur");
    }
    if condition_unit.person.get_flag().value & 1024 != 0 && !pid.contains("ヴェイル") {
        conditions::add_condition("MPID_Lueur");
    }
    if flags.contains(ConditionFlags::Transform) && mode == 1 {
        if has_enemy_tiki(unit) {
            if flags.contains(ConditionFlags::EngageAttack) {
                result.body_model = "oBody_Tik0AF_c567".into();
                result.hair_model = "oHair_h567".into();
                emblem::engage_animation_mode_1(result, emblem::get_emblem_attack_index(unit) as i32, 2);
                result.scale_stuff[16] = 2.6;

            }
            else { result.setup_for_person(1, PersonData::get("PID_E001_Boss"), conditions); }
            return flags;
        }
        let jid = condition_unit.job.jid.to_string();
        if let Some(pos) = MONSTERS.iter().position(|&x| x == jid){
            result.setup_for_person_job_item(1, PersonData::get(MONSTER_PERSONS[pos]), Some(condition_unit.job), None, conditions);
        }
        else if let Some(pos) = EMBLEM_ASSET.iter().position(|&x| jid.contains(x)) {
            if pos != 19 && pos != 23 {
                let pid = concat_string!("PID_闘技場_", EMBLEM_ASSET[pos]);
                result.setup_for_person(1, PersonData::get(pid), conditions);
            }
        }
        else if jid == "JID_裏邪竜ノ子" {
            result.setup_for_person_job_item(1, PersonData::get("PID_ラファール_竜化"), Some(condition_unit.job), None, conditions);
        }
        else if jid == "JID_裏邪竜ノ娘" {
            result.setup_for_person_job_item(1, PersonData::get("PID_エル_竜化"), Some(condition_unit.job), None, conditions);
        }
        return flags;
    }
    if flags.contains(ConditionFlags::Engaged) || unit.status.value & 8388608 != 0 {
        if condition_unit.god_link.is_some() { conditions::add_god_unit_engage_conditions(condition_unit.god_link.unwrap(), &mut flags);  }
        else if condition_unit.get_god_unit().is_some(){ conditions::add_god_unit_engage_conditions(condition_unit.god_unit.unwrap(), &mut flags);}
        if unit.person.parent.hash == 258677212 { remove_condition("チキ"); }
    }
    // Accessories Conditions
    if condition_unit.person.get_asset_force() == 0 {
        let outfit = get_unit_outfit_mode(condition_unit);
        if outfit & 8 != 0 && (condition_unit.status.value & 0x800000 != 0) && !flags.contains(ConditionFlags::Tiki) {
            if let Some(god) = condition_unit.god_link.or(condition_unit.get_god_unit()) {
                result.setup_for_god(mode, Some(god.data), false, conditions);
                remove_condition("女装");
                remove_condition("男装");
                if god.data.female == 1 {
                    flags.set(ConditionFlags::Female, true);
                    flags.set(ConditionFlags::Male, false);
                }
                else {
                    flags.set(ConditionFlags::Male, true);
                    flags.set(ConditionFlags::Female, false);
                }
                return flags;
            }
        }
        if GameUserData::get_sequence() == 4 {
            if (outfit & 1 != 0) && flags.contains(ConditionFlags::UnitInfo) &&
                condition_unit.accessory_list.unit_accessory_array[0].index > 0
            {
                AssetTable::add_condition_key("私服");
                flags.set(ConditionFlags::CausalClothes, true);
            }
        }
        else { flags.set(ConditionFlags::CausalClothes, false); }
        if flags.contains(ConditionFlags::TikiEngage) || flags.contains(ConditionFlags::Transform) {
            for x in 0..*ACCESSORY_COUNT.get().unwrap() {
                remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[x as usize] );
            }
        }
        else if flags.contains(ConditionFlags::Hub) && GameUserData::get_sequence() == 4 {
            if *ACCESSORY_COUNT.get().unwrap() > 5 {
                remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[5] );
            }
        }
        else if outfit & 3 == 0 {   // Default
            for x in 0..*ACCESSORY_COUNT.get().unwrap() {
                remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[x as usize] );
            }
            flags.set(ConditionFlags::CausalClothes, false);
        }
        else if outfit & 3 == 1 {   // Somniel
            if condition_unit.accessory_list.unit_accessory_array[0].index > 0 {
                if AccessoryData::try_index_get(condition_unit.accessory_list.unit_accessory_array[0].index)
                    .map(|data| data.can_equip(condition_unit)).unwrap_or(false)
                {
                    AssetTable::add_condition_key("私服");
                    flags.set(ConditionFlags::CausalClothes, true);
                }
            }
            if *ACCESSORY_COUNT.get().unwrap() > 5 {
                remove_accessory_conditions(condition_unit.accessory_list.unit_accessory_array[5] );
            }
        }
        else if outfit & 3 == 2 {
            if condition_unit.accessory_list.unit_accessory_array[0].index > 0 {
                remove_accessory_conditions(condition_unit.accessory_list.unit_accessory_array[0]);
            }
            if *ACCESSORY_COUNT.get().unwrap() > 5 {
                if condition_unit.accessory_list.unit_accessory_array[5].index > 0 && AccessoryData::try_index_get(condition_unit.accessory_list.unit_accessory_array[5].index)
                    .map(|data| data.can_equip(condition_unit)).unwrap_or(false)
                {
                    flags.set(ConditionFlags::CausalClothes, true);
                    AssetTable::add_condition_key("私服");
                }
            }
        }
        if condition_unit.status.value & 8388608 != 0 && (outfit & 4 != 0 ) && !flags.contains(ConditionFlags::Tiki) {
            if condition_unit.god_link.is_some() { conditions::remove_god_unit_engage_conditions(condition_unit.god_link.unwrap(), &mut flags);  }
            else if condition_unit.get_god_unit().is_some(){ conditions::remove_god_unit_engage_conditions(condition_unit.god_unit.unwrap(), &mut flags);}
            flags.set(ConditionFlags::Engaged, false);
        }
    }
    else { AssetTable::add_condition_key("私服"); }

    // Vision Assets
    if flags.contains(ConditionFlags::Vision) || unit.status.value & 0x8000000 != 0 {
        if ( condition_unit.person.gender == 0 || condition_unit.person.gender > 2 ) || condition_unit.person.get_bmap_size() > 1  {
            result.setup_for_person_job_item(mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, conditions);
        }
        else { 
            result.commit( mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
            illusion_double_dress(result, condition_unit, mode, equipped, conditions, &mut flags);
        }
        return flags;
    }

    // Check if Unit will Transform and adjust conditions
    if condition_unit.person.gender > 0 && transform::is_monster_class( condition_unit ) {
        remove_condition(unit.job.jid);
        sf.condition_flags.add_by_key(unit.person.get_job().unwrap().jid);
        result.commit(mode, Some(condition_unit.person), condition_unit.person.get_job(), equipped);
        flags = ConditionFlags::get_from_conditions(conditions);
        flags.set(ConditionFlags::Transforming, true);
    }

    else if ( equipped.is_some_and(|item| item.iid.str_contains("チキ")) && !flags.contains(ConditionFlags::Engaged)) ||
        transform::has_enemy_tiki( condition_unit )
    {
        result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        flags = ConditionFlags::get_from_conditions(conditions);
        flags.set(ConditionFlags::Transforming, true);
    }
    else { 
        result.commit( mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        flags.set(ConditionFlags::Transforming, false);
    }
    if flags.contains(ConditionFlags::Transforming) { remove_mounts_accs(result);  }
    if !result.body_model.is_null() { if result.body_model.str_contains("AT") { return flags; } }
    if !result.dress_model.is_null() { if result.dress_model.str_contains("AT") { return flags; }  }

    if mode == 2 {
        if !flags.contains(ConditionFlags::Generic) {
            if let Some(v) = result.accessory_list.list.
                iter_mut().find(|acc| acc.model.is_some_and(|f| f.str_contains("uAcc_spine2_HelmAmr0A")))
            {
                v.model = Some("null".into());
            }
        }
        add_personal_outfit_accessory(result, unit, &mut flags);
    };
    adjust_dress( result, mode, unit, &flags);
    // Unit Dress / Head Adjustment
    SEARCH_LIST.get().unwrap().adjust_head(result, condition_unit);
    hair_adjustment(result);

    // Generic Appearance Randomization
    if flags.contains(ConditionFlags::Generic) && !flags.contains(ConditionFlags::TikiEngage) {

        // Generic Appearance
        let generic_mode =  GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
        if generic_mode & 1 != 0 && mode == 2 {  SEARCH_LIST.get().unwrap().random_head(result, condition_unit, flags, true); }
        if generic_mode & 2 != 0 { change_result_colors_by_unit(unit, result); }
        if GameVariableManager::get_number("G_RandAsset") > 1 && unit.person.gender != 0 && unit.person.get_bmap_size() == 1 {
            SEARCH_LIST.get().unwrap().random_aoc(condition_unit, result, flags, 2);
        }
    }
    flags
}

pub fn pad_number(v: i32) -> String {
    if v < 10 { format!("00{}", v) }
    else if v < 100 { format!("0{}", v)}
    else { format!("{}", v) }
}

pub fn add_personal_outfit_accessory(result: &mut AssetTableResult, unit: &Unit, flags: &mut ConditionFlags) {
    if flags.contains(ConditionFlags::TransformedMask) { return; }
    if !result.body_model.is_null() { if result.body_model.str_contains("AT") { return; } }
    if !result.dress_model.is_null() { if result.dress_model.str_contains("AT") { return; }  }
    let outfit_mode = get_unit_outfit_mode(unit);
    let acc_index = unit.accessory_list.unit_accessory_array[0].index;
    if acc_index > 42 || acc_index <= 0 { return; }
    if flags.contains(ConditionFlags::Hub) || (outfit_mode & 1 == 1 && acc_index > 0) {
        let dress_gender = unit_dress_gender(unit);
        if AccessoryData::try_index_get(acc_index).map(|d| d.can_equip(unit)).unwrap_or(false) {
            flags.set(ConditionFlags::CausalClothes, true);
            match acc_index {
                1 => {  if dress_gender == 1 { result.dress_model = "uBody_WearM_c001".into(); } }
                2 => {  if dress_gender == 2 { result.dress_model = "uBody_WearF_c051".into(); } }
                3..43 => {
                    let id = IDS[ acc_index as usize ];
                    if ( id == 303 || (id % 100) >= 50 ) && dress_gender == 2 {
                        result.dress_model =  format!("uBody_WearF_c{}", pad_number(id)).into();
                    }
                    else if ( id != 303 && (id % 100) < 50 ) && dress_gender == 1 {
                        result.dress_model = format!("uBody_WearM_c{}", pad_number(id)).into();
                    }
                }
                _ => {}
            };
        }
    }
}
pub fn adjust_dress(result: &mut AssetTableResult, mode: i32, unit: &Unit, conditions: &ConditionFlags) {
    let search = SEARCH_LIST.get().expect("Search List not initialized yet");
    let dress_gender = unit_dress_gender(unit);
    let changed = search.get_random_appearance(result, mode, unit, conditions);
    let is_engaged = unit.status.value & 0x800000 != 0;
    if is_engaged  {
        if unit.person.parent.hash == 258677212 {   // Changing Chapter 2 Lumera's engage outfit to whatever Sigurd is replaced with
            if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                let god = DVCVariables::get_god_from_index(1, true).unwrap();
                let pos = crate::randomizer::emblem::EMBLEM_LIST.get().unwrap().iter().position(|&hash| hash == god.parent.hash).unwrap();
                search.get_engage_outfit(result, mode, pos as i32, Gender::Female, true, conditions);
                if pos == 13 {  // Add Transformation Animation for Tiki Engage
                    change_result_colors_by_unit(unit, result);
                    result.body_anims.add( Il2CppString::new_static("End0AF-No2_c099_N"));
                }
            }
        }
        else if !conditions.contains(ConditionFlags::TikiEngage) {
            let gen = if dress_gender == 1 { Gender::Male } else { Gender::Female };
            if let Some(god) = unit.god_link.or(unit.god_unit).map(|g| g.data.main_data) {
                if let Some(pos) = crate::randomizer::emblem::EMBLEM_LIST.get().unwrap().iter().position(|&hash| hash == god.parent.hash) {
                    search.get_engage_outfit(
                        result,
                        mode,
                        pos as i32,
                        if dress_gender == 1 { Gender::Male } else { Gender::Female},
                      true,
                        conditions
                    );
                }
                if changed {
                    let ty = if conditions.contains(ConditionFlags::Engaged) { god.parent.hash } else { 20 + god.parent.index };
                    search.get_random_job_dress(result, ty, mode, unit, conditions);
                }
            }
            // When the spine accessory is replaced with engaged accessory and unit does not have a head, regain the spine accessory (Helm for Armors)
            if result.head_model.str_contains("null") {
                if let Some(spine_accessory) = search.job.iter()
                    .find(|w| w.job_hash == unit.job.parent.hash && w.mode ==2)
                    .and_then(|job| job.get_acc(gen, mode, "c_spine2_jnt"))
                { result.accessory_list.try_add(spine_accessory); }
            }
            else { set_accessories_generic(result, unit.person.aid, dress_gender); }
        }
        eve_sforgia_correction(result, mode);
        if conditions.contains(ConditionFlags::TikiEngage) { return; }
    }
    if (is_engaged && !conditions.contains(ConditionFlags::Engaged)) || !is_engaged {
        let random_mode = GameVariableManager::get_number(DVCVariables::RANDOM_CLASS_OUTFITS);
        let gender = if dress_gender == 1 { Gender::Male } else { Gender::Female };
        if !is_engaged { search.set_job_mount_dress(result, unit.job, gender, mode, conditions); }
        if !changed && unit.force.is_some_and(|f| f.force_type != 0 && f.force_type < 3) && unit.person.get_asset_force() != 0 {
            if search.get_default_battle_outfits(result, mode, unit) { return; }
        }
        if !conditions.contains(ConditionFlags::CausalClothes) {
            if !changed && (DVCVariables::is_changed_recruitment_order(false) || GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0) {
                let stating_job_hash = GameVariableManager::get_number(format!("G_JG_{}", unit.person.pid));
                if let Some(default_job) = JobData::try_get_hash(stating_job_hash) {
                    if default_job.parent.hash == unit.job.parent.hash { search.get_default_battle_outfits(result, mode, unit); } else {
                        if let Some(v) = SEARCH_LIST.get().unwrap().personal_outfits.get(unit.person.name.unwrap().to_string().as_str())
                            .and_then(|v| v.split("_c").nth(1))
                        {
                            let str: String = if mode == 1 { result.body_model } else { result.dress_model }.to_string();
                            if !UNIQUE.iter().any(|unique| str.contains(unique)) {
                                if mode == 1 { result.body_model = str.replace(v, "000").into(); } else { result.dress_model = str.replace(v, "000").into(); }
                            }
                        }
                    }
                }
            }
            else if random_mode != 0 {
                search.get_random_job_dress(result, random_mode + 10 * changed as i32, mode, unit, conditions);
            }
        }
        if is_sword_fighter_outfit(result) && !unit.job.jid.str_contains("JID_ソードファイター") {
            search.set_job_dress(result, unit.job, gender, mode, &conditions);
            if is_sword_fighter_outfit(result) { search.get_default_battle_outfits(result, mode, unit); }
        }
    }

    eve_sforgia_correction(result, mode);
    let ident = unit.ident as u32;
    search.check_dress_body(result, Some(unit), ident, mode, conditions.contains(ConditionFlags::Female));
    if conditions.intersects(ConditionFlags::DismountMask) || is_engaged || conditions.contains(ConditionFlags::Engaging) {
        remove_mounts_accs(result);
    }
}

pub fn illusion_double_dress(
    result: &mut AssetTableResult,
    owner: &Unit,
    mode: i32,
    equipped: Option<&ItemData>,
    conditions: &Array<&Il2CppString>,
    flags: &mut ConditionFlags)
{
    if is_tiki_engage(result) || flags.contains(ConditionFlags::TikiEngage) {
        SEARCH_LIST.get().unwrap().replace_with_god(result, mode, 13, false);
        vision_swd_animations(result, Gender::Female, mode);
        return;
    }
    let dress_gender = unit_dress_gender(owner);
    if dress_gender & 3 != 0 && owner.person.get_bmap_size() == 1 && owner.person.parent.index > 0 {
        let gen = if dress_gender == 1 { Gender::Male } else { Gender::Female };
        if flags.contains(ConditionFlags::Generic) {
            let generic_mode = GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
            if generic_mode & 2 != 0 { change_result_colors_by_unit(owner, result); }
        }
        else { adjust_dress(result, mode, owner, &flags); }
        add_personal_outfit_accessory(result, owner, flags);
        vision_swd_animations(result, gen, mode);
    }
    else { result.setup_for_person_job_item(mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, conditions); }
    if GameVariableManager::get_number("G_RandAsset") > 1 && owner.person.gender != 0 && owner.person.get_bmap_size() == 1 {  
        SEARCH_LIST.get().unwrap().random_aoc(owner, result, *flags, 3);
    }
    if is_sword_fighter_outfit(result) &&  dress_gender & 3 != 0 {
        SEARCH_LIST.get().unwrap().set_job_dress(
            result,
            owner.job,
            if dress_gender == 1 { Gender::Male} else { Gender::Female},
            mode,
            &flags
        );
    }
    eve_sforgia_correction(result, mode);
    SEARCH_LIST.get().unwrap().check_dress_body(result, Some(owner), owner.ident as u32, mode, flags.contains(ConditionFlags::Female));
    remove_mounts_accs(result);
}

pub fn change_result_colors_by_unit(unit: &Unit, result: &mut AssetTableResult) {
    let value = if unit.person.get_asset_force() == 0 {
        (unit.person.parent.hash as u32 >> 2) +
            (DVCVariables::get_seed() as u32 >> 2) +
            ( unit.job.parent.hash as u32 >> 2)
    }
    else { unit.grow_seed as u32 };
    let index: [usize; 6] = [0, 1, 4, 5, 6, 7];
    let rng = Random::instantiate().unwrap();
    rng.ctor(value);
    for x in index {
        let value2 = rng.value();
        result.unity_colors[x].r = ( value2 & 255 ) as f32 / 255.0;
        result.unity_colors[x].g = (( value2 >> 4 ) & 255 ) as f32 / 255.0;
        result.unity_colors[x].b = (( value2 >> 8 ) & 255 ) as f32 / 255.0;
    }
}

fn m022_god_dress(result: &mut AssetTableResult, mode: i32, conditions: &mut Array<&'static Il2CppString>) -> bool {
    if result.pid.is_null() { return false; }
    let pid = result.pid.to_string();
    if pid.contains("M022_紋章士") && GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
        if let Some(god) = EMBLEM_ASSET.iter().position(|x| pid.contains(x))
            .and_then(|x| DVCVariables::get_god_from_index(x as i32, true))
        {
            emblem::asset_table_result_god_setup(result, mode+10, Some(god), true, conditions, None);
            return true;
        }
    }
    false
}
fn random_emblem_name_asset_switch(
    result: &mut AssetTableResult,
    condition_unit: &Unit,
    mode: i32,
    equipped: Option<&ItemData>,
    conditions: &mut Array<&'static Il2CppString>
) -> bool {
    let pid =  condition_unit.person.pid.to_string();
    if PIDS.iter().any(|x| pid == *x) || condition_unit.person.name.is_none() || condition_unit.person.get_flag().value & 1536 != 0 { return false; }
    let name = condition_unit.person.name.unwrap().to_string();

    if GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) && !pid.contains(PIDS[0]) && !pid.contains("M022")
    {
        if condition_unit.person.get_summon_rank() > 0 || condition_unit.force.is_some_and(|f| f.force_type == 1) {
            let mid = name.replace("MPID", "MGID");
            if let Some(person) = get_emblem_person(mid.into()) {
                result.setup_for_person_job_item(mode, Some(person), person.get_job(), equipped, conditions);
                condition_unit.edit.set_name(Mess::get_name(person.pid));
                condition_unit.edit.set_gender(person.gender);
                clear_accessory_from_list(result.accessory_list, "Eff_EmblemAura");
                return true;
            }
        }
    }
    condition_unit.edit.set_gender(0);
    false
}

fn hair_adjustment(result: &mut AssetTableResult) {
    if !result.hair_model.is_null() {
        if !result.hair_model.contains("null") {
            if result.accessory_list.list.iter()
                .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("Hair"))) {
                result.hair_model = "uHair_null".into();
            }
        }
    }
}
fn eve_sforgia_correction(result: &mut AssetTableResult, mode: i32) {
    if mode != 1 { return; }
    if !result.body_model.is_null() {
        let body = result.body_model.to_string();
        if body.contains("c451") { result.body_model = body.replace("c451", "c000").into(); }
        if body.contains("c151") { result.body_model = body.replace("c151", "c000").into(); }
    }
}
