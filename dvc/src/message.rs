use unity::prelude::*;
use engage::{
    gamevariable::*, gameuserdata::*, mess::*,
    gamedata::{cook::CookData, *},
    hub::*,
};
use crate::{enums::*, utils::*, randomizer::{data::GameData, names::get_emblem_person, *}, };
use std::{sync::OnceLock, collections::HashMap, };

mod swap;
mod original;
pub(crate) mod talk;
mod swap_kinds;
mod swap_command;

pub use swap::TextSwapper;
pub use talk::*;

pub use swap::RING_PICTURE;
use crate::randomizer::status::RandomizerStatus;

static MID_SWAPS: OnceLock<HashMap<String, (i32, i32)>> = OnceLock::new();
pub static MESSAGE_SWAPPER: OnceLock<RwLock<TextSwapper>> = OnceLock::new();
const LUEUR_MIDS: &[&str] = &["MPID_Lueur", "MGID_Lueur", "MID_SELECTRING_LUEUR_NOTES", "MPID_H_Veyre"];
const STATS: &[&str] = &[
    "_Hp", "_Str", "_Tec", "_Spd", "_Lck", "_Def", "_Mag", "_Res", "_Phy", "_Vis", "_Mov",
    "_H_Hp", "_H_Str", "_H_Tec", "_H_Spd", "_H_Lck", "H_Def", "_H_Mag", "_H_Res", "_H_Phy", "_H_Vis", "_H_Mov"];
const STATS_REPLACE: [i32; 11] = [5, 1, 1, 1, 1, 1, 3, 3, 3, 0, 1];
pub fn mid_swaps_init() -> HashMap<String, (i32, i32)> {
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
    }
    vec.insert("MID_Hub_Next_Go".to_string(), (8, 0));  // Continious Mode
    vec.insert("MID_Hub_Next_Go1".to_string(), (8, 1));  // Continious Mode
    vec.insert("MPID_Il_E006".to_string(), (10, 0));
    GameData::get_playable_god_list().iter().enumerate().for_each(|(index, god)|{ vec.insert(god.mid.to_string(), (11, index as i32)); });
    vec
}
pub fn initialize_mess_hashs() {
    MESSAGE_SWAPPER.get_or_init(|| RwLock::new(TextSwapper::init()));
    MID_SWAPS.get_or_init(|| mid_swaps_init() );
}
#[unity::hook("App", "Mess", "AddTagString")]
pub fn mess_add_tag_to_string(tag_group: u16, tag_id: u16, params: Option<&Array<u8>>, method_info: OptionalMethod) {
    let sf = Il2CppClass::from_name("App", "Mess").unwrap().get_static_fields_mut::<MessStaticFields>();
    if tag_group == 6 && tag_id >= 10 {
        let current_mess = sf.mess.to_string();
        let mut args: Vec<u16> = Vec::new();
        if let Some(params) = params{
            params.iter().step_by(2).for_each(|x|{ args.push(*x as u16); });
        }
        if let Some(replacement) = get_replacement_name(tag_id, &args) {
            sf.mess = format!("{}{}", current_mess, replacement).into();
            return;
        }
    }
    else { call_original!(tag_group, tag_id, params, method_info); }
}

#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&'static Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let mut result = call_original!(label, is_replaced, method_info);
    if !RandomizerStatus::get().init || !DVCVariables::random_enabled() { return result; }
    if let Some(mess_il2cp) = label {
        if mess_il2cp.str_contains("MIID_PieceOfBond") { return call_original!(Some("MID_TUT_KR_KIZUNAPIECE_TITLE".into()), is_replaced, method_info);}
        if mess_il2cp.str_contains("MIID_H_PieceOfBond") { return call_original!(Some("MID_TUT_KR_KIZUNAPIECE_1".into()), is_replaced, method_info);}
        if mess_il2cp.str_contains("EmblemLueur") {
            let lueur_name =
                if GameVariableManager::exist(DVCVariables::LUEUR_NAME) { GameVariableManager::get_string(DVCVariables::LUEUR_NAME) }
                else { call_original!(Some("MPID_Lueur".into()), is_replaced, method_info) };

            Mess::set_argument(0, lueur_name);
            return call_original!(Some("MPID_God_Prefix".into()), is_replaced, None);
        }
        let mess_label = mess_il2cp.to_string();
        if mess_label.starts_with("Morph") {
            let mid = mess_label.replace("Morph", "MPID_");
            let name = call_original!(Some(mid.into()), is_replaced, None);
            Mess::set_argument(0, name);
            return call_original!(Some("MPID_Morph_Prefix".into()), is_replaced, None);
        }
        if result.to_string().len() < 2 && mess_label.contains("MSID") {
            if let Some(s) = STATS.iter().position(|&s| mess_label.contains(s)) {
                if let Some(v) = mess_label.split("_").last() {
                    let index = if s > 10 { s - 11 } else { s };
                    let original = call_original!(
                        Some(format!("MSID{}_{}", STATS[s], STATS_REPLACE[index]).into()),
                        is_replaced,
                        method_info
                    ).to_string();
                    if original.len() > 2 { return original.replace( STATS_REPLACE[s].to_string().as_str(), v).into(); }
                }
            }
        }
        if result.is_null() { return result; }
        let hash_map = MID_SWAPS.get_or_init(|| mid_swaps_init() );
        if let Some(v) = hash_map.get(&mess_label) {
            match v.0 {
                1 => {
                  //  if GameVariableManager::exist(DVCVariables::LUEUR_NAME) { return GameVariableManager::get_string(DVCVariables::LUEUR_NAME); }
                  //  else { return result; }
                }
                2 => {  // Alear Name Swap
                    /*
                    if v.1 == 2 && DVCFlags::GodNames.get_value(){   // Emblem Alear Name Swap
                        if let Some(person) = get_emblem_person(mess_il2cp) {
                            let replacement_name = call_original!(person.get_name(), true, None);
                            return replace_string(result, Mess::get_name(DVCVariables::get_dvc_person(0, false)), replacement_name);
                        }
                    }
                    if DVCVariables::UnitRecruitment.get_value() != 0 {
                        return replace_string(
                            result, Mess::get_name(DVCVariables::get_dvc_person(0, false)),
                            GameVariableManager::get_string(DVCVariables::LUEUR_NAME)
                        );
                    }

                     */
                }
                /*
                3 => {  //Enemy Person Name Swap
                    if DVCVariables::UnitRecruitment.get_value() != 0 {
                        return replace_string(
                            result,
                            Mess::get(MPIDS[v.1 as usize]),
                            Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", PIDS[v.1 as usize])))
                        );
                    }
                }
                4 => {  // Mauvier Replacement
                    if DVCVariables::UnitRecruitment.get_value() != 0 {
                        if v.1 != -1 { result = name_replace(result, v.1); }
                        result = name_replace(result, 33);
                    }
                }

                 */
                5 => {  // Ring/bracelet of XXXX Swaps
                    if DVCVariables::EmblemRecruitment.get_value() != 0 {
                        let new_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[v.1 as usize].to_string(), false);
                        if new_index >= 0 && new_index < 23 {
                            return call_original!(Some(format!("MGID_Ring_{}", RINGS[ new_index as usize ]).into()), true, None);
                        }
                        else { return result; }
                    }
                }
                8 => {
                    if DVCVariables::Continuous.get_value() != 0 {
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
                10 => { return name_replace(result, 37); }
                11 => {
                    if let Some(name) = get_emblem_person(mess_il2cp).and_then(|x| x.name) { return call_original!(Some(name), true, None); }
                }
                _ => {}
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
// Prevents Alear or anyone with no cooking data from cooking 
pub fn cooking_menu_build_attribute(_this: u64, _method_info: OptionalMethod) -> i32 {
    if let Some(chef) = util::HubUtil::get_current_cooking_pid() {
        if CookData::get(chef).is_none() {  return 4; }
    }
    if GameVariableManager::get_bool("G_拠点_料理済み") { 2 } else { 1 }
}
pub fn lol_map_attribute(_this: u64, _method_info: OptionalMethod) -> i32 { 1 }