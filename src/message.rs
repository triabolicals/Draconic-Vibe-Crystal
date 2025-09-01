use concat_string::concat_string;
use unity::prelude::*;
use engage::{
    gamevariable::*, gameuserdata::*, mess::*,
    gamedata::{cook::CookData, *},
    hub::*,
};
use crate::{
    enums::*,
    randomizer::{emblem::emblem_skill::EIRIKA_INDEX, *},
    utils::*,
};
use std::sync::OnceLock;
use std::collections::HashMap;
use crate::talk::NAMES;
use crate::message::names::get_emblem_person;
use crate::message::emblem::EMBLEM_LIST;
static MID_SWAPS: OnceLock<HashMap<String, (i32, i32)>> = OnceLock::new();
const LUEUR_MIDS: &[&str] = &["MPID_Lueur", "MGID_Lueur", "MID_SELECTRING_LUEUR_NOTES", "MPID_H_Veyre"];

pub fn initialize_mess_hashs() {
    MID_SWAPS.get_or_init(||{
        let mut vec: HashMap<String, (i32, i32)> = HashMap::new();
        for x in 0..LUEUR_MIDS.len() {
            if x < 2 { vec.insert(LUEUR_MIDS[x].to_string(), ( 1, 0) ); }  // Lueur Name Swap
            else { vec.insert(LUEUR_MIDS[x].to_string(), ( 2, x as i32) ); }  // Lueur Name Replacement
        }
        vec.insert("MSID_H_EirikEngage".to_string(), ( 9, 0) );
        vec.insert("MID_RULE_M006_LOSE".to_string(), ( 3, 10) );
        vec.insert("MID_RULE_M015_LOSE".to_string(), ( 3, 27) );
        vec.insert("MID_RULE_M015_WIN".to_string(), ( 3, 27) );
        vec.insert("MID_TUT_NAVI_M015_ESCAPE".to_string(), ( 3, 27) );
        vec.insert("MID_TUT_NAVI_VANDRE_FUNDS".to_string(), ( 3, 1) );
        vec.insert("MID_RULE_M007_WIN".to_string(), ( 3, 26) ); 
        vec.insert("MID_RULE_M008_WIN".to_string(), ( 3, 17) );
        vec.insert("MID_RULE_M009_WIN".to_string(), ( 3, 17) );
        vec.insert("MID_RULE_M014_WIN".to_string(), ( 4, 26) ); //Mauvier Name Swap
        vec.insert("MID_RULE_M016_WIN".to_string(), ( 4, -1) );
        vec.insert("MID_RULE_M017_WIN".to_string(), ( 4, -1) );
        for x in 1..12 {
            vec.insert(format!("MTID_Ring_{}",  RINGS[ x as usize ]), ( 5, x)); // Tile Replacement
            vec.insert(format!("MID_TUT_NAVI_M022_GET_{}",  RINGS[ x as usize ]), ( 6, x)); // "Pick up"
        }
        vec.insert("MID_Hub_Next_Go".to_string(), (8, 0));  // Continious Mode
        vec.insert("MID_Hub_Next_Go1".to_string(), (8, 1));  // Continious Mode
        vec.insert("MPID_Il_E006".to_string(), (10, 0));
        EMBLEM_LIST.get().unwrap().iter().enumerate().map(|x| (x.0, GodData::try_get_hash(*x.1).unwrap()))
            .for_each(|(index, god)|{ vec.insert(god.mid.to_string(), (11, index as i32)); });
        println!("Mess Entries: {}", vec.len());
        vec
    });

}

#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&'static Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let mut result = call_original!(label, is_replaced, method_info);
    if !RANDOMIZER_STATUS.read().unwrap().enabled || !DVCVariables::random_enabled() { return result; }

    if let Some(mess_il2cp) = label {
        let mess_label = mess_il2cp.to_string();
        let replace_to_player_name =
            if mess_label.contains("MPID_") && GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 &&
                DVCVariables::get_dvc_person_data(0, false).is_some_and(|x| x.parent.index > 1)
            {
                let ptr = unsafe { get_u16_ptr(mess_il2cp, None) };
                is_player_name_replace(ptr)
            }
            else { false };

        if mess_label.contains("XPID") {
            let original_label = mess_label.replace("XPID", "MPID");
            if GameVariableManager::get_bool(DVCVariables::RANDOM_BOSS_KEY) {
                if let Some(new_name) = NAMES.read().unwrap().other_names.iter().position(|x| *x == original_label)
                    .and_then(|pos| crate::randomizer::names::get_new_npc_person(pos))
                    .and_then(|person| person.get_name() )
                {
                    let original_label = new_name.to_string().replace("XPID", "MPID");
                    return call_original!(Some(original_label.into()), is_replaced, None);
                }
            }
            else { return call_original!(Some(original_label.into()), is_replaced, None); }
        }
        let hash_map = MID_SWAPS.get().unwrap();
        if let Some(v) = hash_map.get(&mess_label) {
            match v.0 {
                1 => {
                    if GameVariableManager::exist(DVCVariables::LUEUR_NAME) { return GameVariableManager::get_string(DVCVariables::LUEUR_NAME); }
                    else { return result; }
                }
                2 => {  // Alear Name Swap
                    if v.1 == 2 && GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) {   // Emblem Alear Name Swap
                        if let Some(person) = get_emblem_person(mess_il2cp) {
                            let replacement_name = call_original!(person.get_name(), true, None);
                            return replace_string(result, Mess::get_name(DVCVariables::get_dvc_person(0, false)), replacement_name);
                        }
                    }
                    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
                        return replace_string(
                            result,
                            Mess::get_name(DVCVariables::get_dvc_person(0, false)),
                            GameVariableManager::get_string(DVCVariables::LUEUR_NAME)
                        );
                    }
                }
                3 => {  //Enemy Person Name Swap
                    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
                        return replace_string(
                            result,
                            Mess::get(MPIDS[v.1 as usize]),
                            Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[v.1 as usize])))
                        );
                    }
                }
                4 => {  // Mauvier Replacement
                    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
                        if v.1 != -1 { result = name_replace(result, v.1); }
                        result = name_replace(result, 33);
                    }
                }
                5 => {  // Ring/bracelet of XXXX Swaps
                    if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                        let new_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[v.1 as usize].to_string(), false);
                        if new_index >= 0 && new_index < 23 {
                            return call_original!(Some(concat_string!("MGID_Ring_", RINGS[ new_index as usize ]).into()), true, None);
                        }
                        else { return result; }
                    }
                }
                6 => {  // You acquired the Ring of the [XXXXXX].
                    if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                        let mock_text = call_original!(Some("MID_TUT_NAVI_M022_GET_Siglud".into()), is_replaced, method_info);
                        let new_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[v.1 as usize].to_string(), false);
                        if new_index < 23 && new_index >= 0 {
                            let sigurd_text = call_original!(Some("MGID_Ring_Siglud".into()), true, None);
                            let new_emblem = call_original!(Some(concat_string!("MGID_Ring_", RINGS[new_index as usize]).into()), false, None);
                            return replace_string(mock_text, sigurd_text, new_emblem);
                        }
                        else { return result; }
                    }
                }
                8 => {
                    if GameVariableManager::get_number(DVCVariables::CONTINUOUS) != 0 {
                        if let Some(next) = GameUserData::get_chapter().get_next_chapter() {
                            if v.1 == 0 {
                                return
                                    format!("{} ({})",
                                            call_original!(Some(next.name), true, None),
                                            call_original!(Some(format!("{}_PREFIX", next.name).into()), true, None)
                                ).into();
                            }
                            else {
                                return format!("{}: {} ({}) #{}",  
                                    call_original!(Some("MID_Hub_Next_Go".into()), true, None), 
                                    call_original!(Some(next.name), true, None), 
                                    call_original!(Some(format!("{}_PREFIX", next.name).into()), true, None),
                                    crate::continuous::get_continious_total_map_complete_count()
                                ).into();
                            }   
                        }
                    }
                }
                9 => {  // Eirika Engage Skill to match the new emblem
                    let eirika = call_original!(Some("MGID_Eirik".into()), true, None);
                    let new_emblem = call_original!( Some(  GodData::get(  EMBLEM_GIDS[ unsafe { EIRIKA_INDEX } ]).unwrap().mid ), true, None);
                    return replace_string(result, eirika, new_emblem);
                }
                10 => { return name_replace(result, 37); }
                11 => {if let Some(name) = get_emblem_person(mess_il2cp).and_then(|x| x.get_name())
                    {
                        return  call_original!(Some(name), true, None);
                    }
                }
                _ => {}
            }
        }
        if GameVariableManager::exist(DVCVariables::LUEUR_NAME) && replace_to_player_name {
            let lueur_name = GameVariableManager::get_string(DVCVariables::LUEUR_NAME);
            if let Some(name) = DVCVariables::get_dvc_person_data(0, false).and_then(|p| p.get_name())
            {
                let main_char_name = call_original!(Some(name), is_replaced, None);
                return replace_string(result, main_char_name, lueur_name);
            }
        }
        if mess_label.contains("MID_RULE_") && mess_label.contains("WIN") {
            if mess_label.contains("E00") {
                if mess_label.contains("E002") {
                    result = name_replace(result, 4);
                    result = name_replace(result, 7);
                }
                else if mess_label.contains("E003") {
                    result = name_replace(result, 11);
                    result = name_replace(result, 14);
                }
                else if mess_label.contains("E004") {
                    result = name_replace(result, 17);
                    result = name_replace(result, 23);
                }
                else if mess_label.contains("E005") || mess_label.contains("E006")
                { result = name_replace(result, 37); }
                return result;
            }
            let count = GameVariableManager::get_number("BossCount");
            if count > 0 && GameVariableManager::get_bool(DVCVariables::RANDOM_BOSS_KEY) {
                result = name_replace(result, 32); // Veyle
                let mut new_result = result.to_string();
                for x in 0..count {
                    let key = format!("DVC_Boss{}", x);
                    if GameVariableManager::exist(key.as_str()){
                        let old_name = GameVariableManager::get_string(key.as_str());
                        let key2 = format!("Old_{}", old_name);
                        if GameVariableManager::exist(key2.as_str()) {
                            let person = GameVariableManager::get_string(key2.as_str());
                            let boss_name = call_original!(Some(person), true, None).to_string();
                            new_result = new_result.replace(&boss_name, &format!("NAME{}", x));
                        }
                    }
                }
                for x in 0..count {
                    let key = format!("DVC_Boss{}", x);
                    if GameVariableManager::exist(key.as_str()){
                        let old_name = GameVariableManager::get_string(key.as_str());
                        let key2 = format!("Ch_{}", old_name);
                        if GameVariableManager::exist(key2.as_str()) {
                            let new_name = GameVariableManager::get_string(key2.as_str());
                            let new_name = call_original!(Some(new_name), true, None).to_string();
                            new_result = new_result.replace(&format!("NAME{}", x), &new_name);
                        }
                    }
                }

                return new_result.into();
            }
        }
        if mess_label.contains("MID_RULE_DLC") && mess_label.contains("LOSE") {
            result = name_replace(result, 36);
            return name_replace(result, 37);
        }
    }
   result
}

fn name_replace(str: &Il2CppString, index: i32) -> &mut Il2CppString {
    let name = if index == 37 { Mess::get("MPID_Il") } else { Mess::get_name(PIDS[index as usize]) };
    let replace_name = Mess::get_name(DVCVariables::get_dvc_person(index, false));
    replace_string(str, name, replace_name)
}

pub fn god_engage_random_str(god: &GodData) -> String {
    format!("* {}:\n\tEngage Atk:{}\n\tLink Engage Atk: {}\n\tLink Emblem / Person: {} / {}",
            Mess::get( god.mid),
            get_skill_name_from_sid(god.get_engage_attack()),
            god_link_engage_atk_str(god),
            god_link_god(god),
            god_link_pid(god)
    )
}

fn is_player_name_replace(ptr: *const u8) -> bool {
    let mut offset = 0;
    loop {
        let value = unsafe { get_u16(ptr, offset, None) };
        match value {
            0..11 => { return false; }
            15 => {
                offset += 6;
            }
            14 => {
                let group = unsafe { get_u16(ptr, offset + 2 , None) };
                let tag = unsafe { get_u16(ptr, offset + 4 , None) };
                if group == 6 && tag == 3 { return true; }
                offset += 8;
            }
            _ => { offset += 2; }
        }
    }
}


pub fn god_link_god(god: &GodData) -> String { 
    god.get_link_god_data()
        .map_or_else(|| String::from(" ------ "), |link| Mess::get(link.mid).to_string())
}

pub fn god_link_engage_atk_str(god: &GodData) -> String { 
    god.get_engage_attack_link()
        .map_or_else(|| String::from(" ------ "), |sid| crate::utils::get_skill_name_from_sid(sid))
}
pub fn god_link_pid(god: &GodData) -> String { 
    god.get_link().map_or_else(||{
        let gid = god.gid.to_string();
        EMBLEM_GIDS.iter().position(|&r| r == gid).filter(|&pos| unsafe { LINKED[ pos ] != -1 })
            .map_or_else(||String::from(" ------ "), |pos| Mess::get_name( PIDS[ unsafe { LINKED[pos] as usize } ] ).to_string() )
    },
    |pid| Mess::get_name(pid).to_string())
}

// Prevents Alear or anyone with no cooking data from cooking 
pub fn cooking_menu_build_attribute(_this: u64, _method_info: OptionalMethod) -> i32 {
    if let Some(chef) = HubUtil::get_current_cooking_pid() {
        if CookData::get(chef).is_none() {  return 4; }
    }
    if GameVariableManager::get_bool("G_拠点_料理済み") { 2 } else { 1 }
}
pub fn lol_map_attribute(_this: u64, _method_info: OptionalMethod) -> i32 { 1 }

pub fn on_str() -> &'static Il2CppString { Mess::get("MID_CONFIG_TUTORIAL_ON") }
pub fn off_str() -> &'static Il2CppString { Mess::get("MID_CONFIG_TUTORIAL_OFF") }

#[skyline::from_offset(0x0336d1f0)]
pub fn get_u16(ptr: *const u8, offset: i32, method_info: OptionalMethod) -> u16;

#[skyline::from_offset(0x025d7f90)]
fn get_u16_ptr(label: &Il2CppString, method_info: OptionalMethod) -> *const u8;