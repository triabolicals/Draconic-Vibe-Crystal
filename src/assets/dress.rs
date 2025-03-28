use accessory::*;
use conditions::*;
use transform::{has_enemy_tiki, MONSTER_PERSONS};
// use crate::assets::data::HEAD_DATA;

use super::*;

pub const IDS: [i32; 40] = [500, 550, 501, 100, 101, 152, 150, 102, 153, 200, 203, 250, 201, 251, 252, 350, 301, 302, 303, 351, 352, 450, 453, 452, 400, 401, 402, 552, 103, 253, 403, 304, 254, 551, 502, 099, 049, 553, 503, 554];


pub fn commit_for_unit_dress(result: &mut AssetTableResult, mode: i32, unit: &mut Unit, equipped: Option<&ItemData>, conditions: &mut Array<&'static Il2CppString>) -> ConditionFlags {
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
    if unit.status.value & 0x8000000 != 0 { flags.set(ConditionFlags::Vision, true); }  // Vision Unit
    let condition_unit = 
        if flags.contains(ConditionFlags::Vision) {  UnitUtil::get_vision_owner(unit).unwrap_or(unit) } // Switch to conditioned unit if Doubles
        else { &unit  };
    // Vision 
    // Main Character Gender Swap for Characters that switch gender clothes (Rosado)
    conditions::set_gender_conditions(condition_unit, &mut flags);
    if flags.contains(ConditionFlags::Transform) && mode == 1 {
        println!("{} is transforming.", Mess::get_name(condition_unit.person.pid));
        if has_enemy_tiki(unit) {
            if flags.contains(ConditionFlags::EngageAttack) {
                result.body_model = "oBody_Tik0AF_c567".into();
                result.hair_model = "oHair_h567".into();
                emblem::engage_animation_mode_1(result, emblem::get_emblem_attack_index(unit) as i32, 2);
                result.scale_stuff[16] = 2.6;

            }
            else {result.setup_for_person(1, PersonData::get("PID_E001_Boss"), conditions); }
            return flags;
        }

        let jid = condition_unit.job.jid.to_string();
        if let Some(pos) = MONSTERS.iter().position(|&x| x == jid){ result.setup_for_person_job_item(1, PersonData::get(MONSTER_PERSONS[pos]), Some(condition_unit.job), None, conditions); }
        else if let Some(pos) = EMBLEM_ASSET.iter().position(|&x| jid.contains(x)) {
            if pos != 19 && pos != 23 {
                let pid = concat_string!("PID_闘技場_", EMBLEM_ASSET[pos]);
                result.setup_for_person(1, PersonData::get(pid), conditions);
            }
        }
        else if jid == "JID_裏邪竜ノ子" { result.setup_for_person_job_item(1, PersonData::get("PID_ラファール_竜化"), Some(condition_unit.job), None, conditions);  }
        else if jid == "JID_裏邪竜ノ娘" { result.setup_for_person_job_item(1, PersonData::get("PID_エル_竜化"), Some(condition_unit.job), None, conditions); }
        return flags;
    }
    if flags.contains(ConditionFlags::Engaged){
        if condition_unit.god_link.is_some() { conditions::add_god_unit_engage_conditions(condition_unit.god_link.unwrap(), &mut flags);  }
        else if condition_unit.get_god_unit().is_some(){ conditions::add_god_unit_engage_conditions(condition_unit.god_unit.unwrap(), &mut flags);}
    }
    // Accessories Conditions
    if condition_unit.person.get_asset_force() == 0 {
        let outfit = get_unit_outfit_mode(condition_unit);
        if flags.contains(ConditionFlags::TikiEngage) || flags.contains(ConditionFlags::Transform) {
            for x in 0..*ACCESSORY_COUNT.get().unwrap() { remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[x as usize] ); }
        }
        else if flags.contains(ConditionFlags::Hub) {
            if *ACCESSORY_COUNT.get().unwrap() > 5 { remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[5] ); }
            AssetTable::add_condition_key("私服");
        }
        else if outfit == 0 {
            for x in 0..*ACCESSORY_COUNT.get().unwrap() { remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[x as usize] ); }
        }
        else if outfit == 1 {
            if condition_unit.accessory_list.unit_accessory_array[0].index > 0 {  AssetTable::add_condition_key("私服"); }
            if *ACCESSORY_COUNT.get().unwrap() > 5 { remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[5] ); }
        }
        else if condition_unit.accessory_list.unit_accessory_array[0].index > 0 {  remove_accessory_conditions( condition_unit.accessory_list.unit_accessory_array[0] ); }
    }
    else { AssetTable::add_condition_key("私服");  }

    // Vision Assets
    if flags.contains(ConditionFlags::Vision) {
        if ( condition_unit.person.gender == 0 || condition_unit.person.gender > 2 ) || condition_unit.person.get_bmap_size() > 1  {
            result.setup_for_person_job_item(mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, conditions);
        }
        else { 
            result.commit( mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
            illusion_double_dress(result, condition_unit, mode, equipped, conditions, flags);
        }

        return flags;
    }
    // Check if Unit will Transform and adjust conditions
    if condition_unit.person.gender > 0 && ( transform::is_emblem_class( condition_unit) || transform::is_monster_class( condition_unit ) ) {
        remove_condition(unit.job.jid);
        sf.condition_flags.add_by_key(unit.person.get_job().unwrap().jid);
        result.commit(mode, Some(condition_unit.person), condition_unit.person.get_job(), equipped);
        flags = ConditionFlags::get_from_conditions(conditions);
        flags.set(ConditionFlags::Transforming, true);
        // println!("{} is Monster tranforming ", Mess::get_name(condition_unit.person.pid));
    }

    else if ( equipped.is_some_and(|item| item.iid.to_string().contains("チキ")) && !flags.contains(ConditionFlags::Engaged) ) || transform::has_enemy_tiki( condition_unit ) {
        result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        flags = ConditionFlags::get_from_conditions(conditions);
        flags.set(ConditionFlags::Transforming, true);
        // println!("{} is Tiki tranforming ", Mess::get_name(condition_unit.person.pid));
    }
    else { 
        result.commit( mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        flags.set(ConditionFlags::Transforming, false);
    }
    if flags.contains(ConditionFlags::Transforming) { remove_mounts_accs(result);  }

    if mode == 2 {  dress::add_personal_outfit_accessory(result, unit, flags); }

    // Unit Dress / Head Adjustment 
    dress::adjust_dress( result, mode, unit, flags);

    SEARCH_LIST.get().unwrap().adjust_head(result, condition_unit);
    // Generic Appearance Randomization
    if flags.contains(ConditionFlags::Generic) && !flags.contains(ConditionFlags::TikiEngage){  // Generic Appearance
        let generic_mode =  GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
        // let can_accessorize = crate::randomizer::RANDOMIZER_STATUS.read().unwrap().accessory;
        if generic_mode & 1 == 1 && mode == 2 {  SEARCH_LIST.get().unwrap().random_head(result, condition_unit, flags, true); }
            //data::HEAD_DATA.get().unwrap().replace_by_rng(unit, result);  }
        if generic_mode & 2 == 2 { change_hair_change(unit, result); }
    }
    // Info Animation Randomization
    if GameVariableManager::get_number("G_RandAsset") > 1 && unit.person.gender != 0 && unit.person.get_bmap_size() == 1 {  
         SEARCH_LIST.get().unwrap().random_aoc(condition_unit, result, flags); 
    }
    flags
}

pub fn pad_number(v: i32) -> String {
    if v < 10 { format!("00{}", v) }
    else if v < 100 { format!("0{}", v)}
    else { format!("{}", v) }
}

pub fn add_personal_outfit_accessory(result: &mut AssetTableResult, unit: &Unit, flags: ConditionFlags) {
    if flags.contains(ConditionFlags::TransformedMask) { return; }
    if !result.body_model.is_null() { if result.body_model.to_string().contains("AT") { return; } }
    if !result.dress_model.is_null() { if result.dress_model.to_string().contains("AT") { return; }  }
    let outfit_mode = get_unit_outfit_mode(unit);

    if flags.contains(ConditionFlags::Hub) || flags.contains(ConditionFlags::CausalClothes) || outfit_mode == 1 {
        let acc_index = unit.accessory_list.unit_accessory_array[0].index;
        if acc_index > 42 || acc_index <= 0 { return; }
        let dress_gender = unit_dress_gender(unit);
        match acc_index {
            1 => {  if dress_gender == 1 { result.dress_model = "uBody_WearM_c001".into(); } }
            2 => {  if dress_gender == 2 { result.dress_model = "uBody_WearF_c051".into(); } }
            3..43 => {
                let id = IDS[ (acc_index - 3) as usize ];
                if ( id == 303 || (id % 100) >= 50 ) && dress_gender == 2 { 
                    result.dress_model =  format!("uBody_WearF_c{}", pad_number(id)).into();
                }
                else if ( id != 303 || (id % 100) < 50 ) && dress_gender == 1 {  
                    result.dress_model = format!("uBody_WearM_c{}", pad_number(id)).into();
                }
            }
            _ => {}
        };
    }
}

pub fn adjust_dress(result: &mut AssetTableResult, mode: i32, unit: &Unit, conditions: ConditionFlags) {
    let dress_gender = unit_dress_gender(unit);
    if conditions.contains(ConditionFlags::Engaged) {
        /* 
        if let Some(link) = unit.god_link {
            println!("{} is Link Engaged with {}.", Mess::get_name(unit.person.pid), Mess::get(link.data.mid));
        }
        if let Some(gunit) = unit.god_unit {
            println!("{} is Engaged with {}.", Mess::get_name(unit.person.pid), Mess::get(gunit.data.mid));
        }
        */
        if unit.person.parent.hash == 258677212 {   //Lumera Chapter 2
            if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                let god = DVCVariables::get_god_from_index(1, true).unwrap();
                let pos = crate::randomizer::emblem::EMBLEM_LIST.get().unwrap().iter().position(|&hash| hash == god.parent.hash).unwrap();
                SEARCH_LIST.get().unwrap().get_engage_outfit(result, mode, pos as i32, Gender::Female, true, conditions);
                if pos == 13 { 
                    change_hair_change(unit, result);  
                    result.body_anims.add( Il2CppString::new_static("End0AF-No2_c099_N")); 
                }
            }
        }
        else {
            if mode == 2 && !conditions.contains(ConditionFlags::TikiEngage) {
                // println!("Dress: {}", result.dress_model);
                let gen = if dress_gender == 1 { Gender::Male } else { Gender::Female };
                let search_data = SEARCH_LIST.get().unwrap();
                if result.head_model.to_string().contains("null") {
                    let _ = search_data.job.iter().find(|w| w.job_hash == unit.job.parent.hash && w.mode == 2 )
                        .map(|w|{
                            if let Some(acc) = w.get_acc(gen, mode, "c_spine2_jnt") {
                                result.accessory_list.try_add(acc);
                            }
                        }
                    );
                }
                else { set_accessories_generic(result, unit.person.aid, dress_gender); }
            }
        }
    }
    else {
        // println!("Searching for Job: {}", Mess::get_name(unit.job.jid));
        SEARCH_LIST.get().unwrap().set_job_dress(result, unit.job, if dress_gender == 1 { Gender::Male} else { Gender::Female}, mode, conditions);
    }
    if conditions.intersects(ConditionFlags::DismountMask) {
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();
        result.ride_anim = None;
    }
}

pub fn illusion_double_dress(result: &mut AssetTableResult, owner: &Unit, mode: i32, equipped: Option<&ItemData>, conditions: &Array<&Il2CppString>, flags: ConditionFlags) {
    if is_tiki_engage(result) || flags.contains(ConditionFlags::TikiEngage) {
        SEARCH_LIST.get().unwrap().replace_with_god(result, mode, 13, false);
        animation::vision_swd_animations(result, Gender::Female, mode);
        return;
    }
    let dress_gender = unit_dress_gender(owner);
    if ( dress_gender == 1 || dress_gender == 2 ) && owner.person.get_bmap_size() == 1 {
        let gen = if dress_gender == 1 { Gender::Male } else { Gender::Female };
        if flags.contains(ConditionFlags::TikiEngage) {
            SEARCH_LIST.get().unwrap().replace_with_god(result, mode, 13, false);  // Replace Assets with Tiki
        }
        else if owner.person.get_flag().value & 2048 != 0 {
            let generic_mode =  GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
            // if generic_mode & 1 == 1 && mode == 2{  HEAD_DATA.get().unwrap().replace_by_rng(owner, result); } 
            if generic_mode & 2 == 2 { change_result_colors_by_unit(owner, result); }
        }
        add_personal_outfit_accessory(result, owner, flags);
        animation::vision_swd_animations(result, gen, mode);
    }
    else { result.setup_for_person_job_item(mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, conditions); }
    if GameVariableManager::get_number("G_RandAsset") > 1 && owner.person.gender != 0 && owner.person.get_bmap_size() == 1 {  
        SEARCH_LIST.get().unwrap().random_aoc(owner, result, flags); 
    }
}

pub fn change_result_colors_by_unit(unit: &Unit, result: &mut AssetTableResult) {
    let value = unit.grow_seed;
    let index: [usize; 6] = [0, 1, 4, 5, 6, 7];
    let rng = Random::instantiate().unwrap();
    rng.ctor(value as u32);
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
        if let Some(emblem_index) = EMBLEM_ASSET.iter().position(|x| pid.contains(x)) {
            let god = DVCVariables::get_god_from_index(emblem_index as i32, true);
            if god.is_some() {
                emblem::asset_table_result_god_setup(result, mode+10, god, true, conditions, None);
                return true;
            }
        }
    }
    false
}
fn random_emblem_name_asset_switch(result: &mut AssetTableResult, condition_unit: &Unit, mode: i32, equipped: Option<&ItemData>, conditions: &mut Array<&'static Il2CppString>) -> bool {
    let pid =  condition_unit.person.pid.to_string();
    if GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) && !pid.contains(PIDS[0]) && !pid.contains("M022") {
        if let Some(emblem_index) = EMBLEM_ASSET.iter().position(|x| pid.contains(x) ){
            let index = if emblem_index > 18 { emblem_index + 1 } else { emblem_index };
            if unsafe { EMBLEM_NAMES[index] } != -1 {
                let emblem_person = PersonData::get(PIDS[ unsafe { EMBLEM_NAMES[index] } as usize ]);
                result.setup_for_person_job_item(mode, emblem_person, emblem_person.unwrap().get_job(), equipped, conditions);
                emblem_person.unwrap().set_name(MPIDS[ unsafe { EMBLEM_NAMES[index] } as usize ].into());
                accessory::clear_accessory_from_list(result.accessory_list, "Eff_EmblemAura");
                return  true;
            }
        }
    }
    false
}
