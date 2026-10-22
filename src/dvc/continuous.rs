use super::*;
use engage::{
    app::{Random_2, BasicMenuItem, BasicMenu_Result, ProcInst, BasicMenuItem_Attribute},
    List_1Ext,
};
mod sequence;

use outfit_core::Randomizer;
use crate::{get_chapter_from_idx, enums::EMBLEM_GIDS};
use crate::utils::{override_vtable, to_opt};

const RANDOM_CHAPTER_CONDITIONS: [(i32, i32); 12] = [
    (11,0x740), (14,0x480), (16,0x480), (17,0x900),
    (19,0x940), (20, 0x900), (21, 0x900), (22,0x3BC940), (23,0x7BE940),
    (24,0x7BE940), (25,0x7BE940), (26,0x7BE940),
];
const UNESCAPE: [i32; 6] = [21, 17, 20, 19, 19, 17];
const JOIN: [i32; 19] = [1, 3, 4, 6, 8, 8, 11, 11, 13, 14, 15, 16, 57, 51, 52, 53, 54, 55, 56];
pub fn continuous_mode_vtable_edit() {
    override_vtable("App", "SortieTopMenu.BackMenuItem", "GetMapAttribute", sortie_back_map_attribute as _);
    /*
    ["Back", "Friend", "God", "Inventory", "PositionChange", "ResetMap", "SaveMap", "SaveMenu", "SelectionUnit", "Shop", "Start", "System"].iter().for_each(|v| {
        let class = format!("App.SortieTopMenu.{}MenuItem", v);
        override_vtable("App", class.as_str(), "BCall", sortie_sequence_b_call as _);
    });
     */
}

pub fn post_chapter_continuous_mode(proc: ProcInst) {
    if DVCVariables::is_main_chapter_complete(26) { return; }
    let mode = DVCVariable::Continuous.get_value();
    if mode == 1 || mode == 2 { sequence::create_continuous_mode_bind(proc); }
}
pub fn set_next_chapter(current_chapter: ChapterData) -> Option<ChapterData> {
    let prefixless = current_chapter.get_prefixless_cid().to_rust_string();
    GameVariableManager::set_bool(format!("G_Cleared_{}", prefixless), true);
    let mut complete_mask = 0i64;
    for x in 1..57 {
        if DVCVariables::is_main_chapter_complete(x) { complete_mask |= 1 << x; }
    }
    let mode = DVCVariable::Continuous.get_value();
    if mode != 2 { DVCVariables::get_next_main_chapter() }
    else {
        let mut available: Vec<i32> = Vec::new();
        available.extend([5, 6, 7, 8, 9, 10, 12, 13, 15, 18, 31, 32].iter().filter(|&&v| complete_mask & (1 << v) == 0));
        RANDOM_CHAPTER_CONDITIONS.into_iter().for_each(|(c, flag)| {
            let f = ((1 << c)|flag) as i64;
            if complete_mask & f == flag as i64 { available.push(c); }
        });
        if complete_mask & 0x900 != 0 {
            available.extend([44, 39, 43, 41, 42, 40, 33, 34, 35, 36, 37, 38].into_iter().enumerate()
                .filter(|(i, v)|is_god_available(EMBLEM_GIDS[*i], false) && complete_mask & (1 << *v) == 0)
                .map(|(_, x)| x)
            );
        }
        if available.len() == 0 { return Some(ChapterData::get("CID_M026".into())); }
        let rng = Random_2::get_game();
        loop {
            if let Some(i) = available.get_random_element(rng).map(|v| get_chapter_from_idx(*v))
                .map(|i| ChapterData::get(format!("CID_{}", i).into()))
            {
                return Some(i);
            }
        }
    }
}
pub fn post_map_emblem_adjustment() {
    let chapter_index = DVCVariables::get_chapter_index();
    if !DVCVariables::is_main_chapter_complete(chapter_index) { return; }
    match chapter_index {
        1 => escape_or_create(0, false),
        3 => escape_or_create(1, false),
        4 => escape_or_create(2, false),
        6 => escape_or_create(3, false),
        8 => {
            escape_or_create(4, false);
            escape_or_create(5, false);
        }
        11 => {
            escape_or_create(6, false);
            escape_or_create(7, false);
        }
        13 => escape_or_create(8, false),
        14 => escape_or_create(9, false),
        15 => escape_or_create(10, false),
        16 => escape_or_create(11, false),
        17 => {
            escape_or_create(1, false);
            escape_or_create(5, false);
        }
        19 => {
            escape_or_create(3, false);
            escape_or_create(4, false);
        }
        20 => escape_or_create(2, false),
        21 => escape_or_create(0, false),
        22 => { for x in 0..12 { escape_or_create(x, false); } }
        33..45|51..57 => {
            if let Some(v) = EmblemPool::get_god_unlocked_from_paralogue(chapter_index){
                let god_unit = GodPool::try_get_2(v, false);
                if god_unit.is_null() {
                    if chapter_index > 50 { GodPool::create(v); }
                }
                else { god_unit.set_m_is_escaping(false); }
            }
        }
        _ => {}
    }
}
pub fn save_load_emblem_adjustment(){
    let sequence = DVCVariables::get_sequence();
    if sequence == 2 || sequence == 3 || sequence == 7 { return; }
    JOIN.iter().enumerate().for_each(|(i, x)| {
        if i != 12 && DVCVariables::is_main_chapter_complete(*x) {
            if let Some(god) = EmblemPool::get_replacement_god_3(i as i32, false){
                GodPool::create(god);
            }
        }
    });
    if DVCVariables::is_main_chapter_complete(22) {
        for i in 0..12 { escape_or_create(i,false); }
    }
    else if (DVCVariable::Continuous.get_value() < 2 && DVCVariables::is_main_chapter_complete(10)) || DVCVariables::is_main_chapter_complete(11) {
        for i in 0..6 {
            escape_or_create(i, !DVCVariables::is_main_chapter_complete(UNESCAPE[i as usize]));
        }
    }
}
pub fn pre_map_emblem_adjustment() {
    let chapter_index = DVCVariables::get_chapter_index();
    match chapter_index {
        11 => { for i in 0..6 { escape_or_create(i, true); } }
        17 => {
            escape_or_create(1, true);
            escape_or_create(5, true);
        }
        19 => {
            escape_or_create(3, true);
            escape_or_create(4, true);
        }
        20 => escape_or_create(2, true),
        21 => escape_or_create(0, true),
        22 => {
            escape_or_create(0, false);
            for x in 1..12 { escape_or_create(x, true); }
        }
        33..45 => {
            if let Some(v) = EmblemPool::get_god_unlocked_from_paralogue(chapter_index){
                let god_unit = GodPool::try_get_2(v, false);
                if !god_unit.is_null() {
                    god_unit.set_m_is_escaping(true);
                    let unit = god_unit.get_parent();
                    if !unit.is_null() { unit.clear_god_unit(); }
                }
            }
        }
        _ => {}
    }
}
pub fn escape_or_create(emblem_index: i32, escape: bool) {
    if emblem_index > 19 { return; }
    let god_data =
        EmblemPool::get_replacement_god_3(emblem_index, false)
            .unwrap_or_else(|| GodData::get(EMBLEM_GIDS[emblem_index as usize].into()));

    let mut god_unit = GodPool::try_get_2(god_data, false);

    if god_unit.is_null() { god_unit = GodPool::create(god_data); }
    god_unit.set_m_is_escaping(escape);
    if escape {
        let parent = god_unit.get_parent();
        if !parent.is_null() { parent.clear_god_unit(); }
    }
}
pub fn is_god_available(gid: &str, randomized: bool) -> bool {
    if let Some(god_data) =
        if DVCVariable::EmblemRecruitment.get_value() == 0 || !randomized { to_opt(GodData::get(gid.into())) }
        else { to_opt(GodData::get(GameVariableManager::get_string(format!("G_R_{}", gid)))) } 
    {
        to_opt(GodPool::try_get_2(god_data, true)).is_some_and(|u| !u.get_is_escaping())
    } else { false }
}
fn sortie_back_map_attribute(this: BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuItem_Attribute {
    let mode = DVCVariable::Continuous.get_value();
    if mode == 1 || mode == 2 { BasicMenuItem_Attribute::hide() }
    else { unsafe { sortie_back_original_map_attribute(this, optional_method) } }
}
/*
fn sortie_sequence_b_call(this: BasicMenuItem, optional_method: OptionalMethod) -> BasicMenu_Result {
    let mode = DVCVariable::Continuous.get_value();
    if mode == 1 || mode == 2 { BasicMenu_Result::se_miss() }
    else { unsafe { sortie_original_b_call(this, optional_method) } }
}
 */
#[skyline::from_offset(0x1d78d40)]
fn sortie_original_b_call(this: BasicMenuItem, optional_method: OptionalMethod) -> BasicMenu_Result;
#[skyline::from_offset(0x01d76320)]
fn sortie_back_original_map_attribute(this: BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuItem_Attribute;