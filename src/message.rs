use concat_string::concat_string;
use unity::prelude::*;
use engage::{
    gamevariable::*, gameuserdata::*, mess::*,
    gamedata::{cook::CookData, *, skill::*},
    hub::*,
};
use crate::{
    enums::*,
    randomizer::{emblem::emblem_skill::EIRIKA_INDEX, *},
    utils::*,
};
use std::sync::OnceLock;
use std::collections::HashMap;
static MID_SWAPS: OnceLock<HashMap<String, (i32, i32)>> = OnceLock::new();
const LUEUR_MIDS: &[&str] = &["MPID_Lueur", "MGID_Lueur", "MID_SELECTRING_LUEUR_NOTES", "MPID_PastLueur", "MPID_H_Veyre", "MID_RULE_M024_WIN"];

pub fn initialize_mess_hashs() {
    MID_SWAPS.get_or_init(||{
        let mut vec: HashMap<String, (i32, i32)> = HashMap::new();
        for x in 0..LUEUR_MIDS.len() {
            if x < 2 { vec.insert(LUEUR_MIDS[x].to_string(), ( 1, 0) ); }  // Lueur Name Swap
            else { vec.insert(LUEUR_MIDS[x].to_string(), ( 2, 0) ); }  // Lueur Name Replacement
        }
        vec.insert("MSID_H_EirikEngage".to_string(), ( 9, 0) );
        vec.insert("MID_RULE_M006_LOSE".to_string(), ( 3, 10) );
        vec.insert("MID_RULE_M015_LOSE".to_string(), ( 3, 27) );
        vec.insert("MID_RULE_M015_WIN".to_string(), ( 3, 27) );
        vec.insert("MID_TUT_NAVI_M015_ESCAPE".to_string(), ( 3, 27) );
        vec.insert("MID_RULE_M007_WIN".to_string(), ( 3, 26) ); 
        vec.insert("MID_RULE_M008_WIN".to_string(), ( 3, 17) );
        vec.insert("MID_RULE_M009_WIN".to_string(), ( 3, 17) );
        vec.insert("MID_RULE_M014_WIN".to_string(), ( 4, 26) ); //Mauvier Name Swap
        vec.insert("MID_RULE_M016_WIN".to_string(), ( 4, -1) );
        vec.insert("MID_RULE_M017_WIN".to_string(), ( 4, -1) );
        vec.insert("MID_RULE_M019_WIN".to_string(), ( 4, -1) );
        for x in 1..12 {
            vec.insert(format!("MTID_Ring_{}",  RINGS[ x as usize ]), ( 5, x)); // Tile Replacement
            vec.insert(format!("MID_TUT_NAVI_M022_GET_{}",  RINGS[ x as usize ]), ( 6, x)); // "Pick up"
        }
        vec.insert("MID_Hub_Next_Go".to_string(), (8, 0));  // COntinous Mode
        vec.insert("MID_Hub_Next_Go1".to_string(), (8, 1));  // COntinous Mode
        println!("Mess Entries: {}", vec.len());
        vec
    });

}

#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&'static Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(label, is_replaced, method_info);
    if !RANDOMIZER_STATUS.read().unwrap().enabled || !DVCVariables::random_enabled() { return result; }

    if let Some(mess_il2cp) = label {
        let mess_label = mess_il2cp.to_string();
        let hash_map = MID_SWAPS.get().unwrap();
        if let Some(v) = hash_map.get(&mess_label) {
            match v.0 {
                1 => {
                    if GameVariableManager::exist(DVCVariables::LUEUR_NAME) { return GameVariableManager::get_string(DVCVariables::LUEUR_NAME); }
                    else { return result; }
                }
                2 => {  // Alear Name Swap
                    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
                        return replace_string(result, Mess::get_name(DVCVariables::get_dvc_person(0, false)), GameVariableManager::get_string(DVCVariables::LUEUR_NAME));
                    }
                }
                3 => {  //Enemy Person Name Swap
                    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
                        return replace_string(result, Mess::get(MPIDS[v.1 as usize]), Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[v.1 as usize]))));
                    }
                }
                4 => {  // Mauvier Replacement
                    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
                        if v.1 != -1 {
                            let str1 = replace_string(result, Mess::get(MPIDS[v.1 as usize]), Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[v.1 as usize]))));
                            return replace_string(str1, Mess::get(MPIDS[33]), Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[33]))));
                        }
                        return replace_string(result, Mess::get(MPIDS[33]), Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[33]))));
                    }
                }
                5 => {  // Ring/bracelet of XXXX Swaps
                    if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                        let new_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[v.1 as usize].to_string(), false);
                        return call_original!(Some(concat_string!("MGID_Ring_", RINGS[ new_index as usize ]).into()), true, None);
                    }
                }
                6 => {  // You acquired the Ring of the [XXXXXX].
                    if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                        let mock_text = call_original!(Some("MID_TUT_NAVI_M022_GET_Siglud".into()), is_replaced, method_info);
                        let new_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[v.1 as usize].to_string(), false);
                        let sigurd_text = call_original!(Some("MGID_Ring_Siglud".into()), true, None);
                        let new_emblem = call_original!(Some(concat_string!("MGID_Ring_", RINGS[new_index as usize]).into()), false, None);
                        return replace_string(mock_text, sigurd_text, new_emblem);
                    }
                }
                8 => {
                    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) != 0 {
                        if let Some(next) = GameUserData::get_chapter().get_next_chapter() {
                            if v.1 == 0 {
                                return format!("{} ({})", call_original!(Some(next.name), true, None), call_original!(Some(format!("{}_PREFIX", next.name).into()), true, None)).into();
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
                9 => {
                    let eirika = call_original!(Some("MGID_Eirik".into()), true, None);
                    let new_emblem = call_original!( Some(  GodData::get(  EMBLEM_GIDS[ unsafe { EIRIKA_INDEX } ]).unwrap().mid ), true, None);
                    return replace_string(result, eirika, new_emblem);
                }
                _ => { return result; }
            }
        }
    }
    return result;
}

pub fn god_engage_random_str(god: &GodData) -> String {
    return format!("* {}:\n\tEngage Atk:{}\n\tLink Engage Atk: {}\n\tLink Emblem / Person: {} / {}", Mess::get( god.mid), crate::utils::get_skill_name_from_sid(god.get_engage_attack()), god_link_engage_atk_str(god), god_link_god(god), god_link_pid(god));
}

pub fn god_link_god(god: &GodData) -> String { 
    god.get_link_god_data().map_or_else(|| String::from(" ------ "), |link| Mess::get(link.mid).to_string()) 
}

pub fn god_link_engage_atk_str(god: &GodData) -> String { 
    god.get_engage_attack_link().map_or_else(|| String::from(" ------ "), |sid| crate::utils::get_skill_name_from_sid(sid)) 
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
pub fn lol_map_attribute(_this: u64, _method_info: OptionalMethod) -> i32 { 
    return 1;
}